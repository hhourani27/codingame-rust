#![allow(warnings, unused)]

#[derive(Copy, Clone, PartialEq, Debug)]
enum Move {
    NONE,
    WAIT,
    REST,
    BREW(u32),
    CAST(u32, u8),
    LEARN(u32),
}

#[derive(Copy, Clone, Default)]
struct Spell {
    id: u32,
    recipe: Ingredients,
    delta_stock: i8,
    tax: u8,
    repeatable: bool,
    active: bool,
}

type Ingredients = [i8; 4];

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

fn main() {
    let mut all_spells: Vec<Spell> = get_tome_spells();
    all_spells.extend_from_slice(&get_basic_spells());
    println!("We have {} spells", all_spells.len());

    // Calculate the max number of spells one can cast given any quantity of stock
    let mut max_spell = 0;
    let mut max_stock: Ingredients = [0, 0, 0, 0];
    let mut max_castable_spells: Vec<Move> = Vec::new();

    let mut possible_stock_states = 0;
    for t0 in 0..=10 {
        for t1 in 0..=(10 - t0) {
            for t2 in 0..=(10 - t0 - t1) {
                for t3 in 0..=(10 - t0 - t1 - t2) {
                    possible_stock_states += 1;
                    let stock = [t0, t1, t2, t3];
                    println!("{:?}", stock);

                    let mut castable_spell_count = 0;
                    let mut castable_spells: Vec<Move> = Vec::new();
                    for spell in all_spells.iter() {
                        let times = how_many_times_can_cast_spell(spell, &stock);
                        castable_spell_count += times;
                        for t in 0..times {
                            castable_spells.push(Move::CAST(spell.id, t));
                        }
                    }

                    if castable_spell_count > max_spell {
                        max_spell = castable_spell_count;
                        max_castable_spells = castable_spells;
                        max_stock = stock.clone();
                    }
                }
            }
        }
    }

    println!("There are {} possible stock states", possible_stock_states);
    println!("Max # castable spells is {}", max_spell);
    println!("They are {:?}", max_castable_spells);
    println!("It is done with the stock {:?}", max_stock);
}
