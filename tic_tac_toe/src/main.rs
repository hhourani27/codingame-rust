mod game_tic_tac_toe;
use game_tic_tac_toe::TicTacToeGame;
mod player;
use common::simulator;
use common::Game;

use std::time::{Duration, Instant};

#[allow(unused_must_use)]

fn main() {
    let players = vec![player::play, player::play];

    let record_path = "C:/Users/hhour/Desktop/codingame-rust/tic_tac_toe/output";

    //let start = Instant::now();

    simulator::run(
        TicTacToeGame::new,
        &players,
        10,
        Some(record_path.to_string()),
        //None,
    );

    //let duration = start.elapsed();
    //println!("Time elapsed is: {:?}", duration);
}
