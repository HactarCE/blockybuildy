pub mod mc4d;
pub mod search;
pub mod sim;
pub mod stackvec;

pub use sim::*;
pub use stackvec::StackVec;

pub use crate::search::*;

pub const MAX_BLOCKS: usize = 21; // 63 bytes (+1 for length)

pub const USE_3D_TWIST_NAMES: bool = false;
