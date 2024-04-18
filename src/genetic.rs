use rand::Rng;

use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    solution::Solution,
};
use std::sync::mpsc;

pub struct EvolveKeyboardArgs<'a> {
    pub solution: Solution,
    pub die_threshold: Penalty,
    pub dictionary: &'a Dictionary,
    pub verbose_print: bool,
}

pub fn evolve_keyboard(args: EvolveKeyboardArgs) -> Solution {
    let mut best = args.solution;
    let mut generations = 0;
    loop {
        let mut current_best = best.clone();
        for child in best.keyboard().every_swap() {
            let child_penalty = child.penalty(args.dictionary, current_best.penalty());
            if child_penalty < current_best.penalty() {
                current_best = Solution::new(
                    child.clone(),
                    child_penalty,
                    format!("| gen {}", generations),
                );
                if args.verbose_print {
                    println!("  > {}", current_best);
                }
            }
        }
        let progress_made = (current_best.penalty().to_f32() - best.penalty().to_f32()).abs()
            > args.die_threshold.to_f32();
        if current_best.penalty() < best.penalty() {
            best = current_best;
            generations = generations + 1;
        }
        if !progress_made {
            break;
        }
    }
    best
}

pub fn find_best(
    dict: &Dictionary,
    print_best: bool,
    die_threshold: Penalty,
    verbose_print: bool,
) -> Solution {
    let alphabet = dict.alphabet();
    let layouts = Partitions {
        parts: 10,
        sum: 27,
        min: 2,
        max: 4,
    }
    .calculate();
    let layout = {
        let mut rng = rand::thread_rng();
        let layout_index = rng.gen_range(0..layouts.len());
        layouts.get(layout_index).unwrap()
    };
    let keys = alphabet.random_subsets(&layout).collect::<Vec<Key>>();
    let keyboard = Keyboard::new_from_keys(keys);
    let penalty = keyboard.penalty(&dict, Penalty::MAX);
    let args = EvolveKeyboardArgs {
        dictionary: &dict,
        solution: keyboard.with_penalty(penalty),
        verbose_print,
        die_threshold,
    };
    let best = evolve_keyboard(args);
    if print_best {
        println!("===========================================");
        println!("{}", best);
        println!("===========================================");
    }
    best
}

pub struct Args {
    pub threads: u32,
    pub die_threshold: Penalty,
    pub verbose_print: bool,
}

pub fn solve(args: Args) -> () {
    let (tx, rx) = mpsc::sync_channel::<Solution>(10);
    let mut best: Option<Solution> = None;
    for _ in 0..args.threads {
        let tx = tx.clone();
        let dictionary = Dictionary::load();
        std::thread::spawn(move || loop {
            let best = find_best(&dictionary, false, args.die_threshold, args.verbose_print);
            tx.send(best).unwrap();
        });
    }
    for solution in rx {
        match best {
            None => {
                println!("{}", solution);
                best = Some(solution);
            }
            Some(ref b) => {
                if solution.penalty() < b.penalty() {
                    println!("{}", solution);
                    best = Some(solution);
                }
            }
        }
    }
}

pub fn smarter_genetic() {
    let dict_full = Dictionary::load();
    let dict_small = Dictionary::load().with_top_n_words(50000);
    let part = Partitions {
        sum: 27,
        parts: 10,
        min: 2,
        max: 4,
    };
    let mut best: Option<Solution> = None;
    let mut seen = 1;
    let mut skipped = 0;
    let initial_screening_limit = Penalty::new(0.035);
    let die_threshold = Penalty::new(0.001);
    let results = Keyboard::random(dict_full.alphabet(), &part)
        .enumerate()
        .filter_map(|(index, k)| {
            // println!("seen {} skipped {}", seen, skipped);
            seen = seen + 1;
            let penalty_partial = k.penalty(&dict_small, Penalty::MAX);
            if penalty_partial < initial_screening_limit {
                let penalty = k.penalty(&dict_full, Penalty::MAX);
                let solution = k.with_penalty_and_notes(penalty, format!("keyboard {}", index));
                Some(solution)
            } else {
                skipped = skipped + 1;
                None
            }
        })
        .map(|s| {
            let args = EvolveKeyboardArgs {
                dictionary: &dict_full,
                die_threshold,
                solution: s,
                verbose_print: false,
            };
            evolve_keyboard(args)
        })
        .filter(move |s| {
            if s.penalty()
                < (&best)
                    .as_ref()
                    .map(|best| best.penalty())
                    .unwrap_or(Penalty::MAX)
            {
                best = Some(s.clone());
                println!("{}", s);
                true
            } else {
                false
            }
        });
    results.count();
}
