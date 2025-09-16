pub mod search;
pub mod sim;
pub mod stackvec;

pub use sim::*;
pub use stackvec::StackVec;

pub use crate::search::*;

// pub const MAX_BLOCKS: usize = 10;
pub const MAX_BLOCKS: usize = 31;
// bigger number is sometimes better???

pub const USE_3D_TWIST_NAMES: bool = false;
