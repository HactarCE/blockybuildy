use super::*;

/// Heuristic for pruning search branches.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Heuristic {
    /// Prune aggressively; prune branches that are unlikely to result in a
    /// solution.
    Fast,
    /// Prune conservatively; never prune a branch that could possibly result in
    /// a solution.
    #[default]
    Correct,
}

impl Heuristic {
    /// Returns whether the heuristic believes that `state` can be reduced to
    /// `expected_blocks` blocks within `remaining_moves`.
    pub fn might_be_solvable(
        self,
        state: PuzzleState,
        expected_blocks: usize,
        remaining_moves: usize,
    ) -> bool {
        let remaining_pairings_needed = state.blocks.len() - expected_blocks;

        remaining_pairings_needed <= self.combinatoric_limit(expected_blocks, remaining_moves)
            && remaining_pairings_needed <= self.grip_theoretic_limit(state, remaining_moves)
    }

    /// Returns the maximum number of block pairings possible using a naive
    /// combinatoric approach.
    fn combinatoric_limit(self, expected_blocks: usize, remaining_moves: usize) -> usize {
        match self {
            Heuristic::Fast => 1 << remaining_moves,
            Heuristic::Correct => (1 << remaining_moves) * expected_blocks,
        }
    }
    /// Returns the maximum number of block pairings using a grip-theoretic
    /// approach.
    fn grip_theoretic_limit(self, state: PuzzleState, remaining_moves: usize) -> usize {
        let blocks_when_solved = state.blocks.map(|b| b.at_solved().layers());
        let mut max_pairings_possible = 0;
        for (i, (b1, b1_when_solved)) in
            std::iter::zip(state.blocks, blocks_when_solved).enumerate()
        {
            // assume `b1` can be made into a larger block within 3 moves.
            // assume other moves are used to make more blocks.
            let mut max_blocks_solvable_using_b1 = remaining_moves.saturating_sub(2);

            'b2: for (b2, b2_when_solved) in
                std::iter::zip(state.blocks, blocks_when_solved).skip(i + 1)
            {
                if let Some((_combined_block, merge_grip)) =
                    b1_when_solved.try_merge_with(b2_when_solved)
                {
                    if b1.attitude() * merge_grip == b2.attitude() * merge_grip {
                        // `b1` and `b2` can be paired in 1 move.
                        // assume other moves are used to make more blocks.
                        max_blocks_solvable_using_b1 = remaining_moves;
                        break 'b2; // not gonna do better than that
                    }
                    if remaining_moves > 1 {
                        for g in b1.layers().separating_axes(b2.layers()).iter() {
                            if g.can_transform_grip_to_grip(
                                b1.attitude() * merge_grip,
                                b2.attitude() * merge_grip,
                            ) {
                                // `b1` and `b2` can be paired in 2 moves.
                                // assume other moves are used to make more blocks
                                max_blocks_solvable_using_b1 = remaining_moves - 1;
                            }
                        }
                    }
                }
            }

            max_pairings_possible += max_blocks_solvable_using_b1;
        }

        max_pairings_possible
    }
}
