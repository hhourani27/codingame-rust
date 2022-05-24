use common::record;
use common::{Game, Message, WinLossTie};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub struct WitchesBrewGame {
    active_player: u8,
}

impl WitchesBrewGame {}

impl Game for WitchesBrewGame {
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

    fn get_board_representation() -> record::BoardRepresentation {
        todo!()
    }
}
