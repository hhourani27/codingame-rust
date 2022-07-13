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

fn main() {
    let mut board: Vec<Option<Cell>> = Vec::new();
    board.push(Some(Cell {
        player: 0,
        tree: Tree::SMALL_TREE,
        is_dormant: false,
    }));
    board.push(None);
    println!("{:?}", board);

    let cell = &mut board[0].as_mut().unwrap();
    cell.player = 1;
    cell.tree = Tree::MEDIUM_TREE;
    println!("{:?}", board);

    for cell in board.iter_mut() {
        if let Some(c) = cell {
            c.is_dormant = true;
        }
    }
    println!("{:?}", board);
}
