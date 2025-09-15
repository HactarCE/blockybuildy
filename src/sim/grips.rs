use std::fmt;

use super::elements::*;
use super::group;
use super::space::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GripId(pub u8);

pub const R: GripId = GripId(0);
pub const L: GripId = GripId(1);
pub const U: GripId = GripId(2);
pub const D: GripId = GripId(3);
pub const F: GripId = GripId(4);
pub const B: GripId = GripId(5);
pub const O: GripId = GripId(6);
pub const I: GripId = GripId(7);

pub const HYPERCUBE_GRIPS: [GripId; 8] = [R, L, U, D, F, B, O, I];
pub const CUBE_GRIPS: [GripId; 6] = [R, L, U, D, F, B];

impl fmt::Display for GripId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.char().to_ascii_lowercase())
        } else {
            write!(f, "{}", self.char())
        }
    }
}

impl GripId {
    pub const fn axis(self) -> usize {
        self.0 as usize >> 1
    }
    pub const fn signum(self) -> i8 {
        if self.0 & 1 == 0 { 1 } else { -1 }
    }

    pub const fn opposite(self) -> Self {
        Self(self.0 ^ 1)
    }

    pub const fn char(self) -> char {
        b"RLUDFBOI"[self.0 as usize] as char
    }

    pub fn vec(self) -> Vec4 {
        let mut ret = ZERO;
        ret[self.axis()] = self.signum();
        ret
    }

    pub fn transforms(self, subgroup: &[ElemId]) -> Vec<ElemId> {
        group::stabilizer(subgroup.iter().copied(), self.vec())
            .filter(|&e| e != IDENT)
            .collect()
    }
}
