use std::sync::mpsc::{Receiver, Sender};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub struct StackVector<T, const MAX_SIZE: usize> {
    pub arr: [T; MAX_SIZE],
    pub len: usize,
}

impl<T, const MAX_SIZE: usize> StackVector<T, MAX_SIZE> {
    pub fn add(&mut self, e: T) {
        self.arr[self.len] = e;
        self.len += 1;
    }

    pub fn get(&self) -> &[T] {
        &self.arr[0..self.len]
    }
}

mod game {

    use super::StackVector;

    // An array of Game Scores, assuming that there'll be always a maxium of 4 players
    pub type GameScore = [f32; 4];

    /* Change this for each new game */
    // The move data type
    pub type Move = u128;
    // Max # of legal moves
    pub const MAX_NB_MOVES: usize = 81;

    /* Optional : Cache */
    pub struct Cache {}

    impl Cache {
        pub fn new() -> Cache {}
    }

    #[derive(Clone, Debug)]
    pub struct State {}

    pub fn new() -> State {}

    pub fn update_state(state: &mut State, player: u8, move_: Move) {}

    pub fn is_terminal(state: &State) -> bool {
        !state.active
    }

    pub fn get_scores(state: &State) -> GameScore {}

    pub fn valid_moves(state: &State) -> (u8, StackVector<Move, 81>) {}

    pub fn random_valid_move(state: &State, cache: &mut Cache) -> (u8, Move) {}

    /* #region(collapsed) [Private game functions] */

    /* #endregion */
}

mod mcts {

    use super::game;
    use rand::Rng;
    use std::time::Instant;

    const MAX_NODE_COUNT: usize = 3000_000;
    const TIME_LIMIT_MS: u128 = 100;

    #[derive(Clone, Copy)]
    struct Node {
        move_: Option<game::Move>,
        player: Option<u8>, // player who did the move

        parent: Option<usize>,
        child_first: Option<usize>,
        child_count: u8,
        expanded: bool,

        visits: u32,
        score: f32,
    }

    impl Default for Node {
        fn default() -> Self {
            Node {
                move_: None,
                player: None,

                parent: None,
                child_first: None,
                child_count: 0,
                expanded: false,

                visits: 0,
                score: 0.0,
            }
        }
    }

    pub struct MCTS {
        arr: Vec<Node>,
        root_idx: usize,
        len: usize,
        nb_simulations: u32,
    }

    pub fn new() -> MCTS {
        MCTS {
            arr: vec![Default::default(); MAX_NODE_COUNT],
            root_idx: 0,
            len: 0,
            nb_simulations: 0,
        }
    }

    impl MCTS {
        pub fn best_move(
            &mut self,
            root_state: &game::State,
            previous_moves: &[game::Move],
            cache: &mut game::Cache,
        ) -> game::Move {
            /*
                Find the best move
                - Starting from State [state],
                - And already given (for optimization) the [valid_moves] that [player] can do
            */

            //eprintln!("[MCTS] init");
            let start = Instant::now();
            self.init(previous_moves);

            while (start.elapsed().as_millis() < TIME_LIMIT_MS)
                & (self.len < MAX_NODE_COUNT - game::MAX_NB_MOVES)
            {
                let mut state = root_state.clone();

                //eprintln!("[MCTS] Selection");

                let selected_node_idx = self.select(&mut state);

                //eprintln!("[MCTS] Expansion");
                let rollout_node_idx = self.expand(selected_node_idx, &mut state);

                //eprintln!("[MCTS] Simulation");
                let score = self.simulate(&mut state, cache);

                self.backpropagate(rollout_node_idx, score);

                self.nb_simulations += 1;
            }

            eprintln!(
                "[MCTS P6] End. Sending best move after expanding {} nodes and running {} simulations in {:?}",
                self.len, self.nb_simulations, start.elapsed()
            );

            // When time is up, choose the move with the best score
            let mut max_score: f32 = -f32::INFINITY;
            let mut max_score_child_idx = 0;
            for c in self.arr[self.root_idx].child_first.unwrap()
                ..self.arr[self.root_idx].child_first.unwrap()
                    + self.arr[self.root_idx].child_count as usize
            {
                let child = &self.arr[c];
                let child_score = match child.visits {
                    0 => -f32::INFINITY,
                    _ => child.score / child.visits as f32,
                };
                if child_score > max_score {
                    max_score = child_score;
                    max_score_child_idx = c;
                }
            }

            self.arr[max_score_child_idx].move_.unwrap()
        }

        fn init(&mut self, previous_moves: &[game::Move]) {
            // Re-initialize the node tree

            self.nb_simulations = 0;
            // If the tree is already empty, just initialize the root
            if self.len == 0 {
                self.arr[0] = Default::default();
                self.arr[0].expanded = false;
                self.len = 1;
            } else {
                // else if the tree is already constructed, move the root index down to the correct position
                for m in previous_moves {
                    let root_node = &self.arr[self.root_idx];

                    let mut down_the_tree = false; // To verify that the child node was created in a previous iteration
                    for c in root_node.child_first.unwrap()
                        ..root_node.child_first.unwrap() + root_node.child_count as usize
                    {
                        let child = &self.arr[c];

                        if child.move_.unwrap() == *m {
                            self.root_idx = c;
                            down_the_tree = true;
                            break;
                        }
                    }

                    // I mostly assume that previous iterations have created all nodes levels until the next turn.
                    // If that's not the case, it will panic
                    // TODO: correct this bit
                    if down_the_tree == false {
                        panic!("[MCTS][ERROR] Couldn't find node when re-initializing the tree");
                    }
                }
            }
        }

        fn create_child(&mut self, parent: usize, move_: game::Move, player: u8) {
            self.arr[self.len] = Node {
                move_: Some(move_),
                player: Some(player),
                parent: Some(parent),
                child_first: None,
                child_count: 0,
                expanded: false,
                visits: 0,
                score: 0.0,
            };
            self.len += 1
        }

        fn select(&self, state: &mut game::State) -> usize {
            /* Go down the tree, selecting each time the node with the largest UCB, until you reach an unexpanded node
             On the way update the state.
            */
            let mut node_idx = self.root_idx;

            while self.arr[node_idx].child_count > 0 {
                let node = &self.arr[node_idx];

                // Identify child with largest UCB
                let mut max_ucb: f32 = -f32::INFINITY;
                let mut max_ucb_node_idx = 0;
                for c in
                    node.child_first.unwrap()..node.child_first.unwrap() + node.child_count as usize
                {
                    let child = &self.arr[c];
                    let child_ucb = MCTS::ucb(node.visits, child.score, child.visits);
                    if child_ucb == f32::INFINITY {
                        //TODO: I'm choosing the first child with ucb=INF. Try to choose a bit more randomly
                        max_ucb_node_idx = c;
                        break;
                    } else if child_ucb >= max_ucb {
                        max_ucb = child_ucb;
                        max_ucb_node_idx = c;
                    }
                }

                node_idx = max_ucb_node_idx;

                game::update_state(
                    state,
                    self.arr[node_idx].player.unwrap(),
                    self.arr[node_idx].move_.unwrap(),
                )
            }

            node_idx
        }

        fn ucb(parent_visit: u32, score: f32, visits: u32) -> f32 {
            match visits {
                0 => f32::INFINITY,
                _ => {
                    (score / visits as f32)
                        + 0.41 * ((parent_visit as f32).ln() / (visits as f32)).sqrt()
                }
            }
        }

        fn expand(&mut self, selected_node_idx: usize, state: &mut game::State) -> usize {
            /*
                Expand the node [selected_node_idx], given its [state]
            */

            let selected_node = &mut self.arr[selected_node_idx];
            if selected_node.expanded == false {
                // This is a non-expanded node, expand it and return it
                selected_node.expanded = true;
                return selected_node_idx;
            } else if game::is_terminal(state) {
                // This is a terminal state, just return the node
                return selected_node_idx;
            } else {
                // This is an already expanded node
                // 1. Create its children, but do not expand them
                let (player, valid_moves) = game::valid_moves(state);

                let child_first = self.len;
                let child_count = valid_moves.len;
                selected_node.child_first = Some(child_first);
                selected_node.child_count = child_count as u8;
                for m in valid_moves.get() {
                    self.create_child(selected_node_idx, *m, player)
                }

                //2. Choose a random child, expand it and return it
                let chosen_child_idx =
                    rand::thread_rng().gen_range(child_first..child_first + child_count);
                self.arr[chosen_child_idx].expanded = true;

                game::update_state(state, player, self.arr[chosen_child_idx].move_.unwrap());

                return chosen_child_idx;
            }
        }

        fn simulate(&self, state: &mut game::State, cache: &mut game::Cache) -> game::GameScore {
            // Simulate the game until the end
            while !game::is_terminal(state) {
                let (player, chosen_move) = game::random_valid_move(state, cache);

                game::update_state(state, player, chosen_move);
            }

            // Get the result
            game::get_scores(state)
        }

        fn backpropagate(&mut self, selected_node_idx: usize, score: game::GameScore) {
            let mut node_idx = selected_node_idx;
            while node_idx != self.root_idx {
                self.arr[node_idx].visits += 1;
                self.arr[node_idx].score += score[self.arr[node_idx].player.unwrap() as usize];

                node_idx = self.arr[node_idx].parent.unwrap();
            }

            // Update visit count for the root node
            self.arr[self.root_idx].visits += 1;
        }
    }
}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(
    ctr_rcv: Receiver<bool>,
    msg_rcv: Receiver<String>,
    msg_snd: Sender<String>,
    params: Option<Vec<String>>,
) {
    // Prepare MCTS
    let mut mcts: mcts::MCTS = mcts::new();
    // Cache is optional
    let mut cache = game::Cache::new();
    // a list of previous moves has to be kept to move through the mcts tree
    let mut previous_moves: Vec<game::Move> = Vec::new();

    while ctr_rcv.recv().unwrap() == true {
        // (1) Read inputs
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();

        //(2) Update my game state & the previous moves vec

        // (3) Determine the next best action
        let best_move = mcts.best_move(&state, &previous_moves, &mut cache);

        // (4) Update state with my action & the previous moves
        game::update_state(&mut state, my_pid, best_move);
        previous_moves.clear();
        previous_moves.push(best_move);

        // (5) Send the move
        let best_move = conv::move81_to_movetuple(best_move);
        msg_snd.send(format!("{} {}", best_move.0, best_move.1));
    }
}
