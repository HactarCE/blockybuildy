use std::fmt;
use std::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, Mul, Not, Sub, SubAssign};

use itertools::Itertools;

use super::elements::{ElemId, IDENT};
use super::grips::*;
use super::layer_mask::PackedLayers;
use crate::{GripStatus, Twist};

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GripSet(pub u8);
impl GripSet {
    pub const NONE: GripSet = GripSet(0);
    pub const ALL: GripSet = GripSet(0xFF);

    pub fn contains(self, grip: GripId) -> bool {
        self.0 & (1 << grip.0) != 0
    }
    pub fn iter(self) -> impl Iterator<Item = GripId> {
        HYPERCUBE_GRIPS
            .into_iter()
            .filter(move |&g| self.contains(g))
    }
    pub fn is_empty(self) -> bool {
        self == GripSet::NONE
    }

    #[track_caller]
    pub fn unwrap_exactly_one(self) -> GripId {
        self.iter().exactly_one().ok().unwrap()
    }
}
impl FromIterator<GripId> for GripSet {
    fn from_iter<T: IntoIterator<Item = GripId>>(iter: T) -> Self {
        Self(iter.into_iter().map(|g| 1 << g.0).sum())
    }
}
impl fmt::Debug for GripSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
impl fmt::Display for GripSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for g in self.iter() {
            g.fmt(f)?;
        }
        if self.is_empty() {
            write!(f, "∅")?;
        }
        Ok(())
    }
}
impl Add<GripId> for GripSet {
    type Output = Self;

    fn add(mut self, rhs: GripId) -> Self::Output {
        self += rhs;
        self
    }
}
impl AddAssign<GripId> for GripSet {
    fn add_assign(&mut self, rhs: GripId) {
        self.0 |= 1 << rhs.0;
    }
}
impl Sub<GripId> for GripSet {
    type Output = Self;

    fn sub(mut self, rhs: GripId) -> Self::Output {
        self -= rhs;
        self
    }
}
impl SubAssign<GripId> for GripSet {
    fn sub_assign(&mut self, rhs: GripId) {
        self.0 &= !(1 << rhs.0);
    }
}
impl BitOr for GripSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl BitAnd for GripSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl BitXor for GripSet {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}
impl Not for GripSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
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

/// Block of pieces with compatible attitudes, displayed as
/// `ACTIVE_GRIPS$blocked_grips!INACTIVE_GRIPS`.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    layers: PackedLayers,
    attitude: ElemId,
}
// same as `#[derive(Default)]` but easier for the compiler to inline
impl Default for Block {
    #[inline]
    fn default() -> Self {
        Self {
            layers: PackedLayers::EMPTY,
            attitude: IDENT,
        }
    }
}
impl From<Piece> for Block {
    fn from(value: Piece) -> Self {
        Self::new(value.grips.iter(), (!value.grips).iter(), value.attitude).unwrap()
    }
}
impl Block {
    pub fn new_solved(
        active_grips: impl IntoIterator<Item = GripId>,
        inactive_grips: impl IntoIterator<Item = GripId>,
    ) -> Option<Self> {
        Self::new(active_grips, inactive_grips, IDENT)
    }
    pub fn new(
        active_grips: impl IntoIterator<Item = GripId>,
        inactive_grips: impl IntoIterator<Item = GripId>,
        attitude: ElemId,
    ) -> Option<Self> {
        let mut layers = PackedLayers::ALL;
        for g in active_grips {
            layers = layers.restrict_to_active_grip(g);
        }
        for g in inactive_grips {
            layers = layers.restrict_to_inactive_grip(g);
        }
        if layers.is_empty_on_any_axis() {
            return None; // block is empty!
        }
        Some(Self { layers, attitude })
    }

    pub fn layers(self) -> PackedLayers {
        self.layers
    }
    pub fn attitude(self) -> ElemId {
        self.attitude
    }

    pub fn at_solved(self) -> Self {
        Block {
            layers: self.attitude.inv() * self.layers,
            attitude: IDENT,
        }
    }

    pub fn is_subset_of(self, other: Self) -> bool {
        self.attitude == other.attitude && self.layers.is_subset_of(other.layers)
    }

    pub fn grip_status(self, grip: GripId) -> Option<GripStatus> {
        self.layers.grip_status(grip)
    }

    pub fn active_grips(self) -> GripSet {
        self.grips_with_status(GripStatus::Active)
    }
    pub fn inactive_grips(self) -> GripSet {
        self.grips_with_status(GripStatus::Inactive)
    }
    pub fn blocked_grips(self) -> GripSet {
        self.grips_with_status(GripStatus::Blocked)
    }
    fn grips_with_status(self, status: GripStatus) -> GripSet {
        HYPERCUBE_GRIPS
            .into_iter()
            .filter(|&g| self.layers.grip_status(g) == Some(status))
            .collect()
    }

    #[must_use]
    pub fn expand_to_active_grip(self, grip: GripId) -> Self {
        Self {
            layers: self.layers.expand_to_active_grip(grip),
            attitude: self.attitude,
        }
    }
    #[must_use]
    pub fn restrict_to_active_grip(self, grip: GripId) -> Option<Self> {
        Some(Self {
            layers: self.layers.restrict_to_active_grip(grip),
            attitude: self.attitude,
        })
    }
    #[must_use]
    pub fn restrict_to_inactive_grip(self, grip: GripId) -> Option<Self> {
        Some(Self {
            layers: self.layers.restrict_to_inactive_grip(grip),
            attitude: self.attitude,
        })
    }

    pub fn contains_piece(self, piece: Piece) -> bool {
        HYPERCUBE_GRIPS.into_iter().all(|g| {
            let forbidden_status = if piece.grips.contains(g) {
                GripStatus::Inactive
            } else {
                GripStatus::Active
            };
            self.layers.grip_status(g) != Some(forbidden_status)
        })
    }

    //// Returns `[inside, outside]`
    pub fn split(self, grip: GripId) -> [Option<Self>; 2] {
        self.layers.split(grip).map(|layers| {
            Some(Block {
                layers: layers?,
                attitude: self.attitude,
            })
        })
    }

    pub fn try_merge(self, other: Self) -> Option<Self> {
        if self.attitude != other.attitude {
            return None;
        }

        Some(Self {
            layers: self.layers.try_merge_with(other.layers)?.0,
            attitude: self.attitude,
        })
    }
}
impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layers, attitude } = self;
        write!(f, "{layers:?}@[{attitude:?}]")
    }
}
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layers, attitude } = self;
        write!(f, "{layers}@[{attitude}]")
    }
}

impl Mul<Block> for Twist {
    type Output = [Option<Block>; 2];

    fn mul(self, rhs: Block) -> Self::Output {
        let [inside, outside] = rhs.split(self.grip);
        [
            inside.map(|b| Block {
                layers: self.transform * b.layers,
                attitude: self.transform * b.attitude,
            }),
            outside,
        ]
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    impl Arbitrary for Block {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            (
                std::array::from_fn::<_, 8, _>(|_| 0..3),
                (0..crate::sim::group::ELEM_COUNT as u8).prop_map(ElemId),
            )
                .prop_filter_map("invalid block", |(grip_statuses, attitude)| {
                    let mut active_grips = GripSet::NONE;
                    let mut inactive_grips = GripSet::NONE;
                    for (i, status) in grip_statuses.into_iter().enumerate() {
                        let g = GripId(i as u8);
                        if status & 1 != 0 {
                            active_grips += g;
                        }
                        if status & 2 != 0 {
                            inactive_grips += g;
                        }
                    }
                    Block::new(active_grips.iter(), inactive_grips.iter(), attitude)
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }

    proptest! {
        #[test]
        fn proptest_merge_blocks(b1: Block, b2: Block) {
            test_merge_blocks(b1, b2);
        }
    }

    fn test_merge_blocks(mut b1: Block, b2: Block) {
        assert!(!b1.layers.is_empty_on_any_axis());
        assert!(!b2.layers.is_empty_on_any_axis());

        if b1.attitude != b2.attitude {
            assert_eq!(None, b1.try_merge(b2));
            assert_eq!(None, b2.try_merge(b1)); // commutativity
            b1.attitude = b2.attitude;
        }

        let layers1 = b1.layers.unpack();
        let layers2 = b2.layers.unpack();

        let possible_merged_block = Block {
            layers: b1.layers | b2.layers,
            attitude: b1.attitude,
        };

        let mut num_same_axes = 0;
        let mut num_mergeable_axes = 0;
        for (l1, l2) in std::iter::zip(layers1, layers2) {
            let same = l1 == l2;
            num_same_axes += same as usize;
            let disjoint = (l1 & l2 == 0) && (l1 | l2 != 0b101);
            num_mergeable_axes += disjoint as usize;
        }

        let actual_merged = b1.try_merge(b2);
        assert_eq!(actual_merged, b2.try_merge(b1)); // commutativity

        assert_eq!(
            num_same_axes == 3 && num_mergeable_axes == 1,
            actual_merged.is_some(),
        );

        if let Some(m) = actual_merged {
            assert_eq!(m, possible_merged_block);
        }
    }
}
