pub mod sim;
pub mod stackvec;

use itertools::Itertools;
pub use sim::*;
pub use stackvec::StackVec;

pub const USE_3D_TWIST_NAMES: bool = true;
pub const MAX_BLOCKS: usize = 6; // 5 is a nice number

const SCRAMBLE: &str = "U2 B2 R2 D' F2 U' L2 U2 L2 U R2 U R F R U2 R F' U' B' U2";

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
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 2);
    iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!("\nStage 2");
    state.add_piece(init_piece(&solution, &[F]));
    state.add_piece(init_piece(&solution, &[F, U]));
    state.add_piece(init_piece(&solution, &[F, U, R]));
    state.add_piece(init_piece(&solution, &[F, R]));
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 4);
    iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 3);
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 2);
    iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!("\nStage 3");
    state.add_piece(init_piece(&solution, &[B]));
    state.add_piece(init_piece(&solution, &[B, U]));
    state.add_piece(init_piece(&solution, &[B, U, R]));
    state.add_piece(init_piece(&solution, &[B, R]));
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 4);
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 3);
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 2);
    iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!("\nStage 4");
    state.add_piece(init_piece(&solution, &[L]));
    state.add_piece(init_piece(&solution, &[L, U]));
    state.add_piece(init_piece(&solution, &[L, U, F]));
    state.add_piece(init_piece(&solution, &[L, F]));
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 4);
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 3);
    iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 2);
    println!("\nStage 5");
    state.add_piece(init_piece(&solution, &[L, U, B]));
    state.add_piece(init_piece(&solution, &[L, B]));
    iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 3);
    // iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 2);
    iddfs_to_solved(&RUBIKS_3D, &mut state, &mut solution, &mut solution_str, 1);
    println!();
    println!();
    println!("whole solution: {solution_str}");
    println!("{} ETM in {:?}", solution.len(), t.elapsed());
}

fn iddfs_to_solved(
    puzzle: &Puzzle,
    state: &mut PuzzleState,
    solution: &mut Vec<Twist>,
    solution_str: &mut String,
    expected_blocks: usize,
) {
    println!(
        "  We currently have {} blocks. Trying to get {expected_blocks} ...",
        state.blocks.len()
    );
    for max_depth in 0.. {
        let t = std::time::Instant::now();
        println!("    Searching at depth {max_depth} ...");
        let mut new_twists = vec![];
        if dfs_to_solved(puzzle, *state, &mut new_twists, expected_blocks, max_depth) {
            solution_str.push_str(&format!(" ({})", new_twists.iter().join(" ")));
            for new_twist in new_twists {
                *state = state.do_twist(new_twist).unwrap();
                solution.push(new_twist);
            }
            let elapsed = t.elapsed();
            println!(
                "    Solution found in {elapsed:?}! We now have {}",
                state.blocks.len()
            );
            println!("  Solution so far: {}", solution.iter().join(" "));
            return;
        }
    }
    unreachable!()
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
