use crate::{record, WinLossTie};
use crate::{Game, Message};
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RunStatistics {
    pub players_win_loss: Vec<(i32, i32, i32)>,
}

impl RunStatistics {
    fn new(player_count: usize) -> RunStatistics {
        RunStatistics {
            players_win_loss: vec![(0, 0, 0); player_count as usize],
        }
    }
}

#[derive(Clone)]
pub struct PlayerPlayFunction {
    pub func: &'static (dyn Fn(
        Receiver<bool>,
        Receiver<String>,
        Sender<(String, Option<HashMap<String, String>>)>,
        Option<Vec<String>>,
    ) + Sync),

    // Parameters that are sent to the player
    pub params: Option<Vec<String>>,
}

fn run_single(
    game: &mut impl Game,
    players: &Vec<PlayerPlayFunction>,
    game_id: u32,
    record_game: bool,
) -> Option<record::GameRun> {
    println!("Run {}", game_id);
    let player_count = players.len();
    // Vector of thread handles
    let mut p_threads: Vec<JoinHandle<()>> = Vec::new();
    // Vector of channels to send messages to the player
    let mut sp_message_senders: Vec<Sender<String>> = Vec::new();
    // Vector of channels to receive messages from the player
    let mut ps_message_receivers: Vec<Receiver<(String, Option<HashMap<String, String>>)>> =
        Vec::new();
    // Vector of channels to send control to the player (telling it to stop or continue)
    let mut sp_control_senders: Vec<Sender<bool>> = Vec::new();

    // For each player
    for pid in 0..player_count {
        // Create all channels between simulator and player
        let (sp_message_sender, sp_message_receiver) = channel();
        let (ps_message_sender, ps_message_receiver) = channel();
        let (sp_control_sender, sp_control_receiver) = channel();

        sp_message_senders.push(sp_message_sender);
        ps_message_receivers.push(ps_message_receiver);
        sp_control_senders.push(sp_control_sender);

        let player_func = players[pid].func;
        let player_params = players[pid].params.clone();

        let th = thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                player_func(
                    sp_control_receiver,
                    sp_message_receiver,
                    ps_message_sender,
                    player_params,
                )
            })
            .unwrap();

        p_threads.push(th);
    }

    // [RECORD] Create the game run record and fill it if record_game is True
    let mut game_run_record = record::GameRun {
        run_id: game_id,
        total_turns: 0,
        turns: Vec::new(),
        final_state: Default::default(),
        winners: Vec::new(),
    };
    /////////// [END RECORD]

    // Start the game
    let mut turn: u32 = 0;
    loop {
        let game_message = game.turn();
        match game_message {
            None => {
                break;
            }
            Some(Message {
                player_id,
                messages,
            }) => {
                sp_control_senders[player_id].send(true).unwrap();

                for msg in messages.iter() {
                    sp_message_senders[player_id].send(msg.to_string()).unwrap();
                }
                let (player_move, player_state) = ps_message_receivers[player_id].recv().unwrap();

                // [RECORD] Record game before playing the move
                if record_game {
                    let game_turn_record = record::GameTurn {
                        turn,
                        game_state: game.get_state(),
                        player: player_id as u32,
                        player_input: messages.clone(),
                        player_state: match player_state {
                            Some(state) => state,
                            None => HashMap::new(),
                        },
                        player_move: player_move.clone(),
                    };

                    game_run_record.turns.push(game_turn_record);
                }
                /////////// [END RECORD]

                turn += 1;
                game.play(player_move);
            }
        }
    }

    // Now that the game is over, terminate all player threads
    for ctrl in sp_control_senders {
        ctrl.send(false).unwrap();
    }

    // [RECORD] Record final result of game
    if record_game {
        game_run_record.total_turns = turn;
        game_run_record.winners = game.winners().unwrap();
        game_run_record.final_state = game.get_state();
    }
    /////////// [END RECORD]

    // Wait for the threads to finish
    for th in p_threads {
        let _ = th.join();
    }

    // Return Record or None
    match record_game {
        false => None,
        true => Some(game_run_record),
    }
}

pub fn run<GC, G>(
    game_constr: GC,
    players: &Vec<PlayerPlayFunction>,
    nb_runs: u32,
    record_path: Option<String>,
    return_stats: bool,
) -> Result<Option<RunStatistics>, Error>
where
    GC: Fn() -> G,
    G: Game,
{
    // [RECORD] Create Record
    let record_game = record_path.is_some();
    let mut record = record::Record {
        board_representation: G::get_board_representation(),
        game_runs: Vec::new(),
    };
    /////////// [END RECORD]

    // [STATS] Create statistics
    let mut stats = RunStatistics::new(players.len());
    /////////// [END STATS]

    for i in 0..nb_runs {
        let mut game = game_constr();
        let run_record = run_single(&mut game, players, i, record_game);

        // [RECORD] After run is over, record run
        if record_game == true {
            record.game_runs.push(run_record.unwrap());
        }
        /////////// [END RECORD]
        //
        // [STATS] After run is over, update stats
        if return_stats == true {
            let winners = game.winners().unwrap();
            for (p, r) in winners.iter().enumerate() {
                match r {
                    WinLossTie::Win => stats.players_win_loss[p].0 += 1,
                    WinLossTie::Loss => stats.players_win_loss[p].1 += 1,
                    WinLossTie::Tie => stats.players_win_loss[p].2 += 1,
                }
            }
        }
        /////////// [END STATS]
    }
    // [RECORD] After all runs are over, print record
    if record_game {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let record_file = format!("{}/record_{}.json", record_path.unwrap(), timestamp);

        let mut _file = File::create(record_file)?;
        serde_json::to_writer(_file, &record)?;
    }
    /////////// [END RECORD]

    Ok(Some(stats))
}

pub fn run_permut<GC, G>(
    game_constr: GC,
    players: &Vec<PlayerPlayFunction>,
    nb_runs: u32,
    record_path: Option<String>,
    return_stats: bool,
) -> Result<Option<RunStatistics>, Error>
where
    GC: Fn() -> G,
    G: Game,
{
    let player_count = players.len();

    let mut stats = RunStatistics::new(player_count);

    let player_ids: Vec<usize> = (0..player_count).collect();
    for perm in player_ids.iter().permutations(player_count) {
        let mut perm_players = Vec::new();
        for p in &perm {
            perm_players.push(players[**p].clone());
        }
        let result = run(
            &game_constr,
            &perm_players,
            nb_runs,
            record_path.clone(),
            return_stats,
        )
        .unwrap();

        // Update stats (by taking the correct player id from the permuted list)
        if return_stats == true {
            let stats_t = result.unwrap();
            for i in 0..player_count {
                let p = *perm[i];
                stats.players_win_loss[p].0 += stats_t.players_win_loss[i].0;
                stats.players_win_loss[p].1 += stats_t.players_win_loss[i].1;
                stats.players_win_loss[p].2 += stats_t.players_win_loss[i].2;
            }
        }
    }
    Ok(Some(stats))
}
