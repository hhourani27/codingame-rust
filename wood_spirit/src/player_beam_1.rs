use std::sync::mpsc::{Receiver, Sender};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

mod game {
    use std::fmt;

    const MAX_VALID_MOVES: usize = 4 + 1;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Move {
        GROW(u8),
        COMPLETE(u8),
        WAIT,
    }

    impl fmt::Display for Move {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Move::GROW(i) => write!(f, "GROW {}", i),
                Move::COMPLETE(i) => write!(f, "COMPLETE {}", i),
                Move::WAIT => write!(f, "WAIT"),
            }
        }
    }

    impl Default for Move {
        fn default() -> Self {
            Move::WAIT
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Tree {
        SMALL_TREE,
        MEDIUM_TREE,
        LARGE_TREE,
    }

    enum SoilRichness {
        LOW_QUALITY,
        MEDIUM_QUALITY,
        HIGH_QUALITY,
    }

    #[derive(Clone, Copy)]
    pub struct Cell {
        pub player: u8,
        pub tree: Tree,
        pub is_dormant: bool,
    }

    #[derive(Clone, Copy)]
    pub struct Player {
        pub sun: u32,
        pub score: u32,

        pub small_tree_count: u8,
        pub medium_tree_count: u8,
        pub large_tree_count: u8,
    }

    #[derive(Clone, Copy)]
    pub struct State {
        pub board: [Option<Cell>; 37],
        pub player: Player,

        pub nutrient: u8,

        pub day: u8,
        pub turn_during_day: u8,
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                board: [None; 37],
                player: Player {
                    sun: 0,
                    score: 0,
                    small_tree_count: 0,
                    medium_tree_count: 0,
                    large_tree_count: 0,
                },
                nutrient: 20,
                day: 0,
                turn_during_day: 0,
            }
        }
    }

    pub fn next_states(state: &State) -> Vec<(Move, State)> {
        let p_sun = state.player.sun;
        let p_medium_tree_count = state.player.medium_tree_count;
        let p_large_tree_count = state.player.large_tree_count;

        let mut valid_moves: Vec<Move> = Vec::new();

        if state.day >= 6 {
        } else {
            for (i, cell) in state.board.iter().enumerate() {
                if let Some(c) = cell {
                    match c.tree {
                        Tree::SMALL_TREE => {
                            if p_sun >= 3 + p_medium_tree_count as u32 {
                                valid_moves.push(Move::GROW(i as u8));
                            }
                        }
                        Tree::MEDIUM_TREE => {
                            if p_sun >= 7 + p_large_tree_count as u32 {
                                valid_moves.push(Move::GROW(i as u8))
                            }
                        }
                        Tree::LARGE_TREE => {
                            if p_sun >= 4 {
                                valid_moves.push(Move::COMPLETE(i as u8))
                            }
                        }
                    }
                }
            }

            valid_moves.push(Move::WAIT);
        }

        valid_moves
            .into_iter()
            .map(|m| (m, update(state, &m)))
            .collect::<Vec<(Move, State)>>()
    }

    fn update(state: &State, move_: &Move) -> State {
        let mut new_state = state.clone();

        match move_ {
            Move::GROW(cell_id) => {
                let cell = new_state.board[*cell_id as usize].as_mut().unwrap();
                match cell.tree {
                    Tree::SMALL_TREE => {
                        new_state.player.sun -= 3 + new_state.player.medium_tree_count as u32;
                        new_state.player.small_tree_count -= 1;
                        new_state.player.medium_tree_count += 1;
                        cell.tree = Tree::MEDIUM_TREE;
                        cell.is_dormant = true;
                    }
                    Tree::MEDIUM_TREE => {
                        new_state.player.sun -= 7 + new_state.player.large_tree_count as u32;
                        new_state.player.medium_tree_count -= 1;
                        new_state.player.large_tree_count += 1;
                        cell.tree = Tree::LARGE_TREE;
                        cell.is_dormant = true;
                    }
                    _ => panic!("This code should not be reached"),
                }

                new_state.turn_during_day += 1;
            }
            Move::COMPLETE(cell_id) => {
                new_state.player.sun -= 4;
                new_state.player.score += new_state.nutrient as u32
                    + match get_cell_richness(*cell_id as usize) {
                        SoilRichness::LOW_QUALITY => 0,
                        SoilRichness::MEDIUM_QUALITY => 2,
                        SoilRichness::HIGH_QUALITY => 4,
                    };
                new_state.player.large_tree_count -= 1;
                new_state.board[*cell_id as usize] = None;
                /* Assume that the other player will also complete a tree */
                new_state.nutrient -= 2;

                new_state.turn_during_day += 1;
            }
            Move::WAIT => {
                new_state.day += 1;
                new_state.turn_during_day = 0;
                for cell in new_state.board.iter_mut() {
                    if let Some(c) = cell {
                        c.is_dormant = false;
                    }
                }

                if new_state.day < 6 {
                    new_state.player.sun += gained_sun_points(
                        new_state.player.small_tree_count,
                        new_state.player.medium_tree_count,
                        new_state.player.large_tree_count,
                    )
                }
            }
        }
        new_state
    }

    pub fn eval(state: &State) -> f32 {
        state.player.score as f32 + (state.player.sun / 3) as f32
    }

    /* #region(collapsed) [Private functions] */
    fn get_cell_richness(cell: usize) -> SoilRichness {
        match cell {
            0..=6 => SoilRichness::HIGH_QUALITY,
            7..=18 => SoilRichness::MEDIUM_QUALITY,
            19..=36 => SoilRichness::LOW_QUALITY,
            _ => panic!("Invalid cell index"),
        }
    }

    fn gained_sun_points(small_tree_count: u8, medium_tree_count: u8, large_tree_count: u8) -> u32 {
        small_tree_count as u32 + medium_tree_count as u32 * 2 + large_tree_count as u32 * 3
    }

    /* #endregion */
}

mod beam {
    use super::game;
    use std::time::Instant;

    const MAX_NODE_COUNT: usize = 300_000;
    const TIME_LIMIT_MS: u128 = 99;

    #[derive(Clone, Copy)]
    struct Node {
        move_: game::Move,
        state: game::State,

        parent: Option<usize>,
        child_first: Option<usize>,
        child_count: usize,
        depth: usize,
        eval: f32,
    }

    impl Default for Node {
        fn default() -> Self {
            Self {
                move_: game::Move::default(),
                state: game::State::default(),
                parent: None,
                child_first: None,
                child_count: 0,
                depth: 0,
                eval: 0.0,
            }
        }
    }

    pub struct Beam {
        arr: Vec<Node>,
        len: usize,
    }

    impl Beam {
        pub fn new() -> Self {
            Self {
                arr: vec![Node::default(); MAX_NODE_COUNT],
                len: 0,
            }
        }

        pub fn best_path(&mut self, start_state: game::State) -> Vec<(game::Move, f32)> {
            const BEAM_SIZE: usize = 1000;

            let start = Instant::now();
            self.init(start_state);

            let mut frontier: Vec<usize> = Vec::with_capacity(BEAM_SIZE);
            frontier.push(0);

            let mut max_eval = -f32::INFINITY;
            let mut most_valuable_node_idx = 0;

            'main: while (start.elapsed().as_millis() < TIME_LIMIT_MS) && (frontier.len() > 0) {
                let mut frontier_temp: Vec<(usize, f32)> = Vec::new();
                let mut min_eval_temp: f32 = f32::INFINITY;

                for node_idx in frontier.iter() {
                    let node = &self.arr[*node_idx];
                    /* Get the next states & moves */
                    let next_states: Vec<(game::Move, game::State)> =
                        game::next_states(&node.state);

                    /* Create children nodes */
                    let mut children: Vec<Node> = next_states
                        .into_iter()
                        .map(|(move_, state)| Node {
                            move_: move_,
                            state: state,
                            parent: Some(*node_idx),
                            child_first: None,
                            child_count: 0,
                            depth: node.depth + 1,
                            eval: Beam::eval(&state, node.depth),
                        })
                        .collect::<Vec<Node>>();

                    /* Check if there's still place for children */
                    if self.len + children.len() > MAX_NODE_COUNT {
                        break 'main;
                    }

                    /* Remove children whose score is so low, they will never be added to the frontier */
                    if frontier_temp.len() > BEAM_SIZE {
                        children.retain(|c| c.eval > min_eval_temp);
                    }

                    /* Add the children nodes to the tree */
                    self.set_children(*node_idx, children);

                    /* Determine the highest & lowest child score so far */
                    let parent_node = &self.arr[*node_idx];
                    for child_idx in parent_node.child_first.unwrap()
                        ..parent_node.child_first.unwrap() + parent_node.child_count as usize
                    {
                        let child: &Node = &self.arr[child_idx];

                        /* Determine if it's the most valuable node so far */
                        if child.eval > max_eval {
                            max_eval = child.eval;
                            most_valuable_node_idx = child_idx;
                        }
                        if child.eval < min_eval_temp {
                            min_eval_temp = child.eval;
                        }

                        /* And add it to the temp frontier */
                        frontier_temp.push((child_idx, child.eval));
                    }
                }

                /* Select the top BEAM_SIZE frontier nodes and add them to the frontier */
                frontier_temp.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                frontier.clear();

                for i in 0..std::cmp::min(frontier_temp.len(), BEAM_SIZE) {
                    frontier.push(frontier_temp[i].0);
                }
            }

            if frontier.len() > 0 {
                eprintln!(
                    "[BEAM P5] BEAM search ended. Expanded {} nodes in {:?}",
                    self.len,
                    start.elapsed()
                );
            } else {
                eprintln!(
                    "[BEAM P5] BEAM search ended. Expanded ALL {} nodes in {:?}",
                    self.len,
                    start.elapsed()
                );
            }

            /* When search is finished, determine the most valuable node, and extract its moves */
            let mut best_path: Vec<(game::Move, f32)> = Vec::new();
            let mut n = most_valuable_node_idx;
            while self.arr[n].parent.is_some() {
                best_path.push((self.arr[n].move_, self.arr[n].eval));
                n = self.arr[n].parent.unwrap();
            }

            best_path.reverse();
            best_path
        }

        fn init(&mut self, start_state: game::State) {
            self.arr[0] = Node {
                move_: game::Move::default(),
                state: start_state,
                parent: None,
                child_first: None,
                child_count: 0,
                depth: 0,
                eval: 0.0,
            };

            self.len = 1;
        }

        fn set_children(&mut self, parent: usize, children: Vec<Node>) {
            self.arr[parent].child_first = Some(self.len);
            self.arr[parent].child_count = children.len();

            for child in children.into_iter() {
                self.arr[self.len] = child;
                self.len += 1;
            }
        }

        fn eval(state: &game::State, node_depth: usize) -> f32 {
            game::eval(state)
        }
    }
}

pub fn play(
    ctr_rcv: Receiver<bool>,
    msg_rcv: Receiver<String>,
    msg_snd: Sender<(String, Option<std::collections::HashMap<String, String>>)>,
    params: Option<Vec<String>>,
) {
    let mut input_line = String::new();
    input_line = msg_rcv.recv().unwrap();
    let number_of_cells = parse_input!(input_line, i32); // 37
    for i in 0..number_of_cells as usize {
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
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

    /* State variables that have to be maintained as they are not sent by the game */
    let mut prev_day = -1;
    let mut turn_during_day = 0;

    let mut beam: beam::Beam = beam::Beam::new();

    // game loop
    while ctr_rcv.recv().unwrap() == true {
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let day = parse_input!(input_line, u8); // the game lasts 24 days: 0-23
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let nutrients = parse_input!(input_line, u8); // the base score you gain from the next COMPLETE action
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let sun = parse_input!(inputs[0], u32); // your sun points
        let score = parse_input!(inputs[1], u32); // your current score
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opp_sun = parse_input!(inputs[0], i32); // opponent's sun points
        let opp_score = parse_input!(inputs[1], i32); // opponent's score
        let opp_is_waiting = parse_input!(inputs[2], i32); // whether your opponent is asleep until the next day
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();

        let mut my_board: [Option<game::Cell>; 37] = [None; 37];
        let mut my_small_tree_count = 0;
        let mut my_medium_tree_count = 0;
        let mut my_large_tree_count = 0;

        let number_of_trees = parse_input!(input_line, i32); // the current amount of trees
        for i in 0..number_of_trees as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let cell_index = parse_input!(inputs[0], usize); // location of this tree
            let size = parse_input!(inputs[1], u8); // size of this tree: 0-3
            let is_mine = parse_input!(inputs[2], i32); // 1 if this is your tree
            let is_dormant = parse_input!(inputs[3], i32); // 1 if this tree is dormant

            if is_mine == 1 {
                my_board[cell_index] = Some(game::Cell {
                    player: 0,
                    tree: match size {
                        1 => {
                            my_small_tree_count += 1;
                            game::Tree::SMALL_TREE
                        }
                        2 => {
                            my_medium_tree_count += 1;
                            game::Tree::MEDIUM_TREE
                        }
                        3 => {
                            my_large_tree_count += 1;
                            game::Tree::LARGE_TREE
                        }
                        _ => panic!(),
                    },
                    is_dormant: match is_dormant {
                        1 => true,
                        0 => false,
                        _ => panic!(),
                    },
                });
            }
        }
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let number_of_possible_actions = parse_input!(input_line, i32); // all legal actions
        let mut possible_actions: Vec<String> = Vec::new();
        for i in 0..number_of_possible_actions as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let possible_action = input_line.trim_matches('\n').to_string(); // try printing something from here to start with
            possible_actions.push(possible_action);
        }

        /* Create State from information above */

        if prev_day < day as i8 {
            turn_during_day = 0;
        } else {
            turn_during_day += 1;
        }
        prev_day = day as i8;

        let state = game::State {
            board: my_board,
            player: game::Player {
                sun: sun,
                score: score,
                small_tree_count: my_small_tree_count,
                medium_tree_count: my_medium_tree_count,
                large_tree_count: my_large_tree_count,
            },
            nutrient: nutrients,
            day: day,
            turn_during_day: turn_during_day,
        };

        /* Extract best path */
        let best_path = beam.best_path(state);

        /* Extract best move */
        let best_move = best_path[0].0;

        let msg = best_move.to_string();
        msg_snd.send((format!("{}", best_move), None));
    }
}
