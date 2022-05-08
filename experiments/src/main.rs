#![allow(warnings, unused)]

use std::thread;

fn f1(count: usize) {
    for i in 0..count { /*do something*/ }
}

fn f2(count: usize) {
    for i in 0..count { /*do something*/ }
}

fn run(ops: &Vec<&'static (dyn Fn(usize) + Sync)>) {
    for i in 0..ops.len() {
        let func = ops[i];
        thread::spawn(move || func(1000));
    }
}

fn main() {
    let ops = vec![f1, f1];

    let ops: Vec<&'static (dyn Fn(usize) + Sync)> = vec![&f1, &f2];

    run(&ops);
}
