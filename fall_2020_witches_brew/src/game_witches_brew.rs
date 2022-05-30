use common::record;
use common::{Game, Message, StackVector, WinLossTie};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::cmp;
use std::collections::HashMap;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

const MAX_VALID_MOVES: usize = 35 + 5 + 6 + 1; // 35 CAST + 5 BREW + 6 LEARN + 1 REST
const MAX_PLAYER_SPELLS: usize = 42 + 4;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Move {
    NONE,
    WAIT,
    REST,
    BREW(u32),
    CAST(u32, u8),
    LEARN(u32),
}

impl Move {
    fn parse_move(msg: &str) -> Move {
        match msg {
            "WAIT" => Move::WAIT,
            "REST" => Move::REST,
            _ => match &msg[0..5] {
                "BREW " => Move::BREW(parse_input!(msg[5..], u32)),
                "LEARN" => Move::LEARN(parse_input!(msg[6..], u32)),
                "CAST " => {
                    let s = msg.split(" ").collect::<Vec<_>>();
                    match s.len() {
                        2 => Move::CAST(parse_input!(s[1], u32), 1),
                        3 => Move::CAST(parse_input!(s[1], u32), parse_input!(s[2], u8)),
                        _ => panic!("Couldn't parse correctly CAST move"),
                    }
                }
                _ => {
                    panic!();
                }
            },
        }
    }

    fn to_string(&self) -> String {
        match self {
            Move::NONE => format!("None"),
            Move::WAIT => format!("WAIT"),
            Move::REST => format!("REST"),
            Move::BREW(i) => format!("BREW {}", i),
            Move::CAST(i, 1) => format!("CAST {}", i),
            Move::CAST(i, t) => format!("CAST {}x{}", i, t),
            Move::LEARN(i) => format!("LEARN {}", i),
        }
    }
}

impl Default for Move {
    fn default() -> Self {
        Move::NONE
    }
}

type Ingredients = [i8; 4];

#[derive(Copy, Clone, Default)]
struct Order {
    id: u32,
    recipe: Ingredients,
    price: u8,
    bonus: u8,
}

#[derive(Copy, Clone, Default, Debug)]
struct Spell {
    id: u32,
    recipe: Ingredients,
    delta_stock: i8,
    tax: u8,
    repeatable: bool,
    active: bool,
}

#[derive(Clone)]
struct Player {
    move_: Move,
    stock: Ingredients,
    spells: StackVector<Spell, MAX_PLAYER_SPELLS>,
    rupees: u32,
    brewed_potions_count: u8,
}

pub struct WitchesBrewGame {
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

impl WitchesBrewGame {
    fn get_tome_spells() -> Vec<Spell> {
        let spells: Vec<Ingredients> = vec![
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

    fn get_basic_spells() -> [Spell; 4] {
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
    }

    fn find_spell(recipe: &Ingredients) -> Option<Spell> {
        for spell in WitchesBrewGame::get_tome_spells().iter() {
            if spell.recipe == *recipe {
                return Some(spell.clone());
            }
        }
        for spell in WitchesBrewGame::get_basic_spells().iter() {
            if spell.recipe == *recipe {
                return Some(spell.clone());
            }
        }

        None
    }

    fn get_all_orders() -> Vec<Order> {
        let orders: Vec<(Ingredients, u8)> = vec![
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

    fn find_order(recipe: &Ingredients) -> Option<Order> {
        for order in WitchesBrewGame::get_all_orders().iter() {
            if order.recipe == *recipe {
                return Some(order.clone());
            }
        }

        None
    }

    fn can_fulfill_order(order: &Order, stock: &Ingredients) -> bool {
        stock[0] >= -order.recipe[0]
            && stock[1] >= -order.recipe[1]
            && stock[2] >= -order.recipe[2]
            && stock[3] >= -order.recipe[3]
    }

    fn can_cast_spell(spell: &Spell, stock: &Ingredients) -> bool {
        if spell.active == false {
            return false;
        }

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
    fn how_many_times_can_cast_spell(spell: &Spell, stock: &Ingredients) -> u8 {
        if spell.active == false {
            return 0;
        }

        if spell.repeatable == false {
            match WitchesBrewGame::can_cast_spell(spell, stock) {
                true => 1,
                false => 0,
            }
        } else {
            let mut times = 0;
            let mut stock = stock.clone();

            while WitchesBrewGame::can_cast_spell(spell, &stock) {
                times += 1;
                WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 1);
            }

            times
        }
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

    fn cast_and_update_stock(stock: &mut Ingredients, recipe: &Ingredients, times: u8) {
        for _ in 0..times {
            for i in 0..4 {
                stock[i] += recipe[i];
            }
        }
    }

    fn brew_and_update_stock(stock: &mut Ingredients, order: &Order) {
        for i in 0..4 {
            stock[i] += order.recipe[i];
        }
    }

    fn valid_moves(
        orders: &[Order],
        tome_spells: &[Spell],
        player_spells: &[Spell],
        stock: &Ingredients,
    ) -> StackVector<Move, MAX_VALID_MOVES> {
        // There's at max 10 possible moves : 5 orders, 4 spells + REST
        let mut valid_moves: StackVector<Move, MAX_VALID_MOVES> = StackVector {
            arr: [Move::NONE; MAX_VALID_MOVES],
            len: 0,
        };

        /* BREW moves */
        // Check which order the player can fulfill and add them as a valid move
        for order in orders.iter() {
            if WitchesBrewGame::can_fulfill_order(order, stock) {
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
                    WitchesBrewGame::how_many_times_can_cast_spell(sp, stock);
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
        for (t, spell) in tome_spells.iter().enumerate() {
            if t as u8 <= stock[0] as u8 {
                valid_moves.add(Move::LEARN(spell.id));
            }
        }

        // At the end, if there's no valid moves, we just send a wait
        if valid_moves.len == 0 {
            valid_moves.add(Move::WAIT);
        }

        valid_moves
    }
}

impl Game for WitchesBrewGame {
    fn new() -> Self {
        /* Create Player's basic spells */
        let mut player_spells: StackVector<Spell, MAX_PLAYER_SPELLS> = StackVector::new();
        let basic_spells = WitchesBrewGame::get_basic_spells();
        for i in 0..4 {
            player_spells.add(basic_spells[i]);
        }

        /* Create Players */
        let player = Player {
            move_: Move::NONE,
            stock: [3, 0, 0, 0],
            rupees: 0,
            brewed_potions_count: 0,
            spells: player_spells,
        };

        /* Create orders */
        let mut all_orders = WitchesBrewGame::get_all_orders();
        all_orders.shuffle(&mut thread_rng());

        let mut counter_orders: StackVector<Order, 5> = StackVector::new();
        for _ in 0..5 {
            counter_orders.add(all_orders.pop().unwrap());
        }

        /* Create tome spells */
        let mut queued_spells = WitchesBrewGame::get_tome_spells();
        queued_spells.shuffle(&mut thread_rng());

        let mut tome_spells: StackVector<Spell, 6> = StackVector::new();
        for _ in 0..6 {
            tome_spells.add(queued_spells.pop().unwrap());
        }

        let game = WitchesBrewGame {
            players: [player.clone(), player.clone()],

            queued_orders: all_orders,
            counter_orders: counter_orders,
            plus_3_bonus_remaining: 4,
            plus_1_bonus_remaining: 4,

            queued_spells: queued_spells,
            tome_spells: tome_spells,

            active: true,
            active_player: 0,
            turn: 0,
            winners: None,
        };

        game
    }

    fn turn(&self) -> Option<Message> {
        // If game is over, return None
        if self.active == false {
            return None;
        }

        let mut out: Vec<String> = Vec::new();

        /* (1) Output number of orders */
        // Count # of orders
        let nb_actions = self.counter_orders.len()
            + self.players[0].spells.len()
            + self.players[1].spells.len()
            + self.tome_spells.len();

        out.push(format!("{}", nb_actions)); // add the 8 spells of the 2 players

        /* (2) Output available orders */
        for order in self.counter_orders.slice().iter() {
            out.push(format!(
                "{} BREW {} {} {} {} {} {} {} 0 0",
                order.id,
                order.recipe[0],
                order.recipe[1],
                order.recipe[2],
                order.recipe[3],
                order.price + order.bonus,
                order.bonus,
                match order.bonus {
                    3 => self.plus_3_bonus_remaining,
                    1 => self.plus_1_bonus_remaining,
                    _ => 0,
                }
            ));
        }

        /* (2) Output available tome spells to learn */
        for (spell_idx, spell) in self.tome_spells.slice().iter().enumerate() {
            out.push(format!(
                "{} LEARN {} {} {} {} 0 {} {} 0 {}",
                spell.id,
                spell.recipe[0],
                spell.recipe[1],
                spell.recipe[2],
                spell.recipe[3],
                spell_idx,
                spell.tax,
                match spell.repeatable {
                    true => 1,
                    false => 0,
                }
            ));
        }

        let active_player: &Player = &self.players[self.active_player as usize];
        let other_player: &Player = &self.players[((self.active_player + 1) % 2) as usize];

        /* (3) Output the active player' spells */
        for spell in active_player.spells.slice().iter() {
            out.push(format!(
                "{} CAST {} {} {} {} 0 -1 -1 {} {}",
                spell.id,
                spell.recipe[0],
                spell.recipe[1],
                spell.recipe[2],
                spell.recipe[3],
                match spell.active {
                    true => 1,
                    false => 0,
                },
                match spell.repeatable {
                    true => 1,
                    false => 0,
                }
            ));
        }

        /* (4) Output the other player' spells */
        for spell in other_player.spells.slice().iter() {
            out.push(format!(
                "{} OPPONENT_CAST {} {} {} {} 0 -1 -1 {} {}",
                spell.id,
                spell.recipe[0],
                spell.recipe[1],
                spell.recipe[2],
                spell.recipe[3],
                match spell.active {
                    true => 1,
                    false => 0,
                },
                match spell.repeatable {
                    true => 1,
                    false => 0,
                }
            ));
        }

        /* (5) Output the active player' ingeredient stock & rupees */
        out.push(format!(
            "{} {} {} {} {}",
            active_player.stock[0],
            active_player.stock[1],
            active_player.stock[2],
            active_player.stock[3],
            active_player.rupees,
        ));

        /* (5) Output the other player' ingeredient stock & rupees */
        out.push(format!(
            "{} {} {} {} {}",
            other_player.stock[0],
            other_player.stock[1],
            other_player.stock[2],
            other_player.stock[3],
            other_player.rupees,
        ));

        /* (X) Send message */
        Some(Message {
            player_id: self.active_player as usize,
            messages: out,
        })
    }

    fn play(&mut self, msg: String) {
        /* (1) Parse move, assuming it is always in the right format */
        let _move = Move::parse_move(msg.as_str());

        /* (2) Record the move */
        self.players[self.active_player as usize].move_ = _move;

        /* (3) If it's player'1 turn, i.e. both players have played =>  update the state */
        if self.active_player == 1 {
            let player0: &Player = &self.players[0];
            let player1: &Player = &self.players[1];

            /* 3.1 Check if moves were valid */
            let player0_valid_moves = WitchesBrewGame::valid_moves(
                &self.counter_orders.slice(),
                &self.tome_spells.slice(),
                &player0.spells.slice(),
                &player0.stock,
            );
            let player1_valid_moves = WitchesBrewGame::valid_moves(
                &self.counter_orders.slice(),
                &self.tome_spells.slice(),
                &player1.spells.slice(),
                &player1.stock,
            );

            let is_move0_valid = player0_valid_moves.slice().contains(&player0.move_);
            let is_move1_valid = player1_valid_moves.slice().contains(&player1.move_);

            if !is_move0_valid && !is_move1_valid {
                eprintln!(
                    "[GAME] Player 0's move {} & Player 1's move {} are both invalid",
                    &player0.move_.to_string(),
                    &player1.move_.to_string()
                );
                self.active = false;
                self.winners = Some((WinLossTie::Loss, WinLossTie::Loss));
                return;
            } else if is_move0_valid && !is_move1_valid {
                eprintln!(
                    "[GAME] Player 1's move {} is invalid",
                    &player1.move_.to_string()
                );
                self.active = false;
                self.winners = Some((WinLossTie::Win, WinLossTie::Loss));
                return;
            } else if !is_move0_valid && is_move1_valid {
                eprintln!(
                    "[GAME] Player 0's move {} is invalid",
                    &player0.move_.to_string()
                );
                self.active = false;
                self.winners = Some((WinLossTie::Loss, WinLossTie::Win));
                return;
            }

            /* 3.2 Update the state */
            // For each player move
            let mut orders_were_fullfilled = false;
            let mut orders_to_remove: [Option<usize>; 2] = [None, None];
            let mut spells_were_learnt = false;
            let mut spells_to_remove: [Option<usize>; 2] = [None, None];
            let mut spell_tax_payed: [Option<usize>; 2] = [None, None];

            for (pid, player) in self.players.iter_mut().enumerate() {
                match player.move_ {
                    Move::BREW(order_id) => {
                        let fullfilled_order_pos = WitchesBrewGame::get_order_position(
                            &self.counter_orders.slice(),
                            order_id,
                        )
                        .unwrap();

                        let fullfilled_order = self.counter_orders.get(fullfilled_order_pos);

                        // Update the player's potion count
                        player.brewed_potions_count += 1;

                        // check if there's a bonus
                        match fullfilled_order.bonus {
                            3 => {
                                self.plus_3_bonus_remaining -= 1;
                            }
                            1 => {
                                self.plus_1_bonus_remaining -= 1;
                            }
                            _ => {}
                        }

                        // Update the player's rupees
                        player.rupees +=
                            fullfilled_order.price as u32 + fullfilled_order.bonus as u32;

                        // Update the player's ingredient stock
                        WitchesBrewGame::brew_and_update_stock(
                            &mut player.stock,
                            &fullfilled_order,
                        );

                        // Save fullfilled orders so that I remove them later
                        orders_were_fullfilled = true;
                        if orders_to_remove[0] == None {
                            orders_to_remove[0] = Some(fullfilled_order_pos);
                        } else if fullfilled_order_pos != orders_to_remove[0].unwrap() {
                            orders_to_remove[1] = Some(fullfilled_order_pos);
                        }
                    }
                    Move::CAST(spell_id, times) => {
                        let cast_spell_idx =
                            WitchesBrewGame::get_spell_position(&player.spells.slice(), spell_id)
                                .unwrap();

                        let cast_spell = player.spells.get_mut(cast_spell_idx);

                        // Update the player's ingredient stock
                        WitchesBrewGame::cast_and_update_stock(
                            &mut player.stock,
                            &cast_spell.recipe,
                            times,
                        );

                        // Spell is now exhausted
                        cast_spell.active = false;
                    }
                    Move::LEARN(spell_id) => {
                        let learnt_spell_pos = WitchesBrewGame::get_spell_position(
                            &self.tome_spells.slice(),
                            spell_id,
                        )
                        .unwrap();

                        let learnt_spell = self.tome_spells.get(learnt_spell_pos);

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
                        // Save learnt spells, so that I replace them later and deal with the tax
                        spells_were_learnt = true;
                        if spells_to_remove[0] == None {
                            spells_to_remove[0] = Some(learnt_spell_pos);
                        } else if learnt_spell_pos != spells_to_remove[0].unwrap() {
                            spells_to_remove[1] = Some(learnt_spell_pos);
                        }
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
                for i in 0..2 {
                    if let Some(oix) = orders_to_remove[i] {
                        self.counter_orders.remove(oix);
                        self.counter_orders.add(self.queued_orders.pop().unwrap());
                    }
                }

                if orders_to_remove[1] == None {
                    // only 1 order to remove
                    self.counter_orders.remove(orders_to_remove[0].unwrap());
                    self.counter_orders.add(self.queued_orders.pop().unwrap());
                } else {
                    // 2 spells to remove
                    self.counter_orders
                        .remove_multi([orders_to_remove[0].unwrap(), orders_to_remove[1].unwrap()]);

                    self.counter_orders.add(self.queued_orders.pop().unwrap());
                    self.counter_orders.add(self.queued_orders.pop().unwrap());
                }

                if self.plus_3_bonus_remaining > 0 {
                    self.counter_orders.get_mut(0).bonus = 3;
                    if self.plus_1_bonus_remaining > 0 {
                        self.counter_orders.get_mut(1).bonus = 1;
                    }
                } else if self.plus_1_bonus_remaining > 0 {
                    self.counter_orders.get_mut(0).bonus = 1;
                    self.counter_orders.get_mut(1).bonus = 0;
                } else {
                    self.counter_orders.get_mut(0).bonus = 0;
                    self.counter_orders.get_mut(0).bonus = 0;
                }
            }

            /* Remove learnt spells and create new one in their place, and update tax */
            if spells_were_learnt == true {
                if spells_to_remove[1] == None {
                    // only 1 spell to remove
                    self.tome_spells.remove(spells_to_remove[0].unwrap());
                    if self.queued_spells.len() > 0 {
                        self.tome_spells.add(self.queued_spells.pop().unwrap());
                    }
                } else {
                    // 2 spells to remove
                    self.tome_spells
                        .remove_multi([spells_to_remove[0].unwrap(), spells_to_remove[1].unwrap()]);

                    if self.queued_spells.len() > 0 {
                        self.tome_spells.add(self.queued_spells.pop().unwrap());
                    }
                    if self.queued_spells.len() > 0 {
                        self.tome_spells.add(self.queued_spells.pop().unwrap());
                    }
                }

                for i in 0..2 {
                    if let Some(t) = spell_tax_payed[i] {
                        for i in 0..t {
                            self.tome_spells.get_mut(i).tax += 1;
                        }
                    }
                }
            }

            /* 3.3 Check terminal condition */
            let player0: &Player = &self.players[0];
            let player1: &Player = &self.players[1];

            if player0.brewed_potions_count == 3
                || player1.brewed_potions_count == 3
                || self.turn == 100
            {
                self.active = false;

                let score0 = player0.rupees
                    + (player0.stock[1] + player0.stock[2] + player0.stock[3]) as u32;
                let score1 = player1.rupees
                    + (player1.stock[1] + player1.stock[2] + player1.stock[3]) as u32;

                if score0 > score1 {
                    self.winners = Some((WinLossTie::Win, WinLossTie::Loss));
                } else if score0 < score1 {
                    self.winners = Some((WinLossTie::Loss, WinLossTie::Win));
                } else {
                    self.winners = Some((WinLossTie::Tie, WinLossTie::Tie));
                }
            }

            /* 3.3 Reinit moves */
            self.players[0].move_ = Move::NONE;
            self.players[1].move_ = Move::NONE;
            self.turn += 1;
        }
        self.active_player = (self.active_player + 1) % 2;
    }

    fn winners(&self) -> Option<Vec<WinLossTie>> {
        match &self.winners {
            Some(w) => Some(vec![w.0, w.1]),
            None => None,
        }
    }

    fn get_state(&self) -> record::GameState {
        let mut state: HashMap<String, String> = HashMap::new();
        state.insert(String::from("Turn"), self.turn.to_string());
        state.insert(String::from("Active"), self.active.to_string());
        state.insert(
            String::from("Bonus left"),
            format!(
                "[{}, {}]",
                self.plus_3_bonus_remaining.to_string(),
                self.plus_1_bonus_remaining.to_string()
            ),
        );
        state.insert(
            String::from("Active_player"),
            self.active_player.to_string(),
        );

        state.insert(
            String::from("Moves"),
            format!(
                "[{}, {}]",
                self.players[0].move_.to_string(),
                self.players[1].move_.to_string()
            ),
        );

        fn fmt_order(order: &Order) -> String {
            format!(
                "[{} | {} | 🔸{}{}]",
                order.id,
                {
                    let tiers = ['🐋', '🍏', '🦧', '💛'];
                    let mut s = String::from("");
                    for i in 0..order.recipe.len() {
                        if order.recipe[i] < 0 {
                            s.push_str(&format!("{}{} ", order.recipe[i], tiers[i]));
                        }
                    }
                    s
                },
                order.price,
                match order.bonus {
                    3 => "+3",
                    1 => "+1",
                    _ => "",
                }
            )
        }

        fn fmt_spell(spell: &Spell) -> String {
            format!(
                "[{}{} {} | {} | {}]",
                match spell.active {
                    true => "🟢",
                    false => "⚪",
                },
                match spell.repeatable {
                    true => "🔁",
                    false => "",
                },
                spell.id,
                {
                    let tiers = ['🐋', '🍏', '🦧', '💛'];
                    let mut s = String::from("");
                    for i in 0..spell.recipe.len() {
                        if spell.recipe[i] < 0 {
                            s.push_str(&format!("{}{} ", spell.recipe[i], tiers[i]));
                        }

                        if spell.recipe[i] > 0 {
                            s.push_str(&format!("+{}{} ", spell.recipe[i], tiers[i]));
                        }
                    }
                    s
                },
                match spell.tax {
                    1 => String::from("🐋"),
                    2.. => format!("🐋x{}", spell.tax),
                    _ => String::from(""),
                }
            )
        }

        state.insert(
            String::from("Orders"),
            self.counter_orders
                .slice()
                .iter()
                .map(|order| fmt_order(order))
                .collect::<Vec<String>>()
                .join(", "),
        );

        state.insert(
            String::from("Tome"),
            self.tome_spells
                .slice()
                .iter()
                .map(|spell| fmt_spell(spell))
                .collect::<Vec<String>>()
                .join(", "),
        );

        for pid in 0..=1 {
            let player: &Player = &self.players[pid];

            fn fmt_stock(ingredients: &Ingredients) -> String {
                format!(
                    "[🐋: {}, 🍏: {}, 🦧: {}, 💛: {}]",
                    ingredients[0], ingredients[1], ingredients[2], ingredients[3]
                )
            }

            state.insert(format!("player[{}]: Stock", pid), fmt_stock(&player.stock));

            state.insert(
                format!("player[{}]: Rupees", pid),
                player.rupees.to_string(),
            );

            state.insert(
                format!("player[{}]: Brewed potion count", pid),
                player.brewed_potions_count.to_string(),
            );

            state.insert(
                format!("player[{}]: Spells", pid),
                player
                    .spells
                    .slice()
                    .iter()
                    .map(|s| fmt_spell(s))
                    .collect::<Vec<String>>()
                    .join(","),
            );

            let valid_moves = WitchesBrewGame::valid_moves(
                &self.counter_orders.slice(),
                &self.tome_spells.slice(),
                &player.spells.slice(),
                &player.stock,
            );
            state.insert(
                format!("player[{}]: Valid moves", pid),
                valid_moves
                    .slice()
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        }

        record::GameState { board: None, state }
    }

    fn get_board_representation() -> Option<record::BoardRepresentation> {
        None
    }
}

#[cfg(test)]
mod tests {
    use common::assert_vec_eq;

    use super::*;

    #[test]
    fn test_parse_move() {
        assert_eq!(Move::parse_move("WAIT"), Move::WAIT);
        assert_eq!(Move::parse_move("REST"), Move::REST);
        assert_eq!(Move::parse_move("BREW 1"), Move::BREW(1));
        assert_eq!(Move::parse_move("BREW 10"), Move::BREW(10));
        assert_eq!(Move::parse_move("LEARN 2"), Move::LEARN(2));
        assert_eq!(Move::parse_move("LEARN 20"), Move::LEARN(20));
        assert_eq!(Move::parse_move("CAST 3"), Move::CAST(3, 1));
        assert_eq!(Move::parse_move("CAST 30"), Move::CAST(30, 1));
        assert_eq!(Move::parse_move("CAST 3 2"), Move::CAST(3, 2));
        assert_eq!(Move::parse_move("CAST 3 12"), Move::CAST(3, 12));
        assert_eq!(Move::parse_move("CAST 30 2"), Move::CAST(30, 2));
        assert_eq!(Move::parse_move("CAST 30 12"), Move::CAST(30, 12));
    }

    #[test]
    fn test_get_tome_spell() {
        let tome_spells = WitchesBrewGame::get_tome_spells();
        assert_eq!(tome_spells.len(), 42);

        assert_eq!(tome_spells[2].repeatable, false);
        assert_eq!(tome_spells[3].repeatable, false);
        assert_eq!(tome_spells[4].repeatable, false);
        assert_eq!(tome_spells[5].repeatable, true);

        assert_eq!(tome_spells[0].delta_stock, -2);
        assert_eq!(tome_spells[2].delta_stock, 2);
        assert_eq!(tome_spells[3].delta_stock, 1);
        assert_eq!(tome_spells[4].delta_stock, 3);
        assert_eq!(tome_spells[5].delta_stock, 3);
    }

    #[test]
    fn test_get_spell() {
        let spell = WitchesBrewGame::find_spell(&[-5, 0, 0, 2]).unwrap();
        assert_eq!(spell.id, 29);

        let spell = WitchesBrewGame::find_spell(&[2, 1, 0, 0]).unwrap();
        assert_eq!(spell.id, 12);
        assert_eq!(spell.repeatable, false);

        let spell = WitchesBrewGame::find_spell(&[-1, 1, 0, 0]).unwrap();
        assert_eq!(spell.id, 43);
        assert_eq!(spell.repeatable, false);

        let spell = WitchesBrewGame::find_spell(&[-5, 0, 0, 0]);
        assert!(spell.is_none());
    }

    #[test]
    fn test_get_order() {
        let order = WitchesBrewGame::find_order(&[-2, 0, 0, -2]).unwrap();
        assert_eq!(order.id, 8);
        assert_eq!(order.price, 10);

        let order = WitchesBrewGame::find_order(&[-3, -2, -1, 0]);
        assert!(order.is_none());
    }

    #[test]
    fn test_can_fulfill_order() {
        let orders = WitchesBrewGame::get_all_orders();

        let stock = [3, 2, 1, 0];

        // order[0] = [2, 2, 0, 0]
        assert_eq!(WitchesBrewGame::can_fulfill_order(&orders[0], &stock), true);
        // order[2] = [0, 4, 0, 0]
        assert_eq!(
            WitchesBrewGame::can_fulfill_order(&orders[2], &stock),
            false
        );

        let stock = [2, 2, 2, 0];
        //order[27] = [2, 2, 2, 0]
        assert_eq!(
            WitchesBrewGame::can_fulfill_order(&orders[27], &stock),
            true
        );
        //order[26] = [1, 0, 2, 1]
        assert_eq!(
            WitchesBrewGame::can_fulfill_order(&orders[26], &stock),
            false
        );

        let stock = [0, 10, 0, 0];
        //order[7] = [0, 5, 0, 0]
        assert_eq!(WitchesBrewGame::can_fulfill_order(&orders[7], &stock), true);
        //order[4] = [2, 3, 0, 0]
        assert_eq!(
            WitchesBrewGame::can_fulfill_order(&orders[4], &stock),
            false
        );
    }

    #[test]
    fn test_can_cast_spell() {
        let tome_spells = WitchesBrewGame::get_tome_spells();

        // Test remove one ingredient
        let spell = &tome_spells[7]; //[3, 0, 1, -1]
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[0, 0, 0, 1]), true);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[1, 1, 1, 1]), true);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[0, 0, 0, 3]), true);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[0, 0, 0, 0]), false);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[1, 1, 1, 0]), false);

        // Test remove 2 ingredients
        let spell = &tome_spells[18]; //[-1, -1, 0, 1]
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[1, 1, 0, 0]), true);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[3, 1, 1, 1]), true);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[0, 3, 1, 1]), false);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[2, 0, 1, 1]), false);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[0, 0, 5, 5]), false);

        // Test inactive spell
        let mut spell = tome_spells[7].clone(); //[3, 0, 1, -1]
        spell.active = false;
        assert_eq!(
            WitchesBrewGame::can_cast_spell(&spell, &[0, 0, 0, 1]),
            false
        );

        // Test no more space
        let spell = &tome_spells[7]; //[3, 0, 1, -1]
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[6, 0, 0, 1]), true);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[7, 0, 0, 1]), false);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[2, 2, 2, 1]), true);
        assert_eq!(WitchesBrewGame::can_cast_spell(spell, &[2, 2, 2, 2]), false);
    }

    #[test]
    fn test_how_many_times_can_cast_spell() {
        let tome_spells = WitchesBrewGame::get_tome_spells();
        let basic_spells = WitchesBrewGame::get_basic_spells();

        /* Spell that just adds a single ingredient without removing another */
        let spell = &tome_spells[14]; //[0, 0, 0, 1]
                                      // Spell is not repeatable
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[0, 0, 0, 0]),
            1
        );

        // let's cheat and make it repeatable
        let mut spell = tome_spells[14].clone(); //[0, 0, 0, 1]
        spell.repeatable = true;
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[0, 0, 0, 0]),
            10
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[0, 0, 0, 5]),
            5
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[3, 3, 3, 0]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[3, 3, 3, 1]),
            0
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[0, 0, 0, 10]),
            0
        );

        /* Spell that  adds 2 ingredients without removing another */
        let spell = &tome_spells[2]; //[1, 1, 0, 0]
                                     // Spell is not repeatable
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[0, 0, 0, 0]),
            1
        );

        // let's cheat and make it repeatable
        let mut spell = tome_spells[2].clone(); //[1, 1, 0, 0]
        spell.repeatable = true;
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[0, 0, 0, 0]),
            5
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[2, 2, 0, 0]),
            3
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[0, 0, 4, 4]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[3, 3, 3, 1]),
            0
        );

        /* Spell that adds 1 ingredient and remove 1 ingredient */
        let spell = &basic_spells[1]; //[-1, 1, 0, 0]
                                      // Spell is not repeatable because it's a basic spell
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[5, 0, 0, 0]),
            1
        );

        // let's cheat and make it repeatable
        let mut spell = basic_spells[1].clone(); //[-1, 1, 0, 0]
        spell.repeatable = true;
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[5, 0, 0, 0]),
            5
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[1, 0, 0, 0]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[10, 0, 0, 0]),
            10
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[5, 5, 0, 0]),
            5
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[0, 5, 0, 0]),
            0
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[0, 0, 0, 0]),
            0
        );

        /* Spell that removes 1 ingredients and add 2 others */
        let spell = &tome_spells[9]; //[2, -3, 2, 0]
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[0, 3, 0, 0]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[0, 5, 0, 0]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[0, 6, 0, 0]),
            2
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[0, 8, 0, 0]),
            2
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[0, 9, 0, 0]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[1, 6, 2, 1]),
            0
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[1, 6, 1, 1]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[2, 3, 2, 2]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[2, 3, 2, 3]),
            0
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[1, 4, 1, 0]),
            1
        );

        /* Another test */
        let spell = &tome_spells[11]; //[-4, 0, 2, 0]
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[4, 0, 0, 0]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[7, 0, 0, 0]),
            1
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[10, 0, 0, 0]),
            2
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[8, 1, 1, 0]),
            2
        );
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(spell, &[3, 0, 0, 0]),
            0
        );

        /* Test inactive spell */
        let mut spell = tome_spells[18].clone(); //[-1, -1, 0, 1]
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[2, 2, 0, 0]),
            2
        );
        spell.active = false;
        assert_eq!(
            WitchesBrewGame::how_many_times_can_cast_spell(&spell, &[2, 2, 0, 0]),
            0
        );
    }

    #[test]
    fn test_get_order_position() {
        let mut orders = WitchesBrewGame::get_all_orders();

        assert_eq!(
            WitchesBrewGame::get_order_position(&orders[0..5], 2),
            Some(2)
        );
        assert_eq!(
            WitchesBrewGame::get_order_position(&orders[0..5], 4),
            Some(4)
        );

        orders[0].id = 200;
        orders[1].id = 201;
        assert_eq!(
            WitchesBrewGame::get_order_position(&orders[0..5], 200),
            Some(0)
        );
        assert_eq!(
            WitchesBrewGame::get_order_position(&orders[0..5], 201),
            Some(1)
        );
        assert_eq!(
            WitchesBrewGame::get_order_position(&orders[0..5], 202),
            None
        );
    }

    #[test]
    fn test_get_spell_position() {
        let mut spells = WitchesBrewGame::get_tome_spells();
        assert_eq!(
            WitchesBrewGame::get_spell_position(&spells[0..6], 2),
            Some(2)
        );
        assert_eq!(
            WitchesBrewGame::get_spell_position(&spells[0..6], 5),
            Some(5)
        );

        spells[0].id = 200;
        spells[2].id = 201;
        assert_eq!(
            WitchesBrewGame::get_spell_position(&spells[0..6], 200),
            Some(0)
        );
        assert_eq!(
            WitchesBrewGame::get_spell_position(&spells[0..6], 201),
            Some(2)
        );
        assert_eq!(
            WitchesBrewGame::get_spell_position(&spells[0..6], 202),
            None
        );
    }

    #[test]
    fn test_cast_and_update_stock() {
        let tome_spells = WitchesBrewGame::get_tome_spells();
        let basic_spells = WitchesBrewGame::get_basic_spells();

        let spell = &tome_spells[14]; //[0, 0, 0, 1]
        let mut stock = [0, 0, 0, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 1);
        assert_eq!(stock, [0, 0, 0, 1]);

        let spell = &tome_spells[2]; //[1, 1, 0, 0]
        let mut stock = [0, 0, 0, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 1);
        assert_eq!(stock, [1, 1, 0, 0]);

        let spell = &basic_spells[1]; //[-1, 1, 0, 0]
        let mut stock = [5, 0, 0, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 1);
        assert_eq!(stock, [4, 1, 0, 0]);
        let mut stock = [5, 0, 0, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 5);
        assert_eq!(stock, [0, 5, 0, 0]);

        let spell = &tome_spells[9]; //[2, -3, 2, 0]
        let mut stock = [0, 3, 0, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 1);
        assert_eq!(stock, [2, 0, 2, 0]);
        let mut stock = [0, 6, 0, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 2);
        assert_eq!(stock, [4, 0, 4, 0]);

        let spell = &tome_spells[11]; //[-4, 0, 2, 0]
        let mut stock = [7, 0, 0, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 1);
        assert_eq!(stock, [3, 0, 2, 0]);
        let mut stock = [8, 1, 1, 0];
        WitchesBrewGame::cast_and_update_stock(&mut stock, &spell.recipe, 2);
        assert_eq!(stock, [0, 1, 5, 0]);
    }

    #[test]
    fn test_brew_and_update_stock() {
        let orders = WitchesBrewGame::get_all_orders();

        let order = orders[0]; //[2, 2, 0, 0]
        let mut stock = [3, 2, 1, 0];
        WitchesBrewGame::brew_and_update_stock(&mut stock, &order);
        assert_eq!(stock, [1, 0, 1, 0]);

        let order = orders[27]; //[2, 2, 2, 0]
        let mut stock = [2, 2, 2, 0];
        WitchesBrewGame::brew_and_update_stock(&mut stock, &order);
        assert_eq!(stock, [0, 0, 0, 0]);

        let order = orders[7]; //[0, 5, 0, 0]
        let mut stock = [0, 10, 0, 0];
        WitchesBrewGame::brew_and_update_stock(&mut stock, &order);
        assert_eq!(stock, [0, 5, 0, 0]);
    }

    #[test]
    fn test_valid_moves() {
        /* Round 1 */
        let player_stock = [3, 0, 0, 0];
        let player_spells = [
            WitchesBrewGame::find_spell(&[2, 0, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -1, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 0, -1, 1]).unwrap(),
        ];
        let tome = [
            WitchesBrewGame::find_spell(&[-5, 0, 0, 2]).unwrap(),
            WitchesBrewGame::find_spell(&[-3, 1, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -3, 0, 2]).unwrap(),
            WitchesBrewGame::find_spell(&[2, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, -1, 0, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[1, 1, 0, 0]).unwrap(),
        ];
        let orders = [
            WitchesBrewGame::find_order(&[-2, 0, 0, -2]).unwrap(),
            WitchesBrewGame::find_order(&[0, 0, -2, -2]).unwrap(),
            WitchesBrewGame::find_order(&[-1, -1, -1, -3]).unwrap(),
            WitchesBrewGame::find_order(&[-2, 0, 0, -3]).unwrap(),
            WitchesBrewGame::find_order(&[-3, -1, -1, -1]).unwrap(),
        ];

        let valid_moves =
            WitchesBrewGame::valid_moves(&orders, &tome, &player_spells, &player_stock);
        let expected_moves = [
            Move::CAST(player_spells[0].id, 1),
            Move::CAST(player_spells[1].id, 1),
            Move::LEARN(tome[0].id),
            Move::LEARN(tome[1].id),
            Move::LEARN(tome[2].id),
            Move::LEARN(tome[3].id),
        ];

        assert_vec_eq!(valid_moves.slice(), &expected_moves);

        /* Round 5 */
        let player_stock = [0, 1, 0, 1];
        let mut player_spells = [
            WitchesBrewGame::find_spell(&[2, 0, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -1, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 0, -1, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -3, 0, 2]).unwrap(),
            WitchesBrewGame::find_spell(&[-3, 1, 1, 0]).unwrap(),
        ];
        player_spells[0].active = false;
        player_spells[3].active = false;
        player_spells[5].active = false;

        let tome = [
            WitchesBrewGame::find_spell(&[-5, 0, 0, 2]).unwrap(),
            WitchesBrewGame::find_spell(&[2, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, -1, 0, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[1, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[2, -2, 0, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[-4, 0, 2, 0]).unwrap(),
        ];
        let orders = [
            WitchesBrewGame::find_order(&[-2, 0, 0, -2]).unwrap(),
            WitchesBrewGame::find_order(&[0, 0, -2, -2]).unwrap(),
            WitchesBrewGame::find_order(&[-1, -1, -1, -3]).unwrap(),
            WitchesBrewGame::find_order(&[-2, 0, 0, -3]).unwrap(),
            WitchesBrewGame::find_order(&[-3, -1, -1, -1]).unwrap(),
        ];

        let valid_moves =
            WitchesBrewGame::valid_moves(&orders, &tome, &player_spells, &player_stock);
        let expected_moves = [
            Move::CAST(player_spells[2].id, 1),
            Move::LEARN(tome[0].id),
            Move::REST,
        ];
        assert_vec_eq!(valid_moves.slice(), &expected_moves);

        /* Round 13 */
        let player_stock = [2, 1, 1, 2];
        let mut player_spells = [
            WitchesBrewGame::find_spell(&[2, 0, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -1, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 0, -1, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -3, 0, 2]).unwrap(),
            WitchesBrewGame::find_spell(&[-3, 1, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-5, 0, 0, 2]).unwrap(),
        ];
        player_spells[0].active = false;
        player_spells[1].active = false;
        player_spells[2].active = false;
        player_spells[3].active = false;

        let tome = [
            WitchesBrewGame::find_spell(&[2, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, -1, 0, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[1, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[2, -2, 0, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[-4, 0, 2, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[3, -1, 0, 0]).unwrap(),
        ];
        let orders = [
            WitchesBrewGame::find_order(&[-2, 0, 0, -2]).unwrap(),
            WitchesBrewGame::find_order(&[0, 0, -2, -2]).unwrap(),
            WitchesBrewGame::find_order(&[-1, -1, -1, -3]).unwrap(),
            WitchesBrewGame::find_order(&[-2, 0, 0, -3]).unwrap(),
            WitchesBrewGame::find_order(&[-3, -1, -1, -1]).unwrap(),
        ];

        let valid_moves =
            WitchesBrewGame::valid_moves(&orders, &tome, &player_spells, &player_stock);
        let expected_moves = [
            Move::LEARN(tome[0].id),
            Move::LEARN(tome[1].id),
            Move::LEARN(tome[2].id),
            Move::BREW(orders[0].id),
            Move::REST,
        ];
        assert_vec_eq!(valid_moves.slice(), &expected_moves);

        /* Round 17 */
        let player_stock = [0, 3, 1, 0];
        let mut player_spells = [
            WitchesBrewGame::find_spell(&[2, 0, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -1, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 0, -1, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 3, 0, -1]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 0, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-2, 0, -1, 2]).unwrap(),
        ];
        let mut tome = [
            WitchesBrewGame::find_spell(&[0, -3, 3, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-2, 0, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[1, 0, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[1, 1, 1, -1]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -2, 2, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 0, 0, 1]).unwrap(),
        ];
        tome[0].tax = 3;
        tome[1].tax = 1;
        tome[2].tax = 1;

        let orders = [
            WitchesBrewGame::find_order(&[0, -2, -2, -2]).unwrap(),
            WitchesBrewGame::find_order(&[0, -2, 0, -2]).unwrap(),
            WitchesBrewGame::find_order(&[0, 0, -2, -3]).unwrap(),
            WitchesBrewGame::find_order(&[-1, -1, -1, -1]).unwrap(),
            WitchesBrewGame::find_order(&[0, 0, 0, -4]).unwrap(),
        ];

        let valid_moves =
            WitchesBrewGame::valid_moves(&orders, &tome, &player_spells, &player_stock);
        let expected_moves = [
            Move::CAST(player_spells[0].id, 1),
            Move::CAST(player_spells[2].id, 1),
            Move::CAST(player_spells[3].id, 1),
            Move::CAST(player_spells[5].id, 1),
            Move::LEARN(tome[0].id),
        ];
        assert_vec_eq!(valid_moves.slice(), &expected_moves);
    }

    #[test]
    fn test_valid_moves_multicast() {
        /* Round 1 */
        let player_stock = [0, 0, 1, 2];
        let mut player_spells = [
            WitchesBrewGame::find_spell(&[2, 0, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-1, 1, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, -1, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 0, -1, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[1, 0, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[-3, 0, 0, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[2, 2, 0, -1]).unwrap(),
        ];
        player_spells[0].active = false;
        player_spells[1].active = false;
        player_spells[2].active = false;
        player_spells[3].active = false;
        player_spells[4].active = false;
        player_spells[5].active = false;
        let tome = [
            WitchesBrewGame::find_spell(&[0, 2, 0, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[3, -2, 1, 0]).unwrap(),
            WitchesBrewGame::find_spell(&[0, 3, 2, -2]).unwrap(),
            WitchesBrewGame::find_spell(&[2, -2, 0, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[-4, 0, 1, 1]).unwrap(),
            WitchesBrewGame::find_spell(&[-2, 0, -1, 2]).unwrap(),
        ];
        let orders = [
            WitchesBrewGame::find_order(&[0, -5, 0, 0]).unwrap(),
            WitchesBrewGame::find_order(&[0, -2, -2, -2]).unwrap(),
            WitchesBrewGame::find_order(&[0, -2, -3, 0]).unwrap(),
            WitchesBrewGame::find_order(&[-1, -1, -1, -1]).unwrap(),
            WitchesBrewGame::find_order(&[-3, 0, -2, 0]).unwrap(),
        ];

        let valid_moves =
            WitchesBrewGame::valid_moves(&orders, &tome, &player_spells, &player_stock);

        let expected_moves = [
            Move::CAST(player_spells[6].id, 1),
            Move::CAST(player_spells[6].id, 2),
            Move::LEARN(tome[0].id),
            Move::REST,
        ];
        assert_vec_eq!(valid_moves.slice(), &expected_moves);
    }
}
