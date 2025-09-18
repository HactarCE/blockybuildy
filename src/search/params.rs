use super::Heuristic;

/// Search parameters for a [`BlockBuildingSearch`];
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BlockBuildingSearchParams {
    /// Heuristic to use for pruning branches.
    pub heuristic: Heuristic,

    /// Maximum depth for IDDFS.
    pub max_depth: usize,

    /// How much to print.
    pub verbosity: u8,
}

impl Default for BlockBuildingSearchParams {
    fn default() -> Self {
        Self {
            heuristic: Heuristic::Fast,
            max_depth: 3,
            verbosity: 1,
        }
    }
}
