use itertools::Itertools;
use rayon::prelude::*;

mod heuristic;
mod meta;
mod params;
mod segment;

pub use heuristic::Heuristic;
use meta::SolutionMetadata;
pub use params::BlockBuildingSearchParams;
pub use segment::SolutionSegment;

use crate::sim::*;
use crate::{MAX_SOLUTION_COUNT, MIN_SOLUTION_COUNT};

pub struct Solver {
    puzzle: &'static Puzzle,
    params: BlockBuildingSearchParams,
    scramble: Vec<Twist>,
    stages: Vec<Vec<SolutionSegment>>,
}
impl Solver {
    fn do_blockbuilding_stage<I: IntoIterator<Item = (Block, SolutionMetadata)>>(
        &mut self,
        target_block_count: usize,
        make_target_blocks: impl Send + Sync + Fn(SolutionMetadata) -> I,
    ) {
        // Add pieces
        log!(self.params);
        log!(self.params, 1, "Adding pieces");
        let new_segments = do_step(
            self.params,
            self.stages.last().unwrap(),
            [()],
            |()| {},
            |(), init, solutions_buffer| {
                for (new_block, new_meta) in make_target_blocks(init.meta) {
                    let setup_moves =
                        self.all_prior_twists_for_segment(init.previous_segment_index);
                    if let Some(new_segment) =
                        init.push_block(&self.puzzle, &setup_moves, new_block, new_meta)
                    {
                        solutions_buffer.push(new_segment);
                    }
                }
            },
        );
        log!(
            self.params,
            1,
            "Done adding pieces: {} options",
            new_segments.len()
        );
        self.stages.push(new_segments);

        // Blockbuild
        self.blockbuild_down_to(target_block_count);
    }

    fn blockbuild_down_to(&mut self, target_block_count: usize) {
        let last_stage = self.stages.last().unwrap();
        let worst_candidate_from_last_stage = last_stage.last().unwrap();
        let init_blocks = worst_candidate_from_last_stage.state.blocks.len();
        for target in (target_block_count..init_blocks).rev() {
            do_step_blockbuilding(self.puzzle, self.params, &mut self.stages, target);
            if self.stages.last().unwrap().is_empty() {
                log!(self.params, 1, "No solutions! Backtracking ...");
                self.stages.pop();
            }
        }
    }

    pub fn new(scramble: impl Into<Vec<Twist>>) -> Self {
        Self {
            puzzle: &*RUBIKS_4D,
            params: BlockBuildingSearchParams {
                heuristic: Heuristic::Fast,
                max_depth: 4,
                verbosity: 5,
            },
            scramble: scramble.into(),
            stages: vec![vec![SolutionSegment::default()]],
        }
    }

    pub fn solve(mut self) -> Vec<Twist> {
        let start = std::time::Instant::now();

        log!(self.params, 0, "STAGE 1: mid + left, 2x2x2x2 block");
        self.do_blockbuilding_stage(1, |meta| meta.stage1());
        log!(self.params);

        log!(self.params, 0, "STAGE 2: mid + left, 2x2x3x2 block");
        self.do_blockbuilding_stage(1, |meta| meta.stage2());
        log!(self.params);

        log!(self.params, 0, "STAGE 3: mid + left, 2x3x3x2 block");
        self.do_blockbuilding_stage(1, |meta| meta.stage3());
        log!(self.params);

        log!(self.params, 0, "STAGE 4: right (mid + left), 2x2x2x1 block");
        self.do_blockbuilding_stage(2, |meta| meta.stage4());
        log!(self.params);

        log!(self.params, 0, "STAGE 5: right (mid + left), 2x2x3x1 block");
        self.do_blockbuilding_stage(2, |meta| meta.stage5());
        log!(self.params);

        log!(self.params, 0, "STAGE 6: front-right, last F2L-a pair");
        self.do_blockbuilding_stage(3, |meta| meta.stage6());
        log!(self.params);

        log!(
            self.params,
            0,
            "STAGE 7: front-right, second-to-last F2L-b pair"
        );
        self.do_blockbuilding_stage(3, |meta| meta.stage7());
        log!(self.params);

        log!(self.params, 0, "STAGE 8: F2L");
        self.do_blockbuilding_stage(1, |meta| meta.stage8());
        log!(self.params);

        println!("Total elapsed time: {:?}", start.elapsed());

        println!();
        println!(
            "Best solution: {}",
            self.stages.last().unwrap().first().unwrap()
        );
        println!("{}", self.solution_twists_for_segment(0).iter().join(" "));

        self.solution_twists_for_segment(0)
    }

    fn solution_twists_for_segment(&self, segment_index: usize) -> Vec<Twist> {
        self.twists_for_segment(&[], segment_index)
    }
    fn all_prior_twists_for_segment(&self, segment_index: usize) -> Vec<Twist> {
        self.twists_for_segment(&self.scramble, segment_index)
    }
    fn twists_for_segment(&self, init: &[Twist], mut segment_index: usize) -> Vec<Twist> {
        let mut reversed_twists = vec![];
        for stage in self.stages.iter().rev() {
            let segment = &stage[segment_index];
            reversed_twists.extend(segment.segment_twists.into_iter().rev());
            segment_index = segment.previous_segment_index;
        }
        init.iter()
            .chain(reversed_twists.iter().rev())
            .copied()
            .collect()
    }
}

fn do_step_blockbuilding(
    puzzle: &Puzzle,
    params: BlockBuildingSearchParams,
    stages: &mut Vec<Vec<SolutionSegment>>,
    block_target: usize,
) {
    log!(params, 1);
    log!(params, 1, "Blockbuilding to {block_target}");
    let new_segments = do_step(
        params,
        stages.last().unwrap(),
        0..=params.max_depth,
        |depth| log!(params, 2, "Searching at depth {depth} ..."),
        |depth, segment, solutions_buffer| {
            dfs_blockbuild(
                params,
                puzzle,
                block_target,
                depth,
                solutions_buffer,
                segment,
            );
        },
    );
    stages.push(new_segments);
}

#[must_use]
pub fn do_step<O: Copy + Send + Sync>(
    params: BlockBuildingSearchParams,
    prev_stage: &[SolutionSegment],
    search_options: impl IntoIterator<Item = O>,
    mut print_fn: impl FnMut(O),
    find_solution_segments: impl Send + Sync + Fn(O, SolutionSegment, &mut Vec<SolutionSegment>),
) -> Vec<SolutionSegment> {
    let t = std::time::Instant::now();

    let mut new_solution_segments = vec![];
    for search_options in search_options {
        print_fn(search_options);
        new_solution_segments.par_extend(prev_stage.par_iter().enumerate().flat_map(
            |(previous_segment_index, prev_segment)| {
                let mut results = vec![];
                find_solution_segments(
                    search_options,
                    prev_segment.next_stage(previous_segment_index),
                    &mut results,
                );
                results
            },
        ));
        if new_solution_segments.len() >= MIN_SOLUTION_COUNT {
            break;
        } else if !new_solution_segments.is_empty() {
            log!(
                params,
                3,
                "Found {} solutions; continuing ...",
                new_solution_segments.len(),
            );
        }
    }

    log!(
        params,
        2,
        "Completed step in {:?} with {} solutions",
        t.elapsed(),
        new_solution_segments.len(),
    );

    new_solution_segments.sort();

    if let Some(best) = new_solution_segments.first() {
        log!(params, 3, "Best solution: {best}");
    }

    if new_solution_segments.len() > MAX_SOLUTION_COUNT {
        log!(params, 3, "Truncating to {MAX_SOLUTION_COUNT} solutions");
        new_solution_segments.truncate(MAX_SOLUTION_COUNT);
    }

    new_solution_segments
}

/// Runs a depth-first search to `remaining_depth` for sequences of moves that
/// results in at most `expected_blocks` blocks.
///
/// Results are accumulated into `solutions_buffer`.
pub fn dfs_blockbuild(
    params: BlockBuildingSearchParams,
    puzzle: &Puzzle,
    expected_blocks: usize,
    remaining_depth: usize,
    solutions_buffer: &mut Vec<SolutionSegment>,
    solution_so_far: SolutionSegment,
) {
    let SolutionSegment {
        state,
        segment_twists,
        ..
    } = solution_so_far;

    if state.blocks.len() <= expected_blocks {
        // found a solution!
        solutions_buffer.push(solution_so_far);
        return;
    }
    if remaining_depth == 0 {
        return; // no more to search; give up
    }

    if !params
        .heuristic
        .might_be_solvable(state, expected_blocks, remaining_depth)
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
        if combined_layer_mask.grip_status(grip.id) == Some(GripStatus::Inactive) {
            return false; // doesn't move any block
        }
        // TODO: don't check opposite if it was 2nd-to-last move
        true
    };

    for grip in puzzle.grips.iter().filter(grip_is_worth_testing) {
        for twist in grip.twists() {
            if let Some(new_partial_solution) = solution_so_far.push_twist(twist, last_grip) {
                dfs_blockbuild(
                    params,
                    puzzle,
                    expected_blocks,
                    remaining_depth - 1,
                    solutions_buffer,
                    new_partial_solution,
                )
            }
        }
    }
}
