use common::record;
use common::{Game, Message, WinLossTie};
use std::collections::HashMap;
use std::fmt;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[derive(Debug, PartialEq)]
enum MoveResult {
    NormalMove,
    MoveWinningSquare,
    MoveFillingSquareWithoutWinningIt,
    MoveWinningBoard,
    MoveFillingBoardWithoutWinning,
    InvalidMove,
}

impl fmt::Display for MoveResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}



#[derive(Debug)]
pub struct TicTacToeGame {
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
    active_player: u8,
    turn: u8,

    last_move: u128,
    last_move_result: Option<MoveResult>,
    winners: Option<(WinLossTie, WinLossTie)>,
}

impl TicTacToeGame {
    fn movetuple_to_move81(move_: (u8, u8)) -> u128 {
        let (r, c) = (move_.0 % 3, move_.1 % 3);
        let (sq_r, sq_c) = (move_.0 / 3, move_.1 / 3);

        let bit = 0b1 << (((2 - sq_r) * 3 + (2 - sq_c)) * 9) + ((2 - r) * 3 + (2 - c));

        bit
    }

    fn move81_to_movetuple(move_:u128) -> (u8,u8) {
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

    fn moves81_to_movetuples(moves:u128) -> Vec<(u8,u8)> {

        let mut valid_moves: Vec<(u8,u8)> = Vec::new();

        let (mut sq_r, mut sq_c) = (0,0);
        let (mut r,mut c) = (0,0);

        for i in 0..81 {

            let v =(moves >> (80-i)) & 0b1;

            if v == 1 {
                valid_moves.push((r+sq_r*3,c+sq_c*3 ));
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
        
valid_moves
    }

    fn square_of_move81(move_: u128) -> u128 {
        match move_ {
            1208925819614629174706176|604462909807314587353088|302231454903657293676544|151115727451828646838272|75557863725914323419136|37778931862957161709568|18889465931478580854784|9444732965739290427392|4722366482869645213696 => 2413129272746388704198656,
            2361183241434822606848|1180591620717411303424|590295810358705651712|295147905179352825856|147573952589676412928|73786976294838206464|36893488147419103232|18446744073709551616|9223372036854775808 => 4713143110832790437888,
            4611686018427387904|2305843009213693952|1152921504606846976|576460752303423488|288230376151711744|144115188075855872|72057594037927936|36028797018963968|18014398509481984 => 9205357638345293824,
            9007199254740992|4503599627370496|2251799813685248|1125899906842624|562949953421312|281474976710656|140737488355328|70368744177664|35184372088832 => 17979214137393152,
            17592186044416|8796093022208|4398046511104|2199023255552|1099511627776|549755813888|274877906944|137438953472|68719476736 => 35115652612096,
            34359738368|17179869184|8589934592|4294967296|2147483648|1073741824|536870912|268435456|134217728 => 68585259008,
            67108864|33554432|16777216|8388608|4194304|2097152|1048576|524288|262144 => 133955584,
            131072|65536|32768|16384|8192|4096|2048|1024|512 => 261632,
            256|128|64|32|16|8|4|2|1 => 511,
            _ => panic!()
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

    fn valid_moves(p_boards: &[u128; 2], locked_squares: u128, last_move: u128) -> u128 {

        let valid_moves = (!(p_boards[0] | p_boards[1] | locked_squares)) & 0b111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111;

        match last_move {
            0 => valid_moves,
            _ => {
                let next_square = TicTacToeGame::square_pointed_by_move81(last_move);

                // If next_square is not a locked square
                if next_square & locked_squares == 0 {
                    valid_moves & next_square 
                }
                else {
                    valid_moves
                }        
            }
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

    fn to_vector(board:u128) -> Vec<Vec<bool>> {
        let (mut sq_r, mut sq_c) = (0,0);
        let (mut r,mut c) = (0,0);

        let mut vec_board: Vec<Vec<bool>>  = Vec::new();
        for _ in 0..9 {
            vec_board.push(vec![false;9]);
        }

        for i in 0..81 {

            let v =(board >> (80-i)) & 0b1;

            vec_board[r+sq_r*3][c+sq_c*3] = match v  {
                1 => true,
                0 => false,
                _ => panic!()
            };

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

vec_board    
}

}

impl Game for TicTacToeGame {
    fn new() -> Self {
        TicTacToeGame {
            p_boards: [0; 2],
            p_squares: [0; 2],
            locked_squares: 0,

            active: true,
            active_player: 0,
            turn: 0,

            last_move: 0,
            last_move_result: None,
            winners: None,
        }
    }

    fn turn(&self) -> Option<Message> {
        // If game is over, return None
        if self.active == false {
            return None;
        }

        let mut out: Vec<String> = Vec::new();

        // (1) Output last move
        if self.last_move == 0 {
            out.push(String::from("-1 -1"));
        }
        else {
            let m = TicTacToeGame::move81_to_movetuple(self.last_move);
            out.push(format!("{} {}", m.0, m.1));
        }

        // (2) Output # of valid moves
        let valid_moves = TicTacToeGame::moves81_to_movetuples(TicTacToeGame::valid_moves(&self.p_boards, self.locked_squares, self.last_move));

        out.push(valid_moves.len().to_string());

        // (3) Output valid moves
        for m in valid_moves.iter() {
            out.push(format!("{} {}", m.0, m.1));
        }

        Some(Message {
            player_id: self.active_player as usize,
            messages: out,
        })
    }

    fn play(&mut self, msg: String) {
        // (1) Extract move information
        let _move = msg.split(" ").collect::<Vec<_>>();
        let row = parse_input!(_move[0], u8);
        let col = parse_input!(_move[1], u8);

        let move81 = TicTacToeGame::movetuple_to_move81((row, col));
        let square81 = TicTacToeGame::square_of_move81(move81);

        let pid = self.active_player;

        // (2) Check if move is valid
        let valid_moves =
            TicTacToeGame::valid_moves(&self.p_boards, self.locked_squares, self.last_move);
        if valid_moves & move81 == 0 {
            eprintln!("[GAME] Received invalid move {}", msg);
            self.last_move_result = Some(MoveResult::InvalidMove);
            self.active = false;
            self.winners = if pid == 0 {
                Some((WinLossTie::Loss, WinLossTie::Win))
            } else {
                Some((WinLossTie::Win, WinLossTie::Loss))
            };

            return;
        }

        self.last_move_result = Some(MoveResult::NormalMove);
        // (3) Perform move and update game state
        //  (3.1) Place move on board
        self.p_boards[pid as usize] |= move81;

        //  (3.2) Check if the player won the square
        if TicTacToeGame::won_the_square(self.p_boards[pid as usize], square81) {
            self.last_move_result = Some(MoveResult::MoveWinningSquare);
            // Update the player's square status
            self.p_squares[pid as usize] |= square81;
            // Update the locked square status
            self.locked_squares |= square81;

        }
        // (3.3) If the player didn't win the square, check if it's filled
        else if (self.p_boards[0] | self.p_boards[1]) & square81 == square81 {
            self.last_move_result = Some(MoveResult::MoveFillingSquareWithoutWinningIt);
            self.locked_squares |= square81;
        }
        // (4) Check if it's a global winning move or a tie
        if TicTacToeGame::won_the_board(self.p_squares[pid as usize]) {
            self.last_move_result = Some(MoveResult::MoveWinningBoard);
            self.active = false;
            self.winners = if pid == 0 {
                Some((WinLossTie::Win, WinLossTie::Loss))
            } else {
                Some((WinLossTie::Loss, WinLossTie::Win))
            }
        } else if self.locked_squares == 0b111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111 {
            self.last_move_result = Some(MoveResult::MoveFillingBoardWithoutWinning);
            self.active = false;
            let won_squares = [
                self.p_squares[0].count_ones(),
                self.p_squares[1].count_ones(),
            ];
            if won_squares[0] > won_squares[1] {
                self.winners = Some((WinLossTie::Win, WinLossTie::Loss));
            } else if won_squares[0] < won_squares[1] {
                self.winners = Some((WinLossTie::Loss, WinLossTie::Win));
            } else {
                self.winners = Some((WinLossTie::Tie, WinLossTie::Tie));
            }
        }

        self.turn += 1;
        self.last_move = move81;

        if self.active == true {
            self.active_player = (self.active_player + 1) % 2;
        }
    }

    fn winners(&self) -> Option<Vec<WinLossTie>> {
        match &self.winners {
            Some(w) => Some(vec![w.0, w.1]),
            None => None,
        }
    }

    fn get_state(&self) -> record::GameState {
        let mut board: Vec<Vec<String>> = Vec::new();

        // Create Record Board
        let board_p0: Vec<Vec<bool>> = TicTacToeGame::to_vector(self.p_boards[0]);
        let board_p1: Vec<Vec<bool>> = TicTacToeGame::to_vector(self.p_boards[1]);
        let squares_p0 :Vec<Vec<bool>> = TicTacToeGame::to_vector(self.p_squares[0]); 
        let squares_p1: Vec<Vec<bool>> = TicTacToeGame::to_vector(self.p_squares[1]);
        let locked_squares :Vec<Vec<bool>> = TicTacToeGame::to_vector(self.locked_squares);

        for r in 0..9 {
            let mut row: Vec<String> = Vec::new();
            for c in 0..9 {
                let mut cell_state = String::new();

                 // (1) Check if cell is occupied
                if board_p0[r][c] == true {
                    cell_state.push('❌');
                }
                else if board_p1[r][c] == true {
                    cell_state.push('⭕');
                }
                else {
                    cell_state.push('.');
                }


                // (2) Check if square is occupied
                if squares_p0[r][c] == true {
                    cell_state.push('❌');
                }
                else if squares_p1[r][c] == true {
                    cell_state.push('⭕');
                }
                else if locked_squares[r][c] == true {
                    cell_state.push('🔒');
                }
                else {
                    cell_state.push('.');
                }

                row.push(cell_state);
            }
            board.push(row);
        }

        // Record other state variables
        let mut state: HashMap<&str, String> = HashMap::new();
        state.insert("turn", self.turn.to_string());
        state.insert("active", self.active.to_string());
        state.insert("active_player", self.active_player.to_string());
        state.insert(
            "last_move",
            match self.last_move {
                0 => String::from("None"),
                _ => {
                    let mt = TicTacToeGame::move81_to_movetuple(self.last_move);
                    format!("({},{})", mt.0, mt.1)},
            },
        );
        state.insert(
            "last_move_result",
            match &self.last_move_result {
                None => String::from("None"),
                Some(mr) => mr.to_string(),
            },
        );

        state.insert("p_board[0]", format!("{:0>81b}", self.p_boards[0]));
        state.insert("p_squares[0]", format!("{:0>81b}", self.p_squares[0]));
        state.insert("p_board[1]", format!("{:0>81b}", self.p_boards[1]));
        state.insert("p_squares[1]", format!("{:0>81b}", self.p_squares[1]));


        state.insert("locked_squares", format!("{:0>81b}", self.locked_squares));

        record::GameState { board, state }
    }

    fn get_board_representation() -> record::BoardRepresentation {
        let mut classes: Vec<HashMap<char, record::CellClass>> = Vec::new();

        // First position
        let mut class_styles: HashMap<char, record::CellClass> = HashMap::new();

        class_styles.insert(
            '❌',
            record::CellClass {
                text: Some('❌'.to_string()),
                text_style: Some({
                    let mut css = HashMap::new();
                    css.insert("color".to_string(), "transparent".to_string());
                    css.insert("textShadow".to_string(), "0 0 0 #F2B213".to_string());
                    css
                }),
                cell_style: None,
            },
        );
        class_styles.insert(
            '⭕',
            record::CellClass {
                text: Some('⭕'.to_string()),
                text_style: Some({
                    let mut css = HashMap::new();
                    css.insert("color".to_string(), "transparent".to_string());
                    css.insert("textShadow".to_string(), "0 0 0 #22A1E4".to_string());
                    css
                }),
                cell_style: None,
            },
        );
        class_styles.insert(
            '.',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: None,
            },
        );

        classes.push(class_styles);

        // Second position
        let mut class_styles: HashMap<char, record::CellClass> = HashMap::new();

        class_styles.insert(
            '❌',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("backgroundColor".to_string(), "#fcf0d0".to_string());
                    css
                }),
            },
        );

        class_styles.insert(
            '⭕',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("backgroundColor".to_string(), "#d3ecfa".to_string());
                    css
                }),
            },
        );

        class_styles.insert(
            '🔒',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("backgroundColor".to_string(), "#adadad".to_string());
                    css
                }),
            },
        );
        class_styles.insert(
            '.',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: None,
            },
        );

        classes.push(class_styles);

        record::BoardRepresentation {
            rows: 9,
            cols: 9,
            classes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::assert_vec_eq;

    #[test]
    fn test_movetuple_to_move81() {
        assert_eq!(TicTacToeGame::movetuple_to_move81((0, 0)),0b100000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((0, 4)),0b000000000_010000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((0, 8)),0b000000000_000000000_001000000_000000000_000000000_000000000_000000000_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((2, 6)),0b000000000_000000000_000000100_000000000_000000000_000000000_000000000_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((4, 1)),0b000000000_000000000_000000000_000010000_000000000_000000000_000000000_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((5, 3)),0b000000000_000000000_000000000_000000000_000000100_000000000_000000000_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((6, 8)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_001000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((7, 0)),0b000000000_000000000_000000000_000000000_000000000_000000000_000100000_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((7, 6)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000100000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((8, 0)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000100_000000000_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((8, 3)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000100_000000000);
        assert_eq!(TicTacToeGame::movetuple_to_move81((8, 8)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000001);
    }

    #[test]
    fn test_move81_to_movetuple() {
        assert_eq!(TicTacToeGame::move81_to_movetuple(0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000001000_000000000),(7,5));
        assert_eq!(TicTacToeGame::move81_to_movetuple(0b000000000_001000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000),(0,5));
        assert_eq!(TicTacToeGame::move81_to_movetuple(0b000000000_000000000_000000000_000000000_000000000_000000000_010000000_000000000_000000000),(6,1));
        assert_eq!(TicTacToeGame::move81_to_movetuple(0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_010000000_000000000),(6,4));
        assert_eq!(TicTacToeGame::move81_to_movetuple(0b100000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000),(0,0));
        assert_eq!(TicTacToeGame::move81_to_movetuple(0b000000000_000000000_000000000_100000000_000000000_000000000_000000000_000000000_000000000),(3,0));
    }

    #[test]
    fn test_moves81_to_movetuples() {
        let moves:u128 = 0b100000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let move_tuples = TicTacToeGame::moves81_to_movetuples(moves);
        let expected_move_tuples = vec![(0,0)];
        assert_vec_eq!(move_tuples,expected_move_tuples);

        let moves:u128 = 0b100011010_010101010_100010010_001101111_011001100_100111101_101000000_001111110_001010001;
        let move_tuples = TicTacToeGame::moves81_to_movetuples(moves);
        let expected_move_tuples = vec![(0,0),(1,1),(1,2),(2,1),(0,4),(1,3),(1,5),(2,4),(0,6),(1,7),(2,7),(3,2),(4,0),(4,2),(5,0),(5,1),(5,2),(3,4),(3,5),(4,5),(5,3),(3,6),(4,6),(4,7),(4,8),(5,6),(5,8),(6,0),(6,2),(6,5),(7,3),(7,4),(7,5),(8,3),(8,4),(6,8),(7,7),(8,8)];
        assert_vec_eq!(move_tuples,expected_move_tuples);

    }


    #[test]
    fn test_square_of_move81() {
        //(0,0)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(80)),0b111111111_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        //(0, 4)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(70)),0b000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        //(0, 8)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(60)),0b000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000);
        //(2, 6)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(56)),0b000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000);
        //(4, 1)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(49)),0b000000000_000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000);
        //(5, 3)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(38)),0b000000000_000000000_000000000_000000000_111111111_000000000_000000000_000000000_000000000);
        //(6, 8)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(6)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111);
        //(7, 0)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(23)),0b000000000_000000000_000000000_000000000_000000000_000000000_111111111_000000000_000000000);
        //(7, 6)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(5)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111);
        //(8, 0)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(20)),0b000000000_000000000_000000000_000000000_000000000_000000000_111111111_000000000_000000000);
        //(8, 3)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(11)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111_000000000);
        //(8, 8)
        assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(0)),0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111);

        for i in 72..=80 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b111111111_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        }
        for i in 63..=71 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        }
        for i in 54..=62 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000);
        }
        for i in 45..=53 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000);
        }
        for i in 36..=44 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_111111111_000000000_000000000_000000000_000000000);
        }
        for i in 27..=35 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_111111111_000000000_000000000_000000000);
        }
        for i in 18..=26 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_000000000_111111111_000000000_000000000);
        }
        for i in 9..=17 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111_000000000);
        }
        for i in 0..=8 {
            assert_eq!(TicTacToeGame::square_of_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111);
        }
    }

    
    #[test]
    fn test_square_pointed_by_move81() {
        for i in (8..=80).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b111111111_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        }
        for i in (7..=79).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
        }
        for i in (6..=78).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000);
        }
        for i in (5..=77).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000);
        }
        for i in (4..=76).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_111111111_000000000_000000000_000000000_000000000);
        }
        for i in (3..=75).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_111111111_000000000_000000000_000000000);
        }
        for i in (2..=74).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_000000000_111111111_000000000_000000000);
        }
        for i in (1..=73).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111_000000000);
        }
        for i in (0..=72).step_by(9) {
            assert_eq!(TicTacToeGame::square_pointed_by_move81(2_u128.pow(i)), 0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000_111111111);
        }
    }

    #[test]
    fn test_valid_moves() {
        let mut game = TicTacToeGame::new();

        let expected_moves:u128= 0b111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111_111111111;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("5 7"));
        game.play(String::from("6 4"));

        let expected_moves:u128= 0b000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("2 5"));
        game.play(String::from("8 7"));

        let expected_moves:u128= 0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_101111111_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("8 4"));
        game.play(String::from("7 4"));

        let expected_moves:u128= 0b000000000_000000000_000000000_000000000_111111111_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("4 5"));
        game.play(String::from("3 8"));

        let expected_moves:u128= 0b000000000_000000000_111111111_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("1 8"));
        game.play(String::from("3 7"));

        let expected_moves:u128= 0b000000000_111111110_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("0 3"));
        game.play(String::from("2 1"));

        let expected_moves:u128= 0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_101101101_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("7 3"));
        game.play(String::from("3 1"));

        let expected_moves:u128= 0b000000000_011111110_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("0 4"));
        game.play(String::from("0 5"));

        let expected_moves:u128= 0b000000000_000000000_111110111_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("2 6"));
        game.play(String::from("7 1"));

        let expected_moves:u128= 0b000000000_000000000_000000000_000000000_111110111_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("4 4"));
        game.play(String::from("3 4"));

        let expected_moves:u128= 0b000000000_000111110_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("2 4"));
        game.play(String::from("6 3"));

        let expected_moves:u128= 0b111111101_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("1 1"));
        game.play(String::from("3 5"));

        let expected_moves:u128= 0b000000000_000000000_111110011_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("0 8"));
        game.play(String::from("1 6"));

        let expected_moves:u128= 0b000000000_000000000_000000000_101111111_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("5 0"));
        game.play(String::from("6 1"));

        let expected_moves:u128= 0b000000000_000111100_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("1 4"));
        game.play(String::from("3 3"));

        let expected_moves:u128= 0b111101101_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("0 1"));
        game.play(String::from("8 1"));

        let expected_moves:u128= 0b000000000_000000000_000000000_000000000_000000000_000000000_000000000_001001101_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("6 5"));
        game.play(String::from("1 7"));

        let expected_moves:u128= 0b101101101_000000000_110000011_101111011_000000000_100111101_000000000_000001101_111111101;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("8 5"));
        game.play(String::from("6 7"));

        let expected_moves:u128= 0b101101101_000000000_110000011_101111011_000000000_100111101_000000000_000001100_101111101;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("5 8"));
        game.play(String::from("7 7"));

        let expected_moves:u128= 0b101101101_000000000_110000011_101111011_000000000_100111100_000000000_000001100_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("4 1"));
        game.play(String::from("3 6"));

        let expected_moves:u128= 0b101101101_000000000_000000000_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

        game.play(String::from("1 0"));
        game.play(String::from("3 2"));

        let expected_moves:u128= 0b000000000_000000000_110000011_000000000_000000000_000000000_000000000_000000000_000000000;
        let valid_moves = TicTacToeGame::valid_moves(&game.p_boards, game.locked_squares, game.last_move);
        assert_eq!(expected_moves,valid_moves);

    }
    
}
