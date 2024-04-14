use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty, solution::Solution,
};

fn go(dict: &Dictionary, keyboard: Keyboard, max_penalty: Penalty) -> Option<Solution> {
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
                .map(|k| go(dict, k.clone(), max_penalty))
                .find_map(|i| i)
        }
    } else {
        None
    }
}

pub struct Args {
    pub max_penalty: Penalty,
}

pub fn solve(args: Args) {
    let d = Dictionary::load();
    let k = Keyboard::new(d.alphabet().map(|r| Key::with_one_letter(r)).collect());
    let result = go(&d, k, args.max_penalty);
    println!("=====================================================");
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
