use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    permutable::Permutable,
};
use std::sync::mpsc;

struct EvolveKeyboardArgs<'a> {
    keyboard: Keyboard,
    keyboard_penalty: Penalty,
    stop_if_stuck: Penalty,
    dictionary: &'a Dictionary,
    print_progress: bool,
}

fn evolve_keyboard(args: EvolveKeyboardArgs) -> (Keyboard, Penalty) {
    let mut parent = args.keyboard.clone();
    let mut best_penalty = args.keyboard_penalty;
    let mut best_keyboard = args.keyboard.clone();
    loop {
        let prior_best = best_penalty;
        for child in parent.every_swap() {
            let child_penalty = child.penalty(args.dictionary, best_penalty);
            if child_penalty < best_penalty {
                best_keyboard = child;
                best_penalty = child_penalty;
                if args.print_progress {
                    println!("{} {}", best_penalty, best_keyboard);
                }
            }
        }
        let stop =
            (best_penalty.to_f32() - prior_best.to_f32()).abs() <= args.stop_if_stuck.to_f32();
        parent = best_keyboard.clone();
        if stop {
            break;
        }
    }
    (best_keyboard, best_penalty)
}

pub fn genetic(print_best: bool) -> (Keyboard, Penalty) {
    let dict = Dictionary::load_large_dictionary();
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
        print_progress: true,
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
    let mut handles = vec![];
    let (sender, receiver) = mpsc::sync_channel::<(Keyboard, Penalty)>(10);
    for _i in 1..threads {
        handles.push(std::thread::spawn(move || {
            let (best_keyboard, best_penalty) = genetic(false);
            let m = sender.clone();
            m.send((best_keyboard, best_penalty));
        }));
    }
    let sender_1 = sender.clone();
    let sender_2 = sender.clone();
    let sender_3 = sender.clone();
    let sender_4 = sender.clone();
    let sender_5 = sender.clone();
    handles.push(std::thread::spawn(move || {
        let (best_keyboard, best_penalty) = genetic(false);
        sender_1.send((best_keyboard, best_penalty));
    }));
    handles.push(std::thread::spawn(move || {
        let (best_keyboard, best_penalty) = genetic(false);
        sender_2.send((best_keyboard, best_penalty));
    }));
    handles.push(std::thread::spawn(move || {
        let (best_keyboard, best_penalty) = genetic(false);
        sender_3.send((best_keyboard, best_penalty));
    }));
    handles.push(std::thread::spawn(move || {
        let (best_keyboard, best_penalty) = genetic(false);
        sender_4.send((best_keyboard, best_penalty));
    }));
    handles.push(std::thread::spawn(move || {
        let (best_keyboard, best_penalty) = genetic(false);
        sender_5.send((best_keyboard, best_penalty));
    }));
    for (keyboard, penalty) in receiver {
        println!("{} {}", penalty, keyboard);
    }
}
