use std::ops::Mul;

use super::elements::ElemId;
use super::grip_set::{GripSet, Piece};
use super::grips::GripId;
use super::group::CHIRAL_BC4;
use super::space::Vec4;
use super::twists::Twist;

impl Mul for ElemId {
    type Output = ElemId;

    fn mul(self, rhs: Self) -> Self::Output {
        CHIRAL_BC4.mul_elem_elem[self.0 as usize][rhs.0 as usize]
    }
}

pub trait TransformByElem {
    fn transform_by(self, elem: ElemId) -> Self;
}

impl Mul<Vec4> for ElemId {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        let mat = CHIRAL_BC4.mul_elem_vec[self.0 as usize];
        mat[0] * rhs.x + mat[1] * rhs.y + mat[2] * rhs.z + mat[3] * rhs.w
    }
}

impl Mul<GripId> for ElemId {
    type Output = GripId;

    fn mul(self, rhs: GripId) -> Self::Output {
        CHIRAL_BC4.mul_elem_grip[self.0 as usize][rhs.0 as usize]
    }
}

impl Mul<GripSet> for ElemId {
    type Output = GripSet;

    fn mul(self, rhs: GripSet) -> Self::Output {
        GripSet(rhs.iter().map(|g| 1 << (self * g).0).sum())
    }
}

impl Mul<Piece> for ElemId {
    type Output = Piece;

    fn mul(self, rhs: Piece) -> Self::Output {
        Piece {
            grips: self * rhs.grips,
            attitude: self * rhs.attitude,
        }
    }
}

impl TransformByElem for Twist {
    fn transform_by(self, elem: ElemId) -> Self {
        Twist {
            grip: elem * self.grip,
            transform: elem * self.transform * elem.inv(),
        }
    }
}

impl Mul<Piece> for Twist {
    type Output = Piece;

    fn mul(self, rhs: Piece) -> Self::Output {
        // GRIP THEORY
        if rhs.grips.contains(self.grip) {
            self.transform * rhs
        } else {
            rhs
        }
    }
}
