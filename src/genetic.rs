use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    permutable::Permutable, solution::Solution,
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
    loop {
        let mut current_best = best.clone();
        for child in best.keyboard().every_swap() {
            let child_penalty = child.penalty(args.dictionary, current_best.penalty());
            if child_penalty < current_best.penalty() {
                current_best = Solution::new(child.clone(), child_penalty);
                if args.verbose_print {
                    println!("  > {}", current_best);
                }
            }
        }
        let progress_made = (current_best.penalty().to_f32() - best.penalty().to_f32()).abs()
            > args.die_threshold.to_f32();
        if current_best.penalty() < best.penalty() {
            best = current_best;
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
    .permute();
    let random_layout = || {
        let random_index: usize = rand::random::<usize>().rem_euclid(layouts.len());
        layouts.get(random_index).unwrap()
    };
    let layout = random_layout();
    let keys = alphabet.random_subsets(layout).collect::<Vec<Key>>();
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
