use std::error::Error;

use itertools::Itertools;
use robodoan::*;

fn main() -> Result<(), Box<dyn Error>> {
    let profile = Profile::Fast;

    PuzzleState::default().do_twist(Twist::new(R, YZ));

    if let Some(filename) = std::env::args().nth(1) {
        let log_file_text = std::fs::read_to_string(&filename)?;
        let scramble: mc4d::Mc4dScramble = log_file_text.parse()?;
        println!("Loaded log file from {filename}");
        println!();
        // let (solve_twists, _elapsed_time) = search_4d(scramble.scramble());
        let solve_twists = robodoan::Solver::new(profile, scramble.scramble()).solve();
        println!();
        std::fs::write("out.log", scramble.to_string(false, solve_twists))?;
        return Ok(());
    }

    let mut results = vec![];
    for i in 0..1 {
        let scramble = RUBIKS_4D.random_moves(&mut rand::rng(), 100);
        println!("\n\n---- STARTING SEARCH #{} ----\n", i + 1);
        println!("Scramble: {}", scramble.iter().join(" "));
        let t = std::time::Instant::now();
        let solution = robodoan::Solver::new(profile, scramble).solve();
        results.push((solution.len(), t.elapsed()));
    }
    println!("\n\n---- RESULTS ----\n");
    for (move_count, time) in results {
        println!("{move_count} ETM in {time:?}");
    }

    // let scramble = parse_twists(
    //     "LF IB2 IDFR LF RBI ID ODFL BUO IBL BR2 OUF BLO IDFL OB FI LD RU2 DFLI FUO IU2 OUBR BD IDFL OUB LDFO BDO FUL IR IUL OL2 LDBI FL BL IU LI2 ODFR OB2 OUF DFLI RI LO RF RB LD IDBL UBRO LDFI FULI FI2 OUF ODFL UFRI LU DL FU LDBO DFRI OB LD UI FLI FO IF IFL DFLI LD FD DBO RUBI DO FD IDFL UBI LUBI BURO BDRI BU BD2 RDBI UBL DB2 LO LDBO OL2 RF BDO ULI UFLO BR2 LB IL DFRI DFR FUI ULI FL IU UBRI LO BURI",
    // );
    // println!("Scramble: {}", scramble.iter().join(" "));
    // println!();
    // robodoan::Solver::new(profile, scramble).solve();

    Ok(())
}

pub fn parse_twists(s: &str) -> Vec<Twist> {
    s.split_whitespace()
        .map(|word| TWISTS_FROM_NAME[word])
        .collect()
}
