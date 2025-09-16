use criterion::{Criterion, criterion_group, criterion_main};

use blockybuildy::*;
use itertools::Itertools;
use rand_pcg::Pcg64Mcg;

fn exec_moves(init_state: PuzzleState, twists: &[Twist]) -> PuzzleState {
    twists
        .iter()
        .fold(init_state, |state, &twist| state.do_twist(twist).unwrap())
}

fn do_and_undo(twists: Vec<Twist>) -> Vec<Twist> {
    twists.iter().chain(twists.iter().rev()).copied().collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    let puzzle = &*RUBIKS_4D;
    let mut rng = Pcg64Mcg::new(0x_5c8d9681_fb970317_36ec6b11_2fa3c0bd);

    let move_count = 32;
    let gen_random_moves = move || puzzle.random_moves(&mut rng, move_count).collect_vec();

    let puzzle_with_2x2x2x2_block = PuzzleState::default()
        .add_block_with_setup_moves(puzzle, &[], Block::new_solved([], [R, U, F, O]).unwrap())
        .unwrap();

    for (init_state, name) in [(puzzle_with_2x2x2x2_block, "2x2x2x2 block")] {
        c.bench_function(&format!("do moves on {name}"), |b| {
            let gen_random_moves = gen_random_moves.clone();
            b.iter_batched(
                gen_random_moves.clone(),
                |input| exec_moves(init_state, &input),
                criterion::BatchSize::SmallInput,
            );
        });

        c.bench_function(&format!("do & undo moves on {name}"), |b| {
            let mut gen_random_moves = gen_random_moves.clone();
            b.iter_batched(
                || do_and_undo(gen_random_moves()),
                |input| exec_moves(init_state, &input),
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
