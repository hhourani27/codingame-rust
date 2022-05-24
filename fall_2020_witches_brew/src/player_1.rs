use std::sync::mpsc::{Receiver, Sender};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

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

    pub type Ingredients = [i32; 4];

    #[derive(Copy, Clone, Default)]
    pub struct Order {
        pub id: i32,
        pub ingredients: Ingredients,
        pub price: i32,
    }

    pub fn can_fulfill_order(order: &Order, ingredient_stock: &Ingredients) -> bool {
        ingredient_stock[0] >= -order.ingredients[0]
            && ingredient_stock[1] >= -order.ingredients[1]
            && ingredient_stock[2] >= -order.ingredients[2]
            && ingredient_stock[3] >= -order.ingredients[3]
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
        let mut current_orders: StackVector<game::Order, 5> = StackVector {
            arr: [Default::default(); 5],
            len: 0,
        };

        let mut my_stock: game::Ingredients = [0, 0, 0, 0];

        let mut input_line = String::new();
        input_line = msg_rcv.recv().unwrap();
        let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play
        for i in 0..action_count as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let action_id = parse_input!(inputs[0], i32); // the unique ID of this spell or recipe
            let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW
            let delta_0 = parse_input!(inputs[2], i32); // tier-0 ingredient change
            let delta_1 = parse_input!(inputs[3], i32); // tier-1 ingredient change
            let delta_2 = parse_input!(inputs[4], i32); // tier-2 ingredient change
            let delta_3 = parse_input!(inputs[5], i32); // tier-3 ingredient change
            let price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
            let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax; For brews, this is the value of the current urgency bonus
            let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell; For brews, this is how many times you can still gain an urgency bonus
            let castable = parse_input!(inputs[9], i32); // in the first league: always 0; later: 1 if this is a castable player spell
            let repeatable = parse_input!(inputs[10], i32); // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

            current_orders.add(game::Order {
                id: action_id,
                ingredients: [delta_0, delta_1, delta_2, delta_3],
                price: price,
            });
        }

        for i in 0..2 as usize {
            let mut input_line = String::new();
            input_line = msg_rcv.recv().unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let inv_0 = parse_input!(inputs[0], i32); // tier-0 ingredients in inventory
            let inv_1 = parse_input!(inputs[1], i32);
            let inv_2 = parse_input!(inputs[2], i32);
            let inv_3 = parse_input!(inputs[3], i32);
            let score = parse_input!(inputs[4], i32); // amount of rupees

            if i == 0 {
                my_stock = [inv_0, inv_1, inv_2, inv_3]
            }
        }

        // in the first league: BREW <id> | WAIT; later: BREW <id> | CAST <id> [<times>] | LEARN <id> | REST | WAIT

        let mut largest_order: Option<game::Order> = None;
        for order in current_orders.get().iter() {
            if game::can_fulfill_order(order, &my_stock) {
                if largest_order.is_none() || order.price > largest_order.unwrap().price {
                    largest_order = Some(order.clone());
                }
            }
        }

        let msg = match largest_order {
            None => format!("WAIT"),
            Some(o) => format!("BREW {}", o.id),
        };

        msg_snd.send(msg);
    }
}
