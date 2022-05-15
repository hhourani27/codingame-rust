mod game_tic_tac_toe;
use game_tic_tac_toe::TicTacToeGame;
mod player_mcts_1;
mod player_mcts_2;
mod player_random;
use common::simulator;
use common::Game;

use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

#[allow(unused_must_use)]

fn main() {
    const STATS: bool = true;
    const RECORD: bool = true;
    const RUNS: u32 = 20;

    let players: Vec<&'static (dyn Fn(Receiver<bool>, Receiver<String>, Sender<String>) + Sync)> =
        vec![&player_mcts_2::play, &player_mcts_1::play];

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
