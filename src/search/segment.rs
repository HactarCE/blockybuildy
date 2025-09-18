use std::fmt;

use super::SolutionMetadata;
use crate::StackVec;
use crate::sim::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SolutionSegment {
    pub state: PuzzleState,
    pub segment_twists: StackVec<Twist, { crate::MAX_SOLUTION_SEGMENT_LEN }>,
    pub previous_segment_index: usize,
    pub total_twist_count: usize,
    pub meta: SolutionMetadata,
}
impl fmt::Display for SolutionSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let block_count = self.state.blocks.len();
        let twist_count = self.total_twist_count;
        write!(f, "{block_count} blocks in {twist_count} ETM")
    }
}
impl SolutionSegment {
    #[must_use]
    pub fn push_twist(&self, twist: Twist, last_grip: Option<GripId>) -> Option<Self> {
        Some(Self {
            state: self.state.do_twist(twist)?,
            segment_twists: self.segment_twists.push(twist)?,
            previous_segment_index: self.previous_segment_index,
            total_twist_count: self.total_twist_count + (last_grip != Some(twist.grip)) as usize,
            meta: self.meta,
        })
    }
    pub fn push_block(
        &self,
        puzzle: &Puzzle,
        setup_moves: &[Twist],
        new_block: Block,
        new_meta: SolutionMetadata,
    ) -> Option<Self> {
        Some(Self {
            state: self
                .state
                .add_block_with_setup_moves(puzzle, setup_moves, new_block)?,
            meta: new_meta,
            ..self.clone()
        })
    }
    pub fn next_stage(&self, previous_segment_index: usize) -> Self {
        Self {
            state: self.state,
            segment_twists: StackVec::new(),
            previous_segment_index,
            total_twist_count: self.total_twist_count,
            meta: self.meta,
        }
    }
}
impl PartialOrd for SolutionSegment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SolutionSegment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}
impl SolutionSegment {
    fn sort_key(&self) -> impl Ord {
        (
            self.total_twist_count,
            self.state.blocks.len(),
            self.state,
            self.previous_segment_index,
            self.segment_twists,
        )
    }
}
