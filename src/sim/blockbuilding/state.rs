use std::collections::HashSet;
use std::fmt;

use itertools::Itertools;

use crate::StackVec;
use crate::sim::*;

/// (64 bytes) State of a puzzle, tracked using [`crate::MAX_BLOCKS`] blocks.
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
    pub fn do_twist(self, twist: Twist, ndim: usize) -> Option<Self> {
        let new_blocks = StackVec::<Block, { crate::MAX_BLOCKS }>::from_iter(
            self.blocks
                .into_iter()
                .flat_map(|block| twist * block)
                .flatten(), // Option<T> -> T
        )?;

        Some(Self { blocks: new_blocks }.merge_blocks(ndim))
    }

    #[must_use]
    fn merge_blocks(mut self, ndim: usize) -> Self {
        loop {
            let mut merged_blocks = StackVec::new();
            'b1: for b1 in self.blocks {
                for b2 in &mut merged_blocks {
                    if let Some(merged) = b1.try_merge(*b2, ndim) {
                        *b2 = merged; // replace with merged block
                        continue 'b1;
                    }
                }
                merged_blocks = merged_blocks.push(b1).unwrap();
            }
            if self.blocks.len() == merged_blocks.len() {
                return self;
            }
            self.blocks = merged_blocks;
        }
    }

    /// Constructs a state containing the given blocks and combines blocks as
    /// much as possible.
    ///
    /// TODO: it may be possible for this to get an "N-perm situation" which
    /// would be bad.
    #[must_use]
    fn from_blocks(blocks: StackVec<Block, { crate::MAX_BLOCKS }>, ndim: usize) -> Self {
        Self { blocks }.merge_blocks(ndim)
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

        Some(Self::from_blocks(
            self.blocks
                .extend(new_pieces.into_iter().map(init_piece).map(Block::from))?,
            puzzle.ndim,
        ))
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
            (true, "F R U R' U' F'"),                       // fruruf
            (true, "R U R' U' R' F R F'"),                  // sexy sledgehammer
            (true, "R U R' F' R U R' U' R' F R2 U' R'"),    // J perm
            (true, "R U R' U R' U' R2 U' R' U R' U R"),     // U perm
            (true, "R U R' U' R' F R2 U' R' U' R U R' F'"), // T perm
        ];

        let ndim = 3;

        for (should_be_solved, last_layer_twist_seq) in last_layer_algs {
            let mut state = PuzzleState {
                blocks: StackVec::from_iter([Block::new_solved([], [U]).unwrap()]).unwrap(),
            };
            for t in last_layer_twist_seq
                .split_ascii_whitespace()
                .map(|s| TWISTS_FROM_NAME[s])
            {
                state = state.do_twist(t, ndim).unwrap();
            }
            assert_eq!(should_be_solved, state.is_solved());
        }
    }
}
