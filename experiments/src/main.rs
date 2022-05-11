#![allow(warnings, unused)]

mod original {

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

    #[derive(Clone, Debug)]
    enum WinLossTie {
        Win,
        Loss,
        Tie,
    }

    fn cell99_to_cell33(cell: (u8, u8)) -> (u8, u8) {
        (cell.0 % 3, cell.1 % 3)
    }

    fn cell33_to_cell99(cell: (u8, u8), square: (u8, u8)) -> (u8, u8) {
        (cell.0 + square.0 * 3, cell.1 + square.1 * 3)
    }

    fn get_bit(bit_9: u16, row: u8, col: u8) -> u8 {
        ((bit_9 >> (2 - row) * 3 + (2 - col)) & 0b1) as u8
    }

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
}

fn main() {
    use std::time::{Duration, Instant};
    let N: i128 = 10_000_000;

    let mut state = original::new();

    let start = Instant::now();
    for i in 0..N {
        original::valid_moves(&state);
    }

    println!("Original : {:?}", start.elapsed());
}
