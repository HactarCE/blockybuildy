use std::fmt;

use super::elements::ElemId;
use super::grips::{GripId, HYPERCUBE_GRIPS};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Twist {
    pub grip: GripId,
    pub transform: ElemId,
}
impl fmt::Display for Twist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(g1) = HYPERCUBE_GRIPS
            .into_iter()
            .find(|&g| self.transform * g != g)
        else {
            return write!(f, "{}[.]", self.grip);
        };
        let g2 = self.transform * g1;
        let g3 = self.transform * g2;
        write!(f, "{}[{g1} -> {g2} -> {g3}]", self.grip)
    }
}
