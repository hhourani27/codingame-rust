use common::record;
use common::{Game, Message, WinLossTie};
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Move {
    GROW(u8),
    COMPLETE(u8),
    SEED(u8, u8),
    WAIT,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::GROW(t) => write!(f, "GROW {}", t),
            Move::COMPLETE(t) => write!(f, "COMPLETE {}", t),
            Move::SEED(t, c) => write!(f, "SEED {} {}", t, c),
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
        } else if msg.starts_with("SEED") {
            let mut s_iter = msg.split_whitespace();
            s_iter.next();
            let tree_pos: u8 = s_iter.next().unwrap().parse().unwrap();
            let seed_pos: u8 = s_iter.next().unwrap().parse().unwrap();
            return Move::SEED(tree_pos, seed_pos);
        } else {
            panic!("Cannot parse move");
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Debug)]
enum Tree {
    SEED = 0,
    SMALL_TREE = 1,
    MEDIUM_TREE = 2,
    LARGE_TREE = 3,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum SoilRichness {
    UNUSABLE,
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

    seed_count: u8,
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

    //Cache
    cache: Cache,
}

/* #region(collapsed) [Helper method] */
fn gained_sun_points(board: &[Option<Cell>; 37], spookied_cells: &[bool; 37]) -> [u32; 2] {
    let mut gained_sun_points_per_players = [0; 2];
    for (cell_pos, cell) in board.iter().enumerate() {
        if let Some(c) = cell {
            if spookied_cells[cell_pos] == false {
                gained_sun_points_per_players[c.player as usize] += match c.tree {
                    Tree::SEED => 0,
                    Tree::SMALL_TREE => 1,
                    Tree::MEDIUM_TREE => 2,
                    Tree::LARGE_TREE => 3,
                }
            }
        }
    }

    gained_sun_points_per_players
}

fn get_spookied_cells(board: &[Option<Cell>; 37], day: u8, cache: &Cache) -> [bool; 37] {
    let mut spookied_cells = [false; 37];

    for (cell_pos, cell) in board.iter().enumerate() {
        if let Some(c) = cell {
            if c.tree != Tree::SEED {
                let shadowed_cells = cache.get_shadowed_cells(cell_pos, c.tree, day as usize);
                for shadowed_cell_pos in shadowed_cells {
                    if let Some(shadowed_cell) = &board[*shadowed_cell_pos] {
                        if c.tree >= shadowed_cell.tree {
                            spookied_cells[*shadowed_cell_pos] = true;
                        }
                    }
                }
            }
        }
    }

    spookied_cells
}

fn get_shadowed_cells(board: &[Option<Cell>; 37], day: u8, cache: &Cache) -> [bool; 37] {
    let mut all_shadowed_cells = [false; 37];

    for (cell_pos, cell) in board.iter().enumerate() {
        if let Some(c) = cell {
            if c.tree != Tree::SEED {
                let shadowed_cells = cache.get_shadowed_cells(cell_pos, c.tree, day as usize);
                for shadowed_cell_pos in shadowed_cells {
                    all_shadowed_cells[*shadowed_cell_pos] = true;
                }
            }
        }
    }

    all_shadowed_cells
}

fn get_initial_soil_richness() -> [SoilRichness; 37] {
    let mut soil_richness: [SoilRichness; 37] = [SoilRichness::UNUSABLE; 37];

    for i in 0..37 {
        soil_richness[i] = match i {
            0..=6 => SoilRichness::HIGH_QUALITY,
            7..=18 => SoilRichness::MEDIUM_QUALITY,
            19..=36 => SoilRichness::LOW_QUALITY,
            _ => panic!("Invalid cell index"),
        }
    }

    soil_richness
}

fn valid_moves(
    board: &[Option<Cell>; 37],
    p_id: u8,
    p_sun: u32,
    p_seed_count: u8,
    p_small_tree_count: u8,
    p_medium_tree_count: u8,
    p_large_tree_count: u8,
    p_is_asleep: bool,
    cache: &Cache,
) -> Vec<Move> {
    let mut valid_moves: Vec<Move> = Vec::new();

    if p_is_asleep == true {
        return valid_moves;
    }

    for (cell_pos, cell) in board.iter().enumerate() {
        match cell {
            Some(c) => {
                if c.player == p_id && c.is_dormant == false {
                    match c.tree {
                        Tree::SEED => {
                            if p_sun >= 1 + p_small_tree_count as u32 {
                                valid_moves.push(Move::GROW(cell_pos as u8));
                            }
                        }
                        Tree::SMALL_TREE => {
                            if p_sun >= 3 + p_medium_tree_count as u32 {
                                valid_moves.push(Move::GROW(cell_pos as u8));
                            }

                            if p_sun >= p_seed_count as u32 {
                                for neighbor in
                                    cache.get_seedable_neighbors(cell_pos, Tree::SMALL_TREE)
                                {
                                    if board[*neighbor].is_none() {
                                        valid_moves
                                            .push(Move::SEED(cell_pos as u8, *neighbor as u8));
                                    }
                                }
                            }
                        }
                        Tree::MEDIUM_TREE => {
                            if p_sun >= 7 + p_large_tree_count as u32 {
                                valid_moves.push(Move::GROW(cell_pos as u8))
                            }

                            if p_sun >= p_seed_count as u32 {
                                for neighbor in
                                    cache.get_seedable_neighbors(cell_pos, Tree::MEDIUM_TREE)
                                {
                                    if board[*neighbor].is_none() {
                                        valid_moves
                                            .push(Move::SEED(cell_pos as u8, *neighbor as u8));
                                    }
                                }
                            }
                        }
                        Tree::LARGE_TREE => {
                            if p_sun >= 4 {
                                valid_moves.push(Move::COMPLETE(cell_pos as u8))
                            }
                            if p_sun >= p_seed_count as u32 {
                                let neighbors =
                                    cache.get_seedable_neighbors(cell_pos, Tree::LARGE_TREE);
                                for neighbor in
                                    cache.get_seedable_neighbors(cell_pos, Tree::LARGE_TREE)
                                {
                                    if board[*neighbor].is_none() {
                                        valid_moves
                                            .push(Move::SEED(cell_pos as u8, *neighbor as u8));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => {}
        }
    }
    valid_moves.push(Move::WAIT);
    valid_moves
}

fn init_with_params(
    players_initial_small_trees: &[[usize; 2]; 2],
    invalid_cells: &[usize],
) -> WoodSpiritGame {
    /* Initialize board & add small trees for each player */
    let mut board: [Option<Cell>; 37] = [None; 37];

    for p_id in 0..2 {
        for cell_pos in players_initial_small_trees[p_id] {
            board[cell_pos] = Some(Cell {
                player: p_id as u8,
                tree: Tree::SMALL_TREE,
                is_dormant: false,
            })
        }
    }

    /* Initialize soil richness */
    let mut soil_richness = get_initial_soil_richness();
    for cell_pos in invalid_cells {
        soil_richness[*cell_pos] = SoilRichness::UNUSABLE;
    }

    /* Creat cache */
    let cache = Cache::new(soil_richness);

    // Initialize players
    let mut players = [Player {
        move_: None,
        sun: 0,
        score: 0,
        seed_count: 0,
        small_tree_count: 4,
        medium_tree_count: 0,
        large_tree_count: 0,
        is_asleep: false,
    }; 2];

    // Update player's gained sun points
    let gained_sun_points = gained_sun_points(&board, &get_spookied_cells(&board, 0, &cache));
    players[0].sun += gained_sun_points[0];
    players[1].sun += gained_sun_points[1];

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

        cache,
    }
}

/* #endregion */

/* #region(collapsed) [Cache] */
struct Cache {
    soil_richness: [SoilRichness; 37],
    cell_neighbors: [[[Option<usize>; 3]; 6]; 37], //[cell_pos][direction][distance] => neighbor position
    seedable_neighbors: Vec<Vec<Vec<usize>>>, //[[Vec<usize>;3];37] : [cell_pos][tree size] => vector of seedable positions
    shadowed_cells: Vec<Vec<Vec<Vec<usize>>>>, // [Vec<usize>;3;6;37]: [cell_pos][day][tree size] => vector of shadowed cell positions
}

impl Cache {
    fn new(soil_richness: [SoilRichness; 37]) -> Self {
        let cell_neighbors = Cache::init_cell_neighbors();
        let seedable_neighbors = Cache::init_seedable_neighbors(&cell_neighbors, &soil_richness);
        let shadowed_cells = Cache::init_shadowed_cells(&cell_neighbors);
        Self {
            soil_richness,
            cell_neighbors,
            seedable_neighbors,
            shadowed_cells,
        }
    }

    fn init_cell_neighbors() -> [[[Option<usize>; 3]; 6]; 37] {
        let distance_1_neighbors: [[isize; 6]; 37] = [
            [1, 2, 3, 4, 5, 6],
            [7, 8, 2, 0, 6, 18],
            [8, 9, 10, 3, 0, 1],
            [2, 10, 11, 12, 4, 0],
            [0, 3, 12, 13, 14, 5],
            [6, 0, 4, 14, 15, 16],
            [18, 1, 0, 5, 16, 17],
            [19, 20, 8, 1, 18, 36],
            [20, 21, 9, 2, 1, 7],
            [21, 22, 23, 10, 2, 8],
            [9, 23, 24, 11, 3, 2],
            [10, 24, 25, 26, 12, 3],
            [3, 11, 26, 27, 13, 4],
            [4, 12, 27, 28, 29, 14],
            [5, 4, 13, 29, 30, 15],
            [16, 5, 14, 30, 31, 32],
            [17, 6, 5, 15, 32, 33],
            [35, 18, 6, 16, 33, 34],
            [36, 7, 1, 6, 17, 35],
            [-1, -1, 20, 7, 36, -1],
            [-1, -1, 21, 8, 7, 19],
            [-1, -1, 22, 9, 8, 20],
            [-1, -1, -1, 23, 9, 21],
            [22, -1, -1, 24, 10, 9],
            [23, -1, -1, 25, 11, 10],
            [24, -1, -1, -1, 26, 11],
            [11, 25, -1, -1, 27, 12],
            [12, 26, -1, -1, 28, 13],
            [13, 27, -1, -1, -1, 29],
            [14, 13, 28, -1, -1, 30],
            [15, 14, 29, -1, -1, 31],
            [32, 15, 30, -1, -1, -1],
            [33, 16, 15, 31, -1, -1],
            [34, 17, 16, 32, -1, -1],
            [-1, 35, 17, 33, -1, -1],
            [-1, 36, 18, 17, 34, -1],
            [-1, 19, 7, 18, 35, -1],
        ];

        let mut cell_neighbors: [[[Option<usize>; 3]; 6]; 37] = [[[None; 3]; 6]; 37];

        for cell_pos in 0..37 {
            for direction in 0..6 {
                for distance in 0..3 {
                    let mut neighbor_cell: Option<usize> = Some(cell_pos);
                    for _ in 0..=distance {
                        let neighbor_cell_temp =
                            distance_1_neighbors[neighbor_cell.unwrap()][direction];
                        if neighbor_cell_temp != -1 {
                            neighbor_cell = Some(neighbor_cell_temp as usize);
                        } else {
                            neighbor_cell = None;
                            break;
                        }
                    }
                    cell_neighbors[cell_pos][direction][distance] = neighbor_cell;
                }
            }
        }

        cell_neighbors
    }

    fn init_seedable_neighbors(
        cell_neighbors: &[[[Option<usize>; 3]; 6]; 37],
        soil_richness: &[SoilRichness; 37],
    ) -> Vec<Vec<Vec<usize>>> {
        let mut seedable_neighbors: Vec<Vec<Vec<usize>>> = vec![vec![vec![]; 3]; 37];

        for cell_pos in 0..37 {
            for tree_size in 1..=3 {
                let mut result: HashSet<usize> = HashSet::new();

                let mut visited: Vec<usize> = Vec::new();
                visited.push(cell_pos);

                for _ in 1..=tree_size {
                    let mut visited_new: Vec<usize> = Vec::new();
                    for c in visited {
                        for direction in 0..6 {
                            let neighbor = cell_neighbors[c][direction][0];
                            if let Some(n) = neighbor {
                                if soil_richness[n] != SoilRichness::UNUSABLE {
                                    result.insert(n);
                                }
                                visited_new.push(n);
                            }
                        }
                    }
                    visited = visited_new;
                }
                result.remove(&cell_pos);
                seedable_neighbors[cell_pos][tree_size - 1] =
                    result.into_iter().collect::<Vec<usize>>();
            }
        }

        seedable_neighbors
    }

    fn init_shadowed_cells(
        cell_neighbors: &[[[Option<usize>; 3]; 6]; 37],
    ) -> Vec<Vec<Vec<Vec<usize>>>> {
        let mut shadowed_cells: Vec<Vec<Vec<Vec<usize>>>> = vec![vec![vec![vec![]; 3]; 6]; 37];
        for cell_pos in 0..37 {
            for day in 0..6 {
                for tree_size in 1..=3 {
                    for distance in 1..=tree_size {
                        let shadowed_cell = cell_neighbors[cell_pos][day][distance - 1];
                        if let Some(shadowed_cell_pos) = shadowed_cell {
                            shadowed_cells[cell_pos][day][tree_size - 1].push(shadowed_cell_pos);
                        }
                    }
                }
            }
        }

        shadowed_cells
    }

    fn get_soil_richness(&self, cell_pos: usize) -> SoilRichness {
        self.soil_richness[cell_pos]
    }

    fn get_neighbor(&self, cell_pos: usize, direction: usize, distance: usize) -> Option<usize> {
        self.cell_neighbors[cell_pos][direction][distance - 1]
    }

    fn get_seedable_neighbors(&self, tree_pos: usize, tree: Tree) -> &[usize] {
        let tree_size: usize = match tree {
            Tree::SEED => 0,
            Tree::SMALL_TREE => 1,
            Tree::MEDIUM_TREE => 2,
            Tree::LARGE_TREE => 3,
        };

        &self.seedable_neighbors[tree_pos][tree_size - 1]
    }

    fn get_shadowed_cells(&self, tree_pos: usize, tree: Tree, day: usize) -> &[usize] {
        let tree_size: usize = match tree {
            Tree::SEED => 0,
            Tree::SMALL_TREE => 1,
            Tree::MEDIUM_TREE => 2,
            Tree::LARGE_TREE => 3,
        };

        // Here I assume that there will never be a seed
        &self.shadowed_cells[tree_pos][day % 6][tree_size - 1]
    }
}

/* #endregion */

impl Game for WoodSpiritGame {
    fn new() -> Self {
        fn generate_random_cell_pairs() -> Vec<(usize, usize)> {
            let mut cell_pairs: Vec<(usize, usize)> = Vec::new();

            let mut high_quality_cells: Vec<usize> = (0..=6).collect::<Vec<usize>>();
            high_quality_cells.shuffle(&mut thread_rng());

            for i in 0..3 {
                cell_pairs.push((high_quality_cells[i * 2], high_quality_cells[i * 2 + 1]));
            }

            let mut medium_quality_cells: Vec<usize> = (7..=18).collect::<Vec<usize>>();
            medium_quality_cells.shuffle(&mut thread_rng());

            for i in 0..6 {
                cell_pairs.push((medium_quality_cells[i * 2], medium_quality_cells[i * 2 + 1]));
            }

            let mut low_quality_cells: Vec<usize> = (19..=36).collect::<Vec<usize>>();
            low_quality_cells.shuffle(&mut thread_rng());

            for i in 0..9 {
                cell_pairs.push((low_quality_cells[i * 2], low_quality_cells[i * 2 + 1]));
            }

            cell_pairs.shuffle(&mut thread_rng());
            cell_pairs
        }

        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut soil_richness: [SoilRichness; 37] = get_initial_soil_richness();

        /* Place the initial 2 small trees for each player*/
        let mut cell_pairs = generate_random_cell_pairs();
        for _ in 0..2 {
            let cells = cell_pairs.pop().unwrap();
            board[cells.0] = Some(Cell {
                player: 0,
                tree: Tree::SMALL_TREE,
                is_dormant: false,
            });

            board[cells.1] = Some(Cell {
                player: 1,
                tree: Tree::SMALL_TREE,
                is_dormant: false,
            });
        }

        /* Chosse invalid cells */
        let invalid_cells_count: usize = thread_rng().gen_range(0..=5);
        for _ in 0..invalid_cells_count {
            let cells = cell_pairs.pop().unwrap();
            soil_richness[cells.0] = SoilRichness::UNUSABLE;
            soil_richness[cells.1] = SoilRichness::UNUSABLE;
        }

        /* Update shadows */
        let cache = Cache::new(soil_richness);

        /* Create players */
        let mut players = [Player {
            move_: None,
            sun: 0,
            score: 0,
            seed_count: 0,
            small_tree_count: 2,
            medium_tree_count: 0,
            large_tree_count: 0,
            is_asleep: false,
        }; 2];

        let gained_sun_points = gained_sun_points(&board, &get_spookied_cells(&board, 0, &cache));
        players[0].sun += gained_sun_points[0];
        players[1].sun += gained_sun_points[1];

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

            cache,
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
                    "{} {} {} {} {} {} {} {}",
                    c,
                    match self.cache.get_soil_richness(c) {
                        SoilRichness::UNUSABLE => 0,
                        SoilRichness::LOW_QUALITY => 1,
                        SoilRichness::MEDIUM_QUALITY => 2,
                        SoilRichness::HIGH_QUALITY => 3,
                    },
                    match self.cache.get_neighbor(c, 0, 1) {
                        Some(n) => n as isize,
                        None => -1,
                    },
                    match self.cache.get_neighbor(c, 1, 1) {
                        Some(n) => n as isize,
                        None => -1,
                    },
                    match self.cache.get_neighbor(c, 2, 1) {
                        Some(n) => n as isize,
                        None => -1,
                    },
                    match self.cache.get_neighbor(c, 3, 1) {
                        Some(n) => n as isize,
                        None => -1,
                    },
                    match self.cache.get_neighbor(c, 4, 1) {
                        Some(n) => n as isize,
                        None => -1,
                    },
                    match self.cache.get_neighbor(c, 5, 1) {
                        Some(n) => n as isize,
                        None => -1,
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

        let tree_count = active_player.seed_count
            + active_player.small_tree_count
            + active_player.medium_tree_count
            + active_player.large_tree_count
            + other_player.seed_count
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
                        Tree::SEED => 0,
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
            &self.board,
            self.active_player,
            active_player.sun,
            active_player.seed_count,
            active_player.small_tree_count,
            active_player.medium_tree_count,
            active_player.large_tree_count,
            active_player.is_asleep,
            &self.cache,
        );

        out.push(format!("{}", valid_moves.len()));

        for vm in valid_moves.iter() {
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
                        &self.board,
                        p_id as u8,
                        player.sun,
                        player.seed_count,
                        player.small_tree_count,
                        player.medium_tree_count,
                        player.large_tree_count,
                        player.is_asleep,
                        &self.cache,
                    )
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
            let player_moves = [self.players[0].move_.clone(), self.players[1].move_.clone()];

            for (p_id, player) in self.players.iter_mut().enumerate() {
                if player.is_asleep == false {
                    match player.move_.unwrap() {
                        Move::SEED(tree_pos, seed_pos) => match player_moves[(p_id + 1) % 2] {
                            Some(Move::SEED(o_tree_pos, o_seed_pos)) if o_seed_pos == seed_pos => {
                                let tree_cell = self.board[tree_pos as usize].as_mut().unwrap();
                                tree_cell.is_dormant = true;
                            }
                            _ => {
                                player.sun -= player.seed_count as u32;
                                player.seed_count += 1;
                                let tree_cell = self.board[tree_pos as usize].as_mut().unwrap();
                                tree_cell.is_dormant = true;
                                self.board[seed_pos as usize] = Some(Cell {
                                    player: p_id as u8,
                                    tree: Tree::SEED,
                                    is_dormant: true,
                                });
                            }
                        },
                        Move::GROW(cell_pos) => {
                            let cell = self.board[cell_pos as usize].as_mut().unwrap();
                            match cell.tree {
                                Tree::SEED => {
                                    player.sun -= 1 + player.small_tree_count as u32;
                                    player.seed_count -= 1;
                                    player.small_tree_count += 1;
                                    cell.tree = Tree::SMALL_TREE;
                                    cell.is_dormant = true;
                                }
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
                        Move::COMPLETE(cell_pos) => {
                            player.sun -= 4;
                            player.score += self.nutrient as u32
                                + match self.cache.get_soil_richness(cell_pos as usize) {
                                    SoilRichness::UNUSABLE => panic!(),
                                    SoilRichness::LOW_QUALITY => 0,
                                    SoilRichness::MEDIUM_QUALITY => 2,
                                    SoilRichness::HIGH_QUALITY => 4,
                                };
                            player.large_tree_count -= 1;
                            self.board[cell_pos as usize] = None;
                            completed_trees_count += 1;
                        }
                        Move::WAIT => {
                            player.is_asleep = true;
                        }
                    }
                }

                player.move_ = None;
            }
            self.nutrient = self.nutrient.saturating_sub(completed_trees_count);
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

            // Reactivate all trees and update shadows
            for (cell_pos, cell) in self.board.iter_mut().enumerate() {
                if let Some(c) = cell {
                    c.is_dormant = false;
                }
            }

            // let the players collect sun points
            if self.day < 24 {
                let gained_sun_points = gained_sun_points(
                    &self.board,
                    &get_spookied_cells(&self.board, self.day, &self.cache),
                );
                self.players[0].sun += gained_sun_points[0];
                self.players[1].sun += gained_sun_points[1];
            }
        } else {
            let next_player = (self.active_player + 1) % 2;
            if self.players[next_player as usize].is_asleep == false {
                self.active_player = next_player;
            }
        }

        /* (5) Check terminal conditions */
        if self.day == 24 {
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

        let mut board_repr: Vec<Vec<record::CellState>> =
            vec![vec![record::CellState::default(); 7]; 7];
        for r in 0..7 {
            for c in 0..7 {
                board_repr[r][c] = match board_pos_to_cell_id(r, c) {
                    Some(cell_pos) => {
                        //1st pos: richness
                        let richness: char = match self.cache.get_soil_richness(cell_pos) {
                            SoilRichness::UNUSABLE => '0',
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
                                Tree::SEED => '🌰',
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

                        //5th pos : Cell is shadowed
                        let shadowed_cells = get_shadowed_cells(&self.board, self.day, &self.cache);
                        let spookied_cells = get_spookied_cells(&self.board, self.day, &self.cache);

                        let shadow: char = {
                            if spookied_cells[cell_pos] == true {
                                '2' // Spookied cell
                            } else if shadowed_cells[cell_pos] == true {
                                '1' // Shadowed non-spookied cell
                            } else {
                                '0' // Non-shadowed cell
                            }
                        };

                        record::CellState {
                            cell_state: format!(
                                "{}{}{}{}{}",
                                richness, player, tree, dormant, shadow
                            ),
                            tooltip: Some(format!("pos: {}", cell_pos)),
                        }
                    }
                    None => record::CellState {
                        cell_state: "....".to_string(),
                        tooltip: None,
                    },
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
                    "[{}🌰, {}🌱, {}🪴, {}🌳]",
                    player.seed_count,
                    player.small_tree_count,
                    player.medium_tree_count,
                    player.large_tree_count
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
            '0',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert("backgroundColor".to_string(), "#999999".to_string());
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
            '🌰',
            record::CellClass {
                text: Some('🌰'.to_string()),
                text_style: Some(text_style.clone()),
                cell_style: None,
            },
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

        // Fifth position : Shadow
        let mut class_styles: HashMap<char, record::CellClass> = HashMap::new();
        class_styles.insert(
            '0',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: None,
            },
        );
        class_styles.insert(
            '1',
            record::CellClass {
                text: None,
                text_style: None,
                cell_style: Some({
                    let mut css = HashMap::new();
                    css.insert(
                        "background-image".to_string(),
                        "radial-gradient(black 1px, transparent 0)".to_string(),
                    );
                    css.insert("background-size".to_string(), "20% 20%".to_string());
                    css.insert("background-repeat".to_string(), "repeat".to_string());
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
                    css.insert(
                        "background-image".to_string(),
                        "radial-gradient(black 1px, transparent 0)".to_string(),
                    );
                    css.insert("background-size".to_string(), "7% 7%".to_string());
                    css.insert("background-repeat".to_string(), "repeat".to_string());
                    css
                }),
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
    use common::assert_vec_eq;

    use super::*;

    #[test]
    fn test_cache_get_neighbor() {
        let cache = Cache::new([SoilRichness::UNUSABLE; 37]);

        assert_eq!(cache.get_neighbor(14, 1, 1), Some(4));
        assert_eq!(cache.get_neighbor(9, 2, 1), Some(23));
        assert_eq!(cache.get_neighbor(28, 3, 3), None);
        assert_eq!(cache.get_neighbor(26, 5, 1), Some(12));
        assert_eq!(cache.get_neighbor(17, 2, 3), Some(3));
        assert_eq!(cache.get_neighbor(26, 1, 2), None);
        assert_eq!(cache.get_neighbor(11, 5, 2), Some(0));
        assert_eq!(cache.get_neighbor(35, 0, 2), None);
        assert_eq!(cache.get_neighbor(5, 2, 1), Some(4));
        assert_eq!(cache.get_neighbor(2, 4, 1), Some(0));
        assert_eq!(cache.get_neighbor(34, 5, 1), None);
        assert_eq!(cache.get_neighbor(36, 0, 1), None);
        assert_eq!(cache.get_neighbor(14, 1, 3), Some(10));
        assert_eq!(cache.get_neighbor(3, 1, 1), Some(10));
        assert_eq!(cache.get_neighbor(15, 4, 3), None);
        assert_eq!(cache.get_neighbor(36, 4, 3), None);
        assert_eq!(cache.get_neighbor(3, 4, 2), Some(14));
        assert_eq!(cache.get_neighbor(7, 2, 3), Some(23));
        assert_eq!(cache.get_neighbor(27, 2, 1), None);
        assert_eq!(cache.get_neighbor(9, 1, 2), None);
    }

    #[test]
    fn test_cache_get_seedable_neighbors_1() {
        let mut soil_richness = [SoilRichness::LOW_QUALITY; 37];
        soil_richness[10] = SoilRichness::UNUSABLE;
        soil_richness[16] = SoilRichness::UNUSABLE;
        soil_richness[20] = SoilRichness::UNUSABLE;
        soil_richness[21] = SoilRichness::UNUSABLE;
        soil_richness[25] = SoilRichness::UNUSABLE;
        soil_richness[26] = SoilRichness::UNUSABLE;
        soil_richness[29] = SoilRichness::UNUSABLE;
        soil_richness[30] = SoilRichness::UNUSABLE;
        soil_richness[34] = SoilRichness::UNUSABLE;
        soil_richness[35] = SoilRichness::UNUSABLE;

        let cache = Cache::new(soil_richness);

        assert_vec_eq!(
            cache.get_seedable_neighbors(0, Tree::SMALL_TREE),
            &[1, 2, 3, 4, 5, 6]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(0, Tree::MEDIUM_TREE),
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 17, 18]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(0, Tree::LARGE_TREE),
            &[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 17, 18, 19, 22, 23, 24, 27, 28, 31,
                32, 33, 36
            ]
        );

        assert_vec_eq!(
            cache.get_seedable_neighbors(1, Tree::SMALL_TREE),
            &[7, 8, 2, 0, 6, 18]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(14, Tree::SMALL_TREE),
            &[5, 4, 13, 15]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(15, Tree::MEDIUM_TREE),
            &[5, 14, 31, 32, 17, 6, 0, 4, 13, 33]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(9, Tree::MEDIUM_TREE),
            &[22, 23, 2, 8, 24, 11, 3, 0, 1, 7]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(4, Tree::MEDIUM_TREE),
            &[0, 3, 12, 13, 14, 5, 1, 2, 11, 27, 28, 15, 6]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(24, Tree::LARGE_TREE),
            &[11, 23, 22, 9, 2, 3, 12, 8, 1, 0, 4, 13, 27]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(14, Tree::SMALL_TREE),
            &[15, 5, 4, 13]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(17, Tree::LARGE_TREE),
            &[18, 6, 33, 36, 7, 1, 0, 5, 15, 32, 19, 8, 2, 3, 4, 14, 31]
        );
    }

    #[test]
    fn test_cache_get_seedable_neighbors_2() {
        let mut soil_richness = [SoilRichness::LOW_QUALITY; 37];
        soil_richness[3] = SoilRichness::UNUSABLE;
        soil_richness[6] = SoilRichness::UNUSABLE;
        soil_richness[7] = SoilRichness::UNUSABLE;
        soil_richness[13] = SoilRichness::UNUSABLE;
        soil_richness[20] = SoilRichness::UNUSABLE;
        soil_richness[24] = SoilRichness::UNUSABLE;
        soil_richness[27] = SoilRichness::UNUSABLE;
        soil_richness[29] = SoilRichness::UNUSABLE;
        soil_richness[33] = SoilRichness::UNUSABLE;
        soil_richness[36] = SoilRichness::UNUSABLE;

        let cache = Cache::new(soil_richness);

        assert_vec_eq!(
            cache.get_seedable_neighbors(28, Tree::SMALL_TREE),
            &Vec::<usize>::new()
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(8, Tree::SMALL_TREE),
            &[21, 9, 2, 1]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(21, Tree::LARGE_TREE),
            &[22, 9, 8, 23, 10, 2, 1, 19, 11, 0, 18]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(4, Tree::MEDIUM_TREE),
            &[0, 12, 14, 5, 1, 2, 10, 11, 26, 28, 30, 15, 16]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(19, Tree::LARGE_TREE),
            &[21, 8, 1, 18, 35, 22, 9, 2, 0, 17, 34]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(32, Tree::LARGE_TREE),
            &[16, 15, 31, 34, 17, 5, 14, 30, 35, 18, 1, 0, 4]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(10, Tree::MEDIUM_TREE),
            &[9, 23, 11, 2, 21, 22, 25, 26, 12, 4, 0, 1, 8]
        );
        assert_vec_eq!(
            cache.get_seedable_neighbors(16, Tree::LARGE_TREE),
            &[5, 15, 32, 17, 35, 18, 1, 0, 4, 14, 30, 31, 34, 8, 2, 12]
        );
    }

    #[test]
    fn test_cache_get_shadowed_cells() {
        let cache = Cache::new([SoilRichness::UNUSABLE; 37]);

        assert_vec_eq!(
            cache.get_shadowed_cells(23, Tree::MEDIUM_TREE, 16),
            &[10, 3]
        );
        assert_vec_eq!(cache.get_shadowed_cells(3, Tree::SMALL_TREE, 13), &[10]);
        assert_vec_eq!(cache.get_shadowed_cells(10, Tree::SMALL_TREE, 3), &[11]);
        assert_vec_eq!(cache.get_shadowed_cells(5, Tree::SMALL_TREE, 6), &[6]);
        assert_vec_eq!(
            cache.get_shadowed_cells(17, Tree::LARGE_TREE, 9),
            &[16, 15, 30]
        );
        assert_vec_eq!(
            cache.get_shadowed_cells(27, Tree::SMALL_TREE, 2),
            &Vec::<usize>::new()
        );
        assert_vec_eq!(
            cache.get_shadowed_cells(17, Tree::LARGE_TREE, 8),
            &[6, 0, 3]
        );
        assert_vec_eq!(
            cache.get_shadowed_cells(21, Tree::MEDIUM_TREE, 23),
            &[20, 19]
        );
        assert_vec_eq!(
            cache.get_shadowed_cells(24, Tree::LARGE_TREE, 7),
            &Vec::<usize>::new()
        );
        assert_vec_eq!(cache.get_shadowed_cells(13, Tree::SMALL_TREE, 11), &[14]);
    }

    #[test]
    fn test_get_shadowed_spookied_cells_1() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![19, 22, 10], vec![12, 28]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![0, 1], vec![4, 27, 31, 34]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![23], vec![32]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![2, 36], vec![5]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let expected_shadowed_cells = [10, 3, 28, 0, 14, 5, 6, 15, 35, 31, 34];
        let mut expected_are_shadowed_cells = [false; 37];
        for sc in expected_shadowed_cells {
            expected_are_shadowed_cells[sc] = true;
        }

        let expected_spookied_cells = [28, 10, 0, 5, 31, 34];
        let mut expected_are_spookied_cells = [false; 37];
        for sc in expected_spookied_cells {
            expected_are_spookied_cells[sc] = true;
        }

        let day = 10;

        assert_eq!(
            get_shadowed_cells(&board, day, &cache),
            expected_are_shadowed_cells
        );

        assert_eq!(
            get_spookied_cells(&board, day, &cache),
            expected_are_spookied_cells
        );
    }

    #[test]
    fn test_get_shadowed_spookied_cells_2() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![14, 15], vec![25, 20, 19, 21]]);
        trees.insert(
            Tree::SMALL_TREE,
            vec![vec![27, 28, 0, 30], vec![22, 23, 24, 29]],
        );
        trees.insert(Tree::MEDIUM_TREE, vec![vec![31, 32], vec![36]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![2, 5, 33], vec![6, 34]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let expected_shadowed_cells = [10, 9, 21, 13, 1, 29, 6, 18, 30, 16, 17, 35, 31, 33, 34];
        let mut expected_are_shadowed_cells = [false; 37];
        for sc in expected_shadowed_cells {
            expected_are_shadowed_cells[sc] = true;
        }

        let expected_spookied_cells = [21, 29, 30, 33, 34];
        let mut expected_are_spookied_cells = [false; 37];
        for sc in expected_spookied_cells {
            expected_are_spookied_cells[sc] = true;
        }

        let day = 17;

        assert_eq!(
            get_shadowed_cells(&board, day, &cache),
            expected_are_shadowed_cells
        );

        assert_eq!(
            get_spookied_cells(&board, day, &cache),
            expected_are_spookied_cells
        );
    }

    #[test]
    fn test_get_shadowed_spookied_cells_3() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![3, 4, 28, 29], vec![1, 6, 20, 35]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![30], vec![21]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![27], vec![36]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![], vec![]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let expected_shadowed_cells = [8, 28, 35, 34];
        let mut expected_are_shadowed_cells = [false; 37];
        for sc in expected_shadowed_cells {
            expected_are_shadowed_cells[sc] = true;
        }

        let expected_spookied_cells = [28, 35];
        let mut expected_are_spookied_cells = [false; 37];
        for sc in expected_spookied_cells {
            expected_are_spookied_cells[sc] = true;
        }

        let day = 4;

        assert_eq!(
            get_shadowed_cells(&board, day, &cache),
            expected_are_shadowed_cells
        );

        assert_eq!(
            get_spookied_cells(&board, day, &cache),
            expected_are_spookied_cells
        );
    }

    #[test]
    fn test_get_shadowed_spookied_cells_4() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![18, 16], vec![0, 28]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![19], vec![26, 32]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![23], vec![30]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![21, 35, 36], vec![27, 31]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let expected_shadowed_cells = [24, 25, 9, 10, 11, 7, 18, 6, 5, 17, 16, 15, 31];
        let mut expected_are_shadowed_cells = [false; 37];
        for sc in expected_shadowed_cells {
            expected_are_shadowed_cells[sc] = true;
        }

        let expected_spookied_cells = [18, 16];
        let mut expected_are_spookied_cells = [false; 37];
        for sc in expected_spookied_cells {
            expected_are_spookied_cells[sc] = true;
        }

        let day = 21;

        assert_eq!(
            get_shadowed_cells(&board, day, &cache),
            expected_are_shadowed_cells
        );

        assert_eq!(
            get_spookied_cells(&board, day, &cache),
            expected_are_spookied_cells
        );
    }

    #[test]
    fn test_get_shadowed_spookied_cells_5() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![14, 18, 36, 15], vec![24, 23, 27]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![30, 32, 33], vec![22]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![31, 34], vec![25, 4, 1]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![6], vec![3, 0]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let expected_shadowed_cells = [23, 22, 10, 9, 21, 3, 2, 8, 1, 14, 5, 36, 15, 16, 17, 35];
        let mut expected_are_shadowed_cells = [false; 37];
        for sc in expected_shadowed_cells {
            expected_are_shadowed_cells[sc] = true;
        }

        let expected_spookied_cells = [23, 22, 1, 14, 36, 15];
        let mut expected_are_spookied_cells = [false; 37];
        for sc in expected_spookied_cells {
            expected_are_spookied_cells[sc] = true;
        }

        let day = 13;

        assert_eq!(
            get_shadowed_cells(&board, day, &cache),
            expected_are_shadowed_cells
        );

        assert_eq!(
            get_spookied_cells(&board, day, &cache),
            expected_are_spookied_cells
        );
    }

    #[test]
    fn test_gained_sun_points_1() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![24, 26, 0, 5, 29], vec![20, 33, 35]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![3, 4, 28], vec![34, 6, 2]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![25], vec![1, 19]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![], vec![]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let day = 8;
        let spookied_cells = get_spookied_cells(&board, day, &cache);

        assert_eq!(gained_sun_points(&board, &spookied_cells), [5, 6]);
    }

    #[test]
    fn test_gained_sun_points_2() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![25, 23, 27], vec![18, 19]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![0], vec![5, 6, 20]]);
        trees.insert(
            Tree::MEDIUM_TREE,
            vec![vec![24, 3, 2, 28, 29], vec![32, 33, 34]],
        );
        trees.insert(Tree::LARGE_TREE, vec![vec![31], vec![36]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let day = 13;
        let spookied_cells = get_spookied_cells(&board, day, &cache);

        assert_eq!(gained_sun_points(&board, &spookied_cells), [13, 10]);
    }

    #[test]
    fn test_gained_sun_points_3() {
        let cache = Cache::new(get_initial_soil_richness());
        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![23, 24], vec![26, 28, 32]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![35], vec![27, 33]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![22, 36], vec![31]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![], vec![]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let day = 4;
        let spookied_cells = get_spookied_cells(&board, day, &cache);

        assert_eq!(gained_sun_points(&board, &spookied_cells), [4, 4]);
    }

    #[test]
    fn test_valid_moves_1() {
        let mut soil_sichness = get_initial_soil_richness();
        soil_sichness[10] = SoilRichness::UNUSABLE;
        soil_sichness[16] = SoilRichness::UNUSABLE;
        let cache = Cache::new(soil_sichness);

        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(Tree::SEED, vec![vec![], vec![]]);
        trees.insert(Tree::SMALL_TREE, vec![vec![27, 32], vec![23, 36]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![], vec![]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![], vec![]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let p_id = 0;
        let p_sun = 2;
        let p_seed_count = 0;
        let p_small_tree_count = 2;
        let p_medium_tree_count = 0;
        let p_large_tree_count = 0;
        let p_is_asleep = false;

        let expected_moves = [
            Move::WAIT,
            Move::SEED(27, 26),
            Move::SEED(27, 13),
            Move::SEED(32, 15),
            Move::SEED(27, 12),
            Move::SEED(32, 31),
            Move::SEED(27, 28),
            Move::SEED(32, 33),
        ];

        assert_vec_eq!(
            valid_moves(
                &board,
                p_id,
                p_sun,
                p_seed_count,
                p_small_tree_count,
                p_medium_tree_count,
                p_large_tree_count,
                p_is_asleep,
                &cache
            ),
            expected_moves
        );
    }

    #[test]
    fn test_valid_moves_2() {
        let mut soil_sichness = get_initial_soil_richness();
        soil_sichness[10] = SoilRichness::UNUSABLE;
        soil_sichness[16] = SoilRichness::UNUSABLE;
        let cache = Cache::new(soil_sichness);

        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(
            Tree::SEED,
            vec![vec![24, 12, 29, 7], vec![3, 4, 35, 36, 32]],
        );
        trees.insert(Tree::SMALL_TREE, vec![vec![27, 28, 13, 23], vec![31]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![2, 20], vec![0, 6, 19]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![], vec![1, 5]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let p_id = 0;
        let p_sun = 0;
        let p_seed_count = 5;
        let p_small_tree_count = 4;
        let p_medium_tree_count = 2;
        let p_large_tree_count = 0;
        let p_is_asleep = false;

        let expected_moves = [Move::WAIT];

        assert_vec_eq!(
            valid_moves(
                &board,
                p_id,
                p_sun,
                p_seed_count,
                p_small_tree_count,
                p_medium_tree_count,
                p_large_tree_count,
                p_is_asleep,
                &cache
            ),
            expected_moves
        );
    }

    #[test]
    fn test_valid_moves_3() {
        let mut soil_sichness = get_initial_soil_richness();
        soil_sichness[24] = SoilRichness::UNUSABLE;
        soil_sichness[33] = SoilRichness::UNUSABLE;
        let cache = Cache::new(soil_sichness);

        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(
            Tree::SEED,
            vec![vec![10, 14, 5, 15, 31, 35], vec![0, 2, 6, 7, 8, 36]],
        );
        trees.insert(Tree::SMALL_TREE, vec![vec![27, 11, 3], vec![26, 25, 1, 18]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![28, 13, 12, 29, 34], vec![19]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![], vec![17]]);

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: false,
                    });
                }
            }
        }

        let p_id = 1;
        let p_sun = 11;
        let p_seed_count = 6;
        let p_small_tree_count = 4;
        let p_medium_tree_count = 1;
        let p_large_tree_count = 1;
        let p_is_asleep = false;

        let expected_moves = [
            Move::WAIT,
            Move::COMPLETE(17),
            Move::GROW(26),
            Move::GROW(18),
            Move::GROW(7),
            Move::GROW(36),
            Move::GROW(25),
            Move::GROW(2),
            Move::GROW(8),
            Move::GROW(1),
            Move::GROW(19),
            Move::GROW(0),
            Move::GROW(6),
            Move::SEED(17, 4),
            Move::SEED(17, 20),
            Move::SEED(19, 20),
            Move::SEED(19, 21),
            Move::SEED(17, 32),
            Move::SEED(17, 30),
            Move::SEED(17, 16),
        ];

        assert_vec_eq!(
            valid_moves(
                &board,
                p_id,
                p_sun,
                p_seed_count,
                p_small_tree_count,
                p_medium_tree_count,
                p_large_tree_count,
                p_is_asleep,
                &cache
            ),
            expected_moves
        );
    }

    #[test]
    fn test_valid_moves_4() {
        let mut soil_sichness = get_initial_soil_richness();
        let cache = Cache::new(soil_sichness);

        let mut board: [Option<Cell>; 37] = [None; 37];
        let mut trees: HashMap<Tree, Vec<Vec<usize>>> = HashMap::new();
        trees.insert(
            Tree::SEED,
            vec![vec![1, 13, 19, 36, 17, 33, 34], vec![24, 10, 8, 28, 5, 31]],
        );
        trees.insert(Tree::SMALL_TREE, vec![vec![14, 7], vec![23, 22, 30]]);
        trees.insert(Tree::MEDIUM_TREE, vec![vec![15, 32, 20], vec![29, 9, 21]]);
        trees.insert(Tree::LARGE_TREE, vec![vec![], vec![]]);

        let mut dormant: [bool; 37] = [false; 37];
        dormant[9] = true;
        dormant[13] = true;
        dormant[14] = true;
        dormant[30] = true;
        dormant[15] = true;

        for tree_size in [
            Tree::SEED,
            Tree::SMALL_TREE,
            Tree::MEDIUM_TREE,
            Tree::LARGE_TREE,
        ] {
            for p_id in 0..2 {
                for tree_pos in trees.get(&tree_size).unwrap()[p_id].iter() {
                    board[*tree_pos] = Some(Cell {
                        player: p_id as u8,
                        tree: tree_size,
                        is_dormant: dormant[*tree_pos],
                    });
                }
            }
        }

        let p_id = 0;
        let p_sun = 4;
        let p_seed_count = 7;
        let p_small_tree_count = 2;
        let p_medium_tree_count = 3;
        let p_large_tree_count = 0;
        let p_is_asleep = false;

        let expected_moves = [
            Move::WAIT,
            Move::GROW(17),
            Move::GROW(19),
            Move::GROW(34),
            Move::GROW(33),
            Move::GROW(36),
            Move::GROW(1),
        ];

        assert_vec_eq!(
            valid_moves(
                &board,
                p_id,
                p_sun,
                p_seed_count,
                p_small_tree_count,
                p_medium_tree_count,
                p_large_tree_count,
                p_is_asleep,
                &cache
            ),
            expected_moves
        );
    }

    #[test]
    fn test_cells_are_dormant_after_both_players_seed() {
        let players_initial_small_trees = [[21, 32], [20, 35]];
        let invalid_cells = [12, 8, 14, 16];

        let mut game = init_with_params(&players_initial_small_trees, &invalid_cells);
        game.play("SEED 21 9".to_string());
        game.play("SEED 20 7".to_string());

        assert_eq!(game.board[9].unwrap().tree, Tree::SEED);
        assert_eq!(game.board[7].unwrap().tree, Tree::SEED);
        assert_eq!(game.board[21].unwrap().is_dormant, true);
        assert_eq!(game.board[9].unwrap().is_dormant, true);
        assert_eq!(game.board[20].unwrap().is_dormant, true);
        assert_eq!(game.board[7].unwrap().is_dormant, true);
        assert_eq!(game.players[0].seed_count, 1);
        assert_eq!(game.players[1].seed_count, 1);
    }

    #[test]
    fn test_players_seed_the_same_cell() {
        let players_initial_small_trees = [[29, 30], [27, 26]];
        let invalid_cells = [];

        let mut game = init_with_params(&players_initial_small_trees, &invalid_cells);
        game.play("SEED 29 13".to_string());
        game.play("SEED 27 13".to_string());

        assert!(game.board[13].is_none());
        assert_eq!(game.board[29].unwrap().is_dormant, true);
        assert_eq!(game.board[27].unwrap().is_dormant, true);
        assert_eq!(game.players[0].seed_count, 0);
        assert_eq!(game.players[1].seed_count, 0);
        assert_eq!(game.players[0].sun, 2);
        assert_eq!(game.players[1].sun, 2);
    }
}
