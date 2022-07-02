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
            arr: [Default::default(); MAX_SIZE],
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

/* #endregion */

mod game {

    pub type GameScore = [f32; 4];

    #[derive(Clone)]
    pub enum WinLossTie {
        Win,
        Loss,
        Tie,
    }

    use super::StackVector;
    use rand::seq::SliceRandom;
    use std::cmp;

    pub const MAX_VALID_MOVES: usize = 35 + 5 + 6 + 1; // 35 CAST + 5 BREW + 6 LEARN + 1 REST
    const EXISTING_SPELL_COUNT: usize = 42 + 4;
    const EXISTING_ORDER_COUNT: usize = 36;

    /* #region [Structs] */
    #[derive(Copy, Clone, PartialEq, Debug)]
    pub enum Move {
        NONE,
        WAIT,
        REST,
        BREW(u32),
        CAST(u32, u8),
        LEARN(u32),
    }

    impl Move {
        pub fn to_string(&self) -> String {
            match self {
                Move::NONE => format!("None"),
                Move::WAIT => format!("WAIT"),
                Move::REST => format!("REST"),
                Move::BREW(i) => format!("BREW {}", i),
                Move::CAST(i, 1) => format!("CAST {}", i),
                Move::CAST(i, t) => format!("CAST {} {}", i, t),
                Move::LEARN(i) => format!("LEARN {}", i),
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

    #[derive(Copy, Clone, Default)]
    pub struct Order {
        pub id: u32,
        pub recipe: Recipe,
        pub price: u8,
        pub bonus: u8,
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct Spell {
        pub id: u32,
        pub recipe: Recipe,
        pub delta_stock: i8,
        pub tax: u8,
        pub repeatable: bool,
        pub active: bool,
    }

    #[derive(Clone, Copy)]
    pub struct Player {
        pub move_: Move,
        pub stock: Stock,
        pub stock_id: usize,
        pub spells: StackVector<Spell, EXISTING_SPELL_COUNT>,
        pub rupees: u32,
        pub brewed_potions_count: u8,
    }

    #[derive(Clone)]
    pub struct State {
        // States per player
        players: [Player; 2],

        // Global states
        queued_orders: Vec<Order>,
        counter_orders: StackVector<Order, 5>,
        plus_3_bonus_remaining: u8,
        plus_1_bonus_remaining: u8,

        queued_spells: Vec<Spell>,
        tome_spells: StackVector<Spell, 6>,

        active: bool,
        active_player: u8,
        turn: u8,
        winners: Option<(WinLossTie, WinLossTie)>,
    }

    /* #endregion */

    /* #region [Public functions] */

    pub fn new(
        players: [Player; 2],
        tome_spells: StackVector<Spell, 6>,
        counter_orders: StackVector<Order, 5>,
        plus_3_bonus_remaining: u8,
        plus_1_bonus_remaining: u8,
        turn: u8,
    ) -> State {
        State {
            players,
            queued_orders: Vec::new(),
            counter_orders,
            plus_3_bonus_remaining,
            plus_1_bonus_remaining,
            queued_spells: Vec::new(),
            tome_spells,
            active: true,
            active_player: 0,
            turn,
            winners: None,
        }
    }

    pub fn update_state(state: &mut State, player: u8, _move: Move, cache: &Cache) {
        /* (2) Record the move */
        state.players[state.active_player as usize].move_ = _move;

        /* (3) If it's player'1 turn, i.e. both players have played =>  update the state */
        if state.active_player == 1 {
            /* 3.2 Update the state */
            // For each player move
            let mut orders_were_fullfilled = false;
            let mut orders_to_remove_pos: [Option<usize>; 2] = [None, None];
            let mut spells_were_learnt = false;
            let mut spells_to_remove_pos: [Option<usize>; 2] = [None, None];
            let mut spell_tax_payed: [Option<usize>; 2] = [None, None];

            for (pid, player) in state.players.iter_mut().enumerate() {
                match player.move_ {
                    Move::BREW(order_id) => {
                        let fullfilled_order_pos =
                            get_order_position(&state.counter_orders.slice(), order_id).unwrap();

                        let fullfilled_order = state.counter_orders.get(fullfilled_order_pos);

                        // Update the player's potion count
                        player.brewed_potions_count += 1;

                        // Update the player's rupees
                        player.rupees +=
                            fullfilled_order.price as u32 + fullfilled_order.bonus as u32;

                        // Update the player's ingredient stock
                        brew_and_update_stock(&mut player.stock, &fullfilled_order.recipe);
                        player.stock_id = cache.getStockId(&player.stock);

                        // Save fullfilled orders so that I remove them later
                        orders_were_fullfilled = true;
                        orders_to_remove_pos[pid] = Some(fullfilled_order_pos);
                    }
                    Move::CAST(spell_id, times) => {
                        let cast_spell_idx =
                            get_spell_position(&player.spells.slice(), spell_id).unwrap();

                        let cast_spell = player.spells.get_mut(cast_spell_idx);

                        // Update the player's ingredient stock
                        cast_and_update_stock(&mut player.stock, &cast_spell.recipe, times);
                        player.stock_id = cache.getStockId(&player.stock);

                        // Spell is now exhausted
                        cast_spell.active = false;
                    }
                    Move::LEARN(spell_id) => {
                        let learnt_spell_pos =
                            get_spell_position(&state.tome_spells.slice(), spell_id).unwrap();

                        let learnt_spell = state.tome_spells.get(learnt_spell_pos);

                        // add the learnt spell to the player's spell
                        let mut player_learnt_spell = learnt_spell.clone();
                        player_learnt_spell.tax = 0;
                        player.spells.add(player_learnt_spell);

                        // pay the tax if needed
                        player.stock[0] -= learnt_spell_pos as i8;
                        // and gain any tier-0 ingredient put on the spell
                        if learnt_spell.tax > 0 {
                            let empty_storage = 10
                                - player.stock[0]
                                - player.stock[1]
                                - player.stock[2]
                                - player.stock[3];
                            player.stock[0] += cmp::min(learnt_spell.tax as i8, empty_storage);
                        }
                        player.stock_id = cache.getStockId(&player.stock);

                        // Save learnt spells, so that I replace them later and deal with the tax
                        spells_were_learnt = true;
                        spells_to_remove_pos[pid] = Some(learnt_spell_pos);
                        spell_tax_payed[pid] = Some(learnt_spell_pos);
                    }
                    Move::REST => {
                        for spell in player.spells.slice_mut().iter_mut() {
                            spell.active = true;
                        }
                    }
                    Move::NONE | Move::WAIT => {}
                }
            }

            /* Remove fullfilled orders and create new one in their place, and update bonus */
            if orders_were_fullfilled == true {
                update_counter_orders(
                    &mut state.counter_orders,
                    &mut state.queued_orders,
                    &mut state.plus_3_bonus_remaining,
                    &mut state.plus_1_bonus_remaining,
                    &orders_to_remove_pos,
                )
            }

            /* Remove learnt spells and create new one in their place, and update tax */
            if spells_were_learnt == true {
                update_tome_spells(
                    &mut state.tome_spells,
                    &mut state.queued_spells,
                    &spells_to_remove_pos,
                    &spell_tax_payed,
                );
            }

            /* 3.3 Check terminal condition */
            let player0: &Player = &state.players[0];
            let player1: &Player = &state.players[1];

            if (player0.rupees as i32 - player1.rupees as i32).abs() >= 20
                || player0.brewed_potions_count == 6
                || player1.brewed_potions_count == 6
                || state.turn == 100
                || (state.players[0].move_ == Move::WAIT && state.players[1].move_ == Move::WAIT)
            {
                state.active = false;

                let score0 = player0.rupees
                    + (player0.stock[1] + player0.stock[2] + player0.stock[3]) as u32;
                let score1 = player1.rupees
                    + (player1.stock[1] + player1.stock[2] + player1.stock[3]) as u32;

                if score0 > score1 {
                    state.winners = Some((WinLossTie::Win, WinLossTie::Loss));
                } else if score0 < score1 {
                    state.winners = Some((WinLossTie::Loss, WinLossTie::Win));
                } else {
                    state.winners = Some((WinLossTie::Tie, WinLossTie::Tie));
                }
            }

            /* 3.3 Reinit moves */
            state.players[0].move_ = Move::NONE;
            state.players[1].move_ = Move::NONE;
            state.turn += 1;
        }
        state.active_player = (state.active_player + 1) % 2;
    }

    pub fn valid_moves(state: &State, cache: &Cache) -> (u8, StackVector<Move, MAX_VALID_MOVES>) {
        let orders = state.counter_orders.slice();
        let tome_spells = state.tome_spells.slice();
        let player = &state.players[state.active_player as usize];
        let player_spells = player.spells.slice();
        let player_stock = &player.stock;
        let player_stock_id = player.stock_id;

        let mut valid_moves: StackVector<Move, MAX_VALID_MOVES> = StackVector::new();

        /* BREW moves */
        // Check which order the player can fulfill and add them as a valid move
        for order in orders.iter() {
            if cache.can_fulfill_order(order.id, player_stock_id) {
                valid_moves.add(Move::BREW(order.id));
            }
        }

        /* CAST moves */
        // Check which spell the player can cast and add them as a valid move
        // in the meantime check if there are inactive spells
        let mut all_spells_are_active = true;
        for sp in player_spells.iter() {
            if sp.active == false {
                all_spells_are_active = false;
            } else {
                let times_can_cast_spell =
                    cache.how_many_times_can_cast_spell(sp.id, player_stock_id);
                if times_can_cast_spell > 0 {
                    for n in 1..=times_can_cast_spell {
                        valid_moves.add(Move::CAST(sp.id, n));
                    }
                }
            }
        }

        /* REST move */
        if all_spells_are_active == false {
            valid_moves.add(Move::REST);
        }

        /* LEARN moves */
        if tome_spells.len() > 0 {
            for t in 0..=cmp::min(player_stock[0] as usize, tome_spells.len() - 1) {
                valid_moves.add(Move::LEARN(tome_spells[t].id));
            }
        }

        // At the end, if there's no valid moves, we just send a wait
        if valid_moves.len == 0 {
            valid_moves.add(Move::WAIT);
        }

        (state.active_player, valid_moves)
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

    #[allow(non_snake_case)]
    pub struct Cache {
        map_stockArr4_stockId: Vec<Vec<Vec<Vec<usize>>>>,
        map_stockId_timesCanCastSpell: [[u8; EXISTING_SPELL_COUNT]; 1001],
        map_stockId_canFullfillOrder: [[bool; EXISTING_ORDER_COUNT]; 1001],
    }

    #[allow(non_snake_case)]
    impl Cache {
        pub fn new() -> Self {
            let (
                map_stockArr4_stockId,
                map_stockId_timesCanCastSpell,
                map_stockId_canFullfillOrder,
            ) = Cache::init_map_stocks();

            Cache {
                map_stockArr4_stockId,
                map_stockId_timesCanCastSpell,
                map_stockId_canFullfillOrder,
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
    }

    /* #endregion */

    /* #region(collapsed) [Private functions] */
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
                tax: 0,
                repeatable: s[0] < 0 || s[1] < 0 || s[2] < 0 || s[3] < 0,
                active: true,
            })
            .collect::<Vec<Spell>>()
    }

    fn get_basic_spells() -> Vec<Spell> {
        [
            Spell {
                id: 42,
                recipe: [2, 0, 0, 0],
                delta_stock: 2,
                tax: 0,
                repeatable: false,
                active: true,
            },
            Spell {
                id: 43,
                recipe: [-1, 1, 0, 0],
                delta_stock: 0,
                tax: 0,
                repeatable: false,
                active: true,
            },
            Spell {
                id: 44,
                recipe: [0, -1, 1, 0],
                delta_stock: 0,
                tax: 0,
                repeatable: false,
                active: true,
            },
            Spell {
                id: 45,
                recipe: [0, 0, -1, 1],
                delta_stock: 0,
                tax: 0,
                repeatable: false,
                active: true,
            },
        ]
        .to_vec()
    }

    fn get_all_tome_spells() -> Vec<Spell> {
        let mut tome_spells = get_learnable_tome_spells();
        tome_spells.extend_from_slice(&get_basic_spells());

        tome_spells
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
                bonus: 0,
            })
            .collect::<Vec<Order>>()
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

    fn can_fulfill_order(order: &Order, stock: &Stock) -> bool {
        stock[0] >= -order.recipe[0]
            && stock[1] >= -order.recipe[1]
            && stock[2] >= -order.recipe[2]
            && stock[3] >= -order.recipe[3]
    }

    fn get_order_position(orders: &[Order], order_id: u32) -> Option<usize> {
        for i in 0..orders.len() {
            if orders[i].id == order_id {
                return Some(i);
            }
        }
        None
    }

    fn get_spell_position(spells: &[Spell], spell_id: u32) -> Option<usize> {
        for i in 0..spells.len() {
            if spells[i].id == spell_id {
                return Some(i);
            }
        }
        None
    }

    fn update_counter_orders(
        counter_orders: &mut StackVector<Order, 5>,
        queued_orders: &mut Vec<Order>,
        plus_3_bonus_remaining: &mut u8,
        plus_1_bonus_remaining: &mut u8,
        orders_to_remove_pos: &[Option<usize>; 2],
    ) {
        /* First remove and determine how many orders & bonuses were used */
        let removed_orders_count;

        match orders_to_remove_pos {
            [None, None] => return,
            [Some(o), None] | [None, Some(o)] => {
                removed_orders_count = 1;
                match counter_orders.get(*o).bonus {
                    3 => {
                        *plus_3_bonus_remaining -= 1;
                    }
                    1 => {
                        *plus_1_bonus_remaining -= 1;
                    }
                    _ => {}
                }
                counter_orders.remove(*o);
            }
            [Some(o1), Some(o2)] if *o1 == *o2 => {
                removed_orders_count = 1;
                match counter_orders.get(*o1).bonus {
                    3 => {
                        *plus_3_bonus_remaining = plus_3_bonus_remaining.saturating_sub(2);
                    }
                    1 => {
                        *plus_1_bonus_remaining = plus_1_bonus_remaining.saturating_sub(2);
                    }
                    _ => {}
                }
                counter_orders.remove(*o1);
            }
            [Some(o1), Some(o2)] => {
                removed_orders_count = 2;
                for o in [o1, o2].iter() {
                    match counter_orders.get(**o).bonus {
                        3 => {
                            *plus_3_bonus_remaining -= 1;
                        }
                        1 => {
                            *plus_1_bonus_remaining -= 1;
                        }
                        _ => {}
                    }
                }
                counter_orders.remove_multi([*o1, *o2]);
            }
        }

        /* Replace removed orders if possible */
        for _ in 0..removed_orders_count {
            if queued_orders.len() > 0 {
                counter_orders.add(queued_orders.pop().unwrap());
            }
        }

        /* Update bonuses if possible */
        if *plus_3_bonus_remaining > 0 {
            if counter_orders.len() >= 1 {
                counter_orders.get_mut(0).bonus = 3;
            }

            if *plus_1_bonus_remaining > 0 {
                if counter_orders.len() >= 2 {
                    counter_orders.get_mut(1).bonus = 1;
                }
            }
        } else if plus_1_bonus_remaining > &mut 0 {
            if counter_orders.len() >= 1 {
                counter_orders.get_mut(0).bonus = 1;
            }

            if counter_orders.len() >= 2 {
                counter_orders.get_mut(1).bonus = 0;
            }
        } else {
            if counter_orders.len() >= 1 {
                counter_orders.get_mut(0).bonus = 0;
            }

            if counter_orders.len() >= 2 {
                counter_orders.get_mut(1).bonus = 0;
            }
        }
    }

    fn update_tome_spells(
        tome_spells: &mut StackVector<Spell, 6>,
        queued_spells: &mut Vec<Spell>,
        spells_to_remove_pos: &[Option<usize>; 2],
        spell_tax_payed: &[Option<usize>; 2],
    ) {
        /* First remove and determine how many spells were learnt */
        let learnt_spells_count;

        match spells_to_remove_pos {
            [None, None] => return,
            [Some(s), None] | [None, Some(s)] => {
                tome_spells.remove(*s);
                learnt_spells_count = 1;
            }
            [Some(s1), Some(s2)] if *s1 == *s2 => {
                tome_spells.remove(*s1);
                learnt_spells_count = 1;
            }
            [Some(s1), Some(s2)] => {
                tome_spells.remove_multi([*s1, *s2]);
                learnt_spells_count = 2;
            }
        }

        /* Replace removed spells if possible */
        for _ in 0..learnt_spells_count {
            if queued_spells.len() > 0 {
                tome_spells.add(queued_spells.pop().unwrap());
            }
        }

        /* Pay taxes */
        for p in 0..2 {
            if let Some(t) = spell_tax_payed[p] {
                for s in 0..cmp::min(t, tome_spells.len()) {
                    tome_spells.get_mut(s).tax += 1;
                }
            }
        }
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

mod mcts {

    use super::game;
    use rand::Rng;
    use std::time::Instant;

    const MAX_NODE_COUNT: usize = 300_000;
    const TIME_LIMIT_MS: u128 = 50;

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
                "[MCTS P2] End. Sending best move after expanding {} nodes and running {} simulations in {:?}",
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

    let mut cache = game::Cache::new();
    let mut mcts: mcts::MCTS = mcts::new();

    // game loop
    while ctr_rcv.recv().unwrap() == true {
        /* Prepare mappings between internal spell & order ids & codingame ids */
        #[allow(non_snake_case)]
        let mut map_order_internalId_cgId: HashMap<u32, u32> = HashMap::new();
        #[allow(non_snake_case)]
        let mut map_spell_internalId_cgId: HashMap<u32, u32> = HashMap::new();

        /* Prepare inputs */
        let mut players: [game::Player; 2] = [game::Player {
            move_: game::Move::NONE,
            stock: [0, 0, 0, 0],
            stock_id: cache.getStockId(&[0, 0, 0, 0]),
            spells: StackVector::new(),
            rupees: 0,
            brewed_potions_count: 0,
        }; 2];

        let mut counter_orders: StackVector<game::Order, 5> = StackVector::new();
        let mut plus_3_bonus_remaining: u8 = 0;
        let mut plus_1_bonus_remaining: u8 = 0;

        let mut tome_spells: StackVector<game::Spell, 6> = StackVector::new();

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
                let mut order = game::find_order(&[delta_0, delta_1, delta_2, delta_3]).unwrap();

                match tome_index {
                    3 => {
                        plus_3_bonus_remaining = tax_count as u8;
                        order.bonus = 3;
                    }
                    1 => {
                        plus_1_bonus_remaining = tax_count as u8;
                        order.bonus = 1;
                    }
                    _ => {}
                }

                map_order_internalId_cgId.insert(order.id, action_id);
                counter_orders.add(order);
            } else if action_type == String::from("CAST")
                || action_type == String::from("OPPONENT_CAST")
            {
                let p = match action_type.as_str() {
                    "CAST" => 0,
                    "OPPONENT_CAST" => 1,
                    _ => panic!(),
                };

                let mut spell = game::find_spell(&[delta_0, delta_1, delta_2, delta_3]).unwrap();
                if p == 0 {
                    map_spell_internalId_cgId.insert(spell.id as u32, action_id);
                }

                if castable == 0 {
                    spell.active = false;
                }

                players[p as usize].spells.add(spell);
            } else if action_type == String::from("LEARN") {
                let mut spell = game::find_spell(&[delta_0, delta_1, delta_2, delta_3]).unwrap();
                map_spell_internalId_cgId.insert(spell.id as u32, action_id);
                spell.tax = tax_count as u8;

                tome_spells.add(spell);
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

            players[i].rupees = score;

            if score > player_rupees[i] {
                player_brewed_potion_count[i] += 1;
            }
            player_rupees[i] = score;

            players[i].brewed_potions_count = player_brewed_potion_count[i];
        }

        /* Initialize states */

        let state = game::new(
            players,
            tome_spells,
            counter_orders,
            plus_3_bonus_remaining,
            plus_1_bonus_remaining,
            turn,
        );

        // (3) Determine the next best action
        let best_move = mcts.best_move(&state, 0, &mut cache);
        turn += 1;

        // convert move to codingame id
        let best_move_cg = match best_move {
            game::Move::NONE | game::Move::WAIT | game::Move::REST => best_move.clone(),
            game::Move::BREW(o) => game::Move::BREW(*map_order_internalId_cgId.get(&o).unwrap()),
            game::Move::CAST(s, t) => {
                game::Move::CAST(*map_spell_internalId_cgId.get(&s).unwrap(), t)
            }
            game::Move::LEARN(s) => game::Move::LEARN(*map_spell_internalId_cgId.get(&s).unwrap()),
        };

        /* #region [Extract player state] */
        let mut player_state: HashMap<String, String> = HashMap::new();
        player_state.insert("Hello".to_string(), "World".to_string());

        /* #endregion */

        let msg = best_move_cg.to_string();
        msg_snd.send((format!("{}", msg), Some(player_state)));
    }
}
