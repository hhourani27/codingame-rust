#![allow(warnings, unused)]

#[derive(Copy, Clone, PartialEq, Debug)]
enum Move {
    NONE,
    WAIT,
    REST,
    BREW(u32),
    CAST(u32, u8),
    LEARN(u32),
}

fn main() {
    let m = Move::LEARN(1);
    if let Move::LEARN(i) = m {
        println!("MATCHES {}", i);
    } else {
        println!("NOT MATCH")
    }
}
