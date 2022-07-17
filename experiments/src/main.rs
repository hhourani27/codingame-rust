#![allow(warnings, unused)]

#[derive(Debug)]
enum Tree {
    SMALL_TREE,
    MEDIUM_TREE,
    LARGE_TREE,
}
#[derive(Debug)]
struct Cell {
    player: u8,
    tree: Tree,
    is_dormant: bool,
}

enum Move {
    GROW(u8),
    COMPLETE(u8),
    SEED(u8, u8),
    WAIT,
}

fn main() {
    let m1 = Move::SEED(0, 1);
    let m2 = Move::SEED(10, 1);

    if let Move::SEED(tree_pos1, seed_pos1) = m1 {
        println!("HERE 1");
        match m2 {
            Move::SEED(tree_pos1, seed_pos2) if seed_pos2 == seed_pos1 => println!("HERE 2"),
            _ => println!("HERE 3"),
        }
    }
}
