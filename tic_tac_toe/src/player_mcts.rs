use rand::seq::SliceRandom;
use std::sync::mpsc::{Receiver, Sender};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

mod Game {
    enum WinLossTie {
        Win,
        Loss,
        Tie,
    }

    pub struct State {
        /*
        2D array
        [pid] => [u16,u16,u16,u16,u16,u16,u16,u16,u16] (9 squares)
            Each u16 correspond to a 9-bit representation of a square.
        */
        pub p_boards: [[u16; 9]; 2],
        // A 2D array : [player_id] => 8-bit number representing which squares are won by a player
        pub p_squares: [u16; 2],
        // represent which squares are locked
        pub locked_squares: u16,

        pub active: bool,
        pub active_player: u8, // player who's turn is to do the next move
        pub turn: u8,

        pub last_move: Option<(u8, u8)>,
        pub winners: Option<(WinLossTie, WinLossTie)>,
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
}

mod MCTS {

    use super::Game;
    use std::time::Instant;

    const MAX_NODE_COUNT: usize = 100_000;
    const TIME_LIMIT_MS: u128 = 100;

    #[derive(Clone, Copy)]
    struct Node<MOVE: Copy> {
        move_: Option<MOVE>,
        player: Option<u8>, // player who did the move

        parent: Option<usize>,
        child_first: Option<usize>,
        child_count: u8,

        visits: u32,
        score: f32,
    }

    impl<MOVE: Copy> Default for Node<MOVE> {
        fn default() -> Self {
            Node {
                move_: None,
                player: None,
                parent: None,
                child_first: None,
                child_count: 0,
                visits: 0,
                score: 0.0,
            }
        }
    }

    pub struct MCTS<MOVE: Copy> {
        arr: [Node<MOVE>; MAX_NODE_COUNT],
        len: usize,
    }

    pub fn new<MOVE: Copy>() -> MCTS<MOVE> {
        MCTS {
            arr: [Default::default(); MAX_NODE_COUNT],
            len: 0,
        }
    }

    impl<MOVE: Copy> MCTS<MOVE> {
        pub fn best_move(&self, state: Game::State, valid_moves: &Vec<MOVE>) -> MOVE {
            let start = Instant::now();

            self.init(&state, valid_moves);

            while start.elapsed().as_millis() < TIME_LIMIT_MS {}

            todo!()
        }

        fn init(&self, state: &Game::State, valid_moves: &Vec<MOVE>) {
            // Re-initilalize the node tree
            self.len = 0;
            self.arr[0] = Node {
                move_: state.last_move,
                player: todo!(),
                parent: todo!(),
                child_first: todo!(),
                child_count: todo!(),
                visits: todo!(),
                score: todo!(),
            }
        }
    }
}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(ctr_rcv: Receiver<bool>, msg_rcv: Receiver<String>, msg_snd: Sender<String>) {
    let mut state = Game::new();
    let mut my_pid = 1; // Assume that I'm player 1
    let mut opp_pid = 0;
    let mcts: MCTS::MCTS<(u8, u8)> = MCTS::new();

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
            Game::update_state(
                &mut state,
                opp_pid,
                (opponent_row as u8, opponent_col as u8),
            );
        }

        // (3) Determine the next best action

        let chosen_move = valid_actions.choose(&mut rand::thread_rng()).unwrap();
        msg_snd.send(format!("{} {}", chosen_move.0, chosen_move.1));
    }
}
