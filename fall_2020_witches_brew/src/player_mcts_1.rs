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
    use rand::seq::SliceRandom;

    pub const MAX_NB_MOVES: usize = 10;

    /* #region [Structs] */
    #[derive(Clone)]
    pub struct State {
        // States per player
        pub players: [Player; 2],

        // Global states
        pub orders: [Option<Order>; 5],

        pub active: bool,
        pub active_player: u8,
        pub turn: u8,
        pub winners: Option<(WinLossTie, WinLossTie)>,
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub enum Move {
        NONE,
        WAIT,
        REST,
        BREW(u32),
        CAST(u32),
    }

    #[derive(Clone, Copy)]
    pub struct Player {
        pub move_: Move,
        pub stock: Ingredients,
        pub empty_slots: i8,
        pub spells: [Spell; 4],
        pub rupees: u32,
        pub brewed_potions_count: u8,
    }

    pub type GameScore = [f32; 4];

    pub type Ingredients = [i8; 4];

    #[derive(Copy, Clone)]
    pub struct Order {
        pub id: u32,
        pub ingredients: Ingredients,
        pub delta_stock: i8,
        pub price: u8,
    }

    #[derive(Copy, Clone, Default)]
    pub struct Spell {
        pub id: u32,
        pub ingredients: Ingredients,
        pub delta_stock: i8,
        pub active: bool,
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

    #[derive(Clone)]
    pub enum WinLossTie {
        Win,
        Loss,
        Tie,
    }
    /* #endregion */

    /* #region [Public functions] */

    pub fn new() -> State {
        let player = Player {
            move_: Move::NONE,
            stock: [0, 0, 0, 0],
            empty_slots: 10,
            spells: [Default::default(); 4],
            rupees: 0,
            brewed_potions_count: 0,
        };

        State {
            // States per player
            players: [player; 2],

            // Global states
            orders: [None; 5],

            active: true,
            active_player: 0,
            turn: 0,
            winners: None,
        }
    }

    pub fn update_state(state: &mut State, player: u8, _move: Move) {
        /* (2) Record the move */
        state.players[player as usize].move_ = _move;

        /* (3) If it's player'1 turn, i.e. both players have played =>  update the state */
        if player == 1 {
            /* 3.2 Update the state */
            // For each player move
            let mut orders_to_remove: [Option<usize>; 2] = [None, None];
            for player in state.players.iter_mut() {
                /* If it's a BREW move */
                if let Move::BREW(order_id) = player.move_ {
                    let fullfilled_order_idx = get_order_idx(&state.orders, order_id).unwrap();

                    let fullfilled_order = &state.orders[fullfilled_order_idx].unwrap();

                    // Update the player's potion count
                    player.brewed_potions_count += 1;

                    // Update the player's rupees
                    player.rupees += fullfilled_order.price as u32;

                    // Update the player's ingredient stock
                    update_ingredient_stock(&mut player.stock, &fullfilled_order.ingredients);

                    // Update the player's empty slots
                    player.empty_slots -= fullfilled_order.delta_stock;

                    if orders_to_remove[0] == None {
                        orders_to_remove[0] = Some(fullfilled_order_idx);
                    } else if fullfilled_order_idx != orders_to_remove[0].unwrap() {
                        orders_to_remove[1] = Some(fullfilled_order_idx);
                    }

                /* If it's a CAST move */
                } else if let Move::CAST(spell_id) = player.move_ {
                    let cast_spell_idx = get_spell_idx(&player.spells, spell_id).unwrap();

                    let cast_spell = &mut player.spells[cast_spell_idx];

                    // Update the player's ingredient stock
                    update_ingredient_stock(&mut player.stock, &cast_spell.ingredients);

                    // Update the player's empty slots
                    player.empty_slots -= cast_spell.delta_stock;

                    // Spell is now exhausted
                    cast_spell.active = false;

                /* If it's a REST move */
                } else if Move::REST == player.move_ {
                    for spell in player.spells.iter_mut() {
                        spell.active = true;
                    }
                }
            }

            // Remove fullfilled orders and create new one in their place
            for i in 0..2 {
                if let Some(oix) = orders_to_remove[i] {
                    state.orders[oix] = None;
                }
            }

            /* 3.3 Check terminal condition */
            let player0: &Player = &state.players[0];
            let player1: &Player = &state.players[1];

            if player0.brewed_potions_count == 3
                || player1.brewed_potions_count == 3
                || state.turn == 100
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

    pub fn valid_moves(state: &State) -> (u8, StackVector<Move, 10>) {
        let orders = &state.orders;
        let player = &state.players[state.active_player as usize];
        let spells = &player.spells;
        let stock = &player.stock;
        let empty_slots = player.empty_slots;

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

        (state.active_player, valid_moves)
    }

    pub fn random_valid_move(state: &State) -> (u8, Move) {
        let valid_moves = valid_moves(state);

        let chosen_move = valid_moves.1.get().choose(&mut rand::thread_rng()).unwrap();

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

    /* #region(collapsed) [Private functions] */

    fn get_order_idx(orders: &[Option<Order>; 5], order_id: u32) -> Option<usize> {
        for i in 0..orders.len() {
            if let Some(o) = orders[i] {
                if o.id == order_id {
                    return Some(i);
                }
            }
        }
        None
    }

    fn get_spell_idx(spells: &[Spell; 4], spell_id: u32) -> Option<usize> {
        for i in 0..spells.len() {
            if spells[i].id == spell_id {
                return Some(i);
            }
        }
        None
    }

    fn update_ingredient_stock(ingredient_stock: &mut Ingredients, order_reqs: &Ingredients) {
        for i in 0..4 {
            ingredient_stock[i] += order_reqs[i];
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
        pub fn best_move(&mut self, root_state: &game::State, player: u8) -> game::Move {
            /*
                Find the best move
                - Starting from State [state],
                - And already given (for optimization) the [valid_moves] that [player] can do
            */

            //eprintln!("[MCTS] init");
            let start = Instant::now();
            self.init(player);

            while (start.elapsed().as_millis() < TIME_LIMIT_MS)
                & (self.len < MAX_NODE_COUNT - game::MAX_NB_MOVES)
            {
                let mut state = root_state.clone();

                //eprintln!("[MCTS] Selection");

                let selected_node_idx = self.select(&mut state);

                //eprintln!("[MCTS] Expansion");
                let rollout_node_idx = self.expand(selected_node_idx, &mut state);

                //eprintln!("[MCTS] Simulation");
                let score = self.simulate(&mut state);

                self.backpropagate(rollout_node_idx, score);

                self.nb_simulations += 1;
            }

            eprintln!(
                "[MCTS P5] End. Sending best move after expanding {} nodes and running {} simulations in {:?}",
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

        fn select(&self, state: &mut game::State) -> usize {
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

        fn expand(&mut self, selected_node_idx: usize, state: &mut game::State) -> usize {
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
                let (player, valid_moves) = game::valid_moves(state);

                let child_first = self.len;
                let child_count = valid_moves.len;
                selected_node.child_first = Some(child_first);
                selected_node.child_count = child_count as u8;
                for m in valid_moves.get() {
                    self.create_child(selected_node_idx, *m, player)
                }

                //2. Choose a random child, expand it and return it
                let chosen_child_idx =
                    rand::thread_rng().gen_range(child_first..child_first + child_count);
                self.arr[chosen_child_idx].expanded = true;

                game::update_state(state, player, self.arr[chosen_child_idx].move_.unwrap());

                return chosen_child_idx;
            }
        }

        fn simulate(&self, state: &mut game::State) -> game::GameScore {
            // Simulate the game until the end
            while !game::is_terminal(state) {
                let (player, chosen_move) = game::random_valid_move(state);

                game::update_state(state, player, chosen_move);
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
    msg_snd: Sender<String>,
    params: Option<Vec<String>>,
) {
    // game loop
    while ctr_rcv.recv().unwrap() == true {
        /* Initialize players & orders */
        let mut state = game::new();
        let mut mcts: mcts::MCTS = mcts::new();

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
                state.orders[order_count] = Some(game::Order {
                    id: action_id,
                    ingredients: [delta_0, delta_1, delta_2, delta_3],
                    delta_stock: delta_0 + delta_1 + delta_2 + delta_3,
                    price: price,
                });
                order_count += 1;
            } else if action_type == String::from("CAST") {
                state.players[0].spells[my_spell_count] = game::Spell {
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
                state.players[1].spells[other_spell_count] = game::Spell {
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
            let score = parse_input!(inputs[4], u32); // amount of rupees

            if i == 0 {
                state.players[0].stock = [inv_0, inv_1, inv_2, inv_3];
                state.players[0].empty_slots = 10 - (inv_0 + inv_1 + inv_2 + inv_3);
                state.players[0].rupees = score;
            } else {
                state.players[1].stock = [inv_0, inv_1, inv_2, inv_3];
                state.players[1].empty_slots = 10 - (inv_0 + inv_1 + inv_2 + inv_3);
                state.players[1].rupees = score;
            }
        }

        // (3) Determine the next best action
        let best_move = mcts.best_move(&state, 0);

        msg_snd.send(format!("{}", best_move.to_string()));
    }
}
