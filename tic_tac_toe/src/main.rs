mod game_tic_tac_toe;
use game_tic_tac_toe::TicTacToeGame;
mod player_mcts_6;
mod player_random;
use common::simulator;
use common::simulator::PlayerPlayFunction;
use common::Game;

use std::time::Instant;

#[allow(unused_must_use)]

fn main() {
    const STATS: bool = true;
    const RECORD: bool = false;
    const RUNS: u32 = 2;

    let record_path = "C:/Users/hhour/Desktop/codingame-rust/tic_tac_toe/output";

    let players: Vec<PlayerPlayFunction> = vec![
        PlayerPlayFunction {
            func: &player_mcts_6::play,
            params: None,
        },
        PlayerPlayFunction {
            func: &player_mcts_6::play,
            params: None,
        },
    ];

    let start = Instant::now();

    let result = simulator::run(
        TicTacToeGame::new,
        &players,
        RUNS,
        match RECORD {
            true => Some(record_path.to_string()),
            false => None,
        },
        STATS,
    );

    let duration = start.elapsed();
    println!("Ran {} games in {:?}", RUNS * 2, duration);

    if STATS == true {
        let stats = result.unwrap().unwrap();
        println!("Win statistics : {:?}", stats.players_win_loss);
    }
}
