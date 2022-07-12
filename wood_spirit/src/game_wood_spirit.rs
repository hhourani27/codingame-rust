use common::record;
use common::{Game, Message, StackVector, WinLossTie};
use rand::prelude::SliceRandom;
use std::fmt;

const MAX_VALID_MOVES: usize = 4 + 4 + 1; //4 GROW + 4 COMPLETE + WAIT

#[derive(Clone, Copy, PartialEq, Eq)]
enum Move {
    GROW(u8),
    COMPLETE(u8),
    WAIT,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::GROW(i) => write!(f, "GROW {}", i),
            Move::COMPLETE(i) => write!(f, "COMPLETE {}", i),
            Move::WAIT => write!(f, "WAIT"),
        }
    }
}

impl Default for Move {
    fn default() -> Self {
        Move::WAIT
    }
}

impl Move {
    fn parse_move(msg: &str) -> Move {
        if msg == "WAIT" {
            return Move::WAIT;
        } else if msg.starts_with("GROW") {
            return Move::GROW(msg[5..].parse().unwrap());
        } else if msg.starts_with("COMPLETE") {
            return Move::COMPLETE(msg[9..].parse().unwrap());
        } else {
            panic!("Cannot parse move");
        }
    }
}

#[derive(Clone, Copy)]
enum Tree {
    SMALL_TREE,
    MEDIUM_TREE,
    LARGE_TREE,
}

enum SoilRichness {
    LOW_QUALITY,
    MEDIUM_QUALITY,
    HIGH_QUALITY,
}

#[derive(Clone, Copy)]
struct Cell {
    player: u8,
    tree: Tree,
    is_dormant: bool,
}

#[derive(Clone, Copy)]
struct Player {
    move_: Option<Move>,

    sun: u32,
    score: u32,

    small_tree_count: u8,
    medium_tree_count: u8,
    large_tree_count: u8,

    is_asleep: bool,
}

pub struct WoodSpiritGame {
    board: [Option<Cell>; 37],
    players: [Player; 2],

    nutrient: u8,

    day: u8,
    turn_during_day: u8,
    turn: u8,

    active: bool,
    active_player: u8,
    winners: Option<(WinLossTie, WinLossTie)>,
}

/* #region(collapsed) [Helper method] */
fn get_cell_indices(richness: SoilRichness) -> Vec<usize> {
    match richness {
        HIGH_QUALITY => (0..=6).collect::<Vec<usize>>(),
        MEDIUM_QUALITY => (7..=18).collect::<Vec<usize>>(),
        LOW_QUALITY => (19..=36).collect::<Vec<usize>>(),
    }
}

fn get_cell_richness(cell: usize) -> SoilRichness {
    match cell {
        0..=6 => SoilRichness::HIGH_QUALITY,
        7..=18 => SoilRichness::MEDIUM_QUALITY,
        19..=36 => SoilRichness::LOW_QUALITY,
        _ => panic!("Invalid cell index"),
    }
}

fn valid_moves(
    board: [Option<Cell>; 37],
    p_id: u8,
    p_sun: u32,
    p_medium_tree_count: u8,
    p_large_tree_count: u8,
    p_is_asleep: bool,
) -> StackVector<Move, MAX_VALID_MOVES> {
    let mut valid_moves: StackVector<Move, MAX_VALID_MOVES> = StackVector::new();

    if p_is_asleep == true {
        return valid_moves;
    }

    let p_cells = board
        .iter()
        .enumerate()
        .filter(|(i, c)| c.is_some() && c.unwrap().player == p_id)
        .map(|(i, c)| (i, c.unwrap()))
        .collect::<Vec<(usize, Cell)>>();

    for (i, cell) in p_cells.iter() {
        match cell.tree {
            Tree::SMALL_TREE => {
                if p_sun >= 3 + p_medium_tree_count as u32 {
                    valid_moves.add(Move::GROW(*i as u8));
                }
            }
            Tree::MEDIUM_TREE => {
                if p_sun >= 7 + p_large_tree_count as u32 {
                    valid_moves.add(Move::GROW(*i as u8))
                }
            }
            Tree::LARGE_TREE => {
                if p_sun >= 4 {
                    valid_moves.add(Move::COMPLETE(*i as u8))
                }
            }
        }
    }

    valid_moves.add(Move::WAIT);

    valid_moves
}

/* #endregion */

impl Game for WoodSpiritGame {
    fn new() -> Self {
        let mut board: [Option<Cell>; 37] = [None; 37];

        let possible_trees_per_richness: Vec<[usize; 3]> = vec![
            [3, 0, 0],
            [2, 1, 0],
            [2, 0, 1],
            [1, 2, 0],
            [1, 1, 1],
            [0, 3, 0],
            [0, 2, 1],
            [0, 1, 2],
        ];

        /* Select randomly where initial tree are placed */
        let rng = &mut rand::thread_rng();

        let [low_quality_tree_count_per_player, medium_quality_tree_count_per_player, high_quality_tree_count_per_player] =
            possible_trees_per_richness.choose(rng).cloned().unwrap();

        let chosen_low_quality_cells = get_cell_indices(SoilRichness::LOW_QUALITY)
            .choose_multiple(rng, low_quality_tree_count_per_player * 2)
            .cloned()
            .collect::<Vec<usize>>();

        for (i, cell_pos) in chosen_low_quality_cells.into_iter().enumerate() {
            board[cell_pos] = Some(Cell {
                player: (i % 2) as u8,
                tree: Tree::SMALL_TREE,
                is_dormant: false,
            })
        }

        let chosen_medium_quality_cells = get_cell_indices(SoilRichness::MEDIUM_QUALITY)
            .choose_multiple(rng, medium_quality_tree_count_per_player * 2)
            .cloned()
            .collect::<Vec<usize>>();

        for (i, cell_pos) in chosen_medium_quality_cells.into_iter().enumerate() {
            board[cell_pos] = Some(Cell {
                player: (i % 2) as u8,
                tree: Tree::SMALL_TREE,
                is_dormant: false,
            })
        }

        let chosen_high_quality_cells = get_cell_indices(SoilRichness::HIGH_QUALITY)
            .choose_multiple(rng, high_quality_tree_count_per_player * 2)
            .cloned()
            .collect::<Vec<usize>>();

        for (i, cell_pos) in chosen_high_quality_cells.into_iter().enumerate() {
            board[cell_pos] = Some(Cell {
                player: (i % 2) as u8,
                tree: Tree::SMALL_TREE,
                is_dormant: false,
            })
        }

        WoodSpiritGame {
            board: board,
            players: [Player {
                move_: None,
                sun: 0,
                score: 0,
                small_tree_count: 0,
                medium_tree_count: 0,
                large_tree_count: 0,
                is_asleep: false,
            }; 2],
            nutrient: 20,
            day: 0,
            turn_during_day: 0,
            turn: 0,

            active_player: 0,
            active: true,
            winners: None,
        }
    }

    fn turn(&self) -> Option<Message> {
        // If game is over, return None
        if self.active == false {
            return None;
        }

        let mut out: Vec<String> = Vec::new();

        if self.turn == 0 {
            out.push("37".to_string());

            for c in 0..37 {
                out.push(format!(
                    "{} {} 0 0 0 0 0 0",
                    c,
                    match get_cell_richness(c) {
                        SoilRichness::LOW_QUALITY => 1,
                        SoilRichness::MEDIUM_QUALITY => 2,
                        SoilRichness::HIGH_QUALITY => 3,
                    }
                ))
            }
        }

        let active_player = &self.players[self.active_player as usize];
        let other_player = &self.players[((self.active_player + 1) % 2) as usize];

        out.push(format!("{}", self.day));
        out.push(format!("{}", self.nutrient));
        out.push(format!("{} {}", active_player.sun, active_player.score));
        out.push(format!(
            "{} {} {}",
            other_player.sun,
            other_player.score,
            match other_player.is_asleep {
                true => 1,
                false => 0,
            }
        ));

        let tree_count = active_player.small_tree_count
            + active_player.medium_tree_count
            + active_player.large_tree_count
            + other_player.small_tree_count
            + other_player.medium_tree_count
            + other_player.large_tree_count;
        out.push(format!("{}", tree_count));

        for (i, cell) in self.board.iter().enumerate() {
            match cell {
                Some(c) => out.push(format!(
                    "{} {} {} {}",
                    i,
                    match c.tree {
                        Tree::SMALL_TREE => 1,
                        Tree::MEDIUM_TREE => 2,
                        Tree::LARGE_TREE => 3,
                    },
                    match c.player == self.active_player {
                        true => 1,
                        false => 0,
                    },
                    match c.is_dormant {
                        true => 1,
                        false => 0,
                    }
                )),
                None => {}
            }
        }

        let valid_moves = valid_moves(
            self.board,
            self.active_player,
            active_player.sun,
            active_player.medium_tree_count,
            active_player.large_tree_count,
            active_player.is_asleep,
        );

        out.push(format!("{}", valid_moves.len()));

        for vm in valid_moves.slice().iter() {
            out.push(format!("{}", vm));
        }

        Some(Message {
            player_id: self.active_player as usize,
            messages: out,
        })
    }

    fn play(&mut self, msg: String) {
        /* (1) Parse move, assuming it is always in the right format */
        let move_ = Move::parse_move(msg.as_str());

        /* (2) Record the move */
        self.players[self.active_player as usize].move_ = Some(move_);

        /* (3) Check if both players have played, so that we update the game */
        if (self.players[0].move_.is_some() && self.players[1].move_.is_some())
            || (self.players[0].move_.is_some() && self.players[1].is_asleep == true)
            || (self.players[0].is_asleep == true && self.players[1].move_.is_some())
        {
            /* 3.1 Check if moves are valid */
            let mut player_did_a_valid_move = [true, true];

            for (p_id, player) in self.players.iter().enumerate() {
                if player.is_asleep == false {
                    player_did_a_valid_move[p_id] = valid_moves(
                        self.board,
                        p_id as u8,
                        player.sun,
                        player.medium_tree_count,
                        player.large_tree_count,
                        player.is_asleep,
                    )
                    .slice()
                    .contains(&player.move_.unwrap());
                }
            }

            if !player_did_a_valid_move[0] && !player_did_a_valid_move[1] {
                eprintln!(
                    "[GAME] Player 0's move {} & Player 1's move {} are both invalid",
                    &self.players[0].move_.unwrap().to_string(),
                    &self.players[1].move_.unwrap().to_string()
                );
                self.active = false;
                self.winners = Some((WinLossTie::Loss, WinLossTie::Loss));
                return;
            } else if player_did_a_valid_move[0] && !player_did_a_valid_move[1] {
                eprintln!(
                    "[GAME] Player 1's move {} is invalid",
                    &self.players[1].move_.unwrap().to_string()
                );
                self.active = false;
                self.winners = Some((WinLossTie::Win, WinLossTie::Loss));
                return;
            } else if !player_did_a_valid_move[0] && player_did_a_valid_move[1] {
                eprintln!(
                    "[GAME] Player 0's move {} is invalid",
                    &self.players[0].move_.unwrap().to_string()
                );
                self.active = false;
                self.winners = Some((WinLossTie::Loss, WinLossTie::Win));
                return;
            }

            /* (3.2) Update the state */
            let mut completed_trees_count = 0;
            for player in self.players.iter_mut() {
                if player.is_asleep == false {
                    match player.move_.unwrap() {
                        Move::GROW(cell_id) => {
                            let cell = &mut self.board[cell_id as usize].unwrap();
                            match cell.tree {
                                Tree::SMALL_TREE => {
                                    player.sun -= 2 + player.medium_tree_count as u32;
                                    player.small_tree_count -= 1;
                                    player.medium_tree_count += 1;
                                    cell.tree = Tree::MEDIUM_TREE;
                                    cell.is_dormant = true;
                                }
                                Tree::MEDIUM_TREE => {
                                    player.sun -= 7 + player.large_tree_count as u32;
                                    player.medium_tree_count -= 1;
                                    player.large_tree_count += 1;
                                    cell.tree = Tree::LARGE_TREE;
                                    cell.is_dormant = true;
                                }
                                _ => panic!("This code should not be reached"),
                            }
                        }
                        Move::COMPLETE(cell_id) => {
                            player.sun -= 4;
                            player.score += self.nutrient as u32
                                + match get_cell_richness(cell_id as usize) {
                                    SoilRichness::LOW_QUALITY => 0,
                                    SoilRichness::MEDIUM_QUALITY => 2,
                                    SoilRichness::HIGH_QUALITY => 4,
                                };
                            player.large_tree_count -= 1;
                            self.board[cell_id as usize] = None;
                            completed_trees_count += 1;
                        }
                        Move::WAIT => {
                            player.is_asleep = true;
                        }
                    }
                }
            }
            self.nutrient -= completed_trees_count;
            self.turn_during_day += 1;
            self.turn += 1;
        }

        /* (4) If both players are asleep update the day */
        if self.players[0].is_asleep == true && self.players[1].is_asleep == true {
            self.day += 1;
            self.turn_during_day = 0;
            self.players[0].move_ = None;
            self.players[1].move_ = None;
            self.players[0].is_asleep = false;
            self.players[1].is_asleep = false;

            for cell in self.board.iter_mut() {
                if let Some(c) = cell {
                    self.players[c.player as usize].sun += match c.tree {
                        Tree::SMALL_TREE => 1,
                        Tree::MEDIUM_TREE => 2,
                        Tree::LARGE_TREE => 3,
                    };
                    c.is_dormant = false;
                }
            }
        }

        /* (5) Check terminal conditions */
        if self.day == 6 {
            let player0 = &self.players[0];
            let player1 = &self.players[1];

            let score0 = player0.score + player0.sun / 3;
            let score1 = player1.score + player1.sun / 3;

            if score0 > score1 {
                self.winners = Some((WinLossTie::Win, WinLossTie::Loss));
            } else if score0 < score1 {
                self.winners = Some((WinLossTie::Loss, WinLossTie::Win));
            } else {
                let tree_count0 =
                    player0.small_tree_count + player0.medium_tree_count + player0.large_tree_count;
                let tree_count1 =
                    player1.small_tree_count + player1.medium_tree_count + player1.large_tree_count;

                if tree_count0 > tree_count1 {
                    self.winners = Some((WinLossTie::Win, WinLossTie::Loss));
                } else if tree_count0 < tree_count1 {
                    self.winners = Some((WinLossTie::Loss, WinLossTie::Win));
                } else {
                    self.winners = Some((WinLossTie::Tie, WinLossTie::Tie));
                }
            }
        }
    }

    fn winners(&self) -> Option<Vec<WinLossTie>> {
        todo!()
    }

    fn get_state(&self) -> record::GameState {
        todo!()
    }

    fn get_board_representation() -> Option<record::BoardRepresentation> {
        todo!()
    }

    fn end_game(&mut self, players_status: Vec<WinLossTie>) {
        self.active = false;
        self.winners = Some((players_status[0], players_status[1]));
    }
}
