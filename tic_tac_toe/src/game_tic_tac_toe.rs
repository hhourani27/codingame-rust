use common::assert_vec_eq;
use common::Game;
use common::Message;
use itertools::iproduct;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[derive(Debug)]
struct TicTacToeGame {
    /*
    2D array
    [pid] => [u16,u16,u16,u16,u16,u16,u16,u16,u16] (9 squares)
        Each u16 correspond to a bit representation of a square.
    */
    p_boards: [[u16; 9]; 2],
    // A 2D array : [player_id] => 8-bit number representing which squares are won by a player
    p_squares: [u16; 2],
    // represent which squares are locked
    locked_squares: u16,

    active: bool,
    active_player: u8,
    turn: u8,

    last_move: Option<(u8, u8)>,
    winners: Option<(bool, bool)>,
}

impl TicTacToeGame {
    fn new() -> TicTacToeGame {
        TicTacToeGame {
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

    fn valid_moves(
        p_boards: &[[u16; 9]; 2],
        locked_squares: u16,
        last_move: Option<(u8, u8)>,
    ) -> Vec<(u8, u8)> {
        // (1) Determine valid squares
        let valid_squares: u16 = match last_move {
            // If it's the first move, all squares are valid
            None => 0b111_111_111,
            // Else, check the last move
            Some(m) => {
                // Get the square referred to by the cell move
                let sq = TicTacToeGame::cell99_to_cell33(m);
                if TicTacToeGame::get_bit(locked_squares, sq.0, sq.1) == 0 {
                    // If the square is not locked, only one square is valid
                    0b1 << (2 - sq.0) * 3 + (2 - sq.1)
                } else {
                    // if square is locked, all non-locked squares are valid
                    !locked_squares
                }
            }
        };

        // (2) For each valid square add list of valid moves
        let mut valid_moves: Vec<(u8, u8)> = Vec::new();
        for (r, c) in iproduct!(0..3, 0..3) {
            if TicTacToeGame::get_bit(valid_squares, r, c) == 1 {
                let sq_ix = (r * 3 + c) as usize;
                let valid_moves_in_square =
                    TicTacToeGame::valid_moves_in_square(p_boards[0][sq_ix] | p_boards[1][sq_ix]);

                for m_sq in valid_moves_in_square {
                    valid_moves.push(TicTacToeGame::cell33_to_cell99(m_sq, (r, c)));
                }
            }
        }
        valid_moves
    }

    /*
        For a 9-bit representation of a square, return empty cells (0..2, 0..2)
    */
    fn valid_moves_in_square(square_bin: u16) -> Vec<(u8, u8)> {
        let mut valid_moves: Vec<(u8, u8)> = Vec::new();
        let sq = square_bin;
        for sh in 0..9 {
            if (sq >> sh) & 0b1 == 0 {
                valid_moves.push(((8 - sh) / 3, (8 - sh) % 3));
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

        winning_configurations.contains(&bit_9)
    }
}

impl Game for TicTacToeGame {
    fn turn(&self) -> Option<Message> {
        // If game is over, return None
        if self.active == false {
            return None;
        }

        let mut out: Vec<String> = Vec::new();

        // (1) Output last move
        out.push(match self.last_move {
            None => String::from("-1 -1"),
            Some(m) => format!("{} {}", m.0, m.1),
        });

        // (2) Output # of valid moves
        let valid_moves =
            TicTacToeGame::valid_moves(&self.p_boards, self.locked_squares, self.last_move);

        out.push(valid_moves.len().to_string());

        // (3) Output valid moves
        for m in valid_moves {
            out.push(format!("{} {}", m.0, m.1));
        }

        Some(Message {
            player_id: self.active_player,
            message: out,
        })
    }

    fn play(&mut self, msg: String) {
        // (1) Extract move information
        let _move = msg.split(" ").collect::<Vec<_>>();
        let row = parse_input!(_move[0], u8);
        let col = parse_input!(_move[1], u8);

        let pid = self.active_player;

        // (2) Check if move is valid
        if !TicTacToeGame::valid_moves(&self.p_boards, self.locked_squares, self.last_move)
            .contains(&(row, col))
        {
            self.active = false;
            self.winners = if pid == 0 {
                Some((false, true))
            } else {
                Some((true, false))
            }
        }

        // (3) Perform move and update game state
        //  (3.1) Place move on board
        let square = TicTacToeGame::square_of_cell((row, col));
        let (row33, col33) = TicTacToeGame::cell99_to_cell33((row, col));
        let sq_idx: usize = (square.0 * 3 + square.1) as usize;
        self.p_boards[pid as usize][sq_idx] =
            TicTacToeGame::set_bit(self.p_boards[pid as usize][sq_idx], row33, col33);

        //  (3.2) Check if the player won the square
        let p_square = self.p_boards[pid as usize][sq_idx];
        if TicTacToeGame::is_won(p_square) {
            // Update the player's square status
            self.p_squares[pid as usize] =
                TicTacToeGame::set_bit(self.p_squares[pid as usize], square.0, square.1);

            // Update the locked square status
            self.locked_squares = TicTacToeGame::set_bit(self.locked_squares, square.0, square.1);
        }
        // (3.3) If the player didn't win the square, check if it's filled
        else if self.p_boards[0][sq_idx] | self.p_boards[1][sq_idx] == 0b111_111_111 {
            self.locked_squares = TicTacToeGame::set_bit(self.locked_squares, square.0, square.1);
        }

        // (4) Check if it's a winning move or a tie
        if TicTacToeGame::is_won(self.p_squares[pid as usize]) {
            self.active = false;
            self.winners = if pid == 0 {
                Some((true, false))
            } else {
                Some((false, true))
            }
        } else if self.locked_squares == 0b111_111_111 {
            self.active = false;
            let won_squares = [
                self.p_squares[0].count_ones(),
                self.p_squares[1].count_ones(),
            ];
            if won_squares[0] > won_squares[1] {
                self.winners = Some((true, false));
            } else if won_squares[0] < won_squares[1] {
                self.winners = Some((false, true));
            } else {
                self.winners = Some((true, true));
            }
        }

        self.last_move = Some((row, col));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell99_to_cell33() {
        assert_eq!(TicTacToeGame::cell99_to_cell33((0, 0)), (0, 0));
        assert_eq!(TicTacToeGame::cell99_to_cell33((0, 4)), (0, 1));
        assert_eq!(TicTacToeGame::cell99_to_cell33((0, 8)), (0, 2));
        assert_eq!(TicTacToeGame::cell99_to_cell33((2, 6)), (2, 0));
        assert_eq!(TicTacToeGame::cell99_to_cell33((4, 1)), (1, 1));
        assert_eq!(TicTacToeGame::cell99_to_cell33((5, 3)), (2, 0));
        assert_eq!(TicTacToeGame::cell99_to_cell33((6, 8)), (0, 2));
        assert_eq!(TicTacToeGame::cell99_to_cell33((7, 0)), (1, 0));
        assert_eq!(TicTacToeGame::cell99_to_cell33((7, 6)), (1, 0));
        assert_eq!(TicTacToeGame::cell99_to_cell33((8, 0)), (2, 0));
        assert_eq!(TicTacToeGame::cell99_to_cell33((8, 3)), (2, 0));
        assert_eq!(TicTacToeGame::cell99_to_cell33((8, 8)), (2, 2));
    }

    #[test]
    fn test_cell33_to_cell_99() {
        let (cell33, sq, exp_cell99) = ((0, 0), (0, 0), (0, 0));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((0, 1), (0, 1), (0, 4));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((0, 2), (0, 2), (0, 8));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((2, 0), (0, 2), (2, 6));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((1, 1), (1, 0), (4, 1));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((2, 0), (1, 1), (5, 3));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((0, 2), (2, 2), (6, 8));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((1, 0), (2, 0), (7, 0));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((1, 0), (2, 2), (7, 6));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((2, 0), (2, 0), (8, 0));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((2, 0), (2, 1), (8, 3));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);

        let (cell33, sq, exp_cell99) = ((2, 2), (2, 2), (8, 8));
        assert_eq!(TicTacToeGame::cell33_to_cell99(cell33, sq), exp_cell99);
    }

    #[test]
    fn test_square_of_cell() {
        assert_eq!(TicTacToeGame::square_of_cell((1, 2)), (0, 0));
        assert_eq!(TicTacToeGame::square_of_cell((4, 4)), (1, 1));
        assert_eq!(TicTacToeGame::square_of_cell((4, 7)), (1, 2));
        assert_eq!(TicTacToeGame::square_of_cell((6, 1)), (2, 0));
        assert_eq!(TicTacToeGame::square_of_cell((8, 7)), (2, 2));
    }

    #[test]
    fn test_get_bit() {
        assert_eq!(TicTacToeGame::get_bit(0b110_111_011, 0, 0), 1);
        assert_eq!(TicTacToeGame::get_bit(0b011_010_001, 0, 1), 1);
        assert_eq!(TicTacToeGame::get_bit(0b000_110_000, 0, 2), 0);
        assert_eq!(TicTacToeGame::get_bit(0b101_111_011, 1, 0), 1);
        assert_eq!(TicTacToeGame::get_bit(0b011_110_000, 1, 1), 1);
        assert_eq!(TicTacToeGame::get_bit(0b000_001_111, 1, 2), 1);
        assert_eq!(TicTacToeGame::get_bit(0b100_000_110, 2, 0), 1);
        assert_eq!(TicTacToeGame::get_bit(0b100_000_011, 2, 1), 1);
        assert_eq!(TicTacToeGame::get_bit(0b110_101_110, 2, 2), 0);
    }

    #[test]
    fn test_set_bit() {
        assert_eq!(TicTacToeGame::set_bit(0b000_000_000, 0, 0), 0b100_000_000);
        assert_eq!(TicTacToeGame::set_bit(0b110_111_011, 0, 2), 0b111_111_011);
        assert_eq!(TicTacToeGame::set_bit(0b011_110_000, 1, 2), 0b011_111_000);
        assert_eq!(TicTacToeGame::set_bit(0b100_000_011, 2, 0), 0b100_000_111);
    }

    #[test]
    fn test_valid_moves_in_square() {
        let valid_moves = TicTacToeGame::valid_moves_in_square(0b110_111_011);
        let expected_moves = vec![(0, 2), (2, 0)];
        assert!(expected_moves.iter().all(|m| valid_moves.contains(m)));

        let valid_moves = TicTacToeGame::valid_moves_in_square(0b011_010_001);
        let expected_moves = vec![(0, 0), (1, 0), (1, 2), (2, 0), (2, 1)];
        assert!(expected_moves.iter().all(|m| valid_moves.contains(m)));

        let valid_moves = TicTacToeGame::valid_moves_in_square(0b000_110_000);
        let expected_moves = vec![(0, 0), (0, 1), (0, 2), (1, 2), (2, 0), (2, 1), (2, 2)];
        assert!(expected_moves.iter().all(|m| valid_moves.contains(m)));

        let valid_moves = TicTacToeGame::valid_moves_in_square(0b101_111_011);
        let expected_moves = vec![(0, 1), (2, 0)];
        assert!(expected_moves.iter().all(|m| valid_moves.contains(m)));

        let valid_moves = TicTacToeGame::valid_moves_in_square(0b100_000_110);
        let expected_moves = vec![(0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (2, 2)];
        assert!(expected_moves.iter().all(|m| valid_moves.contains(m)));

        let valid_moves = TicTacToeGame::valid_moves_in_square(0b000_000_000);
        let expected_moves = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ];
        assert!(expected_moves.iter().all(|m| valid_moves.contains(m)));

        let valid_moves = TicTacToeGame::valid_moves_in_square(0b111_111_111);
        assert_eq!(valid_moves.len(), 0)
    }

    #[test]
    fn test_valid_moves() {
        fn place_move(p_board: &mut [u16; 9], cell: (u8, u8)) {
            let square = TicTacToeGame::square_of_cell(cell);
            let cell33 = TicTacToeGame::cell99_to_cell33(cell);
            let sq_idx: usize = (square.0 * 3 + square.1) as usize;
            p_board[sq_idx] = TicTacToeGame::set_bit(p_board[sq_idx], cell33.0, cell33.1);
        }

        //
        let mut p_boards = [[0b000_000_000; 9]; 2];
        let mut locked_squares = 0b000_000_000;
        let mut last_move: Option<(u8, u8)> = None;

        let expected_moves: Vec<(u8, u8)> = iproduct!(0..9, 0..9).collect();
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (5, 7));
        place_move(&mut p_boards[1], (6, 4));
        locked_squares = 0b000_000_000;
        last_move = Some((6, 4));

        let expected_moves: Vec<(u8, u8)> = vec![
            (2, 3),
            (2, 5),
            (2, 4),
            (0, 3),
            (0, 4),
            (1, 3),
            (0, 5),
            (1, 5),
            (1, 4),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (2, 5));
        place_move(&mut p_boards[1], (8, 7));
        locked_squares = 0b000_000_000;
        last_move = Some((8, 7));

        let expected_moves: Vec<(u8, u8)> = vec![
            (8, 5),
            (7, 3),
            (7, 5),
            (6, 5),
            (7, 4),
            (6, 3),
            (8, 3),
            (8, 4),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (8, 4));
        place_move(&mut p_boards[1], (7, 4));
        locked_squares = 0b000_000_000;
        last_move = Some((7, 4));

        let expected_moves: Vec<(u8, u8)> = vec![
            (4, 3),
            (4, 4),
            (5, 5),
            (5, 3),
            (3, 3),
            (3, 4),
            (5, 4),
            (4, 5),
            (3, 5),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (4, 5));
        place_move(&mut p_boards[1], (3, 8));
        locked_squares = 0b000_000_000;
        last_move = Some((3, 8));

        let expected_moves: Vec<(u8, u8)> = vec![
            (1, 8),
            (0, 7),
            (2, 6),
            (2, 8),
            (1, 7),
            (1, 6),
            (2, 7),
            (0, 6),
            (0, 8),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (1, 8));
        place_move(&mut p_boards[1], (3, 7));
        locked_squares = 0b000_000_000;
        last_move = Some((3, 7));

        let expected_moves: Vec<(u8, u8)> = vec![
            (1, 3),
            (2, 3),
            (0, 5),
            (1, 5),
            (0, 4),
            (2, 4),
            (0, 3),
            (1, 4),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (0, 3));
        place_move(&mut p_boards[1], (2, 1));
        locked_squares = 0b000_000_000;
        last_move = Some((2, 1));

        let expected_moves: Vec<(u8, u8)> = vec![(8, 5), (7, 3), (6, 3), (8, 3), (6, 5), (7, 5)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (7, 3));
        place_move(&mut p_boards[1], (3, 1));
        locked_squares = 0b000_000_000;
        last_move = Some((3, 1));

        let expected_moves: Vec<(u8, u8)> =
            vec![(0, 5), (0, 4), (2, 4), (2, 3), (1, 5), (1, 3), (1, 4)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (0, 4));
        place_move(&mut p_boards[1], (0, 5));
        locked_squares = 0b000_000_000;
        last_move = Some((0, 5));

        let expected_moves: Vec<(u8, u8)> = vec![
            (1, 6),
            (0, 8),
            (0, 7),
            (2, 7),
            (1, 7),
            (2, 8),
            (0, 6),
            (2, 6),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (2, 6));
        place_move(&mut p_boards[1], (7, 1));
        locked_squares = 0b000_000_000;
        last_move = Some((7, 1));

        let expected_moves: Vec<(u8, u8)> = vec![
            (4, 4),
            (3, 4),
            (5, 4),
            (3, 3),
            (3, 5),
            (4, 3),
            (5, 5),
            (5, 3),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (4, 4));
        place_move(&mut p_boards[1], (3, 4));
        locked_squares = 0b000_000_000;
        last_move = Some((3, 4));

        let expected_moves: Vec<(u8, u8)> = vec![(2, 4), (1, 4), (1, 5), (1, 3), (2, 3)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (2, 4));
        place_move(&mut p_boards[1], (6, 3));
        locked_squares = 0b000_000_000;
        last_move = Some((6, 3));

        let expected_moves: Vec<(u8, u8)> = vec![
            (0, 0),
            (2, 0),
            (0, 1),
            (1, 1),
            (0, 2),
            (1, 0),
            (2, 2),
            (1, 2),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (1, 1));
        place_move(&mut p_boards[1], (3, 5));
        locked_squares = 0b000_000_000;
        last_move = Some((3, 5));

        let expected_moves: Vec<(u8, u8)> =
            vec![(0, 8), (1, 6), (1, 7), (2, 8), (2, 7), (0, 7), (0, 6)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (0, 8));
        place_move(&mut p_boards[1], (1, 6));
        locked_squares = 0b000_000_000;
        last_move = Some((1, 6));

        let expected_moves: Vec<(u8, u8)> = vec![
            (4, 1),
            (5, 0),
            (4, 0),
            (3, 2),
            (3, 0),
            (5, 1),
            (4, 2),
            (5, 2),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (5, 0));
        place_move(&mut p_boards[1], (6, 1));
        locked_squares = 0b000_000_000;
        last_move = Some((6, 1));

        let expected_moves: Vec<(u8, u8)> = vec![(2, 3), (1, 4), (1, 5), (1, 3)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (1, 4));
        place_move(&mut p_boards[1], (3, 3));
        locked_squares = 0b010010000;
        last_move = Some((3, 3));

        let expected_moves: Vec<(u8, u8)> =
            vec![(0, 0), (0, 1), (1, 2), (0, 2), (1, 0), (2, 0), (2, 2)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (0, 1));
        place_move(&mut p_boards[1], (8, 1));
        locked_squares = 0b010010100;
        last_move = Some((8, 1));

        let expected_moves: Vec<(u8, u8)> = vec![(6, 5), (7, 5), (8, 3), (8, 5)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (6, 5));
        place_move(&mut p_boards[1], (1, 7));
        locked_squares = 0b010010100;
        last_move = Some((1, 7));

        let expected_moves: Vec<(u8, u8)> = vec![
            (5, 6),
            (2, 2),
            (5, 8),
            (7, 7),
            (0, 2),
            (4, 7),
            (7, 5),
            (5, 2),
            (2, 0),
            (0, 7),
            (0, 6),
            (6, 8),
            (3, 6),
            (4, 1),
            (6, 6),
            (2, 8),
            (3, 2),
            (7, 6),
            (3, 0),
            (8, 3),
            (8, 6),
            (4, 2),
            (8, 8),
            (2, 7),
            (4, 0),
            (1, 2),
            (5, 1),
            (7, 8),
            (1, 0),
            (0, 0),
            (6, 7),
            (8, 5),
            (4, 6),
            (4, 8),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (8, 5));
        place_move(&mut p_boards[1], (6, 7));
        locked_squares = 0b010010100;
        last_move = Some((6, 7));

        let expected_moves: Vec<(u8, u8)> = vec![
            (7, 5),
            (0, 7),
            (2, 0),
            (3, 6),
            (7, 8),
            (8, 3),
            (3, 0),
            (1, 0),
            (2, 7),
            (1, 2),
            (5, 8),
            (2, 2),
            (4, 7),
            (7, 6),
            (8, 6),
            (2, 8),
            (0, 0),
            (6, 6),
            (5, 6),
            (4, 1),
            (0, 6),
            (8, 8),
            (0, 2),
            (6, 8),
            (4, 8),
            (4, 0),
            (7, 7),
            (5, 2),
            (3, 2),
            (4, 6),
            (5, 1),
            (4, 2),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (5, 8));
        place_move(&mut p_boards[1], (7, 7));
        locked_squares = 0b010010101;
        last_move = Some((7, 7));

        let expected_moves: Vec<(u8, u8)> = vec![
            (4, 1),
            (0, 0),
            (2, 2),
            (0, 7),
            (4, 8),
            (0, 6),
            (5, 2),
            (7, 5),
            (0, 2),
            (4, 7),
            (2, 0),
            (5, 6),
            (3, 2),
            (5, 1),
            (4, 6),
            (3, 0),
            (4, 2),
            (2, 8),
            (1, 0),
            (4, 0),
            (1, 2),
            (2, 7),
            (8, 3),
            (3, 6),
        ];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (4, 1));
        place_move(&mut p_boards[1], (3, 6));
        locked_squares = 0b010011101;
        last_move = Some((3, 6));

        let expected_moves: Vec<(u8, u8)> = vec![(2, 0), (0, 2), (1, 2), (1, 0), (2, 2), (0, 0)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);

        //
        place_move(&mut p_boards[0], (1, 0));
        place_move(&mut p_boards[1], (3, 2));
        locked_squares = 0b010011101;
        last_move = Some((3, 2));

        let expected_moves: Vec<(u8, u8)> = vec![(2, 8), (0, 6), (0, 7), (2, 7)];
        let valid_moves = TicTacToeGame::valid_moves(&p_boards, locked_squares, last_move);
        assert_vec_eq!(expected_moves, valid_moves);
    }
}
