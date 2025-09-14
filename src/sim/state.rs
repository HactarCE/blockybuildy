use std::fmt;

use itertools::Itertools;

use super::grip_set::Piece;
use super::twists::Twist;

pub struct PuzzleState {
    pub pieces: Vec<Piece>,
}
impl PuzzleState {
    pub fn do_move(&mut self, twist: Twist) {
        for piece in &mut self.pieces {
            *piece = twist * *piece;
        }
    }

    pub fn is_solved(&self) -> bool {
        self.pieces.iter().map(|p| p.attitude).all_equal()
    }
}
impl fmt::Display for PuzzleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        write!(f, "{}", self.pieces.iter().join(", "))?;
        write!(f, "]")?;
        Ok(())
    }
}
