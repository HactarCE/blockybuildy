use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod dfs;
mod heuristic;
mod params;

use crate::sim::*;
pub use heuristic::Heuristic;
pub use params::BlockBuildingSearchParams;

use crate::{PuzzleState, Twist};

pub struct BlockBuildingSearch {
    /// Puzzle to solve.
    pub puzzle: &'static Puzzle,
    /// Search parameters.
    pub params: BlockBuildingSearchParams,

    /// Scramble (setup moves).
    pub scramble: Vec<Twist>,

    /// Latest puzzle state.
    pub state: PuzzleState,
    /// Latest solution.
    pub solution: Vec<Twist>,
    /// Latest solution as a list of segments.
    pub solution_segments: Vec<Vec<Twist>>,

    /// Whether to display segments with parentheses.
    pub parenthesize_segments: bool,
}

impl BlockBuildingSearch {
    /// Returns the combined scramble + latest solution.
    pub fn all_moves(&self) -> Vec<Twist> {
        itertools::chain(&self.scramble, &self.solution)
            .copied()
            .collect()
    }

    pub fn solution_str(&self) -> String {
        if self.parenthesize_segments {
            self.solution_segments
                .iter()
                .map(|segment| format!("({})", segment.iter().join(" ")))
                .join(" ")
        } else {
            self.solution.iter().join(" ")
        }
    }

    /// Solves one of the candidate blocks using
    /// [`blockbuild_via_repeated_iddfs()`] and returns the block solved.
    ///
    /// All candidates blocks are solved simultaneously, and the one with the
    /// shortest solution is returned.
    ///
    /// If any search succeeded, then the shortest solution is selected: `state`
    /// is updated and the solution is appended to `solution` and a tuple is
    /// returned containing the block that was solved and a list of the new
    /// solution segments. If all candidates failed, then `state` and `solution`
    /// are left unmodified and an error is returned.
    pub fn solve_any_candidate_block(
        &mut self,
        block_name: &str,
        expected_blocks: usize,
        candidate_blocks: &[Block],
    ) -> Result<Block, String> {
        let t = std::time::Instant::now();

        println!(
            "Solving {block_name} block (exploring {} options) ...",
            candidate_blocks.len(),
        );

        let setup_moves = self.all_moves();

        let (new_solved_block, new_state, new_solution, new_segments) = candidate_blocks
            .into_par_iter()
            .filter_map(|&new_block| {
                let mut solution = self.solution.clone();
                let mut state = self
                    .state
                    .add_block_with_setup_moves(&self.puzzle, &setup_moves, new_block)
                    .unwrap();
                match dfs::blockbuild_via_repeated_iddfs(
                    &self.puzzle,
                    &mut state,
                    &mut solution,
                    self.params,
                    expected_blocks,
                ) {
                    Some(new_segments) => Some((new_block, state, solution, new_segments)),
                    None => {
                        if self.params.verbosity >= 2 {
                            println!("  Abandoning {block_name} block attempt");
                        }
                        None
                    }
                }
            })
            .min_by_key(|(_, _, solution, _)| solution.len())
            .ok_or_else(|| format!("no solution to {block_name} block"))?;

        let additional_twist_count: usize = new_segments.iter().map(|segment| segment.len()).sum();

        // Update state solution
        self.state = new_state;
        self.solution = new_solution;

        if self.params.verbosity >= 1 {
            println!("Search completed in {:?}", t.elapsed());
            println!(
                "Best solution solves {new_solved_block} in {additional_twist_count} ETM ({} ETM total)",
                self.solution.len(),
            );
            println!("{}", self.solution_str());
            println!();
        }

        Ok(new_solved_block)
    }
}
