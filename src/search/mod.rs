use std::collections::HashSet;
use std::sync::atomic::AtomicIsize;

use itertools::Itertools;
use rayon::prelude::*;

mod heuristic;
mod meta;
mod params;
mod segment;

pub use heuristic::Heuristic;
use meta::SolutionMetadata;
pub use params::BlockBuildingSearchParams;
pub use segment::{Segment, SegmentId, SegmentStore};

use crate::MAX_SOLUTION_COUNT;
use crate::Profile;
use crate::sim::*;

pub struct Solver {
    profile: Profile,
    puzzle: &'static Puzzle,
    params: BlockBuildingSearchParams,
    segments: SegmentStore,
}
impl Solver {
    pub fn new(profile: Profile, scramble: impl Into<Vec<Twist>>) -> Self {
        Self {
            profile,
            puzzle: &*RUBIKS_4D,
            params: BlockBuildingSearchParams {
                heuristic: Heuristic::Fast,
                max_depth: 4,
                parallel_depth: 2,
                verbosity: 2,
            },
            segments: SegmentStore::new(scramble.into()),
        }
    }

    pub fn solve(mut self) -> Vec<Twist> {
        let start = std::time::Instant::now();

        // Keep the call graph flat for recursion.

        println!("\nSTAGE 1: mid + left, 2x2x2x2 block");
        self.do_blockbuilding_stage(self.profile.select(1, 3), |meta| meta.stage1());

        println!("\nSTAGE 2: mid + left, 2x2x3x2 block");
        self.do_blockbuilding_stage(self.profile.select(1, 3), |meta| meta.stage2());

        println!("\nSTAGE 3: mid + left, 2x3x3x2 block");
        self.do_blockbuilding_stage(self.profile.select(2, 1), |meta| meta.stage3());

        println!("\nSTAGE 4: right (mid + left), 2x2x2x1 block");
        self.do_blockbuilding_stage(self.profile.select(2, 4), |meta| meta.stage4());

        println!("\nSTAGE 5: right (mid + left), 2x2x3x1 block");
        self.do_blockbuilding_stage(self.profile.select(2, 3), |meta| meta.stage5());

        println!("\nSTAGE 6: tripod 2x2x2");
        self.do_blockbuilding_stage(self.profile.select(3, 3), |meta| meta.stage6());

        // println!("\nSTAGE 7: tripod 2x2x3");
        // self.do_blockbuilding_stage(self.profile.select(3, 3), |meta| meta.stage7());

        println!("\nSTAGE 7: solve the rest of the puzzle");
        self.do_blockbuilding_stage(self.profile.select(3, 3), |meta| meta.stage8());

        println!("\nTotal elapsed time: {:?}", start.elapsed());

        println!();
        let best_solution = *self.segments.best_solutions_so_far().first().unwrap();
        println!("Best solution: {}", self.segments[best_solution]);
        println!(
            "{}",
            self.segments
                .solution_twists_for_segment(best_solution)
                .into_iter()
                .join(" ")
        );

        let out_file_path = "out.txt";
        std::fs::write(
            out_file_path,
            self.segments
                .best_solutions_so_far()
                .iter()
                .map(|&s| {
                    self.segments
                        .solution_twists_for_segment(s)
                        .into_iter()
                        .join(" ")
                })
                .join("\n"),
        )
        .unwrap();
        println!("All solutions written to {out_file_path}");

        self.segments.solution_twists_for_segment(best_solution)
    }

    fn do_blockbuilding_stage<I: IntoIterator<Item = (Block, SolutionMetadata)>>(
        &mut self,
        target_block_count: usize,
        make_target_blocks: impl Send + Sync + Fn(SolutionMetadata) -> I,
    ) {
        let t = std::time::Instant::now();

        let step = self.segments.next_step();

        // Add pieces
        let new_segments = self.do_step(|this, prev_segments| {
            prev_segments
                .par_iter()
                .flat_map(|&prev_segment_id| {
                    let prev_segment = &this.segments[prev_segment_id];
                    let mut results = vec![];
                    for (new_block, new_meta) in make_target_blocks(prev_segment.meta) {
                        let setup_moves =
                            this.segments.all_prior_twists_for_segment(prev_segment_id);
                        if let Some(new_segment) =
                            prev_segment.push_block(this.puzzle, &setup_moves, new_block, new_meta)
                        {
                            results.push(new_segment);
                        }
                    }
                    results
                })
                .collect()
        });
        let init_blocks = new_segments
            .iter()
            .map(|segment| segment.state.blocks.len())
            .max()
            .unwrap_or(0);

        println!(
            "  {step:02}. Added pieces ({} options with {} blocks each)",
            new_segments.len(),
            init_blocks,
        );
        if new_segments.is_empty() {
            println!("  WARNING: NO OPTIONS. You may need to increase `MAX_BLOCKS`");
        }

        self.segments.add_segments(step, new_segments);

        // Blockbuild
        for target in (target_block_count..init_blocks).rev() {
            self.do_blockbuilding_step(target);
            // if self.steps.last().unwrap().is_empty() {
            //     log!(self.params, 1, "No solutions! Giving up ...");
            //     std::process::exit(1);
            // }
        }

        println!("  Completed stage in {:?}", t.elapsed());
    }

    fn do_blockbuilding_step(&mut self, block_target: usize) {
        let step = self.segments.next_step(); // TODO: bad

        let mut max_depth = 0;

        let mut new_segments = self.do_step(|this, prev_segments| {
            let mut new_segments = vec![];
            let limit_grip_set = step >= 52;
            let extra_max_depth = if limit_grip_set { 2 } else { 0 };
            for depth in 0..=this.params.max_depth + extra_max_depth {
                let desired_solution_count = match depth.saturating_sub(extra_max_depth) {
                    ..=1 => crate::MIN_SOLUTION_COUNT_DEPTH_1,
                    2 => crate::MIN_SOLUTION_COUNT_DEPTH_2,
                    3 => crate::MIN_SOLUTION_COUNT_DEPTH_3,
                    4.. => crate::MIN_SOLUTION_COUNT_DEPTH_4,
                };
                let solutions_left_to_find =
                    desired_solution_count.saturating_sub(new_segments.len());
                overprint!("  Blockbuilding to {block_target} at depth {depth} ...");

                new_segments.par_extend(
                    prev_segments
                        .par_iter()
                        .flat_map(|&prev_segment| {
                            let allowed_grips = if limit_grip_set {
                                let meta = this.segments[prev_segment].meta;
                                GripSet::from_iter([
                                    meta.front_grip(),
                                    meta.right_grip(),
                                    meta.last_layer(),
                                ])
                            } else {
                                GripSet::ALL
                            };

                            let mut results = vec![];
                            dfs_blockbuild(
                                this.params,
                                this.puzzle,
                                block_target,
                                depth,
                                &mut results,
                                this.segments[prev_segment].next_step(prev_segment),
                                None,
                                if depth > this.params.parallel_depth {
                                    this.params.parallel_depth
                                } else {
                                    0
                                },
                                allowed_grips,
                            );
                            results
                        })
                        .take_any(solutions_left_to_find),
                );

                max_depth = depth;
                if new_segments.len() >= desired_solution_count {
                    break;
                }
            }

            // overprint!("  Processing {} solutions ...", prev_segments.len());
            // let mut solutions_by_initial_state: HashMap<BlockSet, Vec<_>> = HashMap::new();
            // for (initial_state, final_state, twists) in solutions {
            //     solutions_by_initial_state
            //         .entry(initial_state)
            //         .or_default()
            //         .push((final_state, twists));
            // }

            // overprint!("  Applying twists to {} solutions ...", prev_segments.len());
            // prev_segments
            //     .into_iter()
            //     .flat_map(|&id| {
            //         solutions_by_initial_state
            //             .get(&this.segments[id].state)
            //             .into_iter()
            //             .flatten()
            //             .map(move |&(final_state, twists)| {
            //                 this.segments
            //                     .push_twists_onto_segment(id, final_state, twists)
            //             })
            //     })
            //     .take(crate::MIN_SOLUTION_COUNT_DEPTH_1)
            //     .collect()

            new_segments
        });

        overprint!("  Deduplicating {} solutions ...", new_segments.len());
        for s in &mut new_segments {
            s.state.blocks.sort_unstable();
        }
        let unique_states: HashSet<BlockSet> =
            new_segments.iter().map(|segment| segment.state).collect();
        let min_twist_count = new_segments
            .iter()
            .map(|s| s.total_twist_count)
            .min()
            .unwrap_or(0);
        overprintln!(
            "  {step:02}. Blockbuilt to {block_target} with max depth {max_depth} ({} solutions, {} unique; best is {} ETM)",
            new_segments.len(),
            unique_states.len(),
            min_twist_count,
        );
        if step > 64 {
            overprint!("    (finalizing ...)");
            let all = new_segments.into_iter().into_group_map_by(|s| s.state);
            new_segments = all
                .into_values()
                .filter_map(|possible_new_segments| {
                    possible_new_segments
                        .into_iter()
                        .min_by_key(|s| s.total_twist_count)
                })
                .collect_vec();
        }

        self.segments.add_segments(step, new_segments);
    }

    #[must_use]
    fn do_step(
        &mut self,
        continue_solutions: impl Send + Sync + FnOnce(&Self, &[SegmentId]) -> Vec<Segment>,
    ) -> Vec<Segment> {
        let t = std::time::Instant::now();

        let step = self.segments.next_step();
        let last_step = step - 1;
        let segments_to_search_from = self.segments.segment_ids_for_step(last_step).to_vec();

        let mut new_solution_segments = continue_solutions(self, &segments_to_search_from);

        // Sort by twist count and remove duplicates
        new_solution_segments.sort();
        new_solution_segments.dedup();

        log!(
            self.params,
            2,
            "Completed step in {:?} with {} solutions",
            t.elapsed(),
            new_solution_segments.len(),
        );

        if let Some(best) = new_solution_segments.first() {
            log!(self.params, 3, "Best solution: {best}");
            let twist_count_sums = new_solution_segments
                .iter()
                .map(|s| s.total_twist_count)
                .counts()
                .into_iter()
                .sorted()
                .map(|(len, count)| format!("{len}: {count}"))
                .join(", ");
            log!(self.params, 4, "By twist count: {{{twist_count_sums}}}");
        }

        if new_solution_segments.len() > MAX_SOLUTION_COUNT {
            log!(
                self.params,
                3,
                "Truncating to {MAX_SOLUTION_COUNT} solutions"
            );
            new_solution_segments.truncate(MAX_SOLUTION_COUNT);
        }

        new_solution_segments
    }
}

/// Runs a depth-first search to `remaining_depth` for sequences of moves that
/// results in at most `expected_blocks` blocks.
///
/// Results are accumulated into `solutions_buffer`.
#[allow(clippy::too_many_arguments)]
pub fn dfs_blockbuild(
    params: BlockBuildingSearchParams,
    puzzle: &Puzzle,
    expected_blocks: usize,
    remaining_depth: usize,
    solutions_buffer: &mut Vec<Segment>,
    solution_so_far: Segment,
    solutions_left_to_find: Option<&AtomicIsize>,
    remaining_parallel_depth: usize,
    grip_subset: GripSet,
) {
    let Segment {
        state,
        segment_twists,
        ..
    } = solution_so_far;

    if state.blocks.len() <= expected_blocks {
        // found a solution!
        if let Some(count) = solutions_left_to_find {
            count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
        solutions_buffer.push(solution_so_far);
        return;
    }
    if remaining_depth == 0 {
        return; // no more to search; give up
    }

    if solutions_left_to_find.is_some_and(|n| n.load(std::sync::atomic::Ordering::Relaxed) <= 0) {
        return; // give up
    }

    if !params
        .heuristic
        .might_be_solvable(puzzle, state, expected_blocks, remaining_depth)
    {
        return; // probably not solvable; give up
    }

    let combined_layer_mask = state
        .blocks
        .iter()
        .map(|b| b.layers())
        .fold(PackedLayers::EMPTY, |a, b| a | b);

    let mut last_grips = segment_twists.iter().rev().map(|twist| twist.grip);
    let last_grip = last_grips.next();
    let second_to_last_grip = last_grips.next();
    let grip_is_worth_testing = |grip: &&GripData| {
        if last_grip == Some(grip.id) {
            return false; // same grip as last move
        }
        if last_grip == Some(grip.id.opposite()) && second_to_last_grip == Some(grip.id) {
            return false; // opposite grip already moved
        }
        if combined_layer_mask.grip_status(grip.id) == GripStatus::Inactive {
            return false; // doesn't move any block
        }
        if !grip_subset.contains(grip.id) {
            return false;
        }
        // TODO: don't check opposite if it was 2nd-to-last move
        true
    };

    let explore_twist = |twist, solutions_buffer: &mut Vec<_>| {
        if let Some(new_partial_solution) = solution_so_far.push_twist(twist, last_grip) {
            dfs_blockbuild(
                params,
                puzzle,
                expected_blocks,
                remaining_depth - 1,
                solutions_buffer,
                new_partial_solution,
                solutions_left_to_find,
                remaining_parallel_depth.saturating_sub(1),
                grip_subset,
            );
        }
    };

    if remaining_parallel_depth > 0 {
        let grips = puzzle.grips.par_iter().filter(grip_is_worth_testing);
        let twists = grips.flat_map(|grip| grip.par_twists());
        solutions_buffer.par_extend(twists.flat_map_iter(|twist| {
            let mut solutions_buffer = vec![];
            explore_twist(twist, &mut solutions_buffer);
            solutions_buffer
        }));
    } else {
        let grips = puzzle.grips.iter().filter(grip_is_worth_testing);
        let twists = grips.flat_map(|grip| grip.twists());
        twists.for_each(|twist| explore_twist(twist, solutions_buffer));
    }
}
