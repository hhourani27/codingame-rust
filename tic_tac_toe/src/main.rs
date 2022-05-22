mod game_tic_tac_toe;
use game_tic_tac_toe::TicTacToeGame;
mod player_mcts_6_param;
use common::simulator;
use common::simulator::PlayerPlayFunction;
use common::Game;
use itertools::Itertools;
use std::collections::HashMap;

use std::time::Instant;

#[allow(unused_must_use)]

fn main() {
    //run_games();
    try_parameters();
}

fn run_games() {
    const STATS: bool = true;
    const RECORD: bool = false;
    const RUNS: u32 = 2;

    let record_path = "C:/Users/hhour/Desktop/codingame-rust/tic_tac_toe/output";

    let players: Vec<PlayerPlayFunction> = vec![
        PlayerPlayFunction {
            func: &player_mcts_6_param::play,
            params: {
                let mut p: HashMap<String, String> = HashMap::new();
                p.insert(String::from("Exploration coef"), String::from("0.4"));
                Some(p)
            },
        },
        PlayerPlayFunction {
            func: &player_mcts_6_param::play,
            params: {
                let mut p: HashMap<String, String> = HashMap::new();
                p.insert(String::from("Exploration coef"), String::from("0.4"));
                Some(p)
            },
        },
    ];

    /* End prepare players */
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

fn try_parameters() {
    const RUNS_PER_EVAL: u32 = 1;

    /* Prepare parameters */
    let mut exploration_coefs: Vec<f32> = Vec::new();
    for i in 0..20 {
        exploration_coefs.push(0.1 + i as f32 * 0.1);
    }

    let win_scores: Vec<String> = vec![String::from("1.0,0.0,0.5"), String::from("1.0,-1.0,0.0")];

    /* Prepare players */
    let mut players: Vec<PlayerPlayFunction> = Vec::new();
    for ec in exploration_coefs.iter() {
        for ws in win_scores.iter() {
            players.push(PlayerPlayFunction {
                func: &player_mcts_6_param::play,
                params: {
                    let mut p: HashMap<String, String> = HashMap::new();
                    p.insert(String::from("Exploration coef"), ec.to_string());
                    p.insert(String::from("WinLossTie scores"), ws.clone());
                    Some(p)
                },
            })
        }
    }

    let mut final_results: Vec<f32> = vec![0.0; players.len()];

    let player_indices: Vec<usize> = (0..players.len()).collect();

    for selected_players in player_indices.iter().combinations(2) {
        let player0_idx = *selected_players[0];
        let player1_idx = *selected_players[1];

        let player0 = players[player0_idx].clone();
        let player1 = players[player1_idx].clone();

        println!("Run game between players {} & {}", player0_idx, player1_idx);
        let result = simulator::run_permut(
            TicTacToeGame::new,
            &vec![player0, player1],
            RUNS_PER_EVAL,
            None,
            true,
        );

        let result = result.unwrap().unwrap().players_win_loss;

        final_results[player0_idx] += result[0].0 as f32 + 0.5 * result[0].2 as f32;
        final_results[player1_idx] += result[1].0 as f32 + 0.5 * result[1].2 as f32;
    }

    println!("Final results");
    for i in player_indices {
        println! {"Player: {} & {} : {}",
            players[i].params.as_ref().unwrap().get("Exploration coef").unwrap(),
            players[i].params.as_ref().unwrap().get("WinLossTie scores").unwrap(),
            final_results[i]
        };
    }
}
