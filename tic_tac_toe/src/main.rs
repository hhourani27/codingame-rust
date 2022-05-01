mod game_tic_tac_toe;
use game_tic_tac_toe::TicTacToeGame;
mod player;
use common::simulator;
use common::Game;

fn main() {
    let players = vec![player::play, player::play];
    simulator::run(TicTacToeGame::new, &players, 5);
}
