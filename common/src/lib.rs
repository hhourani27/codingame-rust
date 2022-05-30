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
#[derive(Clone)]
pub struct StackVector<T: Copy + Clone + Default, const MAX_SIZE: usize> {
    pub arr: [T; MAX_SIZE],
    pub len: usize,
}

impl<T: Copy + Clone + Default, const MAX_SIZE: usize> StackVector<T, MAX_SIZE> {
    pub fn new() -> Self {
        Self {
            arr: [Default::default(); MAX_SIZE],
            len: 0,
        }
    }

    pub fn add(&mut self, e: T) {
        self.arr[self.len] = e;
        self.len += 1;
    }

    pub fn slice(&self) -> &[T] {
        &self.arr[0..self.len]
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        &mut self.arr[0..self.len]
    }

    pub fn get(&self, idx: usize) -> &T {
        &self.arr[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.arr[idx]
    }

    pub fn remove(&mut self, idx: usize) -> T {
        let removed_element = self.arr[idx];

        for i in idx..self.len - 1 {
            self.arr[i] = self.arr[i + 1];
        }
        self.len -= 1;

        removed_element
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub trait Game {
    fn new() -> Self;

    fn turn(&self) -> Option<Message>;

    fn play(&mut self, msg: String);

    fn winners(&self) -> Option<Vec<WinLossTie>>;

    fn get_state(&self) -> record::GameState;

    fn get_board_representation() -> Option<record::BoardRepresentation>;
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
        pub board_representation: Option<BoardRepresentation>,
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
        pub board: Option<Vec<Vec<String>>>,
        pub state: HashMap<String, String>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stackvector_remove() {
        let mut list: StackVector<u8, 10> = StackVector::new();
        for i in 0..5 {
            list.add(i);
        }

        assert_eq!(list.len(), 5);

        let e = list.remove(2);

        assert_eq!(e, 2);
        assert_eq!(list.len(), 4);
        assert_eq!(*list.get(0), 0);
        assert_eq!(*list.get(1), 1);
        assert_eq!(*list.get(2), 3);
        assert_eq!(*list.get(3), 4);
    }

    #[test]
    fn test_stackvector_slice_after_remove() {
        let mut list: StackVector<u8, 10> = StackVector::new();
        for i in 0..5 {
            list.add(i);
        }

        assert_eq!(list.slice().len(), 5);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![0, 1, 2, 3, 4]);

        list.remove(2);
        assert_eq!(list.slice().len(), 4);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![0, 1, 3, 4]);
    }

    #[test]
    fn test_stackvector_full_remove_last_element() {
        let mut list: StackVector<u8, 10> = StackVector::new();
        for i in 0..10 {
            list.add(i);
        }

        list.remove(9);
        assert_eq!(list.slice().len(), 9);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        list.remove(0);
        assert_eq!(list.slice().len(), 8);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_stackvector_full_remove_first_element() {
        let mut list: StackVector<u8, 10> = StackVector::new();
        for i in 0..10 {
            list.add(i);
        }

        list.remove(0);
        assert_eq!(list.slice().len(), 9);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
