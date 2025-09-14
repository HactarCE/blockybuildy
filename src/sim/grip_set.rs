use std::fmt;

use super::elements::{ElemId, IDENT};
use super::grips::{GripId, HYPERCUBE_GRIPS};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GripSet(pub u8);
impl GripSet {
    pub const CORE: GripSet = GripSet(0);

    pub fn contains(self, grip: GripId) -> bool {
        self.0 & (1 << grip.0) != 0
    }
    pub fn iter(self) -> impl Iterator<Item = GripId> {
        HYPERCUBE_GRIPS
            .into_iter()
            .filter(move |&g| self.contains(g))
    }
    pub fn is_core(self) -> bool {
        self == GripSet::CORE
    }
}
impl FromIterator<GripId> for GripSet {
    fn from_iter<T: IntoIterator<Item = GripId>>(iter: T) -> Self {
        Self(iter.into_iter().map(|g| 1 << g.0).sum())
    }
}
impl fmt::Display for GripSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for g in self.iter() {
            write!(f, "{g}")?;
        }
        if self.is_core() {
            write!(f, "∅")?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    /// Active grip set (changes as the piece moves around)
    pub grips: GripSet,
    pub attitude: ElemId,
}
impl Piece {
    pub fn new_solved(grips: impl IntoIterator<Item = GripId>) -> Self {
        Self {
            grips: GripSet::from_iter(grips),
            attitude: IDENT,
        }
    }
}
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grips)
    }
}
