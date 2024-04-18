use std::fmt;

use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty, solution::Solution,
};

fn go(
    dict: &Dictionary,
    keyboard: Keyboard,
    max_penalty: Penalty,
    never_together: &Vec<Key>,
) -> Option<Solution> {
    println!("{}", keyboard);
    let penalty = keyboard.penalty(dict, max_penalty);
    if penalty < max_penalty {
        if keyboard.key_count() == 10 {
            Some(keyboard.with_penalty(penalty))
        } else {
            keyboard
                .every_combine_two_keys_filter(&never_together)
                .iter()
                .filter(|k| match k.max_key_size() {
                    None => true,
                    Some(k) => k <= 4,
                })
                .map(move |k| go(dict, k.clone(), max_penalty, never_together))
                .find_map(|i| i)
        }
    } else {
        None
    }
}

pub struct Args {
    pub max_penalty: Penalty,
    pub never_together: Vec<Key>,
    pub total_words: usize,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pairs = self
            .never_together
            .iter()
            .filter(|w| w.count_letters() == 2)
            .count();
        let triples = self
            .never_together
            .iter()
            .filter(|w| w.count_letters() == 3)
            .count();
        writeln!(
            f,
            "MaxPenalty:{} TotalWords:{} Pairs:{} Triples:{}",
            self.max_penalty, self.total_words, pairs, triples
        )
    }
}

pub fn solve(args: Args) {
    let d = Dictionary::load().with_top_n_words(args.total_words);
    let k = Keyboard::new_from_keys(d.alphabet().map(|r| Key::with_one_letter(r)).collect());
    let result = go(&d, k, args.max_penalty, &args.never_together);
    println!("=====================================================");
    println!("{}", args);
    match result {
        None => println!(
            "No keyboard found with maximum penalty of {}",
            args.max_penalty
        ),
        Some(keyboard) => {
            println!("{}", keyboard)
        }
    }
}
