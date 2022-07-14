use common::record;
use common::{Game, Message, StackVector, WinLossTie};
use rand::prelude::SliceRandom;
use std::collections::HashMap;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
        SoilRichness::HIGH_QUALITY => (0..=6).collect::<Vec<usize>>(),
        SoilRichness::MEDIUM_QUALITY => (7..=18).collect::<Vec<usize>>(),
        SoilRichness::LOW_QUALITY => (19..=36).collect::<Vec<usize>>(),
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

fn gained_sun_points(small_tree_count: u8, medium_tree_count: u8, large_tree_count: u8) -> u32 {
    small_tree_count as u32 + medium_tree_count as u32 * 2 + large_tree_count as u32 * 3
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

fn init_with_params(players_trees: &[[usize; 4]; 2]) -> WoodSpiritGame {
    let mut board: [Option<Cell>; 37] = [None; 37];

    for p_id in 0..2 {
        for cell_pos in players_trees[p_id] {
            board[cell_pos] = Some(Cell {
                player: p_id as u8,
                tree: Tree::SMALL_TREE,
                is_dormant: false,
            })
        }
    }

    let players = [Player {
        move_: None,
        sun: 4,
        score: 0,
        small_tree_count: 4,
        medium_tree_count: 0,
        large_tree_count: 0,
        is_asleep: false,
    }; 2];

    WoodSpiritGame {
        board: board,
        players: players,
        nutrient: 20,
        day: 0,
        turn_during_day: 0,
        turn: 0,

        active_player: 0,
        active: true,
        winners: None,
    }
}

/* #endregion */

impl Game for WoodSpiritGame {
    fn new() -> Self {
        let mut board: [Option<Cell>; 37] = [None; 37];

        let possible_trees_per_richness: Vec<[usize; 3]> = vec![
            [4, 0, 0],
            [3, 1, 0],
            [3, 0, 1],
            [2, 2, 0],
            [2, 1, 1],
            [2, 0, 2],
            [1, 3, 0],
            [1, 2, 1],
            [1, 1, 2],
            [1, 0, 3],
            [0, 4, 0],
            [0, 3, 1],
            [0, 2, 2],
            [0, 1, 3],
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

        /* Create players */
        let mut players = [Player {
            move_: None,
            sun: 0,
            score: 0,
            small_tree_count: 4,
            medium_tree_count: 0,
            large_tree_count: 0,
            is_asleep: false,
        }; 2];

        for p_id in 0..2 {
            let player = &mut players[p_id];
            player.sun += gained_sun_points(
                player.small_tree_count,
                player.medium_tree_count,
                player.large_tree_count,
            );
        }

        /* Output Game */

        WoodSpiritGame {
            board: board,
            players: players,
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

            if self.end_game_if_invalid_move(
                &vec![self.players[0].move_, self.players[1].move_],
                &player_did_a_valid_move,
            ) == true
            {
                return;
            }

            /* (3.2) Update the state */
            let mut completed_trees_count = 0;
            for player in self.players.iter_mut() {
                if player.is_asleep == false {
                    match player.move_.unwrap() {
                        Move::GROW(cell_id) => {
                            let cell = self.board[cell_id as usize].as_mut().unwrap();
                            match cell.tree {
                                Tree::SMALL_TREE => {
                                    player.sun -= 3 + player.medium_tree_count as u32;
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

                player.move_ = None;
            }
            self.nutrient -= completed_trees_count;
            self.turn_during_day += 1;
            self.turn += 1;
        }

        /* (4) If both players are asleep, update the day, else set the next active player */
        if self.players[0].is_asleep == true && self.players[1].is_asleep == true {
            self.day += 1;
            self.turn_during_day = 0;
            self.players[0].move_ = None;
            self.players[1].move_ = None;
            self.players[0].is_asleep = false;
            self.players[1].is_asleep = false;
            self.active_player = 0;

            // Reactivate all trees
            for cell in self.board.iter_mut() {
                if let Some(c) = cell {
                    c.is_dormant = false;
                }
            }

            // let the players collect sun points
            if self.day < 6 {
                for p_id in 0..2 {
                    let player = &mut self.players[p_id];
                    player.sun += gained_sun_points(
                        player.small_tree_count,
                        player.medium_tree_count,
                        player.large_tree_count,
                    )
                }
            }
        } else {
            let next_player = (self.active_player + 1) % 2;
            if self.players[next_player as usize].is_asleep == false {
                self.active_player = next_player;
            }
        }

        /* (5) Check terminal conditions */
        if self.day == 6 {
            let player0 = &self.players[0];
            let player1 = &self.players[1];

            let score0 = player0.score + player0.sun / 3;
            let score1 = player1.score + player1.sun / 3;

            if score0 > score1 {
                self.end_game(vec![WinLossTie::Win, WinLossTie::Loss]);
            } else if score0 < score1 {
                self.end_game(vec![WinLossTie::Loss, WinLossTie::Win]);
            } else {
                let tree_count0 =
                    player0.small_tree_count + player0.medium_tree_count + player0.large_tree_count;
                let tree_count1 =
                    player1.small_tree_count + player1.medium_tree_count + player1.large_tree_count;

                if tree_count0 > tree_count1 {
                    self.end_game(vec![WinLossTie::Win, WinLossTie::Loss]);
                } else if tree_count0 < tree_count1 {
                    self.end_game(vec![WinLossTie::Loss, WinLossTie::Win]);
                } else {
                    self.end_game(vec![WinLossTie::Tie, WinLossTie::Tie]);
                }
            }
        }
    }

    fn winners(&self) -> Option<Vec<WinLossTie>> {
        match &self.winners {
            Some(w) => Some(vec![w.0, w.1]),
            None => None,
        }
    }

    fn get_state(&self) -> record::GameState {
        /* (1) Output Board */
        fn board_pos_to_cell_id(r: usize, c: usize) -> Option<usize> {
            match (r, c) {
                (3, 3) => Some(0),
                (3, 4) => Some(1),
                (2, 3) => Some(2),
                (2, 2) => Some(3),
                (3, 2) => Some(4),
                (4, 2) => Some(5),
                (4, 3) => Some(6),
                (3, 5) => Some(7),
                (2, 4) => Some(8),
                (1, 3) => Some(9),
                (1, 2) => Some(10),
                (1, 1) => Some(11),
                (2, 1) => Some(12),
                (3, 1) => Some(13),
                (4, 1) => Some(14),
                (5, 1) => Some(15),
                (5, 2) => Some(16),
                (5, 3) => Some(17),
                (4, 4) => Some(18),
                (3, 6) => Some(19),
                (2, 5) => Some(20),
                (1, 4) => Some(21),
                (0, 3) => Some(22),
                (0, 2) => Some(23),
                (0, 1) => Some(24),
                (0, 0) => Some(25),
                (1, 0) => Some(26),
                (2, 0) => Some(27),
                (3, 0) => Some(28),
                (4, 0) => Some(29),
                (5, 0) => Some(30),
                (6, 0) => Some(31),
                (6, 1) => Some(32),
                (6, 2) => Some(33),
                (6, 3) => Some(34),
                (5, 4) => Some(35),
                (4, 5) => Some(36),
                _ => None,
            }
        }

        let mut board_repr: Vec<Vec<String>> = vec![vec!["".to_string(); 7]; 7];
        for r in 0..7 {
            for c in 0..7 {
                board_repr[r][c] = match board_pos_to_cell_id(r, c) {
                    Some(cell_pos) => {
                        //1st pos: richness
                        let richness: char = match get_cell_richness(cell_pos) {
                            SoilRichness::LOW_QUALITY => '1',
                            SoilRichness::MEDIUM_QUALITY => '2',
                            SoilRichness::HIGH_QUALITY => '3',
                        };
                        //2nd pos: player
                        let player: char = match self.board[cell_pos] {
                            Some(cell) => match cell.player {
                                0 => '0',
                                1 => '1',
                                _ => panic!(),
                            },
                            None => '.',
                        };

                        //3nd pos: tree
                        let tree: char = match self.board[cell_pos] {
                            Some(cell) => match cell.tree {
                                Tree::SMALL_TREE => '🌱',
                                Tree::MEDIUM_TREE => '🪴',
                                Tree::LARGE_TREE => '🌳',
                            },
                            None => '.',
                        };
                        //4th pos: Tree is dormant
                        let dormant: char = match self.board[cell_pos] {
                            Some(cell) => match cell.is_dormant {
                                true => '😴',
                                false => '🏃',
                            },
                            None => '.',
                        };
                        format!("{}{}{}{}", richness, player, tree, dormant)
                    }
                    None => "....".to_string(),
                }
            }
        }

        /* Output State */
        let mut state: HashMap<String, String> = HashMap::new();

        state.insert("Nutrient".to_string(), self.nutrient.to_string());
        state.insert("Day".to_string(), self.day.to_string());
        state.insert(
            "Turn during day".to_string(),
            self.turn_during_day.to_string(),
        );
        state.insert("Turn".to_string(), self.turn.to_string());
        state.insert("Active".to_string(), self.active.to_string());
        state.insert("Active player".to_string(), self.active_player.to_string());

        for p in 0..=1 {
            let player = &self.players[p];

            state.insert(format!("player[{}]: Score", p), player.score.to_string());
            state.insert(format!("player[{}]: Sun", p), player.sun.to_string());
            state.insert(
                format!("player[{}]: Asleep", p),
                player.is_asleep.to_string(),
            );
            state.insert(
                format!("player[{}]: Move", p),
                format!(
                    "{}",
                    match player.move_ {
                        Some(m) => format!("{}", m),
                        None => "None".to_string(),
                    }
                ),
            );
            state.insert(
                format!("player[{}]: Tree counts", p),
                format!(
                    "[{}🌱, {}🪴, {}🌳]",
                    player.small_tree_count, player.medium_tree_count, player.large_tree_count
                ),
            );
        }

        /* Output GameState */
        record::GameState {
            board: Some(board_repr),
            state,
        }
    }

    fn get_board_representation() -> Option<record::BoardRepresentation> {
        let mut classes: Vec<HashMap<char, record::CellClass>> = Vec::new();

        // First position : Soil Richness
        let mut class_styles: HashMap<char, record::CellClass> = HashMap::new();

        class_styles.insert(
            '1',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("backgroundColor".to_string(), "#CEB926".to_string());
                    css
                }),
            },
        );
        class_styles.insert(
            '2',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("backgroundColor".to_string(), "#D5EC05".to_string());
                    css
                }),
            },
        );
        class_styles.insert(
            '3',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("backgroundColor".to_string(), "#36DE01".to_string());
                    css
                }),
            },
        );
        class_styles.insert(
            '.',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: None,
            },
        );
        classes.push(class_styles);

        // Second position : Player
        let mut class_styles: HashMap<char, record::CellClass> = HashMap::new();
        class_styles.insert(
            '0',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("border".to_string(), "3px solid #FF552B".to_string());
                    css
                }),
            },
        );
        class_styles.insert(
            '1',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("border".to_string(), "3px solid #2B9AFF".to_string());
                    css
                }),
            },
        );
        class_styles.insert(
            '.',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: None,
            },
        );

        classes.push(class_styles);

        // Third position
        let mut class_styles: HashMap<char, record::CellClass> = HashMap::new();
        let mut text_style: HashMap<String, String> = HashMap::new();
        text_style.insert(
            "text-shadow".to_string(),
            "-1px 0 white, 0 1px white, 1px 0 white, 0 -1px white".to_string(),
        );

        class_styles.insert(
            '🌱',
            record::CellClass {
                text: Some('🌱'.to_string()),
                text_style: Some(text_style.clone()),
                cell_style: None,
            },
        );
        class_styles.insert(
            '🪴',
            record::CellClass {
                text: Some('🪴'.to_string()),
                text_style: Some(text_style.clone()),
                cell_style: None,
            },
        );
        class_styles.insert(
            '🌳',
            record::CellClass {
                text: Some('🌳'.to_string()),
                text_style: Some(text_style.clone()),
                cell_style: None,
            },
        );
        class_styles.insert(
            '.',
            record::CellClass {
                text: Some(' '.to_string()),
                text_style: None,
                cell_style: None,
            },
        );
        classes.push(class_styles);

        // Fourth position
        let mut class_styles: HashMap<char, record::CellClass> = HashMap::new();
        class_styles.insert(
            '😴',
            record::CellClass {
                text: None,
                text_style: Some({
                    let mut css = HashMap::new();
                    css.insert("text-decoration".to_string(), "underline".to_string());
                    css
                }),
                cell_style: None,
            },
        );
        class_styles.insert(
            '🏃',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: None,
            },
        );
        class_styles.insert(
            '.',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: None,
            },
        );

        classes.push(class_styles);

        Some(record::BoardRepresentation {
            board_type: record::BoardType::REGULAR_HEXAGONE_4_SIDES_FLAT_TOP,
            classes,
        })
    }

    fn end_game(&mut self, players_status: Vec<WinLossTie>) {
        self.active = false;
        self.winners = Some((players_status[0], players_status[1]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_1() {
        let mut game = init_with_params(&[[2, 22, 27, 29], [5, 20, 31, 36]]);

        assert_eq!(game.turn, 0);
        assert_eq!(game.day, 0);
        assert_eq!(game.turn_during_day, 0);
        assert_eq!(game.players[0].sun, 4);
        assert_eq!(game.players[0].small_tree_count, 4);
        assert_eq!(game.players[1].sun, 4);
        assert_eq!(game.players[1].small_tree_count, 4);

        game.play("GROW 29".to_string());
        game.play("GROW 20".to_string());
        assert_eq!(game.turn, 1);
        assert_eq!(game.turn_during_day, 1);
        assert_eq!(game.players[0].sun, 1);
        assert_eq!(game.players[0].small_tree_count, 3);
        assert_eq!(game.players[0].medium_tree_count, 1);
        assert_eq!(game.players[1].sun, 1);
        assert_eq!(game.players[1].small_tree_count, 3);
        assert_eq!(game.players[1].medium_tree_count, 1);
        assert_eq!(game.board[29].unwrap().tree, Tree::MEDIUM_TREE);
        assert_eq!(game.board[29].unwrap().is_dormant, true);
        assert_eq!(game.board[20].unwrap().tree, Tree::MEDIUM_TREE);
        assert_eq!(game.board[20].unwrap().is_dormant, true);

        game.play("WAIT".to_string());
        game.play("WAIT".to_string());
        assert_eq!(game.turn, 2);
        assert_eq!(game.day, 1);
        assert_eq!(game.turn_during_day, 0);
        assert_eq!(game.players[0].sun, 6);
        assert_eq!(game.players[1].sun, 6);
        assert_eq!(game.board[29].unwrap().is_dormant, false);
        assert_eq!(game.board[20].unwrap().is_dormant, false);

        game.play("GROW 22".to_string());
        game.play("GROW 31".to_string());
        assert_eq!(game.turn, 3);
        assert_eq!(game.turn_during_day, 1);
        assert_eq!(game.players[0].sun, 2);
        assert_eq!(game.players[0].small_tree_count, 2);
        assert_eq!(game.players[0].medium_tree_count, 2);
        assert_eq!(game.players[1].sun, 2);
        assert_eq!(game.players[1].small_tree_count, 2);
        assert_eq!(game.players[1].medium_tree_count, 2);
        assert_eq!(game.board[22].unwrap().tree, Tree::MEDIUM_TREE);
        assert_eq!(game.board[22].unwrap().is_dormant, true);
        assert_eq!(game.board[31].unwrap().tree, Tree::MEDIUM_TREE);
        assert_eq!(game.board[31].unwrap().is_dormant, true);

        game.play("WAIT".to_string());
        game.play("WAIT".to_string());
        assert_eq!(game.turn, 4);
        assert_eq!(game.day, 2);
        assert_eq!(game.turn_during_day, 0);
        assert_eq!(game.players[0].sun, 8);
        assert_eq!(game.players[1].sun, 8);
        assert_eq!(game.board[22].unwrap().is_dormant, false);
        assert_eq!(game.board[31].unwrap().is_dormant, false);

        game.play("GROW 27".to_string());
        game.play("GROW 20".to_string());
        assert_eq!(game.turn, 5);
        assert_eq!(game.turn_during_day, 1);
        assert_eq!(game.players[0].sun, 3);
        assert_eq!(game.players[0].small_tree_count, 1);
        assert_eq!(game.players[0].medium_tree_count, 3);
        assert_eq!(game.players[1].sun, 1);
        assert_eq!(game.players[1].small_tree_count, 2);
        assert_eq!(game.players[1].medium_tree_count, 1);
        assert_eq!(game.players[1].large_tree_count, 1);
        assert_eq!(game.board[27].unwrap().tree, Tree::MEDIUM_TREE);
        assert_eq!(game.board[27].unwrap().is_dormant, true);
        assert_eq!(game.board[20].unwrap().tree, Tree::LARGE_TREE);
        assert_eq!(game.board[20].unwrap().is_dormant, true);

        game.play("WAIT".to_string());
        game.play("WAIT".to_string());
        assert_eq!(game.turn, 6);
        assert_eq!(game.day, 3);
        assert_eq!(game.turn_during_day, 0);
        assert_eq!(game.players[0].sun, 10);
        assert_eq!(game.players[1].sun, 8);
        assert_eq!(game.board[27].unwrap().is_dormant, false);
        assert_eq!(game.board[20].unwrap().is_dormant, false);

        game.play("GROW 27".to_string());
        game.play("COMPLETE 20".to_string());
        assert_eq!(game.turn, 7);
        assert_eq!(game.turn_during_day, 1);
        assert_eq!(game.players[0].sun, 3);
        assert_eq!(game.players[0].score, 0);
        assert_eq!(game.players[0].small_tree_count, 1);
        assert_eq!(game.players[0].medium_tree_count, 2);
        assert_eq!(game.players[0].large_tree_count, 1);
        assert_eq!(game.players[1].sun, 4);
        assert_eq!(game.players[1].score, 20);
        assert_eq!(game.players[1].small_tree_count, 2);
        assert_eq!(game.players[1].medium_tree_count, 1);
        assert_eq!(game.players[1].large_tree_count, 0);
        assert_eq!(game.board[27].unwrap().tree, Tree::LARGE_TREE);
        assert_eq!(game.board[27].unwrap().is_dormant, true);
        assert_eq!(game.board[20].is_none(), true);

        game.play("WAIT".to_string());
        game.play("GROW 36".to_string());
        assert_eq!(game.turn, 8);
        assert_eq!(game.turn_during_day, 2);
        assert_eq!(game.day, 3);
        assert_eq!(game.players[0].is_asleep, true);
        assert_eq!(game.players[0].sun, 3);
        assert_eq!(game.players[0].score, 0);
        assert_eq!(game.players[0].small_tree_count, 1);
        assert_eq!(game.players[0].medium_tree_count, 2);
        assert_eq!(game.players[0].large_tree_count, 1);
        assert_eq!(game.players[1].is_asleep, false);
        assert_eq!(game.players[1].sun, 0);
        assert_eq!(game.players[1].score, 20);
        assert_eq!(game.players[1].small_tree_count, 1);
        assert_eq!(game.players[1].medium_tree_count, 2);
        assert_eq!(game.players[1].large_tree_count, 0);
        assert_eq!(game.board[36].unwrap().tree, Tree::MEDIUM_TREE);
        assert_eq!(game.board[36].unwrap().is_dormant, true);
        assert_eq!(game.active_player, 1);

        game.play("WAIT".to_string());
        assert_eq!(game.turn, 9);
        assert_eq!(game.day, 4);
        assert_eq!(game.turn_during_day, 0);
        assert_eq!(game.players[0].sun, 11);
        assert_eq!(game.players[1].sun, 5);
        assert_eq!(game.board[27].unwrap().is_dormant, false);
        assert_eq!(game.board[36].unwrap().is_dormant, false);
        assert_eq!(game.active_player, 0);

        game.play("GROW 2".to_string());
        game.play("WAIT".to_string());
        assert_eq!(game.turn, 10);
        assert_eq!(game.turn_during_day, 1);
        assert_eq!(game.day, 4);
        assert_eq!(game.players[0].is_asleep, false);
        assert_eq!(game.players[0].sun, 6);
        assert_eq!(game.players[0].score, 0);
        assert_eq!(game.players[0].small_tree_count, 0);
        assert_eq!(game.players[0].medium_tree_count, 3);
        assert_eq!(game.players[0].large_tree_count, 1);
        assert_eq!(game.players[1].is_asleep, true);
        assert_eq!(game.players[1].sun, 5);
        assert_eq!(game.players[1].score, 20);
        assert_eq!(game.players[1].small_tree_count, 1);
        assert_eq!(game.players[1].medium_tree_count, 2);
        assert_eq!(game.players[1].large_tree_count, 0);
        assert_eq!(game.board[2].unwrap().tree, Tree::MEDIUM_TREE);
        assert_eq!(game.board[2].unwrap().is_dormant, true);
        assert_eq!(game.active_player, 0);

        game.play("WAIT".to_string());
        assert_eq!(game.turn, 11);
        assert_eq!(game.day, 5);
        assert_eq!(game.turn_during_day, 0);
        assert_eq!(game.players[0].sun, 15);
        assert_eq!(game.players[1].sun, 10);
        assert_eq!(game.board[2].unwrap().is_dormant, false);
        assert_eq!(game.active_player, 0);

        game.play("GROW 22".to_string());
        game.play("GROW 31".to_string());
        assert_eq!(game.active, true);
        assert_eq!(game.turn, 12);
        assert_eq!(game.turn_during_day, 1);
        assert_eq!(game.day, 5);
        assert_eq!(game.players[0].is_asleep, false);
        assert_eq!(game.players[0].sun, 7);
        assert_eq!(game.players[0].score, 0);
        assert_eq!(game.players[0].small_tree_count, 0);
        assert_eq!(game.players[0].medium_tree_count, 2);
        assert_eq!(game.players[0].large_tree_count, 2);
        assert_eq!(game.players[1].is_asleep, false);
        assert_eq!(game.players[1].sun, 3);
        assert_eq!(game.players[1].score, 20);
        assert_eq!(game.players[1].small_tree_count, 1);
        assert_eq!(game.players[1].medium_tree_count, 1);
        assert_eq!(game.players[1].large_tree_count, 1);
        assert_eq!(game.board[22].unwrap().tree, Tree::LARGE_TREE);
        assert_eq!(game.board[22].unwrap().is_dormant, true);
        assert_eq!(game.board[31].unwrap().tree, Tree::LARGE_TREE);
        assert_eq!(game.board[31].unwrap().is_dormant, true);
        assert_eq!(game.active_player, 0);

        game.play("WAIT".to_string());
        game.play("WAIT".to_string());
        assert_eq!(game.turn, 13);
        assert_eq!(game.day, 6);
        assert_eq!(game.active, false);
        assert_eq!(game.winners.unwrap(), (WinLossTie::Loss, WinLossTie::Win));
    }
}
