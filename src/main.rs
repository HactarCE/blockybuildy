use std::collections::HashSet;

use itertools::Itertools;
use robodoan::*;

fn main() {
    // let scramble = scramble_3d();
    // // let scramble = "U' B2 U2 L2 B2 U2 L F2 L' U2 R2 F2 L' B L B D' F' D2 L' B";
    // let max_depth = 5;
    // search_3d(&scramble, max_depth, 0);
    // search_3d(&scramble, max_depth, 1);
    // search_3d(&scramble, max_depth, 2);
    // let max_depth = 6;
    // search_3d(&scramble, max_depth, 0);
    // search_3d(&scramble, max_depth, 1);
    // search_3d(&scramble, max_depth, 2);

    // let mut results = vec![];
    // for i in 0..10 {
    //     let scramble = scramble_4d();
    //     println!("\n\n---- STARTING SEARCH #{} ----\n", i + 1);
    //     results.push(search_4d(&scramble));
    // }
    // println!("\n\n---- RESULTS ----\n");
    // for (move_count, time) in results {
    //     println!("{move_count} ETM in {time:?}");
    // }

    // let scramble = scramble_4d();
    let scramble = "LF IB2 IDFR LF RBI ID ODFL BUO IBL BR2 OUF BLO IDFL OB FI LD RU2 DFLI FUO IU2 OUBR BD IDFL OUB LDFO BDO FUL IR IUL OL2 LDBI FL BL IU LI2 ODFR OB2 OUF DFLI RI LO RF RB LD IDBL UBRO LDFI FULI FI2 OUF ODFL UFRI LU DL FU LDBO DFRI OB LD UI FLI FO IF IFL DFLI LD FD DBO RUBI DO FD IDFL UBI LUBI BURO BDRI BU BD2 RDBI UBL DB2 LO LDBO OL2 RF BDO ULI UFLO BR2 LB IL DFRI DFR FUI ULI FL IU UBRI LO BURI";
    println!("Scramble: {scramble}");
    println!();
    // // let scramble = "IDFL LF UBI IUB IL2 RUBO IUBR DO DLO IUBL UI2 ID FULI LUBO OUF DFRI DL BULO FLI LUBI FR BLO UBO UBRO OUFL IU RFI BR2 RI ID IB2 OUBL RUBO DFLO DFRO BO BDL UFRO OUFL ID BURO IBL OUL BD IBL DBLI FDLO BUO DBLO FLO LDF BRO ID FLI IUF OL OL OBL ULO BUO FURI UL2 BR FDLO IUBL UBLO IBL BR2 IUBL BURO IU2 DF ODBL LDBO BR RUB LUBI IU2 FURI IUL OUL ODBR OL2 IB DL UB BO OUB FL2 DFRO ODBR UFRI RI RUFI RDBO IUR UBO BUL RDFI LUFO";
    dbg!(search_4d(&scramble));
}

fn search_4d(scramble_str: &str) -> (usize, std::time::Duration) {
    let t0 = std::time::Instant::now();

    let mut search = BlockBuildingSearch {
        puzzle: &RUBIKS_4D,
        params: BlockBuildingSearchParams {
            heuristic: Heuristic::Fast,
            max_depth: 4,
            parallel_depth: 2,
            force_parallel_determinism: true,
            trim: 0,
            verbosity: 1,
        },
        scramble: parse_twists(scramble_str),
        state: PuzzleState::default(),
        solution: vec![],
        solution_segments: vec![],
        parenthesize_segments: false,
    };

    // Mid + left block
    let block_2222 = search.solve_any_candidate_block("2x2x2x2", 1, &initial_4d_blocks());
    let block_2223 =
        search.solve_any_candidate_block("2x2x2x3", 1, &extensions(block_2222, GripSet::ALL, true));
    let block_2233 =
        search.solve_any_candidate_block("2x2x3x3", 1, &extensions(block_2223, GripSet::ALL, true));

    let right_block_candidate_grips = block_2233.inactive_grips();
    let [rbcg1, rbcg2] = right_block_candidate_grips.iter().collect_array().unwrap();
    let mut mutually_adjacent_axes: HashSet<usize> = HashSet::from_iter([0, 1, 2, 3]);
    mutually_adjacent_axes.remove(&rbcg1.axis());
    mutually_adjacent_axes.remove(&rbcg2.axis());
    let mutually_adjacent_axes = mutually_adjacent_axes.into_iter().collect_vec();
    let grips_within_right_block =
        GripSet::ALL - rbcg1 - rbcg2 - rbcg1.opposite() - rbcg2.opposite();
    let right_block_1222_candidates =
        itertools::iproduct!([(rbcg1, rbcg2), (rbcg2, rbcg1)], [0, 1], [0, 1])
            .map(|((right_block_grip, last_layer_grip), bit0, bit1)| {
                Block::new_solved(
                    [right_block_grip],
                    [
                        last_layer_grip,
                        GripId::new((mutually_adjacent_axes[0] << 1) as u8 | bit0),
                        GripId::new((mutually_adjacent_axes[1] << 1) as u8 | bit1),
                    ],
                )
                .unwrap()
            })
            .collect_vec();

    let right_block_1222 =
        search.solve_any_candidate_block("right 1x2x2x2", 2, &right_block_1222_candidates);

    let (last_f2l_grip, last_layer_grip) =
        if right_block_1222.grip_status(rbcg1) == Some(GripStatus::Active) {
            (rbcg1, rbcg2)
        } else {
            (rbcg2, rbcg1)
        };

    let right_block_1223 = search.solve_any_candidate_block(
        "right 1x2x2x3",
        2,
        &extensions(right_block_1222, grips_within_right_block, true),
    );

    let last_f2l_grip_2 =
        (right_block_1223.inactive_grips() - last_layer_grip - last_f2l_grip.opposite())
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

    search.solve_any_candidate_block("last F2L-A pair", 3, &[last_f2l_a_pair]);
    search.solve_any_candidate_block(
        "second-to-last F2L-b pair",
        3,
        &f2l_b_grips.map(|g| last_f2l_a_pair.expand_to_active_grip(g)),
    );
    let f2l_block = Block::new_solved([], [last_layer_grip]).unwrap();
    search.solve_any_candidate_block("final F2L-b pair", 1, &[f2l_block]);

    println!();
    println!("Whole search completed in {:?}", t0.elapsed());
    println!("Best solution found is {} ETM", search.solution.len());
    println!("{}", search.solution_str());

    (search.solution.len(), t0.elapsed())
}

fn scramble_3d() -> String {
    RUBIKS_3D
        .random_moves(&mut rand::rng(), 100)
        .map(|t| t.to_string())
        .join(" ")
}

fn scramble_4d() -> String {
    RUBIKS_4D
        .random_moves(&mut rand::rng(), 100)
        .map(|t| t.to_string())
        .join(" ")
}

// fn search_3d(scramble: &str, max_depth: usize, trim: usize) {
//     let scramble_twists = parse_twists(scramble);
//     let init_piece = |solution: &[Twist], piece: Piece| {
//         let all_twists = scramble_twists.iter().chain(solution);
//         all_twists.fold(piece, |p, &twist| twist * p)
//     };
//     let add_pieces = |state: &mut PuzzleState, solution: &[Twist], pieces: Vec<Piece>| {
//         for piece in pieces {
//             state.add_piece(init_piece(&solution, piece));
//         }
//     };

//     let solution = vec![];
//     let solution_str = String::new();

//     let t = std::time::Instant::now();

//     let (solution, solution_str) = initial_3d_petrus_blocks()
//         .into_par_iter()
//         .map(|block1| {
//             let mut solution = solution.clone();
//             let mut solution_str = solution_str.clone();
//             let mut state = PuzzleState::default();
//             let new_pieces = pieces_from_block(block1).collect();
//             add_pieces(&mut state, &solution, new_pieces);
//             if !ibb_iddfs(
//                 &RUBIKS_3D,
//                 &mut state,
//                 &mut solution,
//                 &mut solution_str,
//                 1,
//                 max_depth,
//                 trim,
//             ) {
//                 // println!("  Abandoning 2x2x2 block attempt");
//                 return None;
//             }

//             extensions(block1, GripSet::from_iter(CUBE_GRIPS), true)
//                 .into_par_iter()
//                 .map(|block2| {
//                     let mut solution = solution.clone();
//                     let mut solution_str = solution_str.clone();
//                     let mut state = state.clone();
//                     let new_pieces = new_pieces_from_block(block2, block1);
//                     add_pieces(&mut state, &solution, new_pieces);
//                     if !ibb_iddfs(
//                         &RUBIKS_3D,
//                         &mut state,
//                         &mut solution,
//                         &mut solution_str,
//                         1,
//                         max_depth,
//                         trim,
//                     ) {
//                         // println!("  Abandoning 2x2x3 block attempt");
//                         return None;
//                     }

//                     extensions(block2, GripSet::from_iter(CUBE_GRIPS), true)
//                         .into_par_iter()
//                         .map(|block3| {
//                             let mut solution = solution.clone();
//                             let mut solution_str = solution_str.clone();
//                             let mut state = state.clone();
//                             let new_pieces = new_pieces_from_block(block3, block2);
//                             add_pieces(&mut state, &solution, new_pieces);
//                             if !ibb_iddfs(
//                                 &RUBIKS_3D,
//                                 &mut state,
//                                 &mut solution,
//                                 &mut solution_str,
//                                 1,
//                                 max_depth,
//                                 trim,
//                             ) {
//                                 // println!("  Abandoning 2x3x3 block attempt");
//                                 return None;
//                             }

//                             Some((solution, solution_str))
//                         })
//                         .filter_map(|x| x)
//                         .min_by_key(|(solution, _)| solution.len())
//                 })
//                 .filter_map(|x| x)
//                 .min_by_key(|(solution, _)| solution.len())
//         })
//         .filter_map(|x| x)
//         .min_by_key(|(solution, _)| solution.len())
//         .expect("no solution");

//     println!();
//     println!();
//     println!("whole solution: {solution_str}");
//     println!("{} ETM in {:?}", solution.len(), t.elapsed());
// }

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

fn parse_twists(s: &str) -> Vec<Twist> {
    s.split_whitespace()
        .map(|word| TWISTS_FROM_NAME[word])
        .collect()
}
