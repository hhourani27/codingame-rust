use std::sync::mpsc::{Receiver, Sender};

use rand::prelude::SliceRandom;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
pub fn play(
    ctr_rcv: Receiver<bool>,
    msg_rcv: Receiver<String>,
    msg_snd: Sender<(String, Option<std::collections::HashMap<String, String>>)>,
    params: Option<Vec<String>>,
) {
    let p_id = params.unwrap()[0].parse::<u8>().unwrap();

    let mut input_line = String::new();
    input_line = msg_rcv.recv().unwrap();
    eprintln!("[P{}] {}", p_id, &input_line);
    let number_of_cells = parse_input!(input_line, i32); // 37
    for i in 0..number_of_cells as usize {
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        eprintln!("[P{}] {}", p_id, &input_line);
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let index = parse_input!(inputs[0], i32); // 0 is the center cell, the next cells spiral outwards
        let richness = parse_input!(inputs[1], i32); // 0 if the cell is unusable, 1-3 for usable cells
        let neigh_0 = parse_input!(inputs[2], i32); // the index of the neighbouring cell for each direction
        let neigh_1 = parse_input!(inputs[3], i32);
        let neigh_2 = parse_input!(inputs[4], i32);
        let neigh_3 = parse_input!(inputs[5], i32);
        let neigh_4 = parse_input!(inputs[6], i32);
        let neigh_5 = parse_input!(inputs[7], i32);
    }

    // game loop
    while ctr_rcv.recv().unwrap() == true {
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        eprintln!("[P{}] {}", p_id, &input_line);
        let day = parse_input!(input_line, i32); // the game lasts 24 days: 0-23
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        eprintln!("[P{}] {}", p_id, &input_line);
        let nutrients = parse_input!(input_line, i32); // the base score you gain from the next COMPLETE action
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        eprintln!("[P{}] {}", p_id, &input_line);
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let sun = parse_input!(inputs[0], i32); // your sun points
        let score = parse_input!(inputs[1], i32); // your current score
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        eprintln!("[P{}] {}", p_id, &input_line);
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opp_sun = parse_input!(inputs[0], i32); // opponent's sun points
        let opp_score = parse_input!(inputs[1], i32); // opponent's score
        let opp_is_waiting = parse_input!(inputs[2], i32); // whether your opponent is asleep until the next day
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        eprintln!("[P{}] {}", p_id, &input_line);
        let number_of_trees = parse_input!(input_line, i32); // the current amount of trees
        for i in 0..number_of_trees as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            eprintln!("[P{}] {}", p_id, &input_line);
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let cell_index = parse_input!(inputs[0], i32); // location of this tree
            let size = parse_input!(inputs[1], i32); // size of this tree: 0-3
            let is_mine = parse_input!(inputs[2], i32); // 1 if this is your tree
            let is_dormant = parse_input!(inputs[3], i32); // 1 if this tree is dormant
        }
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        eprintln!("[P{}] {}", p_id, &input_line);
        let number_of_possible_actions = parse_input!(input_line, i32); // all legal actions
        let mut possible_actions: Vec<String> = Vec::new();
        for i in 0..number_of_possible_actions as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            eprintln!("[P{}] {}", p_id, &input_line);
            let possible_action = input_line.trim_matches('\n').to_string(); // try printing something from here to start with
            possible_actions.push(possible_action);
        }

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");

        let chosen_move = possible_actions.choose(&mut rand::thread_rng()).unwrap();
        eprintln!("[P{}] Sending move {}", p_id, chosen_move);
        msg_snd.send((format!("{}", chosen_move), None));
    }
}
