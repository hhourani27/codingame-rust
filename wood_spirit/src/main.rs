mod game_wood_spirit;
use common::simulator;
use common::simulator::PlayerPlayFunction;
use common::Game;
use std::time::Instant;

fn run() {
    /*
    const STATS: bool = true;
    const RECORD: bool = false;
    const RUN_PERMUT: bool = false;
    const RUNS: u32 = 10;

    let record_path = "C:/Users/hhour/Desktop/codingame-rust/fall_2020_witches_brew/output";

    let players: Vec<PlayerPlayFunction> = vec![
        PlayerPlayFunction {
            func: &player_beam_5::play,
            params: None,
        },
        PlayerPlayFunction {
            func: &player_beam_4::play,
            params: None,
        },
    ];

    let start = Instant::now();

    let result;
    if RUN_PERMUT == true {
        result = simulator::run_permut(
            WitchesBrewGame::new,
            &players,
            RUNS,
            match RECORD {
                true => Some(record_path.to_string()),
                false => None,
            },
            STATS,
        );
    } else {
        result = simulator::run(
            WitchesBrewGame::new,
            &players,
            RUNS,
            match RECORD {
                true => Some(record_path.to_string()),
                false => None,
            },
            STATS,
        );
    }

    let duration = start.elapsed();
    println!(
        "Ran {} games in {:?}",
        match RUN_PERMUT {
            true => RUNS * 2,
            false => RUNS,
        },
        duration
    );

    if STATS == true {
        let stats = result.unwrap().unwrap();
        println!("Win statistics : {:?}", stats.players_win_loss);
    }
    */
}

fn main() {
    println!("Hello");
    run();
}
