use super::Heuristic;

/// Search parameters for a [`BlockBuildingSearch`];
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BlockBuildingSearchParams {
    /// Heuristic to use for pruning branches.
    pub heuristic: Heuristic,

    /// Maximum depth for IDDFS.
    pub max_depth: usize,

    /// Depth into the DFS to parallelize.
    pub parallel_depth: usize,
    /// Setting this option to `true` will enforce determinism in parallel
    /// searches.
    ///
    /// When `parallel_depth` is a nonzero value, this controls whether the
    /// search will be terminated as soon as any of the parallel searches
    /// succeeds (faster, nondeterministic) or as soon as the sequentially-first
    /// parallel search succeeds (slower, deterministic).
    pub force_parallel_determinism: bool,

    /// Number of moves to trim off the end of the solution in case they cancel
    /// nicely with the next phase.
    pub trim: usize,

    /// How much to print.
    pub verbosity: u8,
}

impl Default for BlockBuildingSearchParams {
    fn default() -> Self {
        Self {
            heuristic: Heuristic::Fast,
            max_depth: 3,
            parallel_depth: 2,
            force_parallel_determinism: true,
            trim: 0,
            verbosity: 1,
        }
    }
}
