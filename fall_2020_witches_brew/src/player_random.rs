use rand::seq::SliceRandom;
use std::sync::mpsc::{Receiver, Sender};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[allow(non_snake_case)]
#[derive(Clone)]
pub struct StackVector<T: Copy + Clone + Default, const MAX_SIZE: usize> {
    pub arr: [T; MAX_SIZE],
    pub len: usize,
}

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
        for i in idx..self.len {
            self.arr[i] = self.arr[i + 1];
        }
        self.len -= 1;

        removed_element
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

mod game {

    use super::StackVector;

    const MAX_VALID_MOVES: usize = 35 + 5 + 6 + 1; // 35 CAST + 5 BREW + 6 LEARN + 1 REST

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

    pub type Ingredients = [i8; 4];

    #[derive(Copy, Clone, Default)]
    pub struct Order {
        pub id: u32,
        pub recipe: Ingredients,
        pub price: u8,
        pub bonus: u8,
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct Spell {
        pub id: u32,
        pub recipe: Ingredients,
        pub delta_stock: i8,
        pub tax: u8,
        pub repeatable: bool,
        pub active: bool,
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

    fn cast_and_update_stock(stock: &mut Ingredients, recipe: &Ingredients, times: u8) {
        for _ in 0..times {
            for i in 0..4 {
                stock[i] += recipe[i];
            }
        }
    }

    /* Return how many times the spell can be cast */
    fn how_many_times_can_cast_spell(spell: &Spell, stock: &Ingredients) -> u8 {
        if spell.active == false {
            return 0;
        }

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

    pub fn valid_moves(
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
            if can_fulfill_order(order, stock) {
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
                let times_can_cast_spell = how_many_times_can_cast_spell(sp, stock);
                if times_can_cast_spell > 0 {
                    valid_moves.add(Move::CAST(sp.id, times_can_cast_spell));
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
        let mut orders: Vec<game::Order> = Vec::new();
        let mut tome_spells_t: [Option<game::Spell>; 6] = [None; 6];
        let mut my_spells: Vec<game::Spell> = Vec::new();
        let mut my_stock: game::Ingredients = [0, 0, 0, 0];

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
                orders.push(game::Order {
                    id: action_id,
                    recipe: [delta_0, delta_1, delta_2, delta_3],
                    price: price - tome_index as u8,
                    bonus: tome_index as u8,
                });
            } else if action_type == String::from("CAST") {
                my_spells.push(game::Spell {
                    id: action_id,
                    recipe: [delta_0, delta_1, delta_2, delta_3],
                    delta_stock: delta_0 + delta_1 + delta_2 + delta_3,
                    tax: 0,
                    repeatable: match repeatable {
                        1 => true,
                        0 => false,
                        _ => panic!(),
                    },
                    active: match castable {
                        1 => true,
                        0 => false,
                        _ => panic!(),
                    },
                })
            } else if action_type == String::from("OPPONENT_CAST") {
            } else if action_type == String::from("LEARN") {
                tome_spells_t[tome_index as usize] = Some(game::Spell {
                    id: action_id,
                    recipe: [delta_0, delta_1, delta_2, delta_3],
                    delta_stock: delta_0 + delta_1 + delta_2 + delta_3,
                    tax: tax_count as u8,
                    repeatable: match repeatable {
                        1 => true,
                        0 => false,
                        _ => panic!(),
                    },
                    active: match castable {
                        1 => true,
                        0 => false,
                        _ => panic!(),
                    },
                });
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
                my_stock = [inv_0, inv_1, inv_2, inv_3];
            }
        }

        let mut tome_spells: Vec<game::Spell> = Vec::new();
        for i in 0..6 {
            if tome_spells_t[i].is_none() {
                break;
            }
            tome_spells.push(tome_spells_t[i].unwrap().clone())
        }

        let valid_moves = game::valid_moves(&orders, &tome_spells, &my_spells, &my_stock);

        let chosen_move = valid_moves.slice().choose(&mut rand::thread_rng()).unwrap();
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
