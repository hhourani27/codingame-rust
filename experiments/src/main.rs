#![allow(warnings, unused)]

fn main() {
    use std::time::{Duration, Instant};

    let a = [1, 2, 3];

    for i in a.iter() {
        println!("{}", 0b1 << *i);
    }
}
