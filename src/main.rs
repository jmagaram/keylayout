use std::time::Instant;

use dictionary::Dictionary;

use key::Key;
use solvers::genetic_threaded;

use crate::permutable::Permutable;

mod dictionary;
mod experiment;
mod frequency;
mod item_count;
mod key;
mod keyboard;
mod letter;
mod partitions;
mod penalty;
mod permutable;
mod solvers;
mod util;
mod utility;
mod word;

fn calc_subsets(print_each: bool, max_items: u32) {
    let start = Instant::now();
    (1..=max_items).for_each(|item_count| {
        println!("");
        println!("== Items: {} ==", item_count);
        let set = Key::with_first_n_letters(item_count);
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

fn find_best_keyboard() -> () {
    let dict = Dictionary::load_large_dictionary();
    let alphabet = dict.alphabet();
    let layouts = partitions::Partitions {
        sum: alphabet.count_items(),
        parts: 10,
        min: 2,
        max: 5,
    };
    for lay in layouts.permute() {
        let distribution = item_count::with_u32_groups(&lay);
        for dist in distribution.permute() {
            // looking for a 5,5,4,4,2,2 set of keys
            // alphabet.subsets_of_size(5)
            // then subsets of 5
            // then subsets of 4
            // then subsets of 4
            // alphabet.subsets_of_size(5);
            println!("dist!")
        }
    }
    println!("done");
}

fn main() {
    genetic_threaded(8);
}
