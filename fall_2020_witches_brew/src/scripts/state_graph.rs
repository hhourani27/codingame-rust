use common::graph::Graph;
use std::fmt::Display;

/*
Generate a DOT graph where :
- There are 1001 stock state nodes
- Edges correspond to SPELL CASTS or BREW orders
*/

#[derive(Copy, Clone, Eq, PartialEq)]
enum Edge {
    CAST(u32),
    BREW(u32),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Node(Stock);

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stock = self.0;
        write!(
            f,
            "[{}, {}, {}, {}]",
            stock[0], stock[1], stock[2], stock[3]
        )
    }
}

impl Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Edge::CAST(i) => write!(f, "C {}", i),
            Edge::BREW(i) => write!(f, "B {}", i),
        }
    }
}

fn get_state_graph() -> Graph<Node, Edge> {
    let mut graph: Graph<Node, Edge> = Graph::new();

    /* (1) Create nodes */
    let map_stockArr4_stockId = get_map_stockArr4_stockId().0;
    for t0 in 0..=10 {
        for t1 in 0..=(10 - t0) {
            for t2 in 0..=(10 - t0 - t1) {
                for t3 in 0..=(10 - t0 - t1 - t2) {
                    let stock_id: StockId = map_stockArr4_stockId[t0][t1][t2][t3];
                    graph.add_node(Node([t0 as i8, t1 as i8, t2 as i8, t3 as i8]));
                }
            }
        }
    }

    /* (2) Create edges */
    let orders = get_all_orders();
    let spells = get_all_tome_spells();
    for t0 in 0..=10 {
        for t1 in 0..=(10 - t0) {
            for t2 in 0..=(10 - t0 - t1) {
                for t3 in 0..=(10 - t0 - t1 - t2) {
                    let stock: Stock = [t0 as i8, t1 as i8, t2 as i8, t3 as i8];
                    let node = Node(stock);
                    let stock_id: StockId = map_stockArr4_stockId[t0][t1][t2][t3];

                    /* (2.1) ADD BREW EDGES */
                    for order in orders.iter() {
                        if can_fulfill_order(order, &stock) {
                            let mut end_stock = stock.clone();
                            update_stock(&mut end_stock, &order.recipe);
                            let end_node = Node(end_stock);
                            let end_stock_id = map_stockArr4_stockId[end_stock[0] as usize]
                                [end_stock[1] as usize][end_stock[2] as usize]
                                [end_stock[3] as usize];

                            graph.add_edge(node, end_node, Edge::BREW(order.id));
                        }
                    }

                    /* (2.2) ADD CAST EDGES */
                    for spell in spells.iter() {
                        if can_cast_spell(spell, &stock) {
                            let mut end_stock = stock.clone();
                            update_stock(&mut end_stock, &spell.recipe);
                            let end_node = Node(end_stock);
                            let end_stock_id = map_stockArr4_stockId[end_stock[0] as usize]
                                [end_stock[1] as usize][end_stock[2] as usize]
                                [end_stock[3] as usize];

                            graph.add_edge(node, end_node, Edge::CAST(spell.id));
                        }
                    }
                }
            }
        }
    }
    graph
}

pub fn compute_shortest_paths() {
    let state_graph = get_state_graph();

    let orders = get_all_orders();

    let stock = [0, 0, 0, 0];
    let node = Node(stock);
    let order_id = 20;

    let path = state_graph
        .bfs_node_to_edge(&node, &Edge::BREW(order_id))
        .unwrap()
        .unwrap();

    println!("{:?}", stock);
    for (e, n) in path.iter() {
        println!("{} -> {}", e, n);
    }
}

pub fn print_state_graph(file_path: &str) {
    let state_graph = get_state_graph();

    println!("Node count : {}", state_graph.node_count());
    println!("Edge count : {}", state_graph.edge_count());

    state_graph.print_dot(file_path);
}

/* #region(collapsed) [Helper functions & structs] */

type Recipe = [i8; 4];

type Stock = [i8; 4];
type StockId = usize;

#[derive(Copy, Clone, Default)]
struct Order {
    id: u32,
    recipe: Recipe,
    price: u8,
    bonus: u8,
}

#[derive(Copy, Clone, Default, Debug)]
struct Spell {
    id: u32,
    recipe: Recipe,
    delta_stock: i8,
    tax: u8,
    repeatable: bool,
    active: bool,
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

fn get_map_stockArr4_stockId() -> (Vec<Vec<Vec<Vec<usize>>>>, [[usize; 4]; 1001]) {
    let mut map_stockArr4_stockId: Vec<Vec<Vec<Vec<usize>>>> =
        vec![vec![vec![vec![0; 11]; 11]; 11]; 11];
    let mut map_stockId_stockArr4: [[usize; 4]; 1001] = [[0; 4]; 1001];

    let mut id = 0;
    for t0 in 0..=10 {
        for t1 in 0..=(10 - t0) {
            for t2 in 0..=(10 - t0 - t1) {
                for t3 in 0..=(10 - t0 - t1 - t2) {
                    map_stockArr4_stockId[t0][t1][t2][t3] = id;
                    map_stockId_stockArr4[id] = [t0, t1, t2, t3];

                    id += 1;
                }
            }
        }
    }

    (map_stockArr4_stockId, map_stockId_stockArr4)
}

fn can_fulfill_order(order: &Order, stock: &Stock) -> bool {
    stock[0] >= -order.recipe[0]
        && stock[1] >= -order.recipe[1]
        && stock[2] >= -order.recipe[2]
        && stock[3] >= -order.recipe[3]
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

fn update_stock(stock: &mut Stock, recipe: &Recipe) {
    for i in 0..4 {
        stock[i] += recipe[i];
    }
}

/* #endregion */
