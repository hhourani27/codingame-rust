use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use rand::prelude::SliceRandom;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

/* #region(collapsed) [General Data Structure] */
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

    pub fn add(&mut self, e: T) {
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
            sv.add(e.clone());
        }
        sv
    }
}

impl<T: Copy + Clone + Default, const MAX_SIZE: usize> IntoIterator for StackVector<T, MAX_SIZE> {
    type Item = T;
    type IntoIter = StackVectorIntoIterator<T, MAX_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        StackVectorIntoIterator {
            stack_vector: self,
            index: 0,
        }
    }
}

pub struct StackVectorIntoIterator<T: Copy + Clone + Default, const MAX_SIZE: usize> {
    stack_vector: StackVector<T, MAX_SIZE>,
    index: usize,
}

impl<T: Copy + Clone + Default, const MAX_SIZE: usize> Iterator
    for StackVectorIntoIterator<T, MAX_SIZE>
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.index > self.stack_vector.len() {
            return None;
        }
        let result = self.stack_vector.arr[self.index];
        self.index += 1;
        Some(result)
    }
}

/* #endregion */

mod game {
    use super::StackVector;
    use std::convert::TryInto;
    use std::fmt::Display;

    const EXISTING_SPELL_COUNT: usize = 42 + 4;
    const EXISTING_ORDER_COUNT: usize = 36;

    pub const MAX_NEXT_MOVES: usize = 35 + 5 + 6 + 1;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Move {
        NONE,
        WAIT,
        REST,
        BREW(u32),
        CAST(u32, u8),
        LEARN(u32),
    }

    impl Display for Move {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Move::NONE => write!(f, "NONE"),
                Move::WAIT => write!(f, "WAIT"),
                Move::REST => write!(f, "REST"),
                Move::BREW(i) => write!(f, "BREW {}", i),
                Move::CAST(i, 1) => write!(f, "CAST {}", i),
                Move::CAST(i, t) => write!(f, "CAST {} {}", i, t),
                Move::LEARN(i) => write!(f, "LEARN {}", i),
            }
        }
    }

    impl Default for Move {
        fn default() -> Self {
            Move::NONE
        }
    }

    type Recipe = [i8; 4];
    pub type Stock = [i8; 4];

    #[derive(Clone, Copy)]
    pub struct Player {
        pub stock: Stock,
        pub stock_id: usize,
        pub spells: StackVector<(u32, bool), EXISTING_SPELL_COUNT>,
    }

    #[derive(Clone, Copy)]
    pub struct State {
        // Player state
        pub player: Player,

        // Game state
        pub counter_order: u32,
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                player: Player {
                    stock: [0, 0, 0, 0],
                    stock_id: 0,
                    spells: StackVector::new(),
                },
                // Game state
                counter_order: 0,
            }
        }
    }

    pub fn next_states(state: &State, cache: &Cache) -> Vec<(Move, State)> {
        let player_spells = state.player.spells.slice();
        let player_stock_id = state.player.stock_id;

        let mut valid_moves: Vec<Move> = Vec::new();

        /* If it's a terminal node, don't return any state */

        /* (A) Determine Valid moves */

        /* BREW moves */
        // Check which order the player can fulfill and add them as a valid move
        if cache.can_fulfill_order(state.counter_order, player_stock_id) {
            valid_moves.push(Move::BREW(state.counter_order));
        }

        /* CAST moves */
        // Check which spell the player can cast and add them as a valid move
        // in the meantime check if there are inactive spells
        let mut all_spells_are_active = true;
        for (spell_id, active) in player_spells.iter() {
            if *active == false {
                all_spells_are_active = false;
            } else {
                let times_can_cast_spell =
                    cache.how_many_times_can_cast_spell(*spell_id, player_stock_id);
                if times_can_cast_spell > 0 {
                    for n in 1..=times_can_cast_spell {
                        valid_moves.push(Move::CAST(*spell_id, n));
                    }
                }
            }
        }

        /* REST move */
        if all_spells_are_active == false {
            valid_moves.push(Move::REST);
        }

        /* (B) Generate state for each move */
        valid_moves
            .into_iter()
            .map(|m| (m, update(state, &m, cache)))
            .collect::<Vec<(Move, State)>>()
    }

    fn update(state: &State, move_: &Move, cache: &Cache) -> State {
        let mut new_state = state.clone();
        match move_ {
            Move::BREW(order_id) => {
                let order = cache.getOrder(*order_id);

                // Update stock
                brew_and_update_stock(&mut new_state.player.stock, &order.recipe);
                new_state.player.stock_id = cache.getStockId(&new_state.player.stock);
            }
            Move::CAST(spell_id, times) => {
                let spell = cache.getSpell(*spell_id);
                let spell_pos =
                    get_player_spell_position(new_state.player.spells.slice(), *spell_id).unwrap();

                // Update stock
                cast_and_update_stock(&mut new_state.player.stock, &spell.recipe, *times);
                new_state.player.stock_id = cache.getStockId(&new_state.player.stock);

                // Update player spells
                new_state.player.spells.get_mut(spell_pos).1 = false;
            }
            Move::LEARN(spell_id) => {
                panic!("LEARN IS NOT ALLOWED");
            }
            Move::REST => {
                for spell in new_state.player.spells.slice_mut().iter_mut() {
                    spell.1 = true;
                }
            }
            Move::NONE | Move::WAIT => {}
        }

        new_state
    }

    /* #region(collapsed) [Cache] */

    #[allow(non_snake_case)]
    pub struct Cache {
        map_stockArr4_stockId: Vec<Vec<Vec<Vec<usize>>>>,
        map_stockId_timesCanCastSpell: [[u8; EXISTING_SPELL_COUNT]; 1001],
        map_stockId_canFullfillOrder: [[bool; EXISTING_ORDER_COUNT]; 1001],

        map_orderId_Order: [Order; EXISTING_ORDER_COUNT],
        map_spellId_Spell: [Spell; EXISTING_SPELL_COUNT],
    }

    #[allow(non_snake_case)]
    impl Cache {
        pub fn new() -> Self {
            let (
                map_stockArr4_stockId,
                map_stockId_timesCanCastSpell,
                map_stockId_canFullfillOrder,
            ) = Cache::init_map_stocks();

            let map_orderId_Order = get_all_orders().try_into().unwrap();
            let map_spellId_Spell = get_all_tome_spells().try_into().unwrap();

            Cache {
                map_stockArr4_stockId,
                map_stockId_timesCanCastSpell,
                map_stockId_canFullfillOrder,

                map_orderId_Order,
                map_spellId_Spell,
            }
        }

        fn init_map_stocks() -> (
            Vec<Vec<Vec<Vec<usize>>>>,
            [[u8; EXISTING_SPELL_COUNT]; 1001],
            [[bool; EXISTING_ORDER_COUNT]; 1001],
        ) {
            let mut map_stockArr4_stockId: Vec<Vec<Vec<Vec<usize>>>> =
                vec![vec![vec![vec![0; 11]; 11]; 11]; 11];

            let mut map_stockId_timesCanCastSpell: [[u8; EXISTING_SPELL_COUNT]; 1001] =
                [[0; EXISTING_SPELL_COUNT]; 1001];

            let mut map_stockId_canFullfillOrder: [[bool; EXISTING_ORDER_COUNT]; 1001] =
                [[false; EXISTING_ORDER_COUNT]; 1001];

            let all_spells = get_all_tome_spells();
            let all_orders = get_all_orders();

            let mut id = 0;
            for t0 in 0..=10 {
                for t1 in 0..=(10 - t0) {
                    for t2 in 0..=(10 - t0 - t1) {
                        for t3 in 0..=(10 - t0 - t1 - t2) {
                            map_stockArr4_stockId[t0][t1][t2][t3] = id;

                            for spell in all_spells.iter() {
                                map_stockId_timesCanCastSpell[id][spell.id as usize] =
                                    how_many_times_can_cast_spell(
                                        &spell,
                                        &[t0 as i8, t1 as i8, t2 as i8, t3 as i8],
                                    );
                            }

                            for order in all_orders.iter() {
                                map_stockId_canFullfillOrder[id][order.id as usize] =
                                    can_fulfill_order(
                                        &order,
                                        &[t0 as i8, t1 as i8, t2 as i8, t3 as i8],
                                    );
                            }

                            id += 1;
                        }
                    }
                }
            }

            (
                map_stockArr4_stockId,
                map_stockId_timesCanCastSpell,
                map_stockId_canFullfillOrder,
            )
        }

        pub fn getStockId(&self, stockArr: &Stock) -> usize {
            self.map_stockArr4_stockId[stockArr[0] as usize][stockArr[1] as usize]
                [stockArr[2] as usize][stockArr[3] as usize]
        }

        pub fn how_many_times_can_cast_spell(&self, spell_id: u32, stock_id: usize) -> u8 {
            self.map_stockId_timesCanCastSpell[stock_id][spell_id as usize]
        }

        pub fn can_fulfill_order(&self, order_id: u32, stock_id: usize) -> bool {
            self.map_stockId_canFullfillOrder[stock_id][order_id as usize]
        }

        pub fn getOrder(&self, order_id: u32) -> Order {
            self.map_orderId_Order[order_id as usize].clone()
        }

        pub fn getSpell(&self, spell_id: u32) -> Spell {
            self.map_spellId_Spell[spell_id as usize].clone()
        }
    }

    /* #region(collapsed) [Private functions] */

    #[derive(Copy, Clone, Default, Debug)]
    pub struct Spell {
        pub id: u32,
        pub recipe: Recipe,
        pub delta_stock: i8,
        pub repeatable: bool,
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct Order {
        pub id: u32,
        pub recipe: Recipe,
        pub price: u8,
    }

    fn get_all_tome_spells() -> Vec<Spell> {
        let mut tome_spells = get_learnable_tome_spells();
        tome_spells.extend_from_slice(&get_basic_spells());

        tome_spells
    }

    fn get_learnable_tome_spells() -> Vec<Spell> {
        let spells: Vec<Recipe> = vec![
            [-3, 0, 0, 1],
            [3, -1, 0, 0],
            [1, 1, 0, 0],
            [0, 0, 1, 0],
            [3, 0, 0, 0],
            [2, 3, -2, 0],
            [2, 1, -2, 1],
            [3, 0, 1, -1],
            [3, -2, 1, 0],
            [2, -3, 2, 0],
            [2, 2, 0, -1],
            [-4, 0, 2, 0],
            [2, 1, 0, 0],
            [4, 0, 0, 0],
            [0, 0, 0, 1],
            [0, 2, 0, 0],
            [1, 0, 1, 0],
            [-2, 0, 1, 0],
            [-1, -1, 0, 1],
            [0, 2, -1, 0],
            [2, -2, 0, 1],
            [-3, 1, 1, 0],
            [0, 2, -2, 1],
            [1, -3, 1, 1],
            [0, 3, 0, -1],
            [0, -3, 0, 2],
            [1, 1, 1, -1],
            [1, 2, -1, 0],
            [4, 1, -1, 0],
            [-5, 0, 0, 2],
            [-4, 0, 1, 1],
            [0, 3, 2, -2],
            [1, 1, 3, -2],
            [-5, 0, 3, 0],
            [-2, 0, -1, 2],
            [0, 0, -3, 3],
            [0, -3, 3, 0],
            [-3, 3, 0, 0],
            [-2, 2, 0, 0],
            [0, 0, -2, 2],
            [0, -2, 2, 0],
            [0, 0, 2, -1],
        ];

        spells
            .iter()
            .enumerate()
            .map(|(i, s)| Spell {
                id: i as u32,
                recipe: s.clone(),
                delta_stock: s[0] + s[1] + s[2] + s[3],
                repeatable: s[0] < 0 || s[1] < 0 || s[2] < 0 || s[3] < 0,
            })
            .collect::<Vec<Spell>>()
    }

    fn get_basic_spells() -> Vec<Spell> {
        [
            Spell {
                id: 42,
                recipe: [2, 0, 0, 0],
                delta_stock: 2,
                repeatable: false,
            },
            Spell {
                id: 43,
                recipe: [-1, 1, 0, 0],
                delta_stock: 0,
                repeatable: false,
            },
            Spell {
                id: 44,
                recipe: [0, -1, 1, 0],
                delta_stock: 0,
                repeatable: false,
            },
            Spell {
                id: 45,
                recipe: [0, 0, -1, 1],
                delta_stock: 0,
                repeatable: false,
            },
        ]
        .to_vec()
    }

    fn get_all_orders() -> Vec<Order> {
        let orders: Vec<(Recipe, u8)> = vec![
            ([2, 2, 0, 0], 6),
            ([3, 2, 0, 0], 7),
            ([0, 4, 0, 0], 8),
            ([2, 0, 2, 0], 8),
            ([2, 3, 0, 0], 8),
            ([3, 0, 2, 0], 9),
            ([0, 2, 2, 0], 10),
            ([0, 5, 0, 0], 10),
            ([2, 0, 0, 2], 10),
            ([2, 0, 3, 0], 11),
            ([3, 0, 0, 2], 11),
            ([0, 0, 4, 0], 12),
            ([0, 2, 0, 2], 12),
            ([0, 3, 2, 0], 12),
            ([0, 2, 3, 0], 13),
            ([0, 0, 2, 2], 14),
            ([0, 3, 0, 2], 14),
            ([2, 0, 0, 3], 14),
            ([0, 0, 5, 0], 15),
            ([0, 0, 0, 4], 16),
            ([0, 2, 0, 3], 16),
            ([0, 0, 3, 2], 17),
            ([0, 0, 2, 3], 18),
            ([0, 0, 0, 5], 20),
            ([2, 1, 0, 1], 9),
            ([0, 2, 1, 1], 12),
            ([1, 0, 2, 1], 12),
            ([2, 2, 2, 0], 13),
            ([2, 2, 0, 2], 15),
            ([2, 0, 2, 2], 17),
            ([0, 2, 2, 2], 19),
            ([1, 1, 1, 1], 12),
            ([3, 1, 1, 1], 14),
            ([1, 3, 1, 1], 16),
            ([1, 1, 3, 1], 18),
            ([1, 1, 1, 3], 20),
        ];

        orders
            .iter()
            .enumerate()
            .map(|(i, o)| Order {
                id: i as u32,
                recipe: [-o.0[0], -o.0[1], -o.0[2], -o.0[3]],
                price: o.1,
            })
            .collect::<Vec<Order>>()
    }

    /* Return how many times the spell can be cast */
    fn how_many_times_can_cast_spell(spell: &Spell, stock: &Stock) -> u8 {
        if spell.repeatable == false {
            match can_cast_spell(spell, stock) {
                true => 1,
                false => 0,
            }
        } else {
            let mut times = 0;
            let mut stock = stock.clone();

            while can_cast_spell(spell, &stock) {
                times += 1;
                cast_and_update_stock(&mut stock, &spell.recipe, 1);
            }

            times
        }
    }

    fn can_cast_spell(spell: &Spell, stock: &Stock) -> bool {
        let empty_slots = 10 - stock[0] - stock[1] - stock[2] - stock[3];

        if spell.delta_stock > empty_slots {
            return false;
        }
        for i in 0..4 {
            if spell.recipe[i] < 0 && stock[i] < -spell.recipe[i] {
                return false;
            }
        }
        true
    }

    fn can_fulfill_order(order: &Order, stock: &Stock) -> bool {
        stock[0] >= -order.recipe[0]
            && stock[1] >= -order.recipe[1]
            && stock[2] >= -order.recipe[2]
            && stock[3] >= -order.recipe[3]
    }

    fn cast_and_update_stock(stock: &mut Stock, recipe: &Recipe, times: u8) {
        for _ in 0..times {
            for i in 0..4 {
                stock[i] += recipe[i];
            }
        }
    }

    fn brew_and_update_stock(stock: &mut Stock, recipe: &Recipe) {
        for i in 0..4 {
            stock[i] += recipe[i];
        }
    }

    fn get_order_position(ids: &[u32], id: u32) -> Option<usize> {
        for i in 0..ids.len() {
            if ids[i] == id {
                return Some(i);
            }
        }
        None
    }

    fn get_player_spell_position(ids: &[(u32, bool)], id: u32) -> Option<usize> {
        for i in 0..ids.len() {
            if ids[i].0 == id {
                return Some(i);
            }
        }
        None
    }

    pub fn get_tome_spell_position(ids: &[(u32, u8)], id: u32) -> Option<usize> {
        for i in 0..ids.len() {
            if ids[i].0 == id {
                return Some(i);
            }
        }
        None
    }

    pub fn find_order(recipe: &Recipe) -> Option<Order> {
        for order in get_all_orders().iter() {
            if order.recipe == *recipe {
                return Some(order.clone());
            }
        }

        None
    }

    pub fn find_spell(recipe: &Recipe) -> Option<Spell> {
        for spell in get_all_tome_spells().iter() {
            if spell.recipe == *recipe {
                return Some(spell.clone());
            }
        }

        None
    }

    /* #endregion */
}

mod search {
    use super::game;
    use std::{
        collections::{HashSet, VecDeque},
        hash::Hash,
        time::Instant,
    };

    #[derive(Clone, Copy)]
    struct Node {
        move_: game::Move,
        state: game::State,

        parent: Option<usize>,
        child_first: Option<usize>,
        child_count: usize,
        depth: usize,
    }

    impl Default for Node {
        fn default() -> Self {
            Self {
                move_: game::Move::default(),
                state: game::State::default(),
                parent: None,
                child_first: None,
                child_count: 0,
                depth: 0,
            }
        }
    }

    pub struct Search {
        arr: Vec<Node>,
    }

    impl Search {
        pub fn new() -> Self {
            Self {
                arr: Vec::with_capacity(1_000_000),
            }
        }

        /* Return the best found path. Path is a vector of Move, eval score */
        pub fn best_path_bfs(
            &mut self,
            start_state: game::State,
            target_order_id: u32,
            cache: &game::Cache,
        ) -> Vec<(game::Move, game::Stock)> {
            let start = Instant::now();
            self.init(start_state);

            let mut frontier: VecDeque<usize> = VecDeque::new();
            frontier.push_back(0);

            let mut visited: HashSet<usize> = HashSet::new();

            while !frontier.is_empty() {
                let node_idx = frontier.pop_front().unwrap();
                let node = &self.arr[node_idx];

                if node.move_ == game::Move::BREW(target_order_id) {
                    let mut best_path: Vec<(game::Move, game::Stock)> = Vec::new();
                    let mut n = node_idx;
                    while self.arr[n].parent.is_some() {
                        best_path.push((self.arr[n].move_, self.arr[n].state.player.stock));
                        n = self.arr[n].parent.unwrap();
                    }

                    best_path.reverse();

                    /*
                    eprintln!(
                        "[BEAM P5] BFS search ended. Expanded {} nodes in {:?}",
                        self.arr.len(),
                        start.elapsed()
                    );*/

                    return best_path;
                } else if !visited.contains(&node.state.player.stock_id) {
                    visited.insert(node.state.player.stock_id);

                    /* Get the next states & moves */
                    let next_states: Vec<(game::Move, game::State)> =
                        game::next_states(&node.state, cache);

                    /* Create children nodes */
                    let children: Vec<Node> = next_states
                        .into_iter()
                        .map(|(move_, state)| Node {
                            move_: move_,
                            state: state,
                            parent: Some(node_idx),
                            child_first: None,
                            child_count: 0,
                            depth: node.depth + 1,
                        })
                        .collect::<Vec<Node>>();

                    /* Add the children nodes to the tree */
                    self.set_children(node_idx, children);

                    let parent_node = &self.arr[node_idx];
                    for child_idx in parent_node.child_first.unwrap()
                        ..parent_node.child_first.unwrap() + parent_node.child_count as usize
                    {
                        let child: &Node = &self.arr[child_idx];

                        frontier.push_back(child_idx);
                    }
                }
            }

            return Vec::new();
        }
        fn init(&mut self, start_state: game::State) {
            self.arr.clear();

            self.arr.push(Node {
                move_: game::Move::default(),
                state: start_state,
                parent: None,
                child_first: None,
                child_count: 0,
                depth: 0,
            });
        }

        fn set_children(&mut self, parent: usize, children: Vec<Node>) {
            self.arr[parent].child_first = Some(self.arr.len());
            self.arr[parent].child_count = children.len();

            for child in children.into_iter() {
                self.arr.push(child);
            }
        }
    }
}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn run() {
    let cache = game::Cache::new();
    let mut search: search::Search = search::Search::new();

    let mut distances: [[i32; 36]; 1001] = [[50; 36]; 1001];

    let spells: Vec<u32> = (0..42).collect();
    let mut rng = &mut rand::thread_rng();
    const SPELL_ITERATIONS: usize = 2000;

    let mut stocks: Vec<game::Stock> = Vec::new();
    for t0 in 0..=10 {
        for t1 in 0..=(10 - t0) {
            for t2 in 0..=(10 - t0 - t1) {
                for t3 in 0..=(10 - t0 - t1 - t2) {
                    stocks.push([t0, t1, t2, t3]);
                }
            }
        }
    }

    //let stocks: Vec<game::Stock> = vec![[0, 0, 0, 7]];

    for start_stock in stocks.into_iter() {
        let start_stock_id = cache.getStockId(&start_stock);

        println!("Analyzing {:?}", start_stock);

        for target_order in 0..36 {
            let mut distances_iteration: [i32; SPELL_ITERATIONS] = [50; SPELL_ITERATIONS];
            for i in 0..SPELL_ITERATIONS {
                let tome_spells: Vec<u32> = spells.choose_multiple(rng, 6).cloned().collect();

                let player = game::Player {
                    stock: start_stock,
                    stock_id: start_stock_id,
                    spells: StackVector::from(&[
                        (42, true),
                        (43, true),
                        (44, true),
                        (45, true),
                        (tome_spells[0], true),
                        (tome_spells[1], true),
                        (tome_spells[2], true),
                        (tome_spells[3], true),
                        (tome_spells[4], true),
                        (tome_spells[5], true),
                    ]),
                };

                let state = game::State {
                    player,
                    counter_order: target_order,
                };

                let shortest_path = search.best_path_bfs(state, target_order, &cache);

                if !shortest_path.is_empty() {
                    distances_iteration[i] = shortest_path.len() as i32;
                }

                /*if shortest_path.is_empty() {
                    println!(
                        "Couldn't find path from {:?} to BREW {}",
                        &start_stock, target_order
                    );
                } else {
                    println!(
                        "\t{:?} → {}",
                        &start_stock,
                        shortest_path
                            .iter()
                            .map(|(m, s)| format!("{} ({:?})", m, s))
                            .collect::<Vec<String>>()
                            .join(" → ")
                            .to_string()
                    )
                }*/
            }
            distances[start_stock_id][target_order as usize] = distances_iteration
                .iter()
                .sum::<i32>()
                .div_euclid(SPELL_ITERATIONS as i32);
        }
    }

    std::fs::write(
        "C:/Users/hhour/Desktop/codingame-rust/fall_2020_witches_brew/distance_stock_order.txt",
        format!(
            "{:?}",
            distances
                .iter()
                .map(|d_s| d_s
                    .iter()
                    .map(|d_s_o| match d_s_o {
                        1..=5 => format!("{}", *d_s_o),
                        6..=10 => "6".to_string(),
                        11..=15 => "7".to_string(),
                        16..=25 => "8".to_string(),
                        26.. => "9".to_string(),
                        _ => "0".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join("")
                    .to_string())
                .collect::<Vec<String>>()
        ),
    );
}
