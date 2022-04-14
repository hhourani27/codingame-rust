use crate::Game;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub fn run(
    //    game: &impl Game,
    players: Vec<impl Fn(Receiver<String>, Sender<String>) + Send + Sync + Copy + 'static>,
) {
    let player_count = players.len();
    let mut simulator_to_player_senders: Vec<Sender<String>> = Vec::new();
    let mut player_to_simulator_receivers: Vec<Receiver<String>> = Vec::new();

    for pid in 0..player_count {
        let (simulator_to_player_sender, simulator_to_player_receiver) = channel();
        let (player_to_simulator_sender, player_to_simulator_receiver) = channel();

        simulator_to_player_senders.push(simulator_to_player_sender);
        player_to_simulator_receivers.push(player_to_simulator_receiver);

        let player_func = players[pid];

        thread::spawn(move || {
            player_func(simulator_to_player_receiver, player_to_simulator_sender)
        });
    }

    for pid in 0..player_count {
        println!("[Simulator] Sending message to Player {}", pid);
        simulator_to_player_senders[pid]
            .send(format!("Message from Simulator : Hello Player {}", pid))
            .unwrap();

        let msg = player_to_simulator_receivers[pid].recv().unwrap();
        println!(
            "[Simulator] Received this message from Player {}: <{}>",
            pid, msg
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator() {
        fn hello(rcv: Receiver<String>, snd: Sender<String>) {
            let msg_from_simulator = rcv.recv().unwrap();
            println!(
                "[Player] Received message from Simulator: <{}>",
                msg_from_simulator
            );

            println!("[Player] Sending message to Simulator");
            snd.send(format!("Message from Player : Hello Simulator"))
                .unwrap();
        }
        let players = vec![hello, hello];
        run(players);
    }
}
