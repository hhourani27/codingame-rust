pub mod simulator;
use serde::Serialize;

#[macro_export]
macro_rules! assert_vec_eq {
    ($v1:expr, $v2:expr) => {
        assert!($v1.iter().all(|m| $v2.contains(m)));
        assert!($v2.iter().all(|m| $v1.contains(m)));
        assert_eq!($v1.len(), $v2.len());
    };
}

#[derive(Debug)]
pub struct Message {
    pub player_id: usize,
    pub messages: Vec<String>,
}

#[allow(non_snake_case)]
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

pub trait Game {
    fn new() -> Self;

    fn turn(&self) -> Option<Message>;

    fn play(&mut self, msg: String);

    fn winners(&self) -> Option<Vec<WinLossTie>>;

    fn get_state(&self) -> record::GameState;

    fn get_board_representation() -> record::BoardRepresentation;
}

#[derive(Serialize, Debug, Clone, Copy)]
pub enum WinLossTie {
    Win,
    Loss,
    Tie,
}

pub mod record {
    use super::WinLossTie;
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Serialize)]
    pub struct Record {
        pub board_representation: BoardRepresentation,
        pub game_runs: Vec<GameRun>,
    }

    #[derive(Serialize)]
    pub struct BoardRepresentation {
        pub rows: u32,
        pub cols: u32,
        pub classes: Vec<HashMap<char, CellClass>>,
    }

    #[derive(Serialize)]
    pub struct CellClass {
        pub text: Option<String>,
        pub text_style: Option<HashMap<String, String>>,
        pub cell_style: Option<HashMap<String, String>>,
    }

    #[derive(Serialize)]
    pub struct GameRun {
        pub run_id: u32,
        pub total_turns: u32,
        pub turns: Vec<GameTurn>,
        pub final_state: GameState,
        pub winners: Vec<WinLossTie>,
    }

    #[derive(Serialize)]
    pub struct GameTurn {
        pub turn: u32,
        pub game_state: GameState,
        pub player: u32,
        pub player_input: Vec<String>,
        pub player_move: String,
    }

    #[derive(Serialize, Default)]
    pub struct GameState {
        pub board: Vec<Vec<String>>,
        pub state: HashMap<&'static str, String>,
    }
}
