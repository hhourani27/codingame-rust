use crate::{Game, Message};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub fn run_single(
    game: &mut impl Game,
    players: &Vec<
        impl Fn(Receiver<bool>, Receiver<String>, Sender<String>) + Send + Sync + Copy + 'static,
    >,
) {
    let player_count = players.len();
    // Vector of thread handles
    let mut p_threads: Vec<JoinHandle<()>> = Vec::new();
    // Vector of channels to send messages to the player
    let mut sp_message_senders: Vec<Sender<String>> = Vec::new();
    // Vector of channels to receive messages from the player
    let mut ps_message_receivers: Vec<Receiver<String>> = Vec::new();
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

        // Start player thread
        let player_func = players[pid];
        let th = thread::spawn(move || {
            player_func(sp_control_receiver, sp_message_receiver, ps_message_sender)
        });

        p_threads.push(th);
    }

    // Start the game
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

                let msg = ps_message_receivers[player_id].recv().unwrap();
                game.play(msg);
            }
        }
    }

    // Now that the game is over, terminate all player threads
    for ctrl in sp_control_senders {
        ctrl.send(false).unwrap();
    }

    // Wait for the threads to finish
    for th in p_threads {
        let _ = th.join();
    }
}

pub fn run<N, G>(
    game_constr: N,
    players: &Vec<
        impl Fn(Receiver<bool>, Receiver<String>, Sender<String>) + Send + Sync + Copy + 'static,
    >,
    nb_runs: u32,
) where
    N: Fn() -> G,
    G: Game,
{
    for i in 0..nb_runs {
        let mut game = game_constr();
        run_single(&mut game, players);

        println!("Winner : {:?}", game.winners());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /*    #[test]
    fn test_simulator() {
        fn hello(ctr_rcv: Receiver<bool>, msg_rcv: Receiver<String>, msg_snd: Sender<String>) {
            while ctr_rcv.recv().unwrap() == true {
                let msg_from_simulator = msg_rcv.recv().unwrap();
                println!(
                    "[Player] Received message from Simulator: <{}>",
                    msg_from_simulator
                );

                println!("[Player] Sending message to Simulator");
                msg_snd
                    .send(format!("Message from Player : Hello Simulator"))
                    .unwrap();
            }
        }
        let players = vec![hello, hello];
        run(&players, 3);
    }
    */
}
