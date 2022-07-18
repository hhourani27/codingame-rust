use std::sync::mpsc::{Receiver, Sender};

use self::game::SoilRichness;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct StackVector<T: Copy + Clone + Default, const MAX_SIZE: usize> {
    pub arr: [T; MAX_SIZE],
    pub len: usize,
}

#[allow(dead_code)]
impl<T: Copy + Clone + Default, const MAX_SIZE: usize> StackVector<T, MAX_SIZE> {
    pub fn new() -> Self {
        Self {
            arr: [T::default(); MAX_SIZE],
            len: 0,
        }
    }

    pub fn push(&mut self, e: T) {
        self.arr[self.len] = e;
        self.len += 1;
    }

    pub fn slice(&self) -> &[T] {
        &self.arr[0..self.len]
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        &mut self.arr[0..self.len]
    }

    pub fn get(&self, idx: usize) -> &T {
        &self.arr[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.arr[idx]
    }

    pub fn remove(&mut self, idx: usize) -> T {
        let removed_element = self.arr[idx];

        for i in idx..self.len - 1 {
            self.arr[i] = self.arr[i + 1];
        }
        self.len -= 1;

        removed_element
    }

    pub fn remove_multi<const NB_ELEMENTS: usize>(
        &mut self,
        mut indices: [usize; NB_ELEMENTS],
    ) -> [T; NB_ELEMENTS] {
        let mut removed_elements: [T; NB_ELEMENTS] = [Default::default(); NB_ELEMENTS];

        indices.sort();
        for i in 0..NB_ELEMENTS {
            removed_elements[i] = self.remove(indices[i] - i);
        }

        removed_elements
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn from(v: &[T]) -> StackVector<T, MAX_SIZE> {
        let mut sv: StackVector<T, MAX_SIZE> = StackVector::new();

        for e in v.iter() {
            sv.push(e.clone());
        }
        sv
    }
}

mod game {

    use super::StackVector;
    use rand::seq::SliceRandom;
    use std::collections::HashSet;
    use std::fmt;

    pub const MAX_VALID_MOVES: usize = 80; // Arbitrary value. TODO: compute the correct value

    pub type GameScore = [f32; 4];

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WinLossTie {
        Win,
        Loss,
        Tie,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Move {
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

    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Debug)]
    pub enum Tree {
        SEED = 0,
        SMALL_TREE = 1,
        MEDIUM_TREE = 2,
        LARGE_TREE = 3,
    }

    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum SoilRichness {
        UNUSABLE,
        LOW_QUALITY,
        MEDIUM_QUALITY,
        HIGH_QUALITY,
    }

    #[derive(Clone, Copy)]
    pub struct Cell {
        pub player: u8,
        pub tree: Tree,
        pub is_dormant: bool,
    }

    #[derive(Clone, Copy, Default)]
    pub struct Player {
        pub move_: Option<Move>,

        pub sun: u32,
        pub score: u32,

        pub seed_count: u8,
        pub small_tree_count: u8,
        pub medium_tree_count: u8,
        pub large_tree_count: u8,

        pub is_asleep: bool,
    }

    #[derive(Clone)]
    pub struct State {
        pub board: [Option<Cell>; 37],
        pub players: [Player; 2],

        pub nutrient: u8,

        pub day: u8,
        pub turn_during_day: u8,

        pub active: bool,
        pub active_player: u8,
        pub winners: Option<(WinLossTie, WinLossTie)>,
    }

    /* #region [Public functions] */
    pub fn valid_moves(state: &State, cache: &Cache) -> (u8, StackVector<Move, MAX_VALID_MOVES>) {
        let board = &state.board;
        let p_id = state.active_player;
        let p_sun = state.players[p_id as usize].sun;
        let p_seed_count = state.players[p_id as usize].seed_count;
        let p_small_tree_count = state.players[p_id as usize].small_tree_count;
        let p_medium_tree_count = state.players[p_id as usize].medium_tree_count;
        let p_large_tree_count = state.players[p_id as usize].large_tree_count;

        let mut valid_moves: StackVector<Move, MAX_VALID_MOVES> = StackVector::new();

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

        (p_id, valid_moves)
    }

    pub fn random_valid_move(state: &State, cache: &Cache) -> (u8, Move) {
        let valid_moves = valid_moves(state, cache);

        let chosen_move = valid_moves
            .1
            .slice()
            .choose(&mut rand::thread_rng())
            .unwrap();

        (valid_moves.0, *chosen_move)
    }

    pub fn update_state(state: &mut State, player: u8, move_: Move, cache: &Cache) {
        /* (2) Record the move */
        state.players[state.active_player as usize].move_ = Some(move_);

        /* (3) Check if both players have played, so that we update the game */
        if (state.players[0].move_.is_some() && state.players[1].move_.is_some())
            || (state.players[0].move_.is_some() && state.players[1].is_asleep == true)
            || (state.players[0].is_asleep == true && state.players[1].move_.is_some())
        {
            /* (3.2) Update the state */
            let mut completed_trees_count = 0;
            let player_moves = [
                state.players[0].move_.clone(),
                state.players[1].move_.clone(),
            ];

            for (p_id, player) in state.players.iter_mut().enumerate() {
                if player.is_asleep == false {
                    match player.move_.unwrap() {
                        Move::SEED(tree_pos, seed_pos) => match player_moves[(p_id + 1) % 2] {
                            Some(Move::SEED(o_tree_pos, o_seed_pos)) if o_seed_pos == seed_pos => {
                                let tree_cell = state.board[tree_pos as usize].as_mut().unwrap();
                                tree_cell.is_dormant = true;
                            }
                            _ => {
                                player.sun -= player.seed_count as u32;
                                player.seed_count += 1;
                                let tree_cell = state.board[tree_pos as usize].as_mut().unwrap();
                                tree_cell.is_dormant = true;
                                state.board[seed_pos as usize] = Some(Cell {
                                    player: p_id as u8,
                                    tree: Tree::SEED,
                                    is_dormant: true,
                                });
                            }
                        },
                        Move::GROW(cell_pos) => {
                            let cell = state.board[cell_pos as usize].as_mut().unwrap();
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
                            player.score += state.nutrient as u32
                                + match cache.get_soil_richness(cell_pos as usize) {
                                    SoilRichness::UNUSABLE => panic!(),
                                    SoilRichness::LOW_QUALITY => 0,
                                    SoilRichness::MEDIUM_QUALITY => 2,
                                    SoilRichness::HIGH_QUALITY => 4,
                                };
                            player.large_tree_count -= 1;
                            state.board[cell_pos as usize] = None;
                            completed_trees_count += 1;
                        }
                        Move::WAIT => {
                            player.is_asleep = true;
                        }
                    }
                }

                player.move_ = None;
            }
            state.nutrient = state.nutrient.saturating_sub(completed_trees_count);
            state.turn_during_day += 1;
        }

        /* (4) If both players are asleep, update the day, else set the next active player */
        if state.players[0].is_asleep == true && state.players[1].is_asleep == true {
            state.day += 1;
            state.turn_during_day = 0;
            state.players[0].move_ = None;
            state.players[1].move_ = None;
            state.players[0].is_asleep = false;
            state.players[1].is_asleep = false;
            state.active_player = 0;

            // Reactivate all trees and update shadows
            for (cell_pos, cell) in state.board.iter_mut().enumerate() {
                if let Some(c) = cell {
                    c.is_dormant = false;
                }
            }

            // let the players collect sun points
            if state.day < 24 {
                let gained_sun_points = gained_sun_points(
                    &state.board,
                    &get_spookied_cells(&state.board, state.day, &cache),
                );
                state.players[0].sun += gained_sun_points[0];
                state.players[1].sun += gained_sun_points[1];
            }
        } else {
            let next_player = (state.active_player + 1) % 2;
            if state.players[next_player as usize].is_asleep == false {
                state.active_player = next_player;
            }
        }

        /* (5) Check terminal conditions */
        if state.day == 24 {
            let player0 = &state.players[0];
            let player1 = &state.players[1];

            let score0 = player0.score + player0.sun / 3;
            let score1 = player1.score + player1.sun / 3;

            if score0 > score1 {
                state.active = false;
                state.winners = Some((WinLossTie::Win, WinLossTie::Loss));
            } else if score0 < score1 {
                state.active = false;
                state.winners = Some((WinLossTie::Loss, WinLossTie::Win));
            } else {
                let tree_count0 =
                    player0.small_tree_count + player0.medium_tree_count + player0.large_tree_count;
                let tree_count1 =
                    player1.small_tree_count + player1.medium_tree_count + player1.large_tree_count;

                if tree_count0 > tree_count1 {
                    state.active = false;
                    state.winners = Some((WinLossTie::Win, WinLossTie::Loss));
                } else if tree_count0 < tree_count1 {
                    state.active = false;
                    state.winners = Some((WinLossTie::Loss, WinLossTie::Win));
                } else {
                    state.active = false;
                    state.winners = Some((WinLossTie::Tie, WinLossTie::Tie));
                }
            }
        }
    }

    /* #endregion */

    /* #region(collapsed) [Private functions] */
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

    pub fn is_terminal(state: &State) -> bool {
        !state.active
    }

    pub fn get_scores(state: &State) -> GameScore {
        match state.winners {
            Some((WinLossTie::Win, WinLossTie::Loss)) => [1.0, 0.0, 0.0, 0.0],
            Some((WinLossTie::Loss, WinLossTie::Win)) => [0.0, 1.0, 0.0, 0.0],
            Some((WinLossTie::Tie, WinLossTie::Tie)) => [0.5, 0.5, 0.0, 0.0],
            _ => panic!(),
        }
    }

    /* #endregion */

    /* #region(collapsed) [Cache] */
    pub struct Cache {
        soil_richness: [SoilRichness; 37],
        cell_neighbors: [[[Option<usize>; 3]; 6]; 37], //[cell_pos][direction][distance] => neighbor position
        seedable_neighbors: Vec<Vec<Vec<usize>>>, //[[Vec<usize>;3];37] : [cell_pos][tree size] => vector of seedable positions
        shadowed_cells: Vec<Vec<Vec<Vec<usize>>>>, // [Vec<usize>;3;6;37]: [cell_pos][day][tree size] => vector of shadowed cell positions
    }

    impl Cache {
        pub fn new(soil_richness: [SoilRichness; 37]) -> Self {
            let cell_neighbors = Cache::init_cell_neighbors();
            let seedable_neighbors =
                Cache::init_seedable_neighbors(&cell_neighbors, &soil_richness);
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
                                shadowed_cells[cell_pos][day][tree_size - 1]
                                    .push(shadowed_cell_pos);
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

        fn get_neighbor(
            &self,
            cell_pos: usize,
            direction: usize,
            distance: usize,
        ) -> Option<usize> {
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
}

mod mcts {

    use super::game;
    use rand::Rng;
    use std::time::Instant;

    const MAX_NODE_COUNT: usize = 300_000;
    const TIME_LIMIT_MS: u128 = 98;

    #[derive(Clone, Copy)]
    struct Node {
        move_: Option<game::Move>,
        player: Option<u8>, // player who did the move

        parent: Option<usize>,
        child_first: Option<usize>,
        child_count: u8,
        expanded: bool,

        visits: u32,
        score: f32,
    }

    impl Default for Node {
        fn default() -> Self {
            Node {
                move_: None,
                player: None,

                parent: None,
                child_first: None,
                child_count: 0,
                expanded: false,

                visits: 0,
                score: 0.0,
            }
        }
    }

    pub struct MCTS {
        arr: Vec<Node>,
        len: usize,
        nb_simulations: u32,
    }

    pub fn new() -> MCTS {
        MCTS {
            arr: vec![Default::default(); MAX_NODE_COUNT],
            len: 0,
            nb_simulations: 0,
        }
    }

    impl MCTS {
        pub fn best_move(
            &mut self,
            root_state: &game::State,
            player: u8,
            cache: &game::Cache,
        ) -> game::Move {
            /*
                Find the best move
                - Starting from State [state],
                - And already given (for optimization) the [valid_moves] that [player] can do
            */

            //eprintln!("[MCTS] init");
            let start = Instant::now();
            self.init(player);

            while (start.elapsed().as_millis() < TIME_LIMIT_MS)
                & (self.len < MAX_NODE_COUNT - game::MAX_VALID_MOVES)
            {
                let mut state = root_state.clone();

                //eprintln!("[MCTS] Selection");

                let selected_node_idx = self.select(&mut state, cache);

                //eprintln!("[MCTS] Expansion");
                let rollout_node_idx = self.expand(selected_node_idx, &mut state, cache);

                //eprintln!("[MCTS] Simulation");
                let score = self.simulate(&mut state, cache);

                self.backpropagate(rollout_node_idx, score);

                self.nb_simulations += 1;
            }

            eprintln!(
                "[MCTS P3] End. Sending best move after expanding {} nodes and running {} simulations in {:?}",
                self.len, self.nb_simulations, start.elapsed()
            );

            // When time is up, choose the move with the best score
            let mut max_score: f32 = -f32::INFINITY;
            let mut max_score_child_idx = 0;
            for c in self.arr[0].child_first.unwrap()
                ..self.arr[0].child_first.unwrap() + self.arr[0].child_count as usize
            {
                let child = &self.arr[c];
                let child_score = match child.visits {
                    0 => -f32::INFINITY,
                    _ => child.score / child.visits as f32,
                };
                if child_score > max_score {
                    max_score = child_score;
                    max_score_child_idx = c;
                }
            }

            self.arr[max_score_child_idx].move_.unwrap()
        }

        fn init(&mut self, player: u8) {
            // Re-initialize the node tree

            // Re-initialize Root
            self.arr[0] = Default::default();
            self.arr[0].expanded = false;
            self.len = 1;
            self.nb_simulations = 0;
        }

        fn create_child(&mut self, parent: usize, move_: game::Move, player: u8) {
            self.arr[self.len] = Node {
                move_: Some(move_),
                player: Some(player),
                parent: Some(parent),
                child_first: None,
                child_count: 0,
                expanded: false,
                visits: 0,
                score: 0.0,
            };
            self.len += 1
        }

        fn select(&self, state: &mut game::State, cache: &game::Cache) -> usize {
            /* Go down the tree, selecting each time the node with the largest UCB, until you reach an unexpanded node
             On the way update the state.
            */
            let mut node_idx = 0;

            while self.arr[node_idx].child_count > 0 {
                let node = &self.arr[node_idx];

                // Identify child with largest UCB
                let mut max_ucb: f32 = 0.0;
                let mut max_ucb_node_idx = 0;
                for c in
                    node.child_first.unwrap()..node.child_first.unwrap() + node.child_count as usize
                {
                    let child = &self.arr[c];
                    let child_ucb = MCTS::ucb(node.visits, child.score, child.visits);
                    if child_ucb == f32::INFINITY {
                        //TODO: I'm choosing the first child with ucb=INF. Try to choose a bit more randomly
                        max_ucb_node_idx = c;
                        break;
                    } else if child_ucb >= max_ucb {
                        max_ucb = child_ucb;
                        max_ucb_node_idx = c;
                    }
                }

                node_idx = max_ucb_node_idx;

                game::update_state(
                    state,
                    self.arr[node_idx].player.unwrap(),
                    self.arr[node_idx].move_.unwrap(),
                    cache,
                )
            }

            node_idx
        }

        fn ucb(parent_visit: u32, score: f32, visits: u32) -> f32 {
            match visits {
                0 => f32::INFINITY,
                _ => {
                    (score / visits as f32)
                        + 0.41 * ((parent_visit as f32).ln() / (visits as f32)).sqrt()
                }
            }
        }

        fn expand(
            &mut self,
            selected_node_idx: usize,
            state: &mut game::State,
            cache: &game::Cache,
        ) -> usize {
            /*
                Expand the node [selected_node_idx], given its [state]
            */

            let selected_node = &mut self.arr[selected_node_idx];
            if selected_node.expanded == false {
                // This is a non-expanded node, expand it and return it
                selected_node.expanded = true;
                return selected_node_idx;
            } else if game::is_terminal(state) {
                // This is a terminal state, just return the node
                return selected_node_idx;
            } else {
                // This is an already expanded node
                // 1. Create its children, but do not expand them
                let (player, valid_moves) = game::valid_moves(state, cache);

                let child_first = self.len;
                let child_count = valid_moves.len;
                selected_node.child_first = Some(child_first);
                selected_node.child_count = child_count as u8;
                for m in valid_moves.slice() {
                    self.create_child(selected_node_idx, *m, player)
                }

                //2. Choose a random child, expand it and return it
                let chosen_child_idx =
                    rand::thread_rng().gen_range(child_first..child_first + child_count);
                self.arr[chosen_child_idx].expanded = true;

                game::update_state(
                    state,
                    player,
                    self.arr[chosen_child_idx].move_.unwrap(),
                    cache,
                );

                return chosen_child_idx;
            }
        }

        fn simulate(&self, state: &mut game::State, cache: &game::Cache) -> game::GameScore {
            // Simulate the game until the end
            while !game::is_terminal(state) {
                let (player, chosen_move) = game::random_valid_move(state, cache);

                game::update_state(state, player, chosen_move, cache);
            }

            // Get the result
            game::get_scores(state)
        }

        fn backpropagate(&mut self, selected_node_idx: usize, score: game::GameScore) {
            let mut node_idx = selected_node_idx;
            while self.arr[node_idx].parent.is_some() {
                self.arr[node_idx].visits += 1;
                self.arr[node_idx].score += score[self.arr[node_idx].player.unwrap() as usize];

                node_idx = self.arr[node_idx].parent.unwrap();
            }

            // Update visit count for the root node
            self.arr[0].visits += 1;
        }

        pub fn get_first_nodes_simulation_results(&self) -> Vec<(game::Move, u32, f32)> {
            let first_child_idx = self.arr[0].child_first.unwrap();
            let child_count = self.arr[0].child_count;

            self.arr[first_child_idx..first_child_idx + child_count as usize]
                .iter()
                .map(|&node| (node.move_.unwrap(), node.visits, node.score))
                .collect::<Vec<(game::Move, u32, f32)>>()
        }
    }
}

pub fn play(
    ctr_rcv: Receiver<bool>,
    msg_rcv: Receiver<String>,
    msg_snd: Sender<(String, Option<std::collections::HashMap<String, String>>)>,
    params: Option<Vec<String>>,
) {
    let mut soil_richness: [game::SoilRichness; 37] = [SoilRichness::UNUSABLE; 37];

    let mut input_line = String::new();
    input_line = msg_rcv.recv().unwrap();
    let number_of_cells = parse_input!(input_line, i32); // 37
    for i in 0..number_of_cells as usize {
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let index = parse_input!(inputs[0], usize); // 0 is the center cell, the next cells spiral outwards
        let richness = parse_input!(inputs[1], i32); // 0 if the cell is unusable, 1-3 for usable cells
        let neigh_0 = parse_input!(inputs[2], i32); // the index of the neighbouring cell for each direction
        let neigh_1 = parse_input!(inputs[3], i32);
        let neigh_2 = parse_input!(inputs[4], i32);
        let neigh_3 = parse_input!(inputs[5], i32);
        let neigh_4 = parse_input!(inputs[6], i32);
        let neigh_5 = parse_input!(inputs[7], i32);

        soil_richness[index] = match richness {
            0 => SoilRichness::UNUSABLE,
            1 => SoilRichness::LOW_QUALITY,
            2 => SoilRichness::MEDIUM_QUALITY,
            3 => SoilRichness::HIGH_QUALITY,
            _ => panic!(),
        }
    }

    /* State variables that have to be maintained as they are not sent by the game */
    let mut prev_day = -1;
    let mut turn_during_day = 0;

    let cache = game::Cache::new(soil_richness);
    let mut mcts: mcts::MCTS = mcts::new();

    // game loop
    while ctr_rcv.recv().unwrap() == true {
        let mut board: [Option<game::Cell>; 37] = [None; 37];
        let mut players: [game::Player; 2] = [game::Player::default(); 2];

        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let day = parse_input!(input_line, u8); // the game lasts 24 days: 0-23
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let nutrients = parse_input!(input_line, u8); // the base score you gain from the next COMPLETE action
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let sun = parse_input!(inputs[0], u32); // your sun points
        let score = parse_input!(inputs[1], u32); // your current score
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opp_sun = parse_input!(inputs[0], u32); // opponent's sun points
        let opp_score = parse_input!(inputs[1], u32); // opponent's score
        let opp_is_waiting = parse_input!(inputs[2], i32); // whether your opponent is asleep until the next day
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();

        players[0].sun = sun;
        players[0].score = score;

        players[1].sun = opp_sun;
        players[1].score = opp_score;
        players[1].is_asleep = match opp_is_waiting {
            1 => true,
            0 => false,
            _ => panic!(),
        };

        let number_of_trees = parse_input!(input_line, i32); // the current amount of trees
        for i in 0..number_of_trees as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let cell_index = parse_input!(inputs[0], usize); // location of this tree
            let size = parse_input!(inputs[1], u8); // size of this tree: 0-3
            let is_mine = parse_input!(inputs[2], i32); // 1 if this is your tree
            let is_dormant = parse_input!(inputs[3], i32); // 1 if this tree is dormant

            board[cell_index] = Some(game::Cell {
                player: match is_mine {
                    1 => 0,
                    0 => 1,
                    _ => panic!(),
                },
                tree: match size {
                    0 => game::Tree::SEED,
                    1 => game::Tree::SMALL_TREE,
                    2 => game::Tree::MEDIUM_TREE,
                    3 => game::Tree::LARGE_TREE,
                    _ => panic!(),
                },
                is_dormant: match is_dormant {
                    1 => true,
                    0 => false,
                    _ => panic!(),
                },
            });
        }

        for cell in board.iter() {
            if let Some(c) = cell {
                match c.tree {
                    game::Tree::SEED => players[c.player as usize].seed_count += 1,
                    game::Tree::SMALL_TREE => players[c.player as usize].small_tree_count += 1,
                    game::Tree::MEDIUM_TREE => players[c.player as usize].medium_tree_count += 1,
                    game::Tree::LARGE_TREE => players[c.player as usize].large_tree_count += 1,
                }
            }
        }

        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let number_of_possible_actions = parse_input!(input_line, i32); // all legal actions
        let mut possible_actions: Vec<String> = Vec::new();
        for i in 0..number_of_possible_actions as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let possible_action = input_line.trim_matches('\n').to_string(); // try printing something from here to start with
            possible_actions.push(possible_action);
        }

        /* Create State from information above */

        if prev_day < day as i8 {
            turn_during_day = 0;
        } else {
            turn_during_day += 1;
        }
        prev_day = day as i8;

        let state = game::State {
            board: board,
            players: players,
            nutrient: nutrients,
            day: day,
            turn_during_day: turn_during_day,

            active: true,
            active_player: 0,
            winners: None,
        };

        /* Extract best path */
        let best_move = mcts.best_move(&state, 0, &cache);

        /* Extract best move */
        let msg = format!("{}", best_move.to_string());
        msg_snd.send((msg, None));
    }
}
