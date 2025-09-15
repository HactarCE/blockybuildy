use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashSet;

pub mod sim;
pub mod stackvec;

pub use sim::*;
pub use stackvec::StackVec;

pub const IS_3D: bool = false;

pub const USE_3D_TWIST_NAMES: bool = IS_3D;
pub const MAX_BLOCKS: usize = 30; // 5 is a nice number

pub const MAX_DEPTH: usize = 3;

const SCRAMBLE_3D: &str = "U' B2 U2 L2 B2 U2 L F2 L' U2 R2 F2 L' B L B D' F' D2 L' B";
const SCRAMBLE_4D: &str = "IDFL LF UBI IUB IL2 RUBO IUBR DO DLO IUBL UI2 ID FULI LUBO OUF DFRI DL BULO FLI LUBI FR BLO UBO UBRO OUFL IU RFI BR2 RI ID IB2 OUBL RUBO DFLO DFRO BO BDL UFRO OUFL ID BURO IBL OUL BD IBL DBLI FDLO BUO DBLO FLO LDF BRO ID FLI IUF OL OL OBL ULO BUO FURI UL2 BR FDLO IUBL UBLO IBL BR2 IUBL BURO IU2 DF ODBL LDBO BR RUB LUBI IU2 FURI IUL OUL ODBR OL2 IB DL UB BO OUB FL2 DFRO ODBR UFRI RI RUFI RDBO IUR UBO BUL RDFI LUFO";

fn main() {
    if IS_3D {
        search_3d();
    } else {
        // scramble_4d();
        search_4d();
    }
}

fn search_4d() {
    let scramble = parse_twists(SCRAMBLE_4D);
    let init_piece = |solution: &[Twist], piece: Piece| {
        let all_twists = scramble.iter().chain(solution);
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

    let (solution, solution_str) = initial_4d_blocks()
        .into_iter()
        .next()
        .map(|block1| {
            let mut solution = solution.clone();
            let mut solution_str = solution_str.clone();
            let mut state = PuzzleState::default();
            let new_pieces = pieces_from_block(block1).collect();
            add_pieces(&mut state, &solution, new_pieces);
            if !ibb_iddfs(&RUBIKS_4D, &mut state, &mut solution, &mut solution_str, 1) {
                println!("  Abandoning 2x2x2x2 block attempt");
                return None;
            }

            // extensions(block1)
            //     .into_par_iter()
            //     .map(|block2| {
            //         let mut solution = solution.clone();
            //         let mut solution_str = solution_str.clone();
            //         let mut state = state.clone();
            //         let new_pieces = new_pieces_from_block(block2, block1);
            //         add_pieces(&mut state, &solution, new_pieces);
            //         if !ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1) {
            //             println!("  Abandoning 2x2x3 block attempt");
            //             return None;
            //         }

            //         extensions(block2)
            //             .into_par_iter()
            //             .map(|block3| {
            //                 let mut solution = solution.clone();
            //                 let mut solution_str = solution_str.clone();
            //                 let mut state = state.clone();
            //                 let new_pieces = new_pieces_from_block(block3, block2);
            //                 add_pieces(&mut state, &solution, new_pieces);
            //                 if !ibb_iddfs(
            //                     &RUBIKS_3D,
            //                     &mut state,
            //                     &mut solution,
            //                     &mut solution_str,
            //                     1,
            //                 ) {
            //                     println!("  Abandoning 2x3x3 block attempt");
            //                     return None;
            //                 }

            Some((solution, solution_str))
            //             })
            //             .filter_map(|x| x)
            //             .min_by_key(|(solution, _)| solution.len())
            //     })
            //     .filter_map(|x| x)
            //     .min_by_key(|(solution, _)| solution.len())
        })
        .into_iter()
        .filter_map(|x| x)
        .min_by_key(|(solution, _)| solution.len())
        .expect("no solution");

    println!();
    println!();
    println!("whole solution: {solution_str}");
    println!("{} ETM in {:?}", solution.len(), t.elapsed());
}

fn scramble_4d() {
    println!(
        "{}",
        RUBIKS_4D.random_moves(100).map(|t| t.to_string()).join(" ")
    );
}

fn search_3d() {
    let scramble = parse_twists(SCRAMBLE_3D);
    let init_piece = |solution: &[Twist], piece: Piece| {
        let all_twists = scramble.iter().chain(solution);
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
            if !ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1) {
                println!("  Abandoning 2x2x2 block attempt");
                return None;
            }

            extensions(block1)
                .into_par_iter()
                .map(|block2| {
                    let mut solution = solution.clone();
                    let mut solution_str = solution_str.clone();
                    let mut state = state.clone();
                    let new_pieces = new_pieces_from_block(block2, block1);
                    add_pieces(&mut state, &solution, new_pieces);
                    if !ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1) {
                        println!("  Abandoning 2x2x3 block attempt");
                        return None;
                    }

                    extensions(block2)
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
                            ) {
                                println!("  Abandoning 2x3x3 block attempt");
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

fn extensions(block: Block) -> Vec<Block> {
    block
        .inactive_grips()
        .iter()
        .map(|g| block.expand_to_active_grip(g))
        .collect()
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
) -> bool {
    while state.blocks.len() > min_blocks {
        println!(
            "  We currently have {} blocks. Trying to get {min_blocks} blocks ...",
            state.blocks.len()
        );
        let t = std::time::Instant::now();
        let Some(new_twists) = (min_blocks..state.blocks.len())
            .rev()
            .map_while(|expected_blocks| {
                println!("    Searching for {expected_blocks} blocks ...");
                iddfs_to_solved(puzzle, *state, expected_blocks)
            })
            .last()
        else {
            return false; // can't even get 1 block within reasonable depth
        };
        confirm_solution(state, solution, solution_str, new_twists, t);
    }
    true
}

fn confirm_solution(
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    solution_str: &mut String,
    new_twists: Vec<Twist>,
    start: std::time::Instant,
) {
    solution_str.push_str(&format!(" ({})", new_twists.iter().join(" ")));
    for new_twist in new_twists {
        *state = state.do_twist(new_twist).unwrap();
        solution.push(new_twist);
    }
    let elapsed = start.elapsed();
    println!(
        "    Solution found in {elapsed:?}! We now have {} blocks",
        state.blocks.len(),
    );
    println!("  Solution so far: {}", solution.iter().join(" "));
    return;
}

fn iddfs_to_solved(
    puzzle: &Puzzle,
    state: PuzzleState,
    expected_blocks: usize,
) -> Option<Vec<Twist>> {
    for max_depth in 0..=MAX_DEPTH {
        println!("      Searching at depth {max_depth} ...");
        let mut new_twists = vec![];
        if dfs_to_solved(puzzle, state, &mut new_twists, expected_blocks, max_depth) {
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
    max_depth: usize,
) -> bool {
    if state.blocks.len() <= expected_blocks {
        return true;
    }
    if max_depth == 0 {
        return false;
    }
    for &twist in &puzzle.twists {
        if let Some(new_state) = state.do_twist(twist) {
            solution.push(twist);
            if dfs_to_solved(puzzle, new_state, solution, expected_blocks, max_depth - 1) {
                return true;
            }
            solution.pop();
        }
    }
    false
}

fn parse_twists(s: &str) -> Vec<Twist> {
    s.split_whitespace()
        .map(|word| TWISTS_FROM_NAME[word])
        .collect()
}
