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

    pub type Move = (u8, u8);
    // Max # of legal moves
    pub const MAX_NB_MOVES: usize = 81;

    // An array of Game Scores, assuming that there'll be always a maxium of 4 players
    pub type GameScore = [f32; 4];

    #[derive(Clone, Debug)]
    pub struct State {
        /*
        2D array
        [pid] => [u16,u16,u16,u16,u16,u16,u16,u16,u16] (9 squares)
            Each u16 correspond to a 9-bit representation of a square.
        */
        p_boards: [[u16; 9]; 2],
        // A 2D array : [player_id] => 8-bit number representing which squares are won by a player
        p_squares: [u16; 2],
        // represent which squares are locked
        locked_squares: u16,

        active: bool,
        active_player: u8, // player who's turn is to do the next move
        turn: u8,

        last_move: Option<(u8, u8)>,
        winners: Option<(WinLossTie, WinLossTie)>,
    }

    pub fn new() -> State {
        State {
            p_boards: [[0b000_000_000; 9]; 2],
            p_squares: [0b000_000_000; 2],
            locked_squares: 0b000_000_000,

            active: true,
            active_player: 0,
            turn: 0,

            last_move: None,
            winners: None,
        }
    }

    pub fn update_state(state: &mut State, player: u8, move_: (u8, u8)) {
        /*
            Update the state with the move done by the player.
            Assume that it's the player's turn and the move is always legal
        */

        // (1) Place move on board
        let square = square_of_cell(move_);
        let (row33, col33) = cell99_to_cell33(move_);
        let sq_idx: usize = (square.0 * 3 + square.1) as usize;

        state.p_boards[player as usize][sq_idx] =
            set_bit(state.p_boards[player as usize][sq_idx], row33, col33);

        // (2) Check if the player won the square
        let p_square = state.p_boards[player as usize][sq_idx];
        if is_won(p_square) {
            // Update the player's square status
            state.p_squares[player as usize] =
                set_bit(state.p_squares[player as usize], square.0, square.1);

            // Update the locked square status
            state.locked_squares = set_bit(state.locked_squares, square.0, square.1);
        }
        // (3) If the player didn't win the square, check if it's filled
        else if state.p_boards[0][sq_idx] | state.p_boards[1][sq_idx] == 0b111_111_111 {
            state.locked_squares = set_bit(state.locked_squares, square.0, square.1);
        }

        // (4) Check if it's a winning move or a tie
        if is_won(state.p_squares[player as usize]) {
            state.active = false;
            state.winners = if player == 0 {
                Some((WinLossTie::Win, WinLossTie::Loss))
            } else {
                Some((WinLossTie::Loss, WinLossTie::Win))
            }
        } else if state.locked_squares == 0b111_111_111 {
            state.active = false;
            let won_squares = [
                state.p_squares[0].count_ones(),
                state.p_squares[1].count_ones(),
            ];
            if won_squares[0] > won_squares[1] {
                state.winners = Some((WinLossTie::Win, WinLossTie::Loss));
            } else if won_squares[0] < won_squares[1] {
                state.winners = Some((WinLossTie::Loss, WinLossTie::Win));
            } else {
                state.winners = Some((WinLossTie::Tie, WinLossTie::Tie));
            }
        }

        state.turn += 1;
        state.last_move = Some(move_);

        if state.active == true {
            state.active_player = (state.active_player + 1) % 2;
        }
    }

    pub fn is_terminal(state: &State) -> bool {
        !state.active
    }

    pub fn get_scores(state: &State) -> GameScore {
        match state.winners {
            Some((WinLossTie::Win, WinLossTie::Loss)) => [1.0, 0.0, 0.0, 0.0],
            Some((WinLossTie::Loss, WinLossTie::Win)) => [0.0, 1.0, 0.0, 0.0],
            Some((WinLossTie::Tie, WinLossTie::Tie)) => [0.5, 0.5, 0.0, 0.0],
            _ => panic!(),
        }
    }

    pub fn valid_moves(state: &State) -> (u8, StackVector<(u8, u8), 81>) {
        let p_boards = &state.p_boards;
        let locked_squares = state.locked_squares;
        let last_move = state.last_move;

        // (1) Determine valid squares
        let valid_squares: u16 = match last_move {
            // If it's the first move, all squares are valid
            None => 0b111_111_111,
            // Else, check the last move
            Some(m) => {
                // Get the square referred to by the cell move
                let sq = cell99_to_cell33(m);
                if get_bit(locked_squares, sq.0, sq.1) == 0 {
                    // If the square is not locked, only one square is valid
                    0b1 << (2 - sq.0) * 3 + (2 - sq.1)
                } else {
                    // if square is locked, all non-locked squares are valid
                    !locked_squares
                }
            }
        };

        // (2) For each valid square add list of valid moves
        let mut valid_moves: StackVector<(u8, u8), 81> = StackVector {
            arr: [(0, 0); 81],
            len: 0,
        };

        for (r, c) in [
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ] {
            if get_bit(valid_squares, r, c) == 1 {
                let sq_ix = (r * 3 + c) as usize;
                let valid_moves_in_square =
                    valid_moves_in_square(p_boards[0][sq_ix] | p_boards[1][sq_ix]);

                for m_sq in valid_moves_in_square.get() {
                    valid_moves.add(cell33_to_cell99(*m_sq, (r, c)));
                }
            }
        }

        (state.active_player, valid_moves)
    }

    /* #region(collapsed) [Private game functions] */
    #[derive(Clone, Debug)]
    enum WinLossTie {
        Win,
        Loss,
        Tie,
    }

    /*
        For a 9-bit representation of a square, return empty cells (0..2, 0..2)
    */
    fn valid_moves_in_square(square_bin: u16) -> StackVector<(u8, u8), 9> {
        let mut valid_moves: StackVector<(u8, u8), 9> = StackVector {
            arr: [(0, 0); 9],
            len: 0,
        };
        let sq = square_bin;
        for sh in 0..9 {
            if (sq >> sh) & 0b1 == 0 {
                valid_moves.add(((8 - sh) / 3, (8 - sh) % 3));
            }
        }
        valid_moves
    }

    /*
    Get the square position (0..2, 0..2) where a cell (0..8, 0..8) is located
    */
    fn square_of_cell(cell: (u8, u8)) -> (u8, u8) {
        (cell.0 / 3, cell.1 / 3)
    }

    /*
    Convert a cell position in a 9x9 board to a cell position in a 3x3 square
    */
    fn cell99_to_cell33(cell: (u8, u8)) -> (u8, u8) {
        (cell.0 % 3, cell.1 % 3)
    }

    /*
    Convert a cell position in a 3x3 square to a cell position in a 9x9 board
    */
    fn cell33_to_cell99(cell: (u8, u8), square: (u8, u8)) -> (u8, u8) {
        (cell.0 + square.0 * 3, cell.1 + square.1 * 3)
    }

    /*
    From a 9-bit representation of a board or square, get the bit referred to by the row & col
    */
    fn get_bit(bit_9: u16, row: u8, col: u8) -> u8 {
        ((bit_9 >> (2 - row) * 3 + (2 - col)) & 0b1) as u8
    }

    fn set_bit(bit_9: u16, row: u8, col: u8) -> u16 {
        (0b1 << ((2 - row) * 3 + (2 - col))) | bit_9
    }

    fn is_won(bit_9: u16) -> bool {
        let winning_configurations: [u16; 8] = [
            0b111000000,
            0b000111000,
            0b000000111,
            0b100100100,
            0b010010010,
            0b001001001,
            0b100010001,
            0b001010100,
        ];

        for wc in winning_configurations {
            if bit_9 & wc == wc {
                return true;
            }
        }
        false
    }

    /* #endregion */
}

mod mcts {

    use super::game;
    use rand::Rng;
    use std::time::Instant;

    const MAX_NODE_COUNT: usize = 80_000;
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
        arr: [Node; MAX_NODE_COUNT],
        len: usize,
        nb_simulations: u32,
    }

    pub fn new() -> MCTS {
        MCTS {
            arr: [Default::default(); MAX_NODE_COUNT],
            len: 0,
            nb_simulations: 0,
        }
    }

    impl MCTS {
        pub fn best_move(
            &mut self,
            root_state: &game::State,
            valid_moves: &Vec<game::Move>,
            player: u8,
        ) -> game::Move {
            /*
                Find the best move
                - Starting from State [state],
                - And already given (for optimization) the [valid_moves] that [player] can do
            */

            //eprintln!("[MCTS] init");
            let start = Instant::now();
            self.init(valid_moves, player);

            while (start.elapsed().as_millis() < TIME_LIMIT_MS)
                & (self.len < MAX_NODE_COUNT - game::MAX_NB_MOVES)
            {
                let mut state = root_state.clone();

                //eprintln!("[MCTS] Selection");

                let selected_node_idx = self.select(&mut state);

                //eprintln!("[MCTS] Expansion");
                let rollout_node_idx = self.expand(selected_node_idx, &mut state);

                //eprintln!("[MCTS] Simulation");
                let score = self.simulate(&mut state);

                self.backpropagate(rollout_node_idx, score);

                self.nb_simulations += 1;
            }

            /*
            eprintln!(
                "[MCTS] End. Sending best move after expanding {} nodes and running {} simulations",
                self.len, self.nb_simulations
            );
            */

            // When time is up, choose the move with the best score
            let mut max_score: f32 = -f32::INFINITY;
            let mut max_score_child_idx = 0;
            for c in self.arr[0].child_first.unwrap()
                ..self.arr[0].child_first.unwrap() + self.arr[0].child_count as usize
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

        fn init(&mut self, valid_moves: &Vec<game::Move>, player: u8) {
            // Re-initialize the node tree

            // Re-initialize Root
            self.arr[0] = Default::default();
            self.arr[0].expanded = true;
            self.len = 1;
            self.nb_simulations = 0;

            // Create the children of root
            self.arr[0].child_first = Some(1);
            self.arr[0].child_count = valid_moves.len() as u8;
            for vm in valid_moves {
                self.create_child(0, *vm, player);
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
            let mut node_idx = 0;

            while self.arr[node_idx].child_count > 0 {
                let node = &self.arr[node_idx];

                // Identify child with largest UCB
                let mut max_ucb: f32 = 0.0;
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
                    } else if child_ucb > max_ucb {
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
                        + 1.41 * ((parent_visit as f32).ln() / (visits as f32)).sqrt()
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

        fn simulate(&self, state: &mut game::State) -> game::GameScore {
            // Simulate the game until the end
            while !game::is_terminal(state) {
                let (player, valid_moves) = game::valid_moves(state);
                let chosen_move = valid_moves.arr[rand::thread_rng().gen_range(0..valid_moves.len)];

                game::update_state(state, player, chosen_move);
            }

            // Get the result
            game::get_scores(state)
        }

        fn backpropagate(&mut self, selected_node_idx: usize, score: game::GameScore) {
            let mut node_idx = selected_node_idx;
            while self.arr[node_idx].parent.is_some() {
                self.arr[node_idx].visits += 1;
                self.arr[node_idx].score += score[self.arr[node_idx].player.unwrap() as usize];

                node_idx = self.arr[node_idx].parent.unwrap();
            }

            // Update visit count for the root node
            self.arr[0].visits += 1;
        }
    }
}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(ctr_rcv: Receiver<bool>, msg_rcv: Receiver<String>, msg_snd: Sender<String>) {
    let mut state = game::new();
    let mut my_pid = 1; // Assume that I'm player 1
    let mut opp_pid = 0;

    // Prepare MCTS
    let mut mcts: mcts::MCTS = mcts::new();

    while ctr_rcv.recv().unwrap() == true {
        // (1) Read inputs
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opponent_row = parse_input!(inputs[0], i32);
        let opponent_col = parse_input!(inputs[1], i32);

        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let valid_action_count = parse_input!(input_line, i32);

        let mut valid_actions: Vec<(u8, u8)> = Vec::new();
        for i in 0..valid_action_count as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let row = parse_input!(inputs[0], u8);
            let col = parse_input!(inputs[1], u8);
            valid_actions.push((row, col));
        }

        //(2) Update my game state
        if opponent_row == -1 {
            // I know now I'm player 0 and it's the first move. no need to update my state
            my_pid = 0;
            opp_pid = 1;
        } else {
            // Update the state with the opponent's last action
            game::update_state(
                &mut state,
                opp_pid,
                (opponent_row as u8, opponent_col as u8),
            );
        }

        // (3) Determine the next best action
        let best_move = mcts.best_move(&state, &valid_actions, my_pid);

        // (4) Update state with my action
        game::update_state(&mut state, my_pid, best_move);

        msg_snd.send(format!("{} {}", best_move.0, best_move.1));
    }
}
