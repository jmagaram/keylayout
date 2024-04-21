use std::collections::HashMap;

use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    penalty_goal::PenaltyGoals, solution::Solution, tally::Tally,
};

pub fn best_n_key(count: u32) -> Option<Solution> {
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
            let solution =
                k.with_penalty_and_notes(penalty, format!("#{} for {} keys", index, count));
            println!("{} > {}", index, solution);
            best = Some(solution);
        }
        if index.rem_euclid(100000) == 0 {
            match best.clone() {
                None => {}
                Some(solution) => {
                    println!("{} > {}", index, solution);
                }
            }
        }
    }
    best
}

fn dfs(
    dictionary: &Dictionary,
    keyboard: Keyboard,
    max_letters_per_key: u32,
    desired_keys: usize,
    penalty_goals: &PenaltyGoals,
) -> Option<Solution> {
    println!("{}", keyboard);
    let penalty_goal = penalty_goals
        .get(keyboard.key_count() as u8)
        .unwrap_or(Penalty::MAX);
    let penalty = keyboard.penalty(&dictionary, penalty_goal);
    if penalty <= penalty_goal {
        if keyboard.key_count() == desired_keys {
            let solution = keyboard.with_penalty(penalty);
            Some(solution)
        } else {
            keyboard
                .every_combine_two_keys()
                .iter()
                .filter(move |k| k.max_key_size().unwrap() <= max_letters_per_key)
                .map(move |k| {
                    dfs(
                        dictionary,
                        k.clone(),
                        max_letters_per_key,
                        desired_keys,
                        penalty_goals,
                    )
                })
                .find_map(|k| k)
        }
    } else {
        None
    }
}

pub fn dumb_run_dfs() {
    let d = Dictionary::load();
    let start = Keyboard::new_every_letter_on_own_key(d.alphabet());
    let penalty_goals = PenaltyGoals::none(d.alphabet())
        .with_random_sampling(12..=26, 10, 0, &d)
        .with_specific(10, Penalty::new(0.5));
    println!("Penalties: {}", penalty_goals);
    let max_letters_per_key = 4;
    let desired_keys = 10;
    let solution = dfs(&d, start, max_letters_per_key, desired_keys, &penalty_goals);
    match solution {
        None => {
            println!("No solution found")
        }
        Some(solution) => {
            println!("{}", solution);
        }
    }
}

pub fn run_dfs() {
    let d = Dictionary::load();
    let start = Keyboard::new_every_letter_on_own_key(d.alphabet());
    let penalty_goals = PenaltyGoals::none(d.alphabet())
        .with_random_sampling(11..=26, 5000, 10, &d)
        .with_specific(10, Penalty::new(0.0240));
    let max_letters_per_key = 5;
    let desired_keys = 10;
    let solution = dfs(&d, start, max_letters_per_key, desired_keys, &penalty_goals);
    match solution {
        None => {
            println!("No solution found")
        }
        Some(solution) => {
            println!("{}", solution);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore]
    fn try_dfs() {
        let d = Dictionary::load();
        let start = Keyboard::new_every_letter_on_own_key(d.alphabet());
        let penalty_goals = PenaltyGoals::none(d.alphabet())
            .with_random_sampling(11..=26, 10, 0, &d)
            .with_adjustment(12..=26, 0.7)
            .with_specific(10, Penalty::new(0.5));
        let max_letters_per_key = 5;
        let desired_keys = 10;
        let solution = dfs(&d, start, max_letters_per_key, desired_keys, &penalty_goals);
        match solution {
            None => {
                println!("No solution found")
            }
            Some(solution) => {
                println!("{}", solution);
            }
        }
    }
}
