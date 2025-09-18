use std::fmt;

use super::elements::*;
use super::group;
use super::space::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GripId(u8);

pub const R: GripId = GripId::new(0);
pub const L: GripId = GripId::new(1);
pub const U: GripId = GripId::new(2);
pub const D: GripId = GripId::new(3);
pub const F: GripId = GripId::new(4);
pub const B: GripId = GripId::new(5);
pub const O: GripId = GripId::new(6);
pub const I: GripId = GripId::new(7);

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
    pub const fn id(self) -> u8 {
        self.0
    }
    pub const fn axis(self) -> usize {
        self.0 as usize >> 1
    }
    pub const fn signum(self) -> i8 {
        if self.0 & 1 == 0 { 1 } else { -1 }
    }

    pub const fn pair_on_axis(axis: usize) -> [Self; 2] {
        assert!(axis < 4, "axis out of range");
        let g1 = Self::new((axis as u8) << 1);
        [g1, g1.opposite()]
    }

    /// Constructs a grip from an ID.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of range (must be strictly less 8).
    pub const fn new(id: u8) -> Self {
        Self::try_new(id).expect("grip ID out of range")
    }
    /// Constructs a grip from an ID, or returns `None` if `id` is out of range
    /// (must be strictly less than 8).
    pub const fn try_new(id: u8) -> Option<Self> {
        if id < 8 { Some(Self(id)) } else { None }
    }
    /// Hint to the compiler that the grip ID is within bounds.
    #[inline]
    pub const fn hint_assert_in_bounds(self) -> Self {
        // SAFETY: `GripId` is only ever constructed using `GripId::try_new()`,
        // which returns `None` if the ID is greater than or equal to 8.
        unsafe { std::hint::assert_unchecked(self.0 < 8) };
        self
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

    pub fn can_transform_grip_to_grip(self, start: GripId, end: GripId) -> bool {
        self.axis() != start.axis() && self.axis() != end.axis()
    }
}
