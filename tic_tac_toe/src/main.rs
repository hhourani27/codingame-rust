mod game_tic_tac_toe;
mod player;
use common::simulator;
use std::thread;

fn main() {
    pub fn testfn(function: (impl Fn() + Send + Sync + 'static)) {
        thread::spawn(move || function());
    }

    fn ftest() {
        println!("Hello");
    }

    testfn(ftest);
}
