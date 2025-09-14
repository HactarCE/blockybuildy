use itertools::Itertools;

use super::group;
use super::space::*;

/// Elements in the group of rotations of a hypercube.
#[static_init::dynamic]
pub static HYPERCUBE_ROTATIONS: [ElemId; 192] = (0..group::ELEM_COUNT)
    .map(|i| ElemId(i as u8))
    .collect_array()
    .unwrap();

/// Elements in the group of rotations of a cube (as a subgroup of BC4).
#[static_init::dynamic]
pub static CUBE_ROTATIONS: [ElemId; 24] = group::stabilizer(HYPERCUBE_ROTATIONS.iter().copied(), W)
    .collect_array()
    .unwrap();

/// Element from the grip group.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ElemId(pub u8);

/// Identity group element.
pub const IDENT: ElemId = ElemId(0);
/// Rotation from +X to +Y.
pub const XY: ElemId = ElemId(1);
/// Rotation from +X to +Z.
pub const XZ: ElemId = ElemId(2);
/// Rotation from +X to +W.
pub const XW: ElemId = ElemId(3);

/// Rotation from +Y to +X.
pub const YX: ElemId = ElemId(13); // XY * XY * XY
/// Rotation from +Z to +X.
pub const ZX: ElemId = ElemId(25); // XZ * XZ * XZ
/// Rotation from +W to +X.
pub const WX: ElemId = ElemId(36); // XW * XW * XW

/// Rotation from +Y to +Z.
pub const YZ: ElemId = ElemId(97); // XZ * YX * ZX
/// Rotation from +Y to +W.
pub const YW: ElemId = ElemId(110); // XW * YX * WX
/// Rotation from +Z to +Y.
pub const ZY: ElemId = ElemId(84); // XY * ZX * YX
/// Rotation from +W to +Y.
pub const WY: ElemId = ElemId(86); // XY * WX * YX

/// Rotation from +Z to +W.
pub const ZW: ElemId = ElemId(133); // XW * ZX * WX
/// Rotation from +W to +Z.
pub const WZ: ElemId = ElemId(128); // XZ * WX * ZX

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rotations() {
        for (elem, init, expected) in [
            (IDENT, [X, Y, Z, W], [X, Y, Z, W]),
            (XY, [X, Y, Z, W], [Y, -X, Z, W]),
            (XZ, [X, Y, Z, W], [Z, Y, -X, W]),
            (XW, [X, Y, Z, W], [W, Y, Z, -X]),
            (YX, [X, Y, Z, W], [-Y, X, Z, W]),
            (ZX, [X, Y, Z, W], [-Z, Y, X, W]),
            (WX, [X, Y, Z, W], [-W, Y, Z, X]),
            (YZ, [X, Y, Z, W], [X, Z, -Y, W]),
            (YW, [X, Y, Z, W], [X, W, Z, -Y]),
            (ZY, [X, Y, Z, W], [X, -Z, Y, W]),
            (WY, [X, Y, Z, W], [X, -W, Z, Y]),
            (ZW, [X, Y, Z, W], [X, Y, W, -Z]),
            (WZ, [X, Y, Z, W], [X, Y, -W, Z]),
        ] {
            for (a, b) in init.into_iter().zip(expected) {
                assert_eq!(elem * a, b);
            }
        }
    }
}
