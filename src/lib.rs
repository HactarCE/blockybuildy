#[macro_use]
mod macros;
pub mod mc4d;
pub mod search;
pub mod sim;
pub mod stackvec;

pub use sim::*;
pub use stackvec::StackVec;

pub use crate::search::*;

/// Maximum number of blocks representable in a puzzle.
pub const MAX_BLOCKS: usize = 21; // 63 bytes (+1 for length)

/// If we get fewer than this many solutions, try searching deeper to generate more solutions.
const MIN_SOLUTION_COUNT: usize = 200;
/// If we get more than this many solutions, truncate to this many.
const MAX_SOLUTION_COUNT: usize = 20_000;

/// Number of moves allowed in a single segment.
const MAX_SOLUTION_SEGMENT_LEN: usize = 11;

/// Whether to print 3D twist names when possible (e.g., `R` instead of `RO`).
pub const USE_3D_TWIST_NAMES: bool = false;
