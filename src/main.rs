pub mod sim;
pub mod stackvec;

use itertools::Itertools;
pub use sim::*;
pub use stackvec::StackVec;

pub const USE_3D_TWIST_NAMES: bool = true;
pub const MAX_BLOCKS: usize = 10; // 5 is a nice number

pub const MAX_DEPTH: usize = 5;

const SCRAMBLE: &str = "R' D2 B U B D F' R' F2 L2 D' R2 D2 F2 D2 L2 B2 D L2 B";

fn main() {
    let scramble = parse_twists(SCRAMBLE);
    let init_piece = |solution: &[Twist], piece: &[GripId]| {
        let piece = Piece::new_solved(piece.iter().copied());
        let all_twists = scramble.iter().chain(solution);
        all_twists.fold(piece, |p, &twist| twist * p)
    };

    let mut solution = vec![];
    let mut solution_str = String::new();

    let t = std::time::Instant::now();

    let mut state = PuzzleState::default();
    println!("\nStage 1");
    state.add_piece(init_piece(&solution, &[]));
    state.add_piece(init_piece(&solution, &[U]));
    state.add_piece(init_piece(&solution, &[U, R]));
    state.add_piece(init_piece(&solution, &[R]));
    ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!("\nStage 2");
    state.add_piece(init_piece(&solution, &[F]));
    state.add_piece(init_piece(&solution, &[F, U]));
    state.add_piece(init_piece(&solution, &[F, U, R]));
    state.add_piece(init_piece(&solution, &[F, R]));
    ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!("\nStage 3");
    state.add_piece(init_piece(&solution, &[B]));
    state.add_piece(init_piece(&solution, &[B, U]));
    state.add_piece(init_piece(&solution, &[B, U, R]));
    state.add_piece(init_piece(&solution, &[B, R]));
    ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!("\nStage 4");
    state.add_piece(init_piece(&solution, &[L]));
    state.add_piece(init_piece(&solution, &[L, U]));
    state.add_piece(init_piece(&solution, &[L, U, F]));
    state.add_piece(init_piece(&solution, &[L, F]));
    ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 2);
    println!("\nStage 5");
    state.add_piece(init_piece(&solution, &[L, U, B]));
    state.add_piece(init_piece(&solution, &[L, B]));
    ibb_iddfs(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!();
    println!();
    println!("whole solution: {solution_str}");
    println!("{} ETM in {:?}", solution.len(), t.elapsed());
}

/// iterative blockbuilding iddfs
fn ibb_iddfs(
    puzzle: &Puzzle,
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    solution_str: &mut String,
    min_blocks: usize,
) {
    while state.blocks.len() > min_blocks {
        println!(
            "  We currently have {} blocks. Trying to get {min_blocks} blocks ...",
            state.blocks.len()
        );
        let t = std::time::Instant::now();
        let new_twists = (min_blocks..state.blocks.len())
            .rev()
            .map_while(|expected_blocks| {
                println!("    Searching for {expected_blocks} blocks ...");
                iddfs_to_solved(puzzle, *state, expected_blocks)
            })
            .last()
            .expect("no solution found within depth");
        confirm_solution(state, solution, solution_str, new_twists, t);
    }
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
