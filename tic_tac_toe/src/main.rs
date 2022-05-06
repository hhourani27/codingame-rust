mod game_tic_tac_toe;
use game_tic_tac_toe::TicTacToeGame;
mod player;
use common::simulator;
use common::Game;

use std::time::Instant;

#[allow(unused_must_use)]

fn main() {
    const STATS: bool = true;
    const RECORD: bool = false;
    const RUNS: u32 = 500;

    let players = vec![player::play, player::play];
    let record_path = "C:/Users/hhour/Desktop/codingame-rust/tic_tac_toe/output";

    let start = Instant::now();

    let result = simulator::run_permut(
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
