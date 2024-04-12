use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    permutable::Permutable,
};

struct EvolveKeyboardArgs<'a> {
    keyboard: Keyboard,
    stop_if_stuck: Penalty,
    dictionary: &'a Dictionary,
}

fn evolve_keyboard(args: EvolveKeyboardArgs) -> (Keyboard, Penalty) {
    let mut parent = args.keyboard.clone();
    let mut best_penalty = Penalty::MAX;
    let mut best_keyboard = args.keyboard.clone();
    loop {
        let prior_best = best_penalty;
        for child in parent.every_swap() {
            let child_penalty = child.penalty(args.dictionary, best_penalty);
            if child_penalty < best_penalty {
                best_keyboard = child;
                best_penalty = child_penalty;
                println!("{} {}", best_penalty, best_keyboard);
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

pub fn genetic() -> () {
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
    let args = EvolveKeyboardArgs {
        dictionary: &dict,
        keyboard: keyboard,
        stop_if_stuck: Penalty::new(0.001),
    };
    let (best_keyboard, best_penalty) = evolve_keyboard(args);
    println!("===========================================");
    println!("{} {}", best_penalty, best_keyboard);
    println!("===========================================");
}
