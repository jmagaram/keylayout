use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    permutable::Permutable, solution::Solution,
};
use std::sync::mpsc;

pub struct EvolveKeyboardArgs<'a> {
    pub solution: Solution,
    pub stop_if_stuck: Penalty,
    pub dictionary: &'a Dictionary,
    pub print_progress: bool,
}

pub fn evolve_keyboard(args: EvolveKeyboardArgs) -> Solution {
    let mut best = args.solution;
    loop {
        let mut current_best = best.clone();
        for child in best.keyboard().every_swap() {
            let child_penalty = child.penalty(args.dictionary, current_best.penalty());
            if child_penalty < current_best.penalty() {
                current_best = Solution::new(child.clone(), child_penalty);
                if args.print_progress {
                    println!("{}", current_best);
                }
            }
        }
        let progress_made = (current_best.penalty().to_f32() - best.penalty().to_f32()).abs()
            > args.stop_if_stuck.to_f32();
        if current_best.penalty() < best.penalty() {
            best = current_best;
        }
        if !progress_made {
            break;
        }
    }
    best
}

pub fn find_best(dict: &Dictionary, print_best: bool) -> Solution {
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
        print_progress: false,
        stop_if_stuck: Penalty::new(0.005),
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
}

pub fn solve(args: Args) -> () {
    println!("Starting genetic solver with {} threads...", args.threads);
    let (tx, rx) = mpsc::sync_channel::<Solution>(10);
    let mut best: Option<Solution> = None;
    for _ in 0..args.threads {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let dictionary = Dictionary::load();
            loop {
                let best = find_best(&dictionary, false);
                tx.send(best).unwrap();
            }
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
