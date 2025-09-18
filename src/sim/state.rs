use std::collections::HashSet;
use std::fmt;

use itertools::Itertools;

use super::grip_set::{Block, Piece};
use super::puzzle::Puzzle;
use super::twists::Twist;
use crate::StackVec;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(align(64))]
pub struct PuzzleState {
    pub blocks: StackVec<Block, { crate::MAX_BLOCKS }>,
}
impl PuzzleState {
    /// Applies a twist and returns the new state.
    ///
    /// Returns `None` if the twist failed because there are too many blocks to
    /// track.
    #[must_use]
    pub fn do_twist(self, twist: Twist) -> Option<Self> {
        let new_blocks = StackVec::<Block, { crate::MAX_BLOCKS }>::from_iter(
            self.blocks
                .into_iter()
                .flat_map(|block| twist * block)
                .flatten(), // Option<T> -> T
        )?
        .sorted_unstable_by_key(|b| b.attitude());

        Self::from_blocks(new_blocks)
    }

    /// Constructs a state containing the given blocks and combines blocks as
    /// much as possible.
    ///
    /// TODO: it may be possible for this to get an "N-perm situation" which
    /// would be bad.
    #[must_use]
    fn from_blocks(blocks_iter: impl IntoIterator<Item = Block>) -> Option<Self> {
        let mut ret = Self::default();
        let mut merge_candidates = StackVec::<Block, { crate::MAX_BLOCKS }>::new();
        for mut new_block in blocks_iter {
            'merge_loop: loop {
                for i in 0..merge_candidates.len() {
                    if let Some(merged_block) = new_block.try_merge(merge_candidates[i]) {
                        new_block = merged_block;
                        merge_candidates = merge_candidates.swap_remove(i);
                        continue 'merge_loop;
                    }
                }
                break 'merge_loop;
            }

            if merge_candidates
                .first()
                .is_some_and(|m| m.attitude() != new_block.attitude())
            {
                ret.blocks = ret.blocks.extend(merge_candidates)?;
                merge_candidates = StackVec::new();
            }

            merge_candidates = merge_candidates.push(new_block)?;
        }

        ret.blocks = ret.blocks.extend(merge_candidates)?;

        Some(ret)
    }

    pub fn is_solved(self) -> bool {
        self.blocks.len() == 1
    }

    /// Applies `setup_moves` to each piece in `block` and then adds all the
    /// pieces into the puzzle state, except for the ones that are already in
    /// the puzzle state.
    ///
    /// `block` must have the identity attitude.
    ///
    /// Returns `None` if the puzzle state would have more than
    /// [`crate::MAX_BLOCKS`] blocks.
    #[must_use]
    pub fn add_block_with_setup_moves(
        self,
        puzzle: &Puzzle,
        setup_moves: &[Twist],
        new_block: Block,
    ) -> Option<Self> {
        let pieces_from_block = |block: Block| {
            assert_eq!(block.attitude(), crate::IDENT);
            let mut blocks = vec![block];
            for g in (puzzle.grip_set() & block.blocked_grips()).iter() {
                blocks = blocks
                    .into_iter()
                    .flat_map(|b| b.split(g))
                    .flatten() // Option<T> -> T
                    .collect();
            }
            blocks
                .into_iter()
                .map(|b| Piece::new_solved(b.active_grips().iter()))
        };

        let mut new_pieces = pieces_from_block(new_block).collect::<HashSet<Piece>>();
        for old_block in self.blocks {
            for piece in pieces_from_block(old_block.at_solved()) {
                new_pieces.remove(&piece);
            }
        }

        let init_piece = |new_piece| setup_moves.iter().fold(new_piece, |p, &twist| twist * p);

        Self::from_blocks(
            self.blocks
                .extend(new_pieces.into_iter().map(init_piece).map(Block::from))?,
        )
    }
}
impl fmt::Display for PuzzleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        write!(f, "{}", self.blocks.iter().join(", "))?;
        write!(f, "]")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::StackVec;
    use crate::sim::*;

    #[test]
    fn test_puzzle_state() {
        let last_layer_algs = [
            (true, ""),
            (false, "R"),
            (false, "D"),
            (true, "U"),
            (true, "R R'"),
            (true, "D D'"),
            (true, "R L R' L'"),
            (true, "F R U R' U' F'"),                    // fruruf
            (true, "R U R' U' R' F R F'"),               // sexy sledgehammer
            (true, "R U R' F' R U R' U' R' F R2 U' R'"), // J perm
            (true, "R U R' U R' U' R2 U' R' U R' U R"),  // U perm
        ];

        for (should_be_solved, last_layer_twist_seq) in last_layer_algs {
            let mut state = PuzzleState {
                blocks: StackVec::from_iter([Block::new_solved([], [U]).unwrap()]).unwrap(),
            };
            for t in last_layer_twist_seq
                .split_ascii_whitespace()
                .map(|s| twists::TWISTS_FROM_NAME[s])
            {
                state = state.do_twist(t).unwrap();
            }
            assert_eq!(should_be_solved, state.is_solved());
        }
    }
}
