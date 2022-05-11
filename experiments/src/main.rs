#![allow(warnings, unused)]

fn main() {
    use rand::{Rng, SeedableRng};

    use std::time::{Duration, Instant};

    let instant = Instant::now();
    let three_secs = Duration::from_secs(3);
    let elapsed = instant.elapsed();
    println!("{:?}, {}", elapsed, elapsed.as_millis());
    let elapsed = instant.elapsed();
    println!("{:?}, {}", elapsed, elapsed.as_millis());

    rand::thread_rng().gen_range(0..100);
}
