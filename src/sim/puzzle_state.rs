use crate::{ElemId, GripId, IDENT, Twist};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PuzzleState {
    /// Piece attitudes, excluding 1 core and 8 centers.
    piece_attitudes: [ElemId; 72],
}
impl Default for PuzzleState {
    fn default() -> Self {
        Self {
            piece_attitudes: [IDENT; 72],
        }
    }
}
impl PuzzleState {
    pub fn do_twist(&mut self, twist: Twist) {
        let indices = INDICES_FOR_GRIP[twist.grip.id() as usize];

        for i in indices {
            self.piece_attitudes[i] = twist.transform * self.piece_attitudes[i];
        }
    }
    pub fn do_twists(&mut self, twists: &[Twist]) {
        for &twist in twists {
            self.do_twist(twist);
        }
    }
    pub fn oriented_pieces(&self, last_layer: GripId) -> [usize; 3] {
        let piece_indices = INDICES_FOR_GRIP[last_layer.id() as usize];
        let is_piece_oriented = |&i: &usize| {
            self.piece_attitudes[piece_indices[i]] * last_layer.vec() == last_layer.vec()
        };

        let ridges = [4, 10, 12, 13, 15, 21]
            .into_iter()
            .filter(is_piece_oriented)
            .count();
        let edges = [1, 3, 5, 7, 9, 11, 14, 16, 18, 20, 22, 24]
            .into_iter()
            .filter(is_piece_oriented)
            .count();
        let corners = [0, 2, 6, 8, 17, 19, 23, 25]
            .into_iter()
            .filter(is_piece_oriented)
            .count();

        [ridges, edges, corners]
    }
}

const INDICES_FOR_GRIP: [[usize; 26]; 8] = [
    indices_for_grip(GripId::new(0)),
    indices_for_grip(GripId::new(1)),
    indices_for_grip(GripId::new(2)),
    indices_for_grip(GripId::new(3)),
    indices_for_grip(GripId::new(4)),
    indices_for_grip(GripId::new(5)),
    indices_for_grip(GripId::new(6)),
    indices_for_grip(GripId::new(7)),
];

const fn indices_for_grip(g: GripId) -> [usize; 26] {
    let mut strides = [1, 3, 9, 27];
    strides.swap(g.axis(), 0);
    let init = strides[0]
        * match g.signum() {
            1 => 0,
            -1 => 2,
            _ => unreachable!(),
        };
    let mut ret = [0; 26];
    let mut i = 0;
    while i < ret.len() {
        let j = i + (i >= 13) as usize;
        let x = j % 3;
        let y = (j / 3) % 3;
        let z = (j / 9) % 3;
        ret[i] = adjust_index(init + x * strides[1] + y * strides[2] + z * strides[3]);
        i += 1;
    }
    ret
}

const fn adjust_index(i: usize) -> usize {
    const CORE: usize = 1 + 3 + 9 + 27;
    if i == CORE {
        panic!()
    } else if i < CORE {
        i
    } else {
        i - 1
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::HYPERCUBE_GRIPS;

    fn unpack_index(mut i: usize) -> [usize; 4] {
        if i >= 1 + 3 + 9 + 27 {
            i += 1;
        }
        [i % 3, (i / 3) % 3, (i / 9) % 3, i / 27]
    }

    #[test]
    fn test_all_pieces_indices() {
        for g1 in HYPERCUBE_GRIPS {
            for g2 in HYPERCUBE_GRIPS {
                if g1 == g2 {
                    continue;
                }
                let set1: HashSet<_> = HashSet::from_iter(INDICES_FOR_GRIP[g1.id() as usize]);
                let set2: HashSet<_> = HashSet::from_iter(INDICES_FOR_GRIP[g2.id() as usize]);
                let expected_intersection = if g1.axis() == g2.axis() { 0 } else { 9 };
                println!(
                    "g1: {:?}",
                    INDICES_FOR_GRIP[g1.id() as usize].map(unpack_index)
                );
                println!(
                    "g2: {:?}",
                    INDICES_FOR_GRIP[g2.id() as usize].map(unpack_index)
                );
                assert_eq!(expected_intersection, set1.intersection(&set2).count());
            }
        }
    }
}
