use std::collections::BTreeMap;
use std::fmt;
use std::ops::Index;
use std::ops::RangeInclusive;

use itertools::Itertools;

use super::SolutionMetadata;
use crate::StackVec;
use crate::sim::*;

/// Segment of a solution.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Segment {
    pub state: PuzzleState, // 64 bytes
    pub segment_twists: StackVec<Twist, { crate::MAX_SOLUTION_SEGMENT_LEN }>, // 23 bytes
    pub previous_segment: SegmentId, // 16 bytes
    pub total_twist_count: usize, // 8 bytes
    pub meta: SolutionMetadata, // 20 bytes
}
impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let block_count = self.state.blocks.len();
        let twist_count = self.total_twist_count;
        write!(f, "{block_count} blocks in {twist_count} ETM")
    }
}
impl Segment {
    /// Assertion that `std::mem::size_of::<Self>() == 128`
    #[allow(unused)]
    const SIZE_ASSERT: [u8; 128] = [0; std::mem::size_of::<Self>()];

    #[must_use]
    pub fn push_twist(&self, twist: Twist, last_grip: Option<GripId>) -> Option<Self> {
        Some(Self {
            state: self.state.do_twist(twist, 4)?, // 4D
            segment_twists: self.segment_twists.push(twist)?,
            previous_segment: self.previous_segment,
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
    pub fn next_step(&self, previous_segment: SegmentId) -> Self {
        Self {
            state: self.state,
            segment_twists: StackVec::new(),
            previous_segment,
            total_twist_count: self.total_twist_count,
            meta: self.meta,
        }
    }
}
impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Segment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}
impl Segment {
    fn sort_key(&self) -> impl Ord {
        (
            self.total_twist_count,
            self.state.blocks.len(),
            self.previous_segment,
            self.segment_twists,
        )
    }
}

/// ID for a [`Segment`].
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SegmentId(usize);
impl SegmentId {
    const INIT: Self = Self(0);
}

pub struct SegmentStore {
    scramble: Vec<Twist>,

    segments: Vec<Segment>,
    // for each step, for each total twist count, for each depth that they've already been searched, a list of solutions
    steps: Vec<BTreeMap<usize, Vec<Vec<SegmentId>>>>,
}
impl Index<SegmentId> for SegmentStore {
    type Output = Segment;

    fn index(&self, index: SegmentId) -> &Self::Output {
        &self.segments[index.0]
    }
}
impl SegmentStore {
    pub fn new(scramble: Vec<Twist>) -> Self {
        Self {
            scramble,
            segments: vec![Segment::default()],
            steps: vec![BTreeMap::from_iter([(0, vec![vec![SegmentId::INIT]])])],
        }
    }

    pub fn add_segments(&mut self, step: usize, segments: Vec<Segment>) {
        let start = self.segments.len();
        self.segments.extend(segments);
        let end = self.segments.len();
        self.push_batch(step, 0, (start..end).map(SegmentId));
    }

    pub fn push_batch(
        &mut self,
        step: usize,
        depth: usize,
        segment_ids: impl Iterator<Item = SegmentId>,
    ) {
        crate::extend_vec_to_index(&mut self.steps, step);
        let twist_counts_at_step = &mut self.steps[step];

        for id in segment_ids {
            let segment = &self.segments[id.0];

            let depths = twist_counts_at_step
                .entry(segment.total_twist_count)
                .or_default();

            crate::extend_vec_to_index(depths, depth);
            depths[depth].push(id);
        }
    }

    pub fn take_all_from_step(&mut self, step: usize) -> Option<Vec<SegmentId>> {
        let mut ret = vec![];
        let twist_counts = std::mem::take(self.steps.get_mut(step)?);
        for (_twist_count, depths) in twist_counts {
            ret.extend(
                depths.into_iter().flatten().sorted_by(|id1, id2| {
                    self.segments[id1.0].cmp(&self.segments[id2.0]).reverse()
                }),
            );
        }
        Some(ret)
    }

    pub fn pop_batch(
        &mut self,
        step: usize,
        twist_count: usize,
        depth: usize,
        batch_size: usize,
    ) -> Option<Vec<SegmentId>> {
        let ids = self
            .steps
            .get_mut(step)?
            .get_mut(&twist_count)?
            .get_mut(depth)?;
        ids.sort_by(|id1, id2| self.segments[id1.0].cmp(&self.segments[id2.0]).reverse());
        Some(ids.drain(ids.len().saturating_sub(batch_size)..).collect())
    }

    pub fn twist_count_range(&self, step: usize) -> Option<RangeInclusive<usize>> {
        Some(
            *self.steps.get(step)?.first_key_value()?.0
                ..=*self.steps.get(step)?.last_key_value()?.0,
        )
    }

    pub fn solution_twists_for_segment(&self, id: SegmentId) -> Vec<Twist> {
        self.twists_for_segment(&[], id)
    }
    pub fn all_prior_twists_for_segment(&self, id: SegmentId) -> Vec<Twist> {
        self.twists_for_segment(&self.scramble, id)
    }
    fn twists_for_segment(&self, init: &[Twist], mut id: SegmentId) -> Vec<Twist> {
        let mut reversed_twists = vec![];
        while id != SegmentId::INIT {
            let segment = &self[id];
            reversed_twists.extend(segment.segment_twists.into_iter().rev());
            id = segment.previous_segment;
        }
        init.iter()
            .chain(reversed_twists.iter().rev())
            .copied()
            .collect()
    }

    pub fn best_solution_so_far(&self) -> Option<SegmentId> {
        self.steps
            .last()? // last step
            .first_key_value()? // shortest move count
            .1
            .iter()
            .flatten()
            .next()
            .copied()
    }

    pub fn next_step(&self) -> usize {
        self.steps.len()
    }
}
