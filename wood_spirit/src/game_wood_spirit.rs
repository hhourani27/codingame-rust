use common::record;
use common::{Game, Message, StackVector, WinLossTie};
use std::fmt;

enum Move {
    GROW(u8),
    COMPLETE(u8),
    WAIT,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::GROW(i) => write!(f, "GROW {}", i),
            Move::COMPLETE(i) => write!(f, "COMPLETE {}", i),
            Move::WAIT => write!(f, "WAIT"),
        }
    }
}

enum Tree {
    SIZE_1_TREE,
    SIZE_2_TREE,
    SIZE_3_TREE,
}

pub struct WoodSpiritGame {}

impl Game for WoodSpiritGame {
    fn new() -> Self {
        todo!()
    }

    fn turn(&self) -> Option<Message> {
        todo!()
    }

    fn play(&mut self, msg: String) {
        todo!()
    }

    fn winners(&self) -> Option<Vec<WinLossTie>> {
        todo!()
    }

    fn get_state(&self) -> record::GameState {
        todo!()
    }

    fn get_board_representation() -> Option<record::BoardRepresentation> {
        todo!()
    }
}
