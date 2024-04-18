use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    solution::Solution, tally::Tally,
};

pub fn best_n_key(count: u32) {
    let dictionary = Dictionary::load();
    let alphabet = dictionary.alphabet();
    let key_sizes = Partitions {
        sum: 27,
        parts: count,
        min: 1,
        max: 27,
    }
    .calculate();
    let keyboards = key_sizes.iter().flat_map(|key_sizes| {
        let arrangements: Tally<u32> = Tally::from(key_sizes);
        alphabet
            .distribute(arrangements)
            .map(|keys| Keyboard::new_from_keys(keys))
    });
    let mut best: Option<Solution> = None;
    for (index, k) in keyboards.enumerate() {
        let best_penalty = best.as_ref().map(|b| b.penalty()).unwrap_or(Penalty::MAX);
        let penalty = k.penalty(&dictionary, best_penalty);
        if penalty < best_penalty {
            let solution = k.with_penalty_and_notes(penalty, format!("#{}", index));
            println!("{}", solution);
            best = Some(solution);
        }
        if index.rem_euclid(10000) == 0 {
            println!("Iteration #{}", index);
        }
    }
}
