mod game_tic_tac_toe;
use game_tic_tac_toe::TicTacToeGame;
mod player_mcts_6_param;
use common::simulator;
use common::Game;
use std::collections::HashMap;

use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

#[allow(unused_must_use)]

fn main() {
    const STATS: bool = true;
    const RECORD: bool = false;
    const RUNS: u32 = 1;

    let players: Vec<
        &'static (dyn Fn(
            Receiver<bool>,
            Receiver<String>,
            Sender<String>,
            Option<HashMap<String, String>>,
        ) + Sync),
    > = vec![&player_mcts_6_param::play, &player_mcts_6_param::play];

    let record_path = "C:/Users/hhour/Desktop/codingame-rust/tic_tac_toe/output";

    /* Player parameters */
    let mut params: Vec<HashMap<String, String>> = Vec::new();
    for _ in 0..2 {
        let mut p_params: HashMap<String, String> = HashMap::new();
        p_params.insert(String::from("Exploration coef"), String::from("0.4"));
        p_params.insert(String::from("Win-Lose score"), String::from("1.0 0.0"));

        params.push(p_params);
    }
    let params = Some(params);
    /* End player parameters */

    let start = Instant::now();

    let result = simulator::run_permut(
        TicTacToeGame::new,
        &players,
        &params,
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
