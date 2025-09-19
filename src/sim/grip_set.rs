use std::fmt;
use std::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, Mul, Not, Sub, SubAssign};

use super::elements::{ElemId, IDENT};
use super::grips::*;
use super::layer_mask::PackedLayers;
use crate::{GripStatus, StackVec, Twist};

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GripSet(pub u8);
impl GripSet {
    pub const NONE: GripSet = GripSet(0);
    pub const ALL: GripSet = GripSet(0xFF);

    pub fn len(self) -> u32 {
        self.0.count_ones()
    }

    pub fn contains(self, grip: GripId) -> bool {
        grip.hint_assert_in_bounds();
        self.0 & (1 << grip.id()) != 0
    }
    pub fn iter(self) -> impl Iterator<Item = GripId> {
        HYPERCUBE_GRIPS
            .into_iter()
            .filter(move |&g| self.contains(g))
    }
    pub fn is_empty(self) -> bool {
        self == GripSet::NONE
    }

    pub fn exactly_one(self) -> Option<GripId> {
        (self.len() == 1).then(|| self.unwrap_exactly_one())
    }
    pub fn exactly_two(self) -> Option<[GripId; 2]> {
        (self.len() == 2).then(|| self.unwrap_exactly_two())
    }

    #[track_caller]
    pub fn unwrap_exactly_one(self) -> GripId {
        assert_eq!(1, self.len(), "expected one grip");
        GripId::new(self.0.trailing_zeros() as u8)
    }
    #[track_caller]
    pub fn unwrap_exactly_two(self) -> [GripId; 2] {
        assert_eq!(2, self.len(), "expected two grips");
        let first = self.0.trailing_zeros();
        let second = (self.0 & (GripSet::ALL.0 << (first + 1))).trailing_zeros();
        [first, second].map(|id| GripId::new(id as u8))
    }
}
impl FromIterator<GripId> for GripSet {
    fn from_iter<T: IntoIterator<Item = GripId>>(iter: T) -> Self {
        Self(iter.into_iter().map(|g| 1 << g.id()).sum())
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
        rhs.hint_assert_in_bounds();
        self.0 |= 1 << rhs.id();
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
        rhs.hint_assert_in_bounds();
        self.0 &= !(1 << rhs.id());
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
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
            layers = layers.restrict_to_active_grip(g)?;
        }
        for g in inactive_grips {
            layers = layers.restrict_to_inactive_grip(g)?;
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

    pub fn grip_status(self, grip: GripId) -> GripStatus {
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
    pub fn is_fully_blocked_on_axis(self, axis: usize) -> bool {
        self.layers.is_fully_blocked_on_axis(axis)
    }
    fn grips_with_status(self, status: GripStatus) -> GripSet {
        HYPERCUBE_GRIPS
            .into_iter()
            .filter(|&g| self.layers.grip_status(g) == status)
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
            layers: self.layers.restrict_to_active_grip(grip)?,
            attitude: self.attitude,
        })
    }
    #[must_use]
    pub fn restrict_to_inactive_grip(self, grip: GripId) -> Option<Self> {
        Some(Self {
            layers: self.layers.restrict_to_inactive_grip(grip)?,
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
            self.layers.grip_status(g) != forbidden_status
        })
    }

    /// Returns a list of indistinguishable attitudes for the block in its
    /// current position, or `None` if the attitude is completely
    /// distinguishable.
    ///
    /// - In 3D, only centers have indistinguishable attitudes.
    /// - In 4D, only centers and ridges has indistinguishable attitudes.
    ///
    /// The core's attitude is always completely distinguishable.
    pub fn indistinguishable_attitudes(&self, ndim: usize) -> impl Iterator<Item = ElemId> {
        self.layers
            .indistinguishable_subgroup()
            .filter(|_| !(ndim == 3 && self.layers == PackedLayers::CORE_3D)) // 3D core is distinguishable
            .unwrap_or(&[IDENT])
            .iter()
            .map(|&rot| rot * self.attitude)
    }
    /// Returns the set of grips constructed by taking all indistinguishable
    /// attitudes for the block in its current position and mutliplying each one
    /// by `g`. Duplicates are not included.
    pub fn mul_indistinguishable_attides_by_grip(
        self,
        ndim: usize,
        g: GripId,
    ) -> StackVec<GripId, 6> {
        let g = self.attitude * g;
        let only_g = || StackVec::from_iter([g]).unwrap();

        // Core is always distinguishable
        if (ndim == 3 && self.layers == PackedLayers::CORE_3D)
            || self.layers == PackedLayers::CORE_4D
        {
            return only_g();
        }

        if self.layers.is_axis_blocked(g.axis()) {
            return only_g();
        }

        let blocked_axes_mask = self.layers.blocked_axes_mask();
        if blocked_axes_mask.count_ones() > 2 {
            return only_g();
        }

        StackVec::from_iter(
            (0..4)
                .filter(|i| (blocked_axes_mask >> i) & 1 != 0)
                .flat_map(GripId::pair_on_axis),
        )
        .unwrap()
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

    pub fn try_merge(self, other: Self, ndim: usize) -> Option<Self> {
        let [head, body] = if self.layers.active_grip_count() > other.layers.active_grip_count() {
            [self, other]
        } else {
            [other, self]
        };

        if !body
            .indistinguishable_attitudes(ndim)
            .any(|a| a == head.attitude)
        {
            return None;
        }

        Some(Self {
            layers: self.layers.try_merge_with(other.layers)?.0,
            attitude: head.attitude,
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
    use itertools::Itertools;
    use proptest::prelude::*;

    use super::*;

    impl Arbitrary for Block {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            (std::array::from_fn::<_, 8, _>(|_| 0..3), any::<ElemId>())
                .prop_filter_map("invalid block", |(grip_statuses, attitude)| {
                    let mut active_grips = GripSet::NONE;
                    let mut inactive_grips = GripSet::NONE;
                    for (i, status) in grip_statuses.into_iter().enumerate() {
                        let g = GripId::new(i as u8);
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
        let ndim = 4;

        assert!(!b1.layers.is_empty_on_any_axis());
        assert!(!b2.layers.is_empty_on_any_axis());

        let mut b1_attitudes = b1.indistinguishable_attitudes(ndim).collect_vec();
        let b2_attitudes = b2.indistinguishable_attitudes(ndim).collect_vec();
        if !b1_attitudes.iter().any(|a| b2_attitudes.contains(a)) {
            assert_eq!(None, b1.try_merge(b2, ndim));
            assert_eq!(None, b2.try_merge(b1, ndim)); // commutativity
            b1.attitude = b2.attitude;
            b1_attitudes = b1.indistinguishable_attitudes(ndim).collect();
        }

        let layers1 = b1.layers.unpack();
        let layers2 = b2.layers.unpack();

        let mut num_same_axes = 0;
        let mut num_mergeable_axes = 0;
        for (l1, l2) in std::iter::zip(layers1, layers2) {
            let same = l1 == l2;
            num_same_axes += same as usize;
            let disjoint = (l1 & l2 == 0) && (l1 | l2 != 0b101);
            num_mergeable_axes += disjoint as usize;
        }

        let actual_merged = b1.try_merge(b2, ndim);
        assert_eq!(actual_merged, b2.try_merge(b1, ndim)); // commutativity

        assert_eq!(
            num_same_axes == 3 && num_mergeable_axes == 1,
            actual_merged.is_some(),
        );

        if let Some(m) = actual_merged {
            assert_eq!(b1.layers | b2.layers, m.layers);
            assert!(b1_attitudes.contains(&m.attitude));
            assert!(b2_attitudes.contains(&m.attitude));
        }
    }

    #[test]
    fn test_grip_set_unwrap() {
        for g1 in HYPERCUBE_GRIPS {
            let one = GripSet::from_iter([g1]);
            assert_eq!(one.exactly_one(), Some(g1));
            assert_eq!(one.exactly_two(), None);
            for g2 in HYPERCUBE_GRIPS {
                if g1 < g2 {
                    let two = GripSet::from_iter([g1, g2]);
                    assert_eq!(two.exactly_one(), None);
                    assert_eq!(two.exactly_two(), Some([g1, g2]));
                }
            }
        }
    }

    #[test]
    fn test_block_indistinguishable_attitudes() {
        fn count(b: Block, ndim: usize) -> usize {
            b.indistinguishable_attitudes(ndim).count()
        }

        // 4D
        for ((ndim, grips, rotations), (center_orientations, ridge_orientations)) in [
            (
                (3, CUBE_GRIPS.as_slice(), crate::CUBE_ROTATIONS.as_slice()),
                (4, 1),
            ),
            ((4, &HYPERCUBE_GRIPS, &*crate::HYPERCUBE_ROTATIONS), (24, 4)),
        ] {
            for &elem in rotations {
                let core = Block::new([], grips.iter().copied(), elem).unwrap();
                assert_eq!(1, count(core, ndim)); // core

                for &g1 in grips {
                    let b1 = core.expand_to_active_grip(g1);
                    let cases = [
                        b1,
                        b1.restrict_to_active_grip(g1).unwrap(),
                        b1.expand_to_active_grip(g1.opposite()),
                    ];
                    for case in cases {
                        assert_eq!(center_orientations, count(case, ndim));
                    }
                    for &g2 in grips.into_iter().filter(|&g2| g2.axis() != g1.axis()) {
                        let blocked = cases.map(|b| b.expand_to_active_grip(g2));
                        let active = blocked.map(|b| b.restrict_to_active_grip(g2).unwrap());
                        let double_blocked =
                            blocked.map(|b| b.expand_to_active_grip(g2.opposite()));
                        let cases = [blocked, active, double_blocked];
                        for &case in cases.as_flattened() {
                            assert_eq!(ridge_orientations, count(case, ndim));
                        }
                        for &g3 in grips
                            .into_iter()
                            .filter(|&g3| g3.axis() != g1.axis() && g3.axis() != g2.axis())
                        {
                            for b in cases.as_flattened() {
                                let blocked = b.expand_to_active_grip(g3);
                                let active = blocked.restrict_to_active_grip(g3).unwrap();
                                let double_blocked = blocked.expand_to_active_grip(g3.opposite());
                                for case in [blocked, active, double_blocked] {
                                    assert_eq!(1, count(case, ndim));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
