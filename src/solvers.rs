use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, letter::Letter, partitions::Partitions,
    penalty::Penalty, permutable::Permutable,
};
use std::sync::mpsc;

pub struct EvolveKeyboardArgs<'a> {
    pub keyboard: Keyboard,
    pub keyboard_penalty: Penalty,
    pub stop_if_stuck: Penalty,
    pub dictionary: &'a Dictionary,
    pub print_progress: bool,
}

pub fn evolve_keyboard(args: EvolveKeyboardArgs) -> (Keyboard, Penalty) {
    let mut best_penalty = args.keyboard_penalty;
    let mut best_keyboard = args.keyboard.clone();
    loop {
        let mut current_best_penalty = best_penalty;
        let mut current_best_keyboard = best_keyboard.clone();
        for child in best_keyboard.every_swap() {
            let child_penalty = child.penalty(args.dictionary, best_penalty);
            if child_penalty < current_best_penalty {
                current_best_keyboard = child;
                current_best_penalty = child_penalty;
                if args.print_progress {
                    println!("{} {}", current_best_penalty, current_best_keyboard);
                }
            }
        }
        let progress_made = (current_best_penalty.to_f32() - best_penalty.to_f32()).abs()
            > args.stop_if_stuck.to_f32();
        if current_best_penalty < best_penalty {
            best_penalty = current_best_penalty;
            best_keyboard = current_best_keyboard;
        }
        if !progress_made {
            break;
        }
    }
    (best_keyboard, best_penalty)
}

pub fn find_best(dict: &Dictionary, print_best: bool) -> (Keyboard, Penalty) {
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
    let keyboard_penalty = keyboard.penalty(&dict, Penalty::MAX);
    let args = EvolveKeyboardArgs {
        dictionary: &dict,
        keyboard,
        keyboard_penalty,
        print_progress: false,
        stop_if_stuck: Penalty::new(0.001),
    };
    let (best_keyboard, best_penalty) = evolve_keyboard(args);
    if print_best {
        println!("===========================================");
        println!("{} {}", best_penalty, best_keyboard);
        println!("===========================================");
    }
    (best_keyboard, best_penalty)
}

pub fn genetic_threaded(threads: u32) -> () {
    let (tx, rx) = mpsc::sync_channel::<(Keyboard, Penalty)>(10);
    let mut best: Option<(Keyboard, Penalty)> = None;
    for _ in 0..threads {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let dictionary = Dictionary::load_large_dictionary();
            loop {
                let (best_keyboard, best_penalty) = find_best(&dictionary, false);
                tx.send((best_keyboard, best_penalty)).unwrap();
            }
        });
    }
    for (keyboard, penalty) in rx {
        match best {
            None => {
                println!("{} {}", penalty, keyboard);
                best = Some((keyboard, penalty));
            }
            Some((_, best_penalty)) => {
                if penalty < best_penalty {
                    println!("{} {}", penalty, keyboard);
                    best = Some((keyboard, penalty));
                }
            }
        }
    }
}

pub fn combine_two_dfs_worker(
    dict: &Dictionary,
    keyboard: Keyboard,
    max_penalty: Penalty,
) -> Option<Keyboard> {
    let penalty = keyboard.penalty(dict, max_penalty);
    println!("{}", keyboard);
    if penalty < max_penalty {
        if keyboard.key_count() == 10 {
            Some(keyboard)
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
            let penalty = keyboard.penalty(&dict, Penalty::MAX);
            println!("{} {}", penalty, keyboard)
        }
    }
}
