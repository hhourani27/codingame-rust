use common::simulator;
use common::simulator::PlayerPlayFunction;
use common::Game;
use std::time::Instant;
mod game_wood_spirit;
mod player_mcts_2;
mod player_mcts_3;
mod player_random;
use game_wood_spirit::WoodSpiritGame;

fn run() {
    const STATS: bool = true;
    const RECORD: bool = false;
    const RUN_PERMUT: bool = true;
    const RUNS: u32 = 10;

    let record_path = "C:/Users/hhour/Desktop/codingame-rust/wood_spirit/output";

    let players: Vec<PlayerPlayFunction> = vec![
        PlayerPlayFunction {
            func: &player_mcts_3::play,
            params: None,
        },
        PlayerPlayFunction {
            func: &player_mcts_2::play,
            params: None,
        },
    ];

    let start = Instant::now();

    let result;
    if RUN_PERMUT == true {
        result = simulator::run_permut(
            WoodSpiritGame::new,
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
            WoodSpiritGame::new,
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
}

fn main() {
    run();
}
