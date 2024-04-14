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
    let keyboard = Keyboard::new(keys);
    let penalty = keyboard.penalty(&dict, Penalty::MAX);
    let args = EvolveKeyboardArgs {
        dictionary: &dict,
        solution: keyboard.with_penalty(penalty),
        print_progress: false,
        stop_if_stuck: Penalty::new(0.001),
    };
    let best = evolve_keyboard(args);
    if print_best {
        println!("===========================================");
        println!("{}", best);
        println!("===========================================");
    }
    best
}

pub fn genetic_threaded(threads: u32) -> () {
    let (tx, rx) = mpsc::sync_channel::<Solution>(10);
    let mut best: Option<Solution> = None;
    for _ in 0..threads {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let dictionary = Dictionary::load_large_dictionary();
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

pub fn combine_two_dfs_worker(
    dict: &Dictionary,
    keyboard: Keyboard,
    max_penalty: Penalty,
) -> Option<Solution> {
    println!("{}", keyboard);
    let penalty = keyboard.penalty(dict, max_penalty);
    if penalty < max_penalty {
        if keyboard.key_count() == 10 {
            Some(keyboard.with_penalty(penalty))
        } else {
            keyboard
                .every_combine_two_keys()
                .iter()
                .filter(|k| match k.max_key_size() {
                    None => true,
                    Some(k) => k <= 4,
                })
                .map(|k| combine_two_dfs_worker(dict, k.clone(), max_penalty))
                .find_map(|i| i)
        }
    } else {
        None
    }
}

pub fn combine_two_dfs(max_penalty: Penalty) {
    let dict = Dictionary::load_large_dictionary();
    let keyboard = Keyboard::new(
        dict.alphabet()
            .map(|r| Key::with_one_letter(r))
            .collect::<Vec<Key>>(),
    );
    let result = combine_two_dfs_worker(&dict, keyboard.clone(), max_penalty);
    println!("=====================================================");
    match result {
        None => println!("No keyboard with maximum penalty of {}", max_penalty),
        Some(keyboard) => {
            println!("{}", keyboard)
        }
    }
}
