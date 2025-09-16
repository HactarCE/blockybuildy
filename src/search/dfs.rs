use super::*;

/// Runs iterative-deepening depth-first searches repeatedly via
/// [`iddfs_blockbuild()`] to blockbuild from `state` until there are at most
/// `target_blocks` remaining.
///
/// If the search succeeded, then `state` is updated and the solution is
/// appended to `solution` and a list is returned containing the new segments of
/// the solution. If it failed, then `state` and `solution` are left unmodified
/// and `None` is returned.
pub fn blockbuild_via_repeated_iddfs(
    puzzle: &Puzzle,
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    params: BlockBuildingSearchParams,
    target_blocks: usize,
) -> Option<Vec<Vec<Twist>>> {
    let mut solution_segments = vec![];

    let mut current_max_depth = params.max_depth;
    while state.blocks.len() > target_blocks {
        if params.verbosity >= 2 {
            println!(
                "  We currently have {} blocks. Trying to get to {target_blocks} blocks ...",
                state.blocks.len(),
            );
        }

        'deepen_loop: loop {
            if let Some(additional_twist_count) =
                (target_blocks..state.blocks.len()).find_map(|expected_blocks| {
                    iddfs_blockbuild(
                        puzzle,
                        state,
                        solution,
                        params,
                        expected_blocks,
                        current_max_depth,
                    )
                })
            {
                // Reset max depth.
                current_max_depth = params.max_depth;
                solution_segments
                    .push(solution[solution.len() - additional_twist_count..].to_vec());
                break 'deepen_loop;
            } else if params.deepen_when_stuck {
                if params.verbosity >= 2 {
                    println!(
                        "  Stuck at {} blocks; increasing depth tempoarily. Partial solution: {}",
                        state.blocks.len(),
                        solution.iter().join(" "),
                    );
                }
                current_max_depth += 1;
                continue 'deepen_loop; // can't even get 1 block within reasonable depth
            } else {
                return None; // give up
            }
        }
    }

    Some(solution_segments)
}

/// Runs an iterative-deepening depth-first search to `max_depth` for a sequence
/// of moves that results in at most `expected_blocks` blocks. Returns the
/// number of moves in the solution if one is found, or `None` if the search
/// failed.
///
/// If the search succeeded, then `state` is updated and the solution is
/// appended to `solution`. If it failed, then `state` and `solution` are left
/// unmodified.
pub fn iddfs_blockbuild(
    puzzle: &Puzzle,
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    params: BlockBuildingSearchParams,
    expected_blocks: usize,
    max_depth: usize,
) -> Option<usize> {
    let t = std::time::Instant::now();
    if params.verbosity >= 3 {
        println!("    Searching for {expected_blocks} blocks ...");
    }
    for depth in 0..=max_depth {
        if params.verbosity >= 4 {
            println!("      Searching at depth {depth} ...");
        }
        let dfs_result = dfs_blockbuild(
            puzzle,
            state,
            solution,
            params,
            expected_blocks,
            depth,
            None,
        );
        if let Some(mut additional_twist_count) = dfs_result {
            let trim = std::cmp::min(params.trim, additional_twist_count.saturating_sub(1));
            if params.verbosity >= 3 {
                println!(
                    "    Solution found using {}{} ETM in {:?}! We now have {} blocks",
                    additional_twist_count,
                    if trim > 0 {
                        format!("-{trim} (trimmed)")
                    } else {
                        String::new()
                    },
                    t.elapsed(),
                    state.blocks.len(),
                );
                println!("    Solution so far: {}", solution.iter().join(" "));
            }
            if trim > 0 {
                additional_twist_count -= trim;
                solution.truncate(solution.len() - trim);
                if params.verbosity >= 3 {
                    println!("    (trimmed by {trim} moves for next stage)");
                }
            }

            return Some(additional_twist_count);
        }
    }

    None
}

/// Runs a depth-first search to `remaining_depth` for a sequence of moves that
/// results in at most `expected_blocks` blocks. Returns the number of moves in
/// the solution if one is found, or `None` if the search failed.
///
/// If the search succeeded, then `state` is updated and the solution is
/// appended to `solution`. If it failed, then `state` and `solution` are left
/// unmodified.
pub fn dfs_blockbuild(
    puzzle: &Puzzle,
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    params: BlockBuildingSearchParams,
    expected_blocks: usize,
    remaining_depth: usize,
    last_grip: Option<GripId>,
) -> Option<usize> {
    if state.blocks.len() <= expected_blocks {
        return Some(0); // done!
    }
    if remaining_depth == 0 {
        return None;
    }

    if !params
        .heuristic
        .might_be_solvable(*state, expected_blocks, remaining_depth)
    {
        return None;
    }

    let combined_layer_mask = state
        .blocks
        .iter()
        .map(|b| b.layers())
        .fold(PackedLayers::EMPTY, |a, b| a | b);

    for grip in &puzzle.grips {
        if last_grip == Some(grip.id) {
            continue; // same grip as last move
        }
        if combined_layer_mask.grip_status(grip.id) == Some(GripStatus::Inactive) {
            continue; // doesn't move any block
        }

        for twist in grip.twists() {
            if let Some(mut new_state) = state.do_twist(twist) {
                solution.push(twist);
                let dfs_result = dfs_blockbuild(
                    puzzle,
                    &mut new_state,
                    solution,
                    params,
                    expected_blocks,
                    remaining_depth - 1,
                    Some(twist.grip),
                );
                if let Some(additional_twist_count) = dfs_result {
                    *state = new_state;
                    return Some(additional_twist_count + 1);
                }
                solution.pop();
            }
        }
    }

    None
}
