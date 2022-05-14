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
    use rand::prelude::SliceRandom;

    pub type Move = u128;
    // Max # of legal moves
    pub const MAX_NB_MOVES: usize = 81;

    // An array of Game Scores, assuming that there'll be always a maxium of 4 players
    pub type GameScore = [f32; 4];

    pub struct Cache {
        random_indices : Vec<[u8;81]>,
        random_indices_i: usize
    }

    impl Cache {
        pub fn new() -> Cache {
            let mut random_indices : Vec<[u8;81]> = Vec::new();
            for _ in 0..100_000 {
                let mut arr:[u8;81] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80];
                arr.shuffle(&mut rand::thread_rng());
                random_indices.push(arr);
            }
            
        Cache {
            random_indices,
            random_indices_i:0
        }
        }

    }

    #[derive(Clone, Debug)]
    pub struct State {
        /*
        2D array
        [pid] => [u16,u16,u16,u16,u16,u16,u16,u16,u16] (9 squares)
            Each u16 correspond to a 9-bit representation of a square.
        */
        p_boards: [u128; 2],
        // A 2D array : [player_id] => 8-bit number representing which squares are won by a player
        p_squares: [u128; 2],
        // represent which squares are locked
        locked_squares: u128,

        active: bool,
        active_player: u8, // player who's turn is to do the next move
        turn: u8,

        last_move: u128,
        winners: Option<(WinLossTie, WinLossTie)>,
    }

    pub fn new() -> State {
        State {
            p_boards: [0; 2],
            p_squares: [0; 2],
            locked_squares: 0,

            active: true,
            active_player: 0,
            turn: 0,

            last_move: 0,
            winners: None,
        }
    }

    pub fn update_state(state: &mut State, player: u8, move_: Move) {
        /*
            Update the state with the move done by the player.
            Assume that it's the player's turn and the move is always legal
        */

        // (1) Place move on board
        let square81 = square_of_move81(move_);

        state.p_boards[player as usize] |= move_;

        // (2) Check if the player won the square
        if won_the_square(state.p_boards[player as usize], square81) {
            // Update the player's square status
            state.p_squares[player as usize] |= square81;
            // Update the locked square status
            state.locked_squares |= square81;
        }
        // (3.3) If the player didn't win the square, check if it's filled
        else if (state.p_boards[0] | state.p_boards[1]) & square81 == square81 {
            state.locked_squares |= square81;
        }
        // (4) Check if it's a global winning move or a tie
        if won_the_board(state.p_squares[player as usize]) {
            state.active = false;
            state.winners = if player == 0 {
                Some((WinLossTie::Win, WinLossTie::Loss))
            } else {
                Some((WinLossTie::Loss, WinLossTie::Win))
            }
        } else if state.locked_squares == 0b111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111 {
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
        state.last_move = move_;

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

    pub fn valid_moves(state: &State) -> (u8, StackVector<Move, 81>) {
        let p_boards = &state.p_boards;
        let locked_squares = state.locked_squares;
        let last_move = state.last_move;

        // (1) Determine valid moves
        let valid_moves81 = (!(p_boards[0] | p_boards[1] | locked_squares)) & 0b111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111;

        let valid_moves81 = match last_move {
            0 => valid_moves81,
            _ => {
                let next_square = square_pointed_by_move81(last_move);

                // If next_square is not a locked square
                if next_square & locked_squares == 0 {
                    valid_moves81 & next_square 
                }
                else {
                    valid_moves81
                }        
            }
        };

        // (2) Transform the move mask to an array of moves
        let mut valid_moves_vec: StackVector<Move, 81> = StackVector {
            arr: [0; 81],
            len: 0,
        };

        let mut m = 0b1;
        for _ in 0..81 {
            if valid_moves81 & m > 0 {
                valid_moves_vec.add(m);
            }
            m <<= 1;
        }

        (state.active_player, valid_moves_vec)
    }

    pub fn random_valid_move(state: &State, cache: &mut Cache) -> (u8, Move) {
        let p_boards = &state.p_boards;
        let locked_squares = state.locked_squares;
        let last_move = state.last_move;

        // (1) Determine valid moves
        let valid_moves81 = (!(p_boards[0] | p_boards[1] | locked_squares)) & 0b111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111;

        let valid_moves81 = match last_move {
            0 => valid_moves81,
            _ => {
                let next_square = square_pointed_by_move81(last_move);

                // If next_square is not a locked square
                if next_square & locked_squares == 0 {
                    valid_moves81 & next_square 
                }
                else {
                    valid_moves81
                }        
            }
        };

        // (2) Get a random valid move
        let indices = &cache.random_indices[cache.random_indices_i];
        cache.random_indices_i = (cache.random_indices_i +  1)%100_000;

        for i in indices.iter() {
            let m:u128 = 0b1 << i;
            if valid_moves81 & m > 0 {
                return (state.active_player,m);
            }        
        }

        panic!("Couldn't pick a random move");

    }



    /* #region(collapsed) [Private game functions] */
    #[derive(Clone, Debug)]
    enum WinLossTie {
        Win,
        Loss,
        Tie,
    }

    fn square_of_move81(move_: u128) -> u128 {
        match move_ {
            1208925819614629174706176
            | 604462909807314587353088
            | 302231454903657293676544
            | 151115727451828646838272
            | 75557863725914323419136
            | 37778931862957161709568
            | 18889465931478580854784
            | 9444732965739290427392
            | 4722366482869645213696 => 2413129272746388704198656,
            2361183241434822606848
            | 1180591620717411303424
            | 590295810358705651712
            | 295147905179352825856
            | 147573952589676412928
            | 73786976294838206464
            | 36893488147419103232
            | 18446744073709551616
            | 9223372036854775808 => 4713143110832790437888,
            4611686018427387904 | 2305843009213693952 | 1152921504606846976
            | 576460752303423488 | 288230376151711744 | 144115188075855872 | 72057594037927936
            | 36028797018963968 | 18014398509481984 => 9205357638345293824,
            9007199254740992 | 4503599627370496 | 2251799813685248 | 1125899906842624
            | 562949953421312 | 281474976710656 | 140737488355328 | 70368744177664
            | 35184372088832 => 17979214137393152,
            17592186044416 | 8796093022208 | 4398046511104 | 2199023255552 | 1099511627776
            | 549755813888 | 274877906944 | 137438953472 | 68719476736 => 35115652612096,
            34359738368 | 17179869184 | 8589934592 | 4294967296 | 2147483648 | 1073741824
            | 536870912 | 268435456 | 134217728 => 68585259008,
            67108864 | 33554432 | 16777216 | 8388608 | 4194304 | 2097152 | 1048576 | 524288
            | 262144 => 133955584,
            131072 | 65536 | 32768 | 16384 | 8192 | 4096 | 2048 | 1024 | 512 => 261632,
            256 | 128 | 64 | 32 | 16 | 8 | 4 | 2 | 1 => 511,
            _ => panic!(),
        }
    }

    fn square_pointed_by_move81(move_: u128) -> u128 {
        match move_{
            1208925819614629174706176 | 2361183241434822606848 | 4611686018427387904 | 9007199254740992 | 17592186044416 | 34359738368 | 67108864 | 131072 | 256 => 2413129272746388704198656,
            604462909807314587353088 | 1180591620717411303424 | 2305843009213693952 | 4503599627370496 | 8796093022208 | 17179869184 | 33554432 | 65536 | 128 => 4713143110832790437888,
            302231454903657293676544 | 590295810358705651712 | 1152921504606846976 | 2251799813685248 | 4398046511104 | 8589934592 | 16777216 | 32768 | 64 => 9205357638345293824,
            151115727451828646838272 | 295147905179352825856 | 576460752303423488 | 1125899906842624 | 2199023255552 | 4294967296 | 8388608 | 16384 | 32 => 17979214137393152,
            75557863725914323419136 | 147573952589676412928 | 288230376151711744 | 562949953421312 | 1099511627776 | 2147483648 | 4194304 | 8192 | 16 => 35115652612096,
            37778931862957161709568 | 73786976294838206464 | 144115188075855872 | 281474976710656 | 549755813888 | 1073741824 | 2097152 | 4096 | 8 => 68585259008,
            18889465931478580854784 | 36893488147419103232 | 72057594037927936 | 140737488355328 | 274877906944 | 536870912 | 1048576 | 2048 | 4 => 133955584,
            9444732965739290427392 | 18446744073709551616 | 36028797018963968 | 70368744177664 | 137438953472 | 268435456 | 524288 | 1024 | 2 => 261632,
            4722366482869645213696 | 9223372036854775808 | 18014398509481984 | 35184372088832 | 68719476736 | 134217728 | 262144 | 512 | 1 => 511,
            _ => panic!()
        }
    }

    fn won_the_square(p_board81:u128, square81:u128) -> bool {

        let sq_idx = match square81 {
            2413129272746388704198656 => 0,
            4713143110832790437888 => 1,
            9205357638345293824 => 2,
            17979214137393152 => 3,
            35115652612096 => 4,
            68585259008 => 5,
            133955584 => 6,
            261632 => 7,
            511 => 8,
            _ => panic!()
        };

        const ALL_WINNING_CONFIGURATIONS: [[u128; 8]; 9] = [
        [0b111000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000111000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000111_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b100100100_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b010010010_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b001001001_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b100010001_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b001010100_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000],
        
        [0b000000000_111000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000111000_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000111_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_100100100_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_010010010_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_001001001_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_100010001_000000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_001010100_000000000_000000000_000000000_000000000_000000000_000000000_000000000],
        
        [0b000000000_000000000_111000000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000111000_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000111_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_100100100_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_010010010_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_001001001_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_100010001_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_001010100_000000000_000000000_000000000_000000000_000000000_000000000],

        [0b000000000_000000000_000000000_111000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000111000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000111_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_100100100_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_010010010_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_001001001_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_100010001_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_001010100_000000000_000000000_000000000_000000000_000000000,],

        [0b000000000_000000000_000000000_000000000_111000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000111000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000111_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_100100100_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_010010010_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_001001001_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_100010001_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_001010100_000000000_000000000_000000000_000000000,],

        [0b000000000_000000000_000000000_000000000_000000000_111000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000111000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000111_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_100100100_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_010010010_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_001001001_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_100010001_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_001010100_000000000_000000000_000000000,],

        [0b000000000_000000000_000000000_000000000_000000000_000000000_111000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000111000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000111_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_100100100_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_010010010_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_001001001_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_100010001_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_001010100_000000000_000000000,],

        [0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_111000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000111000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000111_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_100100100_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_010010010_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_001001001_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_100010001_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_001010100_000000000,],

        [0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_111000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000111000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000111,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_100100100,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_010010010,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_001001001,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_100010001,
        0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_001010100,]
        ];

        let winning_configurations = ALL_WINNING_CONFIGURATIONS[sq_idx as usize];

        for wc in winning_configurations {
            if p_board81 & wc == wc {
                return true;
            }
        }
        false    
    }

    fn won_the_board(p_squares81:u128) -> bool {
        const WINNING_CONFIGURATIONS :[u128;8] = [0b111111111_111111111_111111111_000000000_000000000_000000000_000000000_000000000_000000000,
        0b000000000_000000000_000000000_111111111_111111111_111111111_000000000_000000000_000000000,
        0b000000000_000000000_000000000_000000000_000000000_000000000_111111111_111111111_111111111,
        0b111111111_000000000_000000000_111111111_000000000_000000000_111111111_000000000_000000000,
        0b000000000_111111111_000000000_000000000_111111111_000000000_000000000_111111111_000000000,
        0b000000000_000000000_111111111_000000000_000000000_111111111_000000000_000000000_111111111,
        0b111111111_000000000_000000000_000000000_111111111_000000000_000000000_000000000_111111111,
        0b000000000_000000000_111111111_000000000_111111111_000000000_111111111_000000000_000000000];

        for wc in WINNING_CONFIGURATIONS {
            if p_squares81 & wc == wc {
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

    const MAX_NODE_COUNT: usize = 55_000;
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
            cache: &mut game::Cache
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
                let score = self.simulate(&mut state, cache);

                self.backpropagate(rollout_node_idx, score);

                self.nb_simulations += 1;
            }

            
            eprintln!(
                "[MCTS P3] End. Sending best move after expanding {} nodes and running {} simulations",
                self.len, self.nb_simulations
            );
            

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

mod conv {

    use super::game;

    pub fn movetuple_to_move81(move_: (u8, u8)) -> game::Move {
        let (r, c) = (move_.0 % 3, move_.1 % 3);
        let (sq_r, sq_c) = (move_.0 / 3, move_.1 / 3);

        let bit = 0b1 << (((2 - sq_r) * 3 + (2 - sq_c)) * 9) + ((2 - r) * 3 + (2 - c));

        bit
    }

    pub fn move81_to_movetuple(move_:game::Move) -> (u8,u8) {
        let (mut sq_r, mut sq_c) = (0,0);
        let (mut r,mut c) = (0,0);

        for i in 0..81 {

            let v =(move_ >> (80-i)) & 0b1;

            if v == 1 {
                return (r+sq_r*3,c+sq_c*3 )
            }

            c+=1;
            if c == 3 {
                c = 0;
                r += 1;
            }
            if r == 3 {
                r = 0;
                sq_c += 1;
            }
            if sq_c == 3 {
                sq_c = 0;
                sq_r += 1;
            }
        }
        
        panic!();
    }
}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(ctr_rcv: Receiver<bool>, msg_rcv: Receiver<String>, msg_snd: Sender<String>) {
    let mut state = game::new();
    let mut my_pid = 1; // Assume that I'm player 1
    let mut opp_pid = 0;

    // Prepare MCTS
    let mut mcts: mcts::MCTS = mcts::new();
    let mut cache = game::Cache::new();

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

        let mut valid_actions: Vec<game::Move> = Vec::new();
        for i in 0..valid_action_count as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let row = parse_input!(inputs[0], u8);
            let col = parse_input!(inputs[1], u8);
            valid_actions.push(conv::movetuple_to_move81((row, col)));
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
                conv::movetuple_to_move81((opponent_row as u8, opponent_col as u8)),
            );
        }

        // (3) Determine the next best action
        let best_move = mcts.best_move(&state, &valid_actions, my_pid, &mut cache);

        // (4) Update state with my action
        game::update_state(&mut state, my_pid, best_move);

        // (5) Send the move
        let best_move = conv::move81_to_movetuple(best_move);
        msg_snd.send(format!("{} {}", best_move.0, best_move.1));
    }
}
