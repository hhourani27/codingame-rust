use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

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

    #[derive(Clone, Copy)]
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
    type Stock = [i8; 4];

    #[derive(Clone, Copy)]
    pub struct Player {
        pub stock: Stock,
        pub stock_id: usize,
        pub spells: StackVector<(u32, bool), EXISTING_SPELL_COUNT>,
        pub rupees: u8,
        pub brewed_potions_count: u8,
    }

    #[derive(Clone, Copy)]
    pub struct State {
        // Player state
        pub player: Player,

        // Game state
        pub counter_orders: StackVector<u32, 5>,
        pub plus_3_bonus_remaining: u8,
        pub plus_1_bonus_remaining: u8,

        pub tome_spells: StackVector<(u32, u8), 6>,

        pub turn: u8,
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                player: Player {
                    stock: [0, 0, 0, 0],
                    stock_id: 0,
                    spells: StackVector::new(),
                    rupees: 0,
                    brewed_potions_count: 0,
                },
                // Game state
                counter_orders: StackVector::new(),
                plus_3_bonus_remaining: 0,
                plus_1_bonus_remaining: 0,

                tome_spells: StackVector::new(),

                turn: 0,
            }
        }
    }

    pub fn next_states(state: &State, cache: &Cache) -> Vec<(Move, State)> {
        let orders = state.counter_orders.slice();
        let tome_spells = state.tome_spells.slice();
        let player_spells = state.player.spells.slice();
        let player_stock = &state.player.stock;
        let player_stock_id = state.player.stock_id;

        let mut valid_moves: Vec<Move> = Vec::new();

        /* If it's a terminal node, don't return any state */

        if state.turn == 100 || state.player.brewed_potions_count == 6 {
        } else if player_spells.len() < 10 {
            valid_moves.push(Move::LEARN(tome_spells[0].0))
        } else {
            /* (A) Determine Valid moves */

            /* BREW moves */
            // Check which order the player can fulfill and add them as a valid move
            for order_id in orders.iter() {
                if cache.can_fulfill_order(*order_id, player_stock_id) {
                    valid_moves.push(Move::BREW(*order_id));
                }
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

            /* LEARN moves */
            if tome_spells.len() > 0 {
                for t in 0..=std::cmp::min(player_stock[0] as usize, tome_spells.len() - 1) {
                    valid_moves.push(Move::LEARN(tome_spells[t].0));
                }
            }
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
                let order_pos =
                    get_order_position(new_state.counter_orders.slice(), *order_id).unwrap();

                // Update stock
                brew_and_update_stock(&mut new_state.player.stock, &order.recipe);
                new_state.player.stock_id = cache.getStockId(&new_state.player.stock);

                // Update score
                let mut bonus = 0;
                if order_pos == 0 {
                    if new_state.plus_3_bonus_remaining > 0 {
                        bonus = 3;
                        new_state.plus_3_bonus_remaining -= 1;
                    } else if new_state.plus_1_bonus_remaining > 0 {
                        bonus = 1;
                        new_state.plus_1_bonus_remaining -= 1;
                    }
                } else if order_pos == 1 {
                    if new_state.plus_3_bonus_remaining > 0 && new_state.plus_1_bonus_remaining > 0
                    {
                        bonus = 1;
                        new_state.plus_1_bonus_remaining -= 1;
                    }
                }

                new_state.player.brewed_potions_count += 1;
                new_state.player.rupees += order.price + bonus;

                // Update counter
                new_state.counter_orders.remove(order_pos);
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
                let spell_pos =
                    get_tome_spell_position(new_state.tome_spells.slice(), *spell_id).unwrap();

                // add the learnt spell to the player's spell
                new_state.player.spells.add((*spell_id, true));

                // pay the tax if needed
                new_state.player.stock[0] -= spell_pos as i8;
                // and gain any tier-0 ingredient put on the spell
                let tax = new_state.tome_spells.get(spell_pos).1;
                if tax > 0 {
                    let empty_storage = 10
                        - new_state.player.stock[0]
                        - new_state.player.stock[1]
                        - new_state.player.stock[2]
                        - new_state.player.stock[3];
                    new_state.player.stock[0] += std::cmp::min(tax as i8, empty_storage);
                }
                new_state.player.stock_id = cache.getStockId(&new_state.player.stock);

                // remove learnt spell from tome spells
                new_state.tome_spells.remove(spell_pos);
            }
            Move::REST => {
                for spell in new_state.player.spells.slice_mut().iter_mut() {
                    spell.1 = true;
                }
            }
            Move::NONE | Move::WAIT => {}
        }

        new_state.turn += 1;

        new_state
    }

    pub fn eval(state: &State) -> f32 {
        const TIER0_FACTOR: f32 = 1.0;
        const TIER1_FACTOR: f32 = 3.0;
        const TIER2_FACTOR: f32 = 3.0;
        const TIER3_FACTOR: f32 = 3.0;
        const RUPEES_FACTOR: f32 = 20.0;

        RUPEES_FACTOR * state.player.rupees as f32
            + TIER0_FACTOR * state.player.stock[0] as f32
            + TIER1_FACTOR * state.player.stock[1] as f32
            + TIER2_FACTOR * state.player.stock[2] as f32
            + TIER3_FACTOR * state.player.stock[3] as f32
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

mod beam {
    use super::game;
    use std::time::Instant;

    const MAX_NODE_COUNT: usize = 300_000;
    const TIME_LIMIT_MS: u128 = 49;

    #[derive(Clone, Copy)]
    struct Node {
        move_: game::Move,
        state: game::State,

        parent: Option<usize>,
        child_first: Option<usize>,
        child_count: usize,
        depth: usize,
        eval: f32,
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
                eval: 0.0,
            }
        }
    }

    pub struct Beam {
        arr: Vec<Node>,
        len: usize,
    }

    impl Beam {
        pub fn new() -> Self {
            Self {
                arr: vec![Node::default(); MAX_NODE_COUNT],
                len: 0,
            }
        }

        /* Return the best found path. Path is a vector of Move, eval score */
        pub fn best_path(
            &mut self,
            start_state: game::State,
            cache: &game::Cache,
        ) -> Vec<(game::Move, f32)> {
            const BEAM_SIZE: usize = 1000;

            let start = Instant::now();
            self.init(start_state);

            let mut frontier: Vec<usize> = Vec::with_capacity(BEAM_SIZE);
            frontier.push(0);

            let mut max_eval = -f32::INFINITY;
            let mut most_valuable_node_idx = 0;

            'main: while (start.elapsed().as_millis() < TIME_LIMIT_MS) && (frontier.len() > 0) {
                let mut frontier_temp: Vec<(usize, f32)> = Vec::new();

                for node_idx in frontier.iter() {
                    let node = &self.arr[*node_idx];
                    /* Get the next states & moves */
                    let next_states: Vec<(game::Move, game::State)> =
                        game::next_states(&node.state, cache);

                    /* Create children nodes */
                    let children: Vec<Node> = next_states
                        .into_iter()
                        .map(|(move_, state)| Node {
                            move_: move_,
                            state: state,
                            parent: Some(*node_idx),
                            child_first: None,
                            child_count: 0,
                            depth: node.depth + 1,
                            eval: 0.0,
                        })
                        .collect::<Vec<Node>>();

                    /* Check if there's still place for children */
                    if self.len + children.len() > MAX_NODE_COUNT {
                        break 'main;
                    }

                    /* Add the children nodes to the tree */
                    self.set_children(*node_idx, children);

                    /* Evaluate each children node */
                    let parent_node = &self.arr[*node_idx];
                    for child_idx in parent_node.child_first.unwrap()
                        ..parent_node.child_first.unwrap() + parent_node.child_count as usize
                    {
                        let child: &mut Node = &mut self.arr[child_idx];
                        child.eval = Beam::eval(&child);

                        /* Determine if it's the most valuable node so far */
                        if child.eval > max_eval {
                            max_eval = child.eval;
                            most_valuable_node_idx = child_idx;
                        }

                        /* And add it to the temp frontier */
                        frontier_temp.push((child_idx, child.eval));
                    }
                }

                /* Select the top BEAM_SIZE frontier nodes and add them to the frontier */
                frontier_temp.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                frontier.clear();

                for i in 0..std::cmp::min(frontier_temp.len(), BEAM_SIZE) {
                    frontier.push(frontier_temp[i].0);
                }
            }

            if frontier.len() > 0 {
                eprintln!(
                    "[BEAM P5] End. Sending best path after expanding {} nodes in {:?}",
                    self.len,
                    start.elapsed()
                );
            } else {
                eprintln!(
                    "[BEAM P5] End. Sending best path after expanding ALL {} nodes in {:?}",
                    self.len,
                    start.elapsed()
                );
            }

            /* When search is finished, determine the most valuable node, and extract its moves */
            let mut best_path: Vec<(game::Move, f32)> = Vec::new();
            let mut n = most_valuable_node_idx;
            while self.arr[n].parent.is_some() {
                best_path.push((self.arr[n].move_, self.arr[n].eval));
                n = self.arr[n].parent.unwrap();
            }

            best_path.reverse();
            best_path
        }

        fn init(&mut self, start_state: game::State) {
            self.arr[0] = Node {
                move_: game::Move::default(),
                state: start_state,
                parent: None,
                child_first: None,
                child_count: 0,
                depth: 0,
                eval: 0.0,
            };

            self.len = 1;
        }

        fn set_children(&mut self, parent: usize, children: Vec<Node>) {
            self.arr[parent].child_first = Some(self.len);
            self.arr[parent].child_count = children.len();

            for child in children.into_iter() {
                self.arr[self.len] = child;
                self.len += 1;
            }
        }

        fn eval(node: &Node) -> f32 {
            const PATH_LEN_FACTOR: f32 = 0.99;

            game::eval(&node.state) * PATH_LEN_FACTOR.powi(node.depth as i32)
        }
    }
}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(
    ctr_rcv: Receiver<bool>,
    msg_rcv: Receiver<String>,
    msg_snd: Sender<(String, Option<std::collections::HashMap<String, String>>)>,
    params: Option<Vec<String>>,
) {
    /* State variables that have to be maintained as they are not sent by the game */
    let mut turn: u8 = 0;
    let mut player_rupees: [u32; 2] = [0, 0];
    let mut player_brewed_potion_count: [u8; 2] = [0, 0];

    let cache = game::Cache::new();
    let mut beam: beam::Beam = beam::Beam::new();

    // game loop
    while ctr_rcv.recv().unwrap() == true {
        /* Prepare mappings between internal spell & order ids & codingame ids */
        #[allow(non_snake_case)]
        let mut map_order_internalId_cgId: HashMap<u32, u32> = HashMap::new();
        #[allow(non_snake_case)]
        let mut map_learn_spell_internalId_cgId: HashMap<u32, u32> = HashMap::new();
        #[allow(non_snake_case)]
        let mut map_cast_spell_internalId_cgId: HashMap<u32, u32> = HashMap::new();

        /* Prepare inputs */
        let mut players: [game::Player; 2] = [game::Player {
            stock: [0, 0, 0, 0],
            stock_id: cache.getStockId(&[0, 0, 0, 0]),
            spells: StackVector::new(),
            rupees: 0,
            brewed_potions_count: 0,
        }; 2];

        let mut counter_orders: StackVector<u32, 5> = StackVector::new();
        let mut plus_3_bonus_remaining: u8 = 0;
        let mut plus_1_bonus_remaining: u8 = 0;

        let mut tome_spells: StackVector<(u32, u8), 6> = StackVector::new();

        /* Read inputs */
        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play
        for i in 0..action_count as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let action_id = parse_input!(inputs[0], u32); // the unique ID of this spell or recipe
            let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW
            let delta_0 = parse_input!(inputs[2], i8); // tier-0 ingredient change
            let delta_1 = parse_input!(inputs[3], i8); // tier-1 ingredient change
            let delta_2 = parse_input!(inputs[4], i8); // tier-2 ingredient change
            let delta_3 = parse_input!(inputs[5], i8); // tier-3 ingredient change
            let price = parse_input!(inputs[6], u8); // the price in rupees if this is a potion
            let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax; For brews, this is the value of the current urgency bonus
            let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell; For brews, this is how many times you can still gain an urgency bonus
            let castable = parse_input!(inputs[9], i32); // in the first league: always 0; later: 1 if this is a castable player spell
            let repeatable = parse_input!(inputs[10], i32); // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

            /* READ INPUT AND UPDATE STATE */
            if action_type == String::from("BREW") {
                let order = game::find_order(&[delta_0, delta_1, delta_2, delta_3]).unwrap();

                match tome_index {
                    3 => {
                        plus_3_bonus_remaining = tax_count as u8;
                    }
                    1 => {
                        plus_1_bonus_remaining = tax_count as u8;
                    }
                    _ => {}
                }

                map_order_internalId_cgId.insert(order.id, action_id);
                counter_orders.add(order.id);
            } else if action_type == String::from("CAST")
                || action_type == String::from("OPPONENT_CAST")
            {
                let p = match action_type.as_str() {
                    "CAST" => 0,
                    "OPPONENT_CAST" => 1,
                    _ => panic!(),
                };

                let spell = game::find_spell(&[delta_0, delta_1, delta_2, delta_3]).unwrap();
                if p == 0 {
                    map_cast_spell_internalId_cgId.insert(spell.id as u32, action_id);
                }
                let mut active = true;
                if castable == 0 {
                    active = false;
                }

                players[p as usize].spells.add((spell.id, active));
            } else if action_type == String::from("LEARN") {
                let spell = game::find_spell(&[delta_0, delta_1, delta_2, delta_3]).unwrap();
                map_learn_spell_internalId_cgId.insert(spell.id as u32, action_id);

                tome_spells.add((spell.id, tax_count as u8));
            }
        }
        for i in 0..2 as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let inv_0 = parse_input!(inputs[0], i8); // tier-0 ingredients in inventory
            let inv_1 = parse_input!(inputs[1], i8);
            let inv_2 = parse_input!(inputs[2], i8);
            let inv_3 = parse_input!(inputs[3], i8);
            let score = parse_input!(inputs[4], u32); // amount of rupees

            players[i].stock = [inv_0, inv_1, inv_2, inv_3];
            players[i].stock_id = cache.getStockId(&players[i].stock);

            players[i].rupees = score as u8;

            if score > player_rupees[i] {
                player_brewed_potion_count[i] += 1;
            }
            player_rupees[i] = score;

            players[i].brewed_potions_count = player_brewed_potion_count[i];
        }

        /* Initialize states */

        let state = game::State {
            player: players[0].clone(),
            counter_orders,
            plus_3_bonus_remaining,
            plus_1_bonus_remaining,

            tome_spells: tome_spells.clone(),
            turn,
        };

        /* Extract best path */
        let mut best_path = beam.best_path(state, &cache);

        /* Extract best move */
        let best_move = best_path[0].0;

        /* Update turn */
        turn += 1;

        // convert move to codingame id
        let best_move_cg = match best_move {
            game::Move::NONE | game::Move::WAIT | game::Move::REST => best_move.clone(),
            game::Move::BREW(o) => game::Move::BREW(*map_order_internalId_cgId.get(&o).unwrap()),
            game::Move::CAST(s, t) => {
                game::Move::CAST(*map_cast_spell_internalId_cgId.get(&s).unwrap(), t)
            }
            game::Move::LEARN(s) => {
                game::Move::LEARN(*map_learn_spell_internalId_cgId.get(&s).unwrap())
            }
        };

        /* #region [Extract player state] */

        let mut player_state: HashMap<String, String> = HashMap::new();

        player_state.insert(
            "Best path".to_string(),
            best_path
                .iter()
                .map(|(m, e)| format!("{} ({:.1})", m, e))
                .collect::<Vec<String>>()
                .join(" → "),
        );

        /* #endregion */

        let msg = best_move_cg.to_string();
        //msg_snd.send((format!("{}", msg), None));
        msg_snd.send((format!("{}", msg), Some(player_state)));
    }
}
