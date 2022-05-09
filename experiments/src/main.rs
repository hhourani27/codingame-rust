#![allow(warnings, unused)]

fn main() {
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    let instant = Instant::now();
    let three_secs = Duration::from_secs(3);
    sleep(three_secs);
    let elapsed = instant.elapsed();
    println!("{:?}, {}", elapsed, elapsed.as_millis());
    sleep(three_secs);
    let elapsed = instant.elapsed();
    println!("{:?}, {}", elapsed, elapsed.as_millis());
}
