use super::Heuristic;

/// Search parameters for a [`BlockBuildingSearch`];
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BlockBuildingSearchParams {
    /// Heuristic to use for pruning branches.
    pub heuristic: Heuristic,

    /// Maximum depth for IDDFS.
    pub max_depth: usize,
    /// Increase the IDDFS depth if a block cannot be formed in `max_depth`
    /// moves.
    ///
    /// When this flag is false, the search is aborted if a block cannot be
    /// formed in `max_depth` moves.
    ///
    /// This is recommended when there are very few candidate blocks. If there
    /// are many candidate blocks, then settings this to true will cause the
    /// search to wait until every candidate has a solution.
    pub deepen_when_stuck: bool,

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
            deepen_when_stuck: false,
            trim: 0,
            verbosity: 1,
        }
    }
}
