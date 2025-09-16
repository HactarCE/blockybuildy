use std::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Mul};

use super::elements::ElemId;
use super::grip_set::GripSet;
use super::grips::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GripStatus {
    Active,
    Blocked,
    Inactive,
}

/// Layer mask of a block, represented using 12 bits: `0www_0zzz_0yyy_0xxx`
/// (MSb..LSb)
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PackedLayers([u8; 2]);

impl PackedLayers {
    pub const ALL: Self = Self::from_u16(0b_0111_0111_0111_0111);
    pub const EMPTY: Self = Self::from_u16(0);

    const fn to_u16(self) -> u16 {
        u16::from_ne_bytes(self.0)
    }
    const fn from_u16(i: u16) -> Self {
        Self(i.to_ne_bytes())
    }

    #[must_use]
    pub const fn restrict_to_active_grip(self, g: GripId) -> Self {
        Self::from_u16(self.to_u16() & !inactive_grip_mask(g))
    }
    #[must_use]
    pub const fn restrict_to_inactive_grip(self, g: GripId) -> Self {
        Self::from_u16(self.to_u16() & !active_grip_mask(g))
    }
    #[must_use]
    pub fn expand_to_active_grip(self, g: GripId) -> Self {
        Self::from_u16(
            self.to_u16()
                | if self.grip_status(g.opposite()) == Some(GripStatus::Active) {
                    inactive_grip_mask(g.opposite())
                } else {
                    active_grip_mask(g)
                },
        )
    }

    pub fn is_empty_on_any_axis(self) -> bool {
        let bits = self.to_u16();
        (bits | (bits >> 1) | (bits >> 2)) & 0b_0001_0001_0001_0001 != 0b_0001_0001_0001_0001
    }

    const fn bits_for_axis(self, axis: usize) -> u8 {
        assert!(axis < 4, "axis out of range");
        ((self.to_u16() >> (axis * 4)) & 0xF) as u8
    }
    const fn is_empty_on_axis(self, axis: usize) -> bool {
        self.bits_for_axis(axis) == 0
    }

    const fn bits_for_grip(self, g: GripId) -> u8 {
        let bits = self.bits_for_axis(g.axis());
        if g.id() & 1 == 0 { bits } else { rev3(bits) }
    }

    /// Returns `[inside, outside]`
    #[must_use]
    pub fn split(self, g: GripId) -> [Option<Self>; 2] {
        debug_assert!(g.id() < 8);
        [
            self.restrict_to_active_grip(g),
            self.restrict_to_inactive_grip(g),
        ]
        .map(|l| (!l.is_empty_on_axis(g.axis())).then_some(l))
    }

    pub fn is_subset_of(self, other: Self) -> bool {
        self.to_u16() & !other.to_u16() == 0
    }

    #[must_use]
    pub fn try_merge_with(self, other: Self) -> Option<(Self, GripId)> {
        if self == other {
            return None; // same blocks
        }

        let lhs = self.to_u16();
        let rhs = other.to_u16();
        let layer_difference = lhs ^ rhs;
        let merge_grip = GripId::try_new(
            (layer_difference & 0b_0101_0101_0101_0101).trailing_zeros() as u8 / 2,
        )?;
        let merge_axis = layer_difference.trailing_zeros() as usize / 4;

        let mut lhs_bits = lhs >> (merge_axis * 4);
        let mut rhs_bits = rhs >> (merge_axis * 4);
        if lhs_bits ^ rhs_bits > 0b111 {
            return None; // multiple axes have different layers
        }
        lhs_bits &= 0b111;
        rhs_bits &= 0b111;

        if lhs_bits & rhs_bits != 0 {
            return None; // `self` and `other` overlap
        }

        if lhs_bits | rhs_bits == 0b101 {
            return None; // disconnected blocks not allowed (but they could be, if we had slice moves)
        }

        Some((self | other, merge_grip))
    }

    /// Returns the positive grips on the separating axes of the blocks.
    pub fn separating_axes(self, other: Self) -> GripSet {
        let diff = self ^ other;
        let mut ret = GripSet::NONE;
        for g in [R, U, F, O] {
            if diff.bits_for_grip(g) & 0b111 != 0 {
                ret += g;
            }
        }
        ret
    }

    /// Returns the status of a grip, or `None` if the layer mask is invalid.
    pub const fn grip_status(self, g: GripId) -> Option<GripStatus> {
        match self.bits_for_grip(g) {
            0b001 => Some(GripStatus::Active),
            0b011 | 0b111 => Some(GripStatus::Blocked),
            0b110 | 0b100 | 0b010 => Some(GripStatus::Inactive),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn unpack(self) -> [u8; 4] {
        let bits = self.to_u16();
        [0, 4, 8, 12].map(|i| (bits >> i) as u8 & 0b111)
    }
}

const fn active_grip_mask(g: GripId) -> u16 {
    g.hint_assert_in_bounds();
    0b01 << (g.id() as usize * 2)
}
const fn inactive_grip_mask(g: GripId) -> u16 {
    g.hint_assert_in_bounds();
    (0b0111 << (g.axis() * 4)) ^ active_grip_mask(g)
}

impl fmt::Debug for PackedLayers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:03b}", self.bits_for_axis(3))?;
        write!(f, "_{:03b}", self.bits_for_axis(2))?;
        write!(f, "_{:03b}", self.bits_for_axis(1))?;
        write!(f, "_{:03b}", self.bits_for_axis(0))?;
        assert_eq!(self.to_u16() & !Self::ALL.to_u16(), 0);
        Ok(())
    }
}
impl fmt::Display for PackedLayers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut active = GripSet::NONE;
        let mut blocked = GripSet::NONE;
        let mut inactive = GripSet::NONE;
        for g in HYPERCUBE_GRIPS {
            match self.grip_status(g) {
                Some(GripStatus::Active) => active += g,
                Some(GripStatus::Blocked) => blocked += g,
                Some(GripStatus::Inactive) => inactive += g,
                None => return write!(f, "bad_mask_{:#b}", self.to_u16()),
            }
        }
        write!(f, "{active}${blocked:#}!{inactive}")
    }
}

impl BitOr for PackedLayers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_u16(self.to_u16() | rhs.to_u16())
    }
}

impl BitAnd for PackedLayers {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_u16(self.to_u16() & rhs.to_u16())
    }
}

impl BitXor for PackedLayers {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_u16(self.to_u16() ^ rhs.to_u16())
    }
}

impl Mul<PackedLayers> for ElemId {
    type Output = PackedLayers;

    fn mul(self, rhs: PackedLayers) -> Self::Output {
        let mut resulting_bits = 0_u16;
        for axis in 0..4 {
            let old_grip = GripId::new(axis << 1);
            let new_grip = self * old_grip;
            let old_axis_bits = rhs.bits_for_axis(axis as usize);
            let new_axis_bits = if new_grip.id() & 1 == 0 {
                old_axis_bits
            } else {
                rev3(old_axis_bits)
            };
            let new_axis = new_grip.axis();
            resulting_bits |= (new_axis_bits as u16) << (new_axis * 4);
        }
        PackedLayers::from_u16(resulting_bits)
    }
}

/// Reverses the lowest 3 bits of a byte.
const fn rev3(x: u8) -> u8 {
    x.reverse_bits() >> 5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HYPERCUBE_ROTATIONS, IDENT};

    #[test]
    fn test_transform_layer_mask() {
        let layers = PackedLayers::from_u16(0b_0111_0100_0010_0110);
        assert_eq!(IDENT * layers, layers);
        for &t1 in &*HYPERCUBE_ROTATIONS {
            for &t2 in &*HYPERCUBE_ROTATIONS {
                assert_eq!(t1 * (t2 * layers), (t1 * t2) * layers);
            }
        }
    }
}
