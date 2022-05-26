use rand::seq::SliceRandom;
use std::sync::mpsc::{Receiver, Sender};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[allow(non_snake_case)]
pub struct StackVector<T, const MAX_SIZE: usize> {
    pub arr: [T; MAX_SIZE],
    pub len: usize,
}

impl<T, const MAX_SIZE: usize> StackVector<T, MAX_SIZE> {
    pub fn add(&mut self, e: T) {
        self.arr[self.len] = e;
        self.len += 1;
    }

    pub fn get(&self) -> &[T] {
        &self.arr[0..self.len]
    }
}

mod game {

    use super::StackVector;

    pub type Ingredients = [i8; 4];

    #[derive(Copy, Clone)]
    pub struct Order {
        pub id: u32,
        pub ingredients: Ingredients,
        pub delta_stock: i8,
        pub price: u8,
    }

    #[derive(Clone, Default)]
    pub struct Spell {
        pub id: u32,
        pub ingredients: Ingredients,
        pub delta_stock: i8,
        pub active: bool,
    }

    #[derive(Clone, Default)]
    pub struct Player {
        pub stock: Ingredients,
        pub empty_slots: i8,
        pub spells: [Spell; 4],
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub enum Move {
        NONE,
        WAIT,
        REST,
        BREW(u32),
        CAST(u32),
    }

    impl Move {
        pub fn to_string(&self) -> String {
            match self {
                Move::NONE => format!("None"),
                Move::WAIT => format!("WAIT"),
                Move::REST => format!("REST"),
                Move::BREW(i) => format!("BREW {}", i),
                Move::CAST(i) => format!("CAST {}", i),
            }
        }
    }

    fn can_fulfill_order(order: &Order, stock: &Ingredients) -> bool {
        stock[0] >= -order.ingredients[0]
            && stock[1] >= -order.ingredients[1]
            && stock[2] >= -order.ingredients[2]
            && stock[3] >= -order.ingredients[3]
    }

    fn can_cast_spell(spell: &Spell, stock: &Ingredients, empty_slots: i8) -> bool {
        if spell.delta_stock > empty_slots {
            return false;
        }
        if spell.active == false {
            return false;
        }

        for i in 0..4 {
            if spell.ingredients[i] < 0 && stock[i] < -spell.ingredients[i] {
                return false;
            }
        }

        true
    }

    pub fn valid_moves(
        orders: &[Option<Order>; 5],
        spells: &[Spell; 4],
        stock: &Ingredients,
        empty_slots: i8,
    ) -> StackVector<Move, 10> {
        // There's at max 10 possible moves : 5 orders, 4 spells + REST
        let mut valid_moves: StackVector<Move, 10> = StackVector {
            arr: [Move::NONE; 10],
            len: 0,
        };

        // Check which order the player can fulfill and add them as a valid move
        for order in orders.iter() {
            if let Some(o) = order {
                if can_fulfill_order(o, stock) {
                    valid_moves.add(Move::BREW(o.id));
                }
            }
        }

        // Check which spell the player can fulfill and add them as a valid move
        // in the meantime check if there are inactive spells
        let mut all_spells_are_active = true;
        for spell in spells.iter() {
            if spell.active == false {
                all_spells_are_active = false;
            } else {
                if can_cast_spell(spell, stock, empty_slots) {
                    valid_moves.add(Move::CAST(spell.id));
                }
            }
        }

        if all_spells_are_active == false {
            valid_moves.add(Move::REST);
        }

        // At the end, if there's no valid moves, we just send a wait
        if valid_moves.len == 0 {
            valid_moves.add(Move::WAIT);
        }

        valid_moves
    }
}

#[allow(unused_variables, unused_assignments, unused_must_use)]
pub fn play(
    ctr_rcv: Receiver<bool>,
    msg_rcv: Receiver<String>,
    msg_snd: Sender<String>,
    params: Option<Vec<String>>,
) {
    // game loop
    while ctr_rcv.recv().unwrap() == true {
        /* Initialize players & orders */
        let mut my_player: game::Player = Default::default();
        let mut other_player: game::Player = Default::default();
        let mut orders: [Option<game::Order>; 5] = [None; 5];
        let mut order_count = 0;
        let mut my_spell_count = 0;
        let mut other_spell_count = 0;

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
                orders[order_count] = Some(game::Order {
                    id: action_id,
                    ingredients: [delta_0, delta_1, delta_2, delta_3],
                    delta_stock: delta_0 + delta_1 + delta_2 + delta_3,
                    price: price,
                });
                order_count += 1;
            } else if action_type == String::from("CAST") {
                my_player.spells[my_spell_count] = game::Spell {
                    id: action_id,
                    ingredients: [delta_0, delta_1, delta_2, delta_3],
                    delta_stock: delta_0 + delta_1 + delta_2 + delta_3,
                    active: match castable {
                        1 => true,
                        _ => false,
                    },
                };
                my_spell_count += 1;
            } else if action_type == String::from("OPPONENT_CAST") {
                other_player.spells[other_spell_count] = game::Spell {
                    id: action_id,
                    ingredients: [delta_0, delta_1, delta_2, delta_3],
                    delta_stock: delta_0 + delta_1 + delta_2 + delta_3,
                    active: match castable {
                        1 => true,
                        _ => false,
                    },
                };
                other_spell_count += 1;
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
            let score = parse_input!(inputs[4], i32); // amount of rupees

            if i == 0 {
                my_player.stock = [inv_0, inv_1, inv_2, inv_3];
                my_player.empty_slots = 10 - (inv_0 + inv_1 + inv_2 + inv_3);
            } else {
                other_player.stock = [inv_0, inv_1, inv_2, inv_3];
                other_player.empty_slots = 10 - (inv_0 + inv_1 + inv_2 + inv_3);
            }
        }

        let valid_moves = game::valid_moves(
            &orders,
            &my_player.spells,
            &my_player.stock,
            my_player.empty_slots,
        );

        let chosen_move = valid_moves.get().choose(&mut rand::thread_rng()).unwrap();
        /*eprintln!(
            "valid moves: {}",
            valid_moves
                .get()
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );*/

        // in the first league: BREW <id> | WAIT; later: BREW <id> | CAST <id> [<times>] | LEARN <id> | REST | WAIT
        msg_snd.send(format!("{}", chosen_move.to_string()));
    }
}
