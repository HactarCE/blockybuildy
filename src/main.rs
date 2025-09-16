use std::collections::HashSet;

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub mod sim;
pub mod stackvec;

pub use sim::*;
pub use stackvec::StackVec;

pub const IS_3D: bool = false;
const HEURISTICS_MODE: HeuristicsMode = HeuristicsMode::Fast;

// pub const MAX_BLOCKS: usize = 10;
pub const MAX_BLOCKS: usize = 31;
// bigger number is sometimes better???

const TRIM: usize = 0;
const DEEPEN_WHEN_STUCK: bool = false;

pub const USE_3D_TWIST_NAMES: bool = IS_3D;

fn main() {
    if IS_3D {
        let scramble = scramble_3d();
        // let scramble = "U' B2 U2 L2 B2 U2 L F2 L' U2 R2 F2 L' B L B D' F' D2 L' B";
        let max_depth = 5;
        search_3d(&scramble, max_depth, 0);
        search_3d(&scramble, max_depth, 1);
        search_3d(&scramble, max_depth, 2);
        let max_depth = 6;
        search_3d(&scramble, max_depth, 0);
        search_3d(&scramble, max_depth, 1);
        search_3d(&scramble, max_depth, 2);
    } else {
        // let scramble = scramble_4d();
        let scramble = "IDFL LF UBI IUB IL2 RUBO IUBR DO DLO IUBL UI2 ID FULI LUBO OUF DFRI DL BULO FLI LUBI FR BLO UBO UBRO OUFL IU RFI BR2 RI ID IB2 OUBL RUBO DFLO DFRO BO BDL UFRO OUFL ID BURO IBL OUL BD IBL DBLI FDLO BUO DBLO FLO LDF BRO ID FLI IUF OL OL OBL ULO BUO FURI UL2 BR FDLO IUBL UBLO IBL BR2 IUBL BURO IU2 DF ODBL LDBO BR RUB LUBI IU2 FURI IUL OUL ODBR OL2 IB DL UB BO OUB FL2 DFRO ODBR UFRI RI RUFI RDBO IUR UBO BUL RDFI LUFO";
        let max_depth = 4;
        search_4d(&scramble, max_depth, TRIM);
    }
}

fn search_4d(scramble: &str, max_depth: usize, trim: usize) {
    let scramble_twists = parse_twists(scramble);

    fn add_pieces(
        state: &mut PuzzleState,
        scramble_twists: &[Twist],
        partial_solution: &[Twist],
        pieces: Vec<Piece>,
    ) {
        for piece in pieces {
            let all_twists = scramble_twists.iter().chain(partial_solution);
            let piece = all_twists.fold(piece, |p, &twist| twist * p);
            state.add_piece(piece);
        }
    }

    let mut solution = vec![];

    #[track_caller]
    fn search_and_update_solution(
        block_name: &str,
        candidate_blocks: &[Block],
        scramble_twists: &[Twist],
        state: &mut PuzzleState,
        solution: &mut Vec<Twist>,
        expected_blocks: usize,
        max_depth: usize,
        trim: usize,
    ) -> Block {
        let t = std::time::Instant::now();

        println!(
            "Solving {block_name} block (exploring {} options) ...",
            candidate_blocks.len(),
        );

        let (new_solved_block, new_state, new_solution) = candidate_blocks
            .into_par_iter()
            .filter_map(|&new_block| {
                let mut solution = solution.clone();
                let mut state = state.clone();
                add_new_pieces_from_block(&mut state, scramble_twists, &solution, new_block);
                ibb_iddfs(
                    &RUBIKS_4D,
                    &mut state,
                    &mut solution,
                    &mut String::new(), // don't care
                    expected_blocks,
                    max_depth,
                    trim,
                )
                .then_some((new_block, state, solution))
            })
            .min_by_key(|(_, _, solution)| solution.len())
            .expect(&format!("no solution to {block_name} block"));

        let old_len = solution.len();

        // Update state solution
        *state = new_state;
        *solution = new_solution;

        println!("Search completed in {:?}", t.elapsed());
        println!(
            "Best solution solves {new_solved_block} in {} ETM ({} ETM total)",
            solution.len() - old_len,
            solution.len(),
        );
        println!("{}", solution.iter().join(" "));
        println!();

        new_solved_block
    }

    let mut state = PuzzleState::default();

    let t0 = std::time::Instant::now();
    let block_2222 = search_and_update_solution(
        "2x2x2x2",
        &initial_4d_blocks(),
        &scramble_twists,
        &mut state,
        &mut solution,
        1,
        max_depth,
        trim,
    );

    let block_2223 = search_and_update_solution(
        "2x2x2x3",
        &extensions(block_2222, GripSet::ALL, true),
        &scramble_twists,
        &mut state,
        &mut solution,
        1,
        max_depth,
        trim,
    );

    let block_2233 = search_and_update_solution(
        "2x2x3x3",
        &extensions(block_2223, GripSet::ALL, true),
        &scramble_twists,
        &mut state,
        &mut solution,
        1,
        max_depth,
        trim,
    );

    let right_block_candidate_grips = block_2233.inactive_grips();
    assert_eq!(right_block_candidate_grips.iter().count(), 2);
    let right_block_1222_candidates = right_block_candidate_grips
        .iter()
        .flat_map(|g| {
            [R, U, F, O]
                .into_iter()
                .filter(move |g1| g1.axis() != g.axis())
                .map(|g1| [g1, g1.opposite()])
                .multi_cartesian_product()
                .map(move |new_grips| Block::new_solved([g], new_grips).unwrap())
        })
        .collect_vec();

    let right_block_1222 = search_and_update_solution(
        "right 1x2x2x2",
        &right_block_1222_candidates,
        &scramble_twists,
        &mut state,
        &mut solution,
        2,
        max_depth,
        trim,
    );

    let last_f2l_grip = right_block_1222.active_grips().unwrap_exactly_one();
    let grips_within_right_block = GripSet::from_iter(HYPERCUBE_GRIPS) - last_f2l_grip.opposite();

    let right_block_1223 = search_and_update_solution(
        "right 1x2x2x3",
        &extensions(right_block_1222, grips_within_right_block, true),
        &scramble_twists,
        &mut state,
        &mut solution,
        2,
        max_depth,
        trim,
    );

    let last_layer_grip = (right_block_candidate_grips - last_f2l_grip).unwrap_exactly_one();
    dbg!(last_f2l_grip);
    dbg!(last_layer_grip);
    let last_f2l_grip_2 =
        dbg!(right_block_1223.inactive_grips() - last_layer_grip - last_f2l_grip.opposite())
            .unwrap_exactly_one();
    let last_six_pieces =
        Block::new_solved([last_f2l_grip, last_f2l_grip_2], [last_layer_grip]).unwrap();
    let f2l_b_grips: [GripId; 2] = (last_six_pieces.blocked_grips() - last_layer_grip.opposite())
        .iter()
        .collect_array()
        .unwrap();
    let last_f2l_a_pair = last_six_pieces
        .restrict_to_inactive_grip(f2l_b_grips[0])
        .unwrap()
        .restrict_to_inactive_grip(f2l_b_grips[1])
        .unwrap();

    dbg!(right_block_1223);
    dbg!(last_f2l_a_pair);

    search_and_update_solution(
        "last F2L-A pair",
        &[last_f2l_a_pair],
        &scramble_twists,
        &mut state,
        &mut solution,
        2,
        max_depth + 2,
        trim,
    );

    search_and_update_solution(
        "second-to-last F2L-B pair",
        &f2l_b_grips.map(|g| last_f2l_a_pair.expand_to_active_grip(g)),
        &scramble_twists,
        &mut state,
        &mut solution,
        2,
        max_depth + 1,
        trim,
    );

    let _f2l_block = search_and_update_solution(
        "F2L",
        &[Block::new_solved([], [last_layer_grip]).unwrap()],
        &scramble_twists,
        &mut state,
        &mut solution,
        1,
        max_depth + 2,
        trim,
    );

    println!();
    println!("Whole search completed in {:?}", t0.elapsed());
    println!("Best solution found is {} ETM", solution.len());
    println!("{}", solution.iter().join(" "));
}

fn scramble_3d() -> String {
    RUBIKS_3D.random_moves(100).map(|t| t.to_string()).join(" ")
}

fn scramble_4d() -> String {
    RUBIKS_4D.random_moves(100).map(|t| t.to_string()).join(" ")
}

fn search_3d(scramble: &str, max_depth: usize, trim: usize) {
    let scramble_twists = parse_twists(scramble);
    let init_piece = |solution: &[Twist], piece: Piece| {
        let all_twists = scramble_twists.iter().chain(solution);
        all_twists.fold(piece, |p, &twist| twist * p)
    };
    let add_pieces = |state: &mut PuzzleState, solution: &[Twist], pieces: Vec<Piece>| {
        for piece in pieces {
            state.add_piece(init_piece(&solution, piece));
        }
    };

    let solution = vec![];
    let solution_str = String::new();

    let t = std::time::Instant::now();

    let (solution, solution_str) = initial_3d_petrus_blocks()
        .into_par_iter()
        .map(|block1| {
            let mut solution = solution.clone();
            let mut solution_str = solution_str.clone();
            let mut state = PuzzleState::default();
            let new_pieces = pieces_from_block(block1).collect();
            add_pieces(&mut state, &solution, new_pieces);
            if !ibb_iddfs(
                &RUBIKS_3D,
                &mut state,
                &mut solution,
                &mut solution_str,
                1,
                max_depth,
                trim,
            ) {
                // println!("  Abandoning 2x2x2 block attempt");
                return None;
            }

            extensions(block1, GripSet::from_iter(CUBE_GRIPS), true)
                .into_par_iter()
                .map(|block2| {
                    let mut solution = solution.clone();
                    let mut solution_str = solution_str.clone();
                    let mut state = state.clone();
                    let new_pieces = new_pieces_from_block(block2, block1);
                    add_pieces(&mut state, &solution, new_pieces);
                    if !ibb_iddfs(
                        &RUBIKS_3D,
                        &mut state,
                        &mut solution,
                        &mut solution_str,
                        1,
                        max_depth,
                        trim,
                    ) {
                        // println!("  Abandoning 2x2x3 block attempt");
                        return None;
                    }

                    extensions(block2, GripSet::from_iter(CUBE_GRIPS), true)
                        .into_par_iter()
                        .map(|block3| {
                            let mut solution = solution.clone();
                            let mut solution_str = solution_str.clone();
                            let mut state = state.clone();
                            let new_pieces = new_pieces_from_block(block3, block2);
                            add_pieces(&mut state, &solution, new_pieces);
                            if !ibb_iddfs(
                                &RUBIKS_3D,
                                &mut state,
                                &mut solution,
                                &mut solution_str,
                                1,
                                max_depth,
                                trim,
                            ) {
                                // println!("  Abandoning 2x3x3 block attempt");
                                return None;
                            }

                            Some((solution, solution_str))
                        })
                        .filter_map(|x| x)
                        .min_by_key(|(solution, _)| solution.len())
                })
                .filter_map(|x| x)
                .min_by_key(|(solution, _)| solution.len())
        })
        .filter_map(|x| x)
        .min_by_key(|(solution, _)| solution.len())
        .expect("no solution");

    println!();
    println!();
    println!("whole solution: {solution_str}");
    println!("{} ETM in {:?}", solution.len(), t.elapsed());
}

fn initial_4d_blocks() -> Vec<Block> {
    itertools::iproduct!([R, L], [U, D], [F, B], [I, O])
        .filter_map(|(x, y, z, w)| Block::new_solved([], [x, y, z, w]))
        .collect()
}

fn initial_3d_petrus_blocks() -> Vec<Block> {
    itertools::iproduct!([R, L], [U, D], [F, B])
        .filter_map(|(x, y, z)| Block::new_solved([], [x, y, z]))
        .collect()
}

fn extensions(block: Block, mut allowed_grips: GripSet, allow_opposites: bool) -> Vec<Block> {
    for g in allowed_grips.iter() {
        if block.grip_status(g) != Some(GripStatus::Inactive) {
            allowed_grips -= g;
            if !allow_opposites {
                allowed_grips -= g.opposite();
            }
        }
    }
    allowed_grips
        .iter()
        .map(|g| block.expand_to_active_grip(g))
        .collect()
}

fn add_new_pieces_from_block(
    state: &mut PuzzleState,
    scramble_twists: &[Twist],
    partial_solution: &[Twist],
    new_block: Block,
) {
    let mut new_pieces = pieces_from_block(new_block).collect::<HashSet<Piece>>();
    for old_block in state.blocks {
        for piece in pieces_from_block(old_block.at_solved()) {
            new_pieces.remove(&piece);
        }
    }

    let init_piece = |new_piece| {
        let twists = itertools::chain(scramble_twists, partial_solution);
        twists.fold(new_piece, |p, &twist| twist * p)
    };

    state.blocks = state
        .blocks
        .extend(new_pieces.into_iter().map(init_piece).map(Block::from))
        .expect("too many new pieces! increase MAX_BLOCKS");
}

fn new_pieces_from_block(new_block: Block, old_block: Block) -> Vec<Piece> {
    assert!(old_block.is_subset_of(new_block));
    let mut pieces = pieces_from_block(new_block).collect::<HashSet<Piece>>();
    for piece in pieces_from_block(old_block) {
        pieces.remove(&piece);
    }
    pieces.into_iter().collect()
}

fn pieces_from_block(block: Block) -> impl Iterator<Item = Piece> {
    assert_eq!(block.attitude(), IDENT);
    let mut blocks = vec![block];
    for g in block.blocked_grips().iter() {
        if IS_3D && (g == I || g == O) {
            continue;
        }
        blocks = blocks
            .into_iter()
            .flat_map(|b| b.split(g))
            .filter_map(|b| b)
            .collect();
    }
    blocks
        .into_iter()
        .map(|b| Piece::new_solved(b.active_grips().iter()))
}

/// iterative blockbuilding iddfs
fn ibb_iddfs(
    puzzle: &Puzzle,
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    solution_str: &mut String,
    min_blocks: usize,
    max_depth: usize,
    trim: usize,
) -> bool {
    let mut current_max_depth = max_depth;
    while state.blocks.len() > min_blocks {
        println!(
            "  We currently have {} blocks. Trying to get {min_blocks} blocks ...",
            state.blocks.len()
        );
        let t = std::time::Instant::now();
        if let Some(new_twists) = (min_blocks..state.blocks.len())
            .rev()
            .map_while(|expected_blocks| {
                println!("    Searching for {expected_blocks} blocks ...");
                iddfs_to_solved(
                    puzzle,
                    *state,
                    expected_blocks,
                    HEURISTICS_MODE,
                    current_max_depth,
                )
            })
            .last()
        {
            confirm_solution(state, solution, solution_str, new_twists, t, trim);
            current_max_depth = max_depth;
        } else if DEEPEN_WHEN_STUCK {
            println!(
                "Stuck at {} blocks; increasing depth tempoarily. Partial solution: {}",
                state.blocks.len(),
                solution.iter().join(" "),
            );
            current_max_depth += 1;
            continue; // can't even get 1 block within reasonable depth
        } else {
            return false; // give up
        }
    }
    true
}

fn confirm_solution(
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    solution_str: &mut String,
    mut new_twists: Vec<Twist>,
    start: std::time::Instant,
    trim: usize,
) {
    for _ in 0..trim {
        if new_twists.len() > 1 {
            new_twists.pop();
        }
    }
    solution_str.push_str(&format!(" ({})", new_twists.iter().join(" ")));
    for new_twist in new_twists {
        *state = state.do_twist(new_twist).unwrap();
        solution.push(new_twist);
    }
    let elapsed = start.elapsed();
    // println!(
    //     "    Solution found in {elapsed:?}! We now have {} blocks",
    //     state.blocks.len(),
    // );
    // println!("  Solution so far: {}", solution.iter().join(" "));
    return;
}

fn iddfs_to_solved(
    puzzle: &Puzzle,
    state: PuzzleState,
    expected_blocks: usize,
    heuristics_mode: HeuristicsMode,
    max_depth: usize,
) -> Option<Vec<Twist>> {
    for depth in 0..=max_depth {
        // println!("      Searching at depth {depth_limit} ...");
        let mut new_twists = vec![];
        if dfs_to_solved(
            puzzle,
            state,
            &mut new_twists,
            expected_blocks,
            depth,
            heuristics_mode,
            None,
        ) {
            return Some(new_twists);
        }
    }
    None
}

fn dfs_to_solved(
    puzzle: &Puzzle,
    state: PuzzleState,
    solution: &mut Vec<Twist>,
    expected_blocks: usize,
    remaining_depth: usize,
    heuristics_mode: HeuristicsMode,
    last_grip: Option<GripId>,
) -> bool {
    if state.blocks.len() <= expected_blocks {
        return true;
    }
    if remaining_depth == 0 {
        return false;
    }

    if !solvable_by_heuristic(state, expected_blocks, remaining_depth, heuristics_mode) {
        return false;
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
            if let Some(new_state) = state.do_twist(twist) {
                solution.push(twist);
                if dfs_to_solved(
                    puzzle,
                    new_state,
                    solution,
                    expected_blocks,
                    remaining_depth - 1,
                    heuristics_mode,
                    Some(twist.grip),
                ) {
                    return true;
                }
                solution.pop();
            }
        }
    }
    false
}

/// Do we have a chance of solving `expected_blocks` within `remaining_moves`?
fn solvable_by_heuristic(
    state: PuzzleState,
    expected_blocks: usize,
    remaining_moves: usize,
    heuristics_mode: HeuristicsMode,
) -> bool {
    let remaining_pairings_needed = state.blocks.len() - expected_blocks;
    let max_pairings_possible = match heuristics_mode {
        HeuristicsMode::Fast => 1 << remaining_moves,
        HeuristicsMode::Correct => (1 << remaining_moves) * expected_blocks,
    };
    if remaining_pairings_needed > max_pairings_possible {
        return false;
    }

    let blocks_when_solved = state.blocks.map(|b| b.at_solved().layers());
    let mut max_blocks_solvable = 0;
    for (i, (b1, b1_when_solved)) in std::iter::zip(state.blocks, blocks_when_solved).enumerate() {
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

        max_blocks_solvable += max_blocks_solvable_using_b1;
    }
    if max_blocks_solvable < remaining_pairings_needed {
        return false;
    }

    true
}

fn parse_twists(s: &str) -> Vec<Twist> {
    s.split_whitespace()
        .map(|word| TWISTS_FROM_NAME[word])
        .collect()
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
enum HeuristicsMode {
    Fast,
    #[default]
    Correct,
}
