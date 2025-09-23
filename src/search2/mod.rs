use std::{
    collections::{BTreeSet, HashMap},
    ops::{BitOr, BitOrAssign},
    sync::{Arc, atomic::AtomicUsize},
    u8,
};

use itertools::Itertools;

use crate::{
    Block, BlockSet, GripId, MAX_TWISTS_PER_SEGMENT, PuzzleState, SegmentId, StackVec, Twist,
};

pub fn do_blockbuilding_stage(
    states_from_previous_stage: &[(BlockBuildingState, PuzzleState, Stage1Meta)],
) -> impl Iterator<Item = (BlockSet, Stage2Meta)> {
    let initial_states = states_from_previous_stage
        .iter()
        .flat_map(|&(blocks, last_moved_grip, state, meta)| {
            meta.next_stage().map(move |(new_block, new_meta)| {
                let new_blocks = add_scrambled_block_to_puzzle(blocks, new_block, state);
                (new_blocks, last_moved_grip, new_meta)
            })
        })
        .collect_vec();

    let mut step = Step::new(initial_states);
    step.search();
    // for block_count in (1..16).rev() {
    // }

    vec![].into_iter()
}

pub trait GenericStage {
    fn append_reversed_twists_for_segment(
        &self,
        twists: &mut Vec<Twist>,
        s: SegmentId,
    ) -> Option<SegmentId>;
    fn generate_more(&self) -> Vec<Twist>;
}

fn add_scrambled_block_to_puzzle(
    blocks: BlockSet,
    new_block: Block,
    state: PuzzleState,
) -> BlockSet {
    todo!()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stage1Meta {}
impl Stage1Meta {
    pub fn next_stage(self) -> impl Iterator<Item = (Block, Stage2Meta)> {
        [].into_iter()
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stage2Meta {}

// impl<T: Stage> GenericStage for T {
//     fn append_reversed_twists_for_segment(
//         &self,
//         twists: &mut Vec<Twist>,
//         s: SegmentId,
//     ) -> Option<SegmentId> {
//         todo!()
//     }

//     fn generate_more(&self) {
//         todo!()
//     }
// }

// pub trait Stage {
//     type State;
//     type Meta;

//     fn initial_states(&self, twists: &[(SegmentId, Vec<Twist>)]) -> Vec<State>;
//     fn step(&self, prev_segment: State) -> Segment<State, Meta>;
// }

// pub struct Stage<S, M> {
//     pub gen_initial_states_fn: Box<dyn Fn(&[Vec<Twist>]) -> Vec<S>>,
//     pub states: Vec<(S, M, Vec)>,
//     pub step_fn: Box<dyn Fn(&[S]) -> Vec<S>>,
//     pub segments_at_each_step: Vec<Vec<Segment<S, M>>>,
// }

pub struct Stage {
    steps: Vec<Step>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockBuildingState {
    pub blocks: BlockSet,
    pub last_moved_grip: LastMovedGrip,
}

struct Step {
    initial_states: Vec<(BlockBuildingState, Stage2Meta)>,
    state_indices: HashMap<BlockBuildingState, usize>,
    states: Vec<(BlockBuildingState, Vec<StatePredecessor>)>,
}
impl Step {
    pub fn new(initial_states: Vec<(BlockBuildingState, Stage2Meta)>) -> Self {
        Self {
            initial_states,
            state_indices: Default::default(),
            states: Default::default(),
        }
    }

    pub fn add_state_predecessor(
        &mut self,
        mut state: BlockBuildingState,
        predecessor: StatePredecessor,
    ) {
        state.blocks.blocks.sort_unstable(); // make some effort to canonicalize
        let state_index = *self.state_indices.entry(state).or_insert_with(|| {
            let new_id = self.states.len();
            self.states.push((state, vec![]));
            new_id
        });
        self.states[state_index].1.push(predecessor);
    }

    pub fn search(&mut self) {
        const MAX_DEPTH: usize = 4;

        for depth in 0..MAX_DEPTH {
            for (i, &(state, meta)) in self.initial_states.iter().enumerate() {
                let segments = dfs(state, depth);
                for (new_state, extension) in segments {
                    self.add_state_predecessor(
                        new_state,
                        StatePredecessor {
                            prev_state_id: i,
                            twists: extension.new_twists,
                            additional_twist_count: extension.additional_twist_count,
                            min_total_twist_count: ,
                        },
                    );
                }
            }
        }
    }

    // pub fn sort_predecessors

    // pub fn state_predecessors
}

pub struct SomethingState {
    blocks: BlockSet,
    min_twist_count: usize,
    last_moved_grip: LastMovedGrip,
}

fn dfs(state: BlockBuildingState, depth: usize) -> Vec<(BlockBuildingState, SolutionExtension)> {
    vec![]
}

pub struct SolutionExtension {
    pub new_twists: StackVec<Twist, MAX_TWISTS_PER_SEGMENT>,
    pub additional_twist_count: usize,
    pub last_moved_grip: LastMovedGrip,
}

struct Segment {
    resulting_state: BlockSet,
    predecessors: Vec<StatePredecessor>,
    min_twist_count: usize,
}
impl Segment {
    pub fn add_predecessor(&mut self, predecessor: StatePredecessor) {
        self.min_twist_count =
            std::cmp::min(self.min_twist_count, predecessor.min_total_twist_count);
        self.predecessors.push(predecessor);
    }
    pub fn sort(&mut self) {
        self.predecessors
            .sort_unstable_by_key(|pred| pred.min_total_twist_count);
    }
    pub fn predecessors(&self, max_twist_count: usize) -> impl Iterator<Item = &StatePredecessor> {
        self.predecessors
            .iter()
            .take_while(move |pred| pred.min_total_twist_count <= max_twist_count)
    }
}

#[derive(Debug)]
pub struct StatePredecessor {
    pub prev_state_id: usize,
    pub twists: StackVec<Twist, MAX_TWISTS_PER_SEGMENT>,
    pub additional_twist_count: usize,
    pub min_total_twist_count: usize,
}
impl StatePredecessor {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LastMovedGrip(u8);
impl Default for LastMovedGrip {
    fn default() -> Self {
        Self(u8::MAX) // no grip
    }
}
impl LastMovedGrip {
    pub fn push_twist(&mut self, grip_of_new_twist: GripId) -> usize {
        let axis_of_new_twist = grip_of_new_twist.axis() as u8 + 8;

        if self.0 == grip_of_new_twist.id() || self.0 == axis_of_new_twist {
            // next twist on this axis has no cost
            0 // no cost!
        } else if self.0 == grip_of_new_twist.opposite().id() {
            self.0 = axis_of_new_twist; // next twist on this axis has no cost
            0 // no cost!
        } else {
            self.0 = grip_of_new_twist.id(); // next twist on this grip has no cost
            1 // cost = 1
        }
    }
}

// pub struct Segment<S, M> {
//     pub prev_state_id: S,
//     pub last_moved_grip: LastMovedGrip,

//     pub twists: StackVec<Twist, MAX_TWISTS_PER_SEGMENT>,
//     pub min_total_twist_count: usize,
//     pub resulting_state_id: S,
//     pub meta: M,
// }

/// Bitmask indicating the last moves of a solution, for canceling purposes.
///
/// Each bit represents the last few moves of a different solution, so applying
/// a twist always collapses the set to a single bit.
///
/// The bits are arranged in the following order: `0000_xyzw_IOBF_DULR`.
///
/// - Lowercase letters represent axes where the positive and negative grip have
///   *both* been moved already.
/// - Uppercase letters represent grips that have been moved where the opposite
///   grip has not been.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LastMovedGripSet(u16);
impl LastMovedGripSet {
    pub fn push_twist(&mut self, grip_of_new_twist: GripId) -> usize {
        let axis_bit = 1_u16 << (grip_of_new_twist.axis() + 8);
        if self.0 & axis_bit != 0 {
            self.0 = axis_bit; // next twist on this axis is free
            return 0; // free!
        }

        let grip_bit = 1_u16 << grip_of_new_twist.id();
        if self.0 & grip_bit != 0 {
            // last twist was on
            self.0 = grip_bit; // next twist on this grip is free
            return 0; // free!
        }

        let opposite_grip_bit = 1_u16 << grip_of_new_twist.opposite().id();
        if self.0 & opposite_grip_bit != 0 {
            // last twist was on opposite grip
            self.0 = axis_bit; // next twist on this axis is free
            return 1;
        }

        // last twist was on a different axis
        self.0 = grip_bit; // next twist on this grip is free
        return 1;
    }
}
impl BitOr for LastMovedGripSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for LastMovedGripSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub fn simplify_twist_sequence(twists: &[Twist]) -> Vec<Twist> {
    let mut ret: Vec<Twist> = vec![];
    for &twist in twists {
        let mut last_twists_on_axis = ret
            .iter_mut()
            .rev()
            .take_while(|t| t.grip.axis() == twist.grip.axis());

        if let Some(prior) = last_twists_on_axis.find(|t| t.grip == twist.grip) {
            prior.transform = twist.transform * prior.transform;
        } else {
            ret.push(twist);
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::PuzzleState;

    use super::*;

    proptest! {
        #[test]
        fn proptest_last_moved_grip_set(twists: Vec<Twist>) {
            test_last_moved_grip_set(twists);
        }
    }

    fn test_last_moved_grip_set(twists: Vec<Twist>) {
        // Test that `LastMovedGripSet` and `simplify_twist_sequence()` agree
        let mut last_moved = LastMovedGripSet::default();
        let mut count = 0;
        for &twist in &twists {
            count += last_moved.push_twist(twist.grip);
        }
        let simplified = simplify_twist_sequence(&twists);
        assert_eq!(simplified.len(), count);

        // Test that `simplify_twist_sequence()` agrees with the original
        let state1 = twists
            .iter()
            .fold(PuzzleState::default(), |mut state, &twist| {
                state.do_twist(twist);
                state
            });
        let state2 = simplified
            .iter()
            .fold(PuzzleState::default(), |mut state, &twist| {
                state.do_twist(twist);
                state
            });

        assert_eq!(state1, state2);
    }
}
