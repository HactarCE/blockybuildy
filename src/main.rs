use itertools::Itertools;

pub mod sim;

pub use sim::*;

pub const USE_3D_TWIST_NAMES: bool = false;

fn main() {
    // for t in &RUBIKS_3D.twists {
    //     println!("{t}");
    // }

    let edge_ufr = Piece::new_solved([U, F]);
    let corner_ufro = Piece::new_solved([U, F, R]);
    let corner_ufri = Piece::new_solved([U, F, L]);

    let mut state = PuzzleState {
        pieces: vec![edge_ufr, corner_ufro, corner_ufri],
    };

    for m in RUBIKS_3D.random_moves(100) {
        state.do_move(m);
    }

    println!("{state}");

    let t = std::time::Instant::now();

    let solution = iddfs_to_solved(
        &RUBIKS_3D,
        [state.pieces[0], state.pieces[1], state.pieces[2]],
    );
    for t in solution {
        println!("{t}");
    }

    dbg!(t.elapsed());
}

fn iddfs_to_solved<const N: usize>(puzzle: &Puzzle, pieces: [Piece; N]) -> Vec<Twist> {
    for max_depth in 1.. {
        let mut solution = vec![];
        if dfs_to_solved(puzzle, pieces, &mut solution, max_depth) {
            return solution;
        }
    }
    unreachable!()
}

fn dfs_to_solved<const N: usize>(
    puzzle: &Puzzle,
    pieces: [Piece; N],
    solution: &mut Vec<Twist>,
    max_depth: usize,
) -> bool {
    if max_depth == 0 {
        return false;
    }
    if pieces.map(|p| p.attitude).iter().all_equal() {
        return true;
    }
    for &twist in &puzzle.twists {
        if !pieces
            .map(|p| p.grips.contains(twist.grip))
            .iter()
            .all_equal()
        {
            solution.push(twist);
            if dfs_to_solved(puzzle, pieces.map(|p| twist * p), solution, max_depth - 1) {
                return true;
            }
            solution.pop();
        }
    }
    false
}
