pub struct StackVector<T: Copy + Clone + Default, const MAX_SIZE: usize> {
    pub arr: [T; MAX_SIZE],
    pub len: usize,
}

pub enum Move {
    NONE,
    WAIT,
    REST,
    BREW(u32),
    CAST(u32, u8),
    LEARN(u32),
}

const EXISTING_SPELL_COUNT: usize = 42 + 4;
const EXISTING_ORDER_COUNT: usize = 36;

type Stock = [i8; 4];
type Recipe = [i8; 4];

struct Order {
    pub id: u32,
    pub recipe: Recipe,
    pub price: u8,
}

struct Spell {
    pub id: u32,
    pub recipe: Recipe,
    pub delta_stock: i8,
    pub repeatable: bool,
}

pub struct Player {
    pub stock: Stock,
    pub stock_id: usize,
    pub spells: StackVector<(u32, bool), EXISTING_SPELL_COUNT>,
    pub rupees: u8,
    pub brewed_potions_count: u8,
}

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

struct Node {
    move_: Move,
    //state: State,
    parent: Option<usize>,
    child_first: Option<usize>,
    child_count: usize,
    depth: usize,
    eval: f32,
}

struct Cache {
    map_stockArr4_stockId: [[[[usize; 11]; 11]; 11]; 11],
    map_stockId_timesCanCastSpell: [[u8; EXISTING_SPELL_COUNT]; 1001],
    map_stockId_canFullfillOrder: [[bool; EXISTING_ORDER_COUNT]; 1001],

    map_orderId_Order: [Order; EXISTING_ORDER_COUNT],
    map_spellId_Spell: [Spell; EXISTING_SPELL_COUNT],
}

pub fn run() {
    println!("Memory taken by Node: {}", std::mem::size_of::<Node>());
    println!("Memory taken by Cache: {} ", std::mem::size_of::<Cache>());
}
