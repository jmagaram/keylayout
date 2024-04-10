use std::time::Instant;

use dictionary::Dictionary;

use item_count::ItemCount;
use keyboard::Keyboard;
use partitions::Partitions;
use penalty::Penalty;
use permutable::Permutable;
use set32::Set32;

mod dictionary;
mod experiment;
mod frequency;
mod item_count;
mod keyboard;
mod partitions;
mod penalty;
mod permutable;
mod set32;
mod u5;
mod utility;
mod word;

fn calc_subsets(print_each: bool, max_items: u32) {
    let start = Instant::now();
    (1..=max_items).for_each(|item_count| {
        println!("");
        println!("== Items: {} ==", item_count);
        let set = Set32::fill(item_count);
        let mut subsets_found = 0;
        (1..=item_count).for_each(|subset_size| {
            println!("  items:{} choose:{}", item_count, subset_size,);
            set.subsets_of_size(subset_size.into()).for_each(|subset| {
                subsets_found += 1;
                match print_each {
                    true => println!("     {}", subset.to_string()),
                    false => (),
                }
            });
        });
        let duration = start.elapsed();
        println!("     subsets:{} duration: {:?}", subsets_found, duration);
    });
}

fn use_dictionary() {
    let dict = Dictionary::load_large_dictionary().with_top_n_words(100);
    dict.words().iter().for_each(|w| println!("{}", w));
}

fn how_to_spell_words() {
    let dict = Dictionary::load_large_dictionary().with_top_n_words(100);
    let keyboard = Keyboard::with_layout(&dict, "abc,def,ghi,jkl,mnop,qrst,uv,wx,yz'"); // error!
    dict.words().iter().for_each(|w| {
        let spell = keyboard.spell(&dict, w);
        println!("{} : {}", w, spell);
    })
}

fn try_keyboard() {
    let dictionary = Dictionary::load_large_dictionary();
    let k = Keyboard::with_layout(&dictionary, "abc,def,ghi,jkl");
    let keys = dictionary.letters().count_items();
    let partitions = Partitions {
        sum: keys.into(),
        parts: 10,
        min: 1,
        max: 5,
    }
    .permute();
    partitions.iter().for_each(|p| println!("{:?}", p))
    // divide a set into those pieces
    //
    // let aa = keys.subsets_of_size(size)
    // let keyboard = Keyboard::new(vec![])
}

fn find_best_two_key_keyboard() {
    let d = Dictionary::load_large_dictionary();
    let mut best = Penalty::MAX;
    for first_key in d.letters().subsets_of_size(13) {
        let second_key = d.letters().difference(first_key);
        let k = Keyboard::new(vec![first_key, second_key]);
        let penalty = k.penalty(&d, best);
        if penalty.to_f32() < best.to_f32() {
            best = penalty;
            println!("{} {}", k.format(&d), penalty);
        }
    }
}

fn main() {
    calc_subsets(false, 12);
    use_dictionary();
    try_keyboard();
    how_to_spell_words();
    find_best_two_key_keyboard();
}
