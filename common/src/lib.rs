pub mod graph;
pub mod simulator;
use serde::Serialize;
use std::fmt::Display;

#[macro_export]
macro_rules! assert_vec_eq {
    ($v1:expr, $v2:expr) => {
        assert_eq!(
            $v1.len(),
            $v2.len(),
            "LEFT & RIGHT do not have the same number of elements. LEFT: {:?}, RIGHT: {:?}",
            &$v1,
            &$v2,
        );
        assert!(
            $v1.iter().all(|m| $v2.contains(m)),
            "Some elements in LEFT are not present in RIGHT. LEFT: {:?}, RIGHT: {:?}",
            &$v1,
            &$v2,
        );
        assert!(
            $v2.iter().all(|m| $v1.contains(m)),
            "Some elements in RIGHT are not present in LEFT. LEFT: {:?}, RIGHT: {:?}",
            &$v1,
            &$v2,
        );
    };
}

#[derive(Debug)]
pub struct Message {
    pub player_id: usize,
    pub messages: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct StackVector<T: Copy + Clone + Default, const MAX_SIZE: usize> {
    pub arr: [T; MAX_SIZE],
    pub len: usize,
}

#[allow(dead_code)]
impl<T: Copy + Clone + Default, const MAX_SIZE: usize> StackVector<T, MAX_SIZE> {
    pub fn new() -> Self {
        Self {
            arr: [T::default(); MAX_SIZE],
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

    pub fn remove_multi<const NB_ELEMENTS: usize>(
        &mut self,
        mut indices: [usize; NB_ELEMENTS],
    ) -> [T; NB_ELEMENTS] {
        let mut removed_elements: [T; NB_ELEMENTS] = [Default::default(); NB_ELEMENTS];

        indices.sort();
        for i in 0..NB_ELEMENTS {
            removed_elements[i] = self.remove(indices[i] - i);
        }

        removed_elements
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn from(v: &[T]) -> StackVector<T, MAX_SIZE> {
        let mut sv: StackVector<T, MAX_SIZE> = StackVector::new();

        for e in v.iter() {
            sv.add(e.clone());
        }
        sv
    }
}

pub trait Game {
    fn new() -> Self;

    fn turn(&self) -> Option<Message>;

    fn play(&mut self, msg: String);

    fn winners(&self) -> Option<Vec<WinLossTie>>;

    fn get_state(&self) -> record::GameState;

    fn get_board_representation() -> Option<record::BoardRepresentation>;

    fn end_game(&mut self, players_status: Vec<WinLossTie>);

    fn end_game_if_invalid_move<M: Display>(
        &mut self,
        players_moves: &[Option<M>],
        did_players_do_valid_moves: &[bool],
    ) -> bool {
        if did_players_do_valid_moves.contains(&false) {
            let mut players_status: Vec<WinLossTie> = Vec::new();
            for p_valid_move in did_players_do_valid_moves.iter() {
                players_status.push(match *p_valid_move {
                    true => WinLossTie::Win,
                    false => WinLossTie::Loss,
                })
            }

            eprintln!(
                "[GAME] The following players did invalid moves : {}",
                players_status
                    .iter()
                    .enumerate()
                    .zip(players_moves.iter())
                    .filter(|((p_id, p_status), p_move)| **p_status == WinLossTie::Loss)
                    .map(|((p_id, p_status), p_move)| format!(
                        "({}, {})",
                        p_id,
                        match p_move {
                            Some(m) => format!("{}", m),
                            None => "None".to_string(),
                        }
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            self.end_game(players_status);
            return true;
        }
        false
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
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
    pub enum BoardType {
        SQUARE(u32, u32),
        REGULAR_HEXAGONE_4_SIDES_FLAT_TOP,
    }

    #[derive(Serialize)]
    pub struct BoardRepresentation {
        pub board_type: BoardType,
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
        pub player_state: HashMap<String, String>,
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

    #[test]
    fn test_stackvector_remove_multiple_elements() {
        let mut list: StackVector<u8, 10> = StackVector::new();
        for i in 0..10 {
            list.add(i);
        }
        list.remove_multi([0, 4]);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![1, 2, 3, 5, 6, 7, 8, 9]);

        //
        let mut list: StackVector<u8, 10> = StackVector::new();
        for i in 0..10 {
            list.add(i);
        }
        list.remove_multi([7, 0, 4]);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![1, 2, 3, 5, 6, 8, 9]);

        //
        let mut list: StackVector<u8, 10> = StackVector::new();
        for i in 0..10 {
            list.add(i);
        }
        list.remove_multi([5]);
        let v: Vec<u8> = list.slice().to_vec();
        assert_vec_eq!(v, vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
    }

    #[test]
    fn test_stackvector_from() {
        let v = vec![3, 4, 5, 6];
        let sv: StackVector<i32, 10> = StackVector::from(&v);

        assert_vec_eq!(v, sv.slice());
        assert_eq!(sv.len(), 4)
    }
}
