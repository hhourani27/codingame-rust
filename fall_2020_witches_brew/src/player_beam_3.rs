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

mod game {}

mod beam {}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(
    ctr_rcv: Receiver<bool>,
    msg_rcv: Receiver<String>,
    msg_snd: Sender<(String, Option<std::collections::HashMap<String, String>>)>,
    params: Option<Vec<String>>,
) {
    /*
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
        let mut first_nodes = mcts.get_first_nodes_simulation_results();

        first_nodes.sort_by(|n1, n2| {
            (n1.2 / n1.1 as f32)
                .partial_cmp(&(n2.2 / n2.1 as f32))
                .unwrap()
        });
        first_nodes.reverse();

        for (i, (move_, visits, score)) in first_nodes.iter().enumerate() {
            player_state.insert(
                format!("({:02}) {}", i + 1, move_.to_string()),
                format!("{:.2} {} {}", score / *visits as f32, *visits, *score),
            );
        }

        /* #endregion */

        let msg = best_move_cg.to_string();
        msg_snd.send((format!("{}", msg), Some(player_state)));
    }
    */
}
