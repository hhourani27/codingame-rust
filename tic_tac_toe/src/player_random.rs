use rand::seq::SliceRandom;
use std::sync::mpsc::{Receiver, Sender};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(ctr_rcv: Receiver<bool>, msg_rcv: Receiver<String>, msg_snd: Sender<String>) {
    // game loop
    while ctr_rcv.recv().unwrap() == true {
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opponent_row = parse_input!(inputs[0], i32);
        let opponent_col = parse_input!(inputs[1], i32);

        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let valid_action_count = parse_input!(input_line, i32);

        let mut valid_actions: Vec<(i32, i32)> = Vec::new();
        for i in 0..valid_action_count as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let row = parse_input!(inputs[0], i32);
            let col = parse_input!(inputs[1], i32);
            valid_actions.push((row, col));
        }

        let chosen_move = valid_actions.choose(&mut rand::thread_rng()).unwrap();
        msg_snd.send(format!("{} {}", chosen_move.0, chosen_move.1));
    }
}
