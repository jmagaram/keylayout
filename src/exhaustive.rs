use std::time::Instant;

use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    penalty_goal::PenaltyGoals, solution::Solution, tally::Tally,
};

pub fn best_n_key(count: u32) -> Option<Solution> {
    let dictionary = Dictionary::load();
    let alphabet = dictionary.alphabet();
    let start_time = Instant::now();
    let max = ((alphabet.count_letters() / count) + 3).min(alphabet.count_letters());
    let key_sizes = Partitions {
        sum: alphabet.count_letters(),
        parts: count,
        min: 1,
        max,
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
            let solution = k.with_penalty_and_notes(
                penalty,
                format!("{} keys, kbd {}, {:?}", count, index, start_time.elapsed()),
            );
            println!("{}", solution);
            best = Some(solution);
        }
        if index > 0 && index.rem_euclid(100000) == 0 {
            println!(
                "> seen {} keyboards with {} keys, {:?}",
                index,
                count,
                start_time.elapsed()
            );
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
    let start_time = Instant::now();
    let d = Dictionary::load();
    let start = Keyboard::new_every_letter_on_own_key(d.alphabet());
    let penalty_goals = PenaltyGoals::none(d.alphabet())
        .with_specific(26, Penalty::new(0.00006))
        .with_specific(25, Penalty::new(0.000174))
        .with_specific(24, Penalty::new(0.000385))
        .with_specific(23, Penalty::new(0.0007))
        .with_specific(22, Penalty::new(0.0012))
        .with_specific(21, Penalty::new(0.001985))
        .with_specific(20, Penalty::new(0.0003152))
        .with_specific(19, Penalty::new(0.0037))
        .with_specific(18, Penalty::new(0.004739))
        .with_specific(17, Penalty::new(0.005092))
        .with_specific(16, Penalty::new(0.00825))
        .with_specific(15, Penalty::new(0.009746))
        .with_specific(14, Penalty::new(0.013445))
        .with_specific(13, Penalty::new(0.016709))
        .with_specific(12, Penalty::new(0.02109))
        .with_adjustment(12..=20, 0.8)
        .with_adjustment(21..=25, 0.5)
        .with_specific(10, Penalty::new(0.0245));
    let max_letters_per_key = 4;
    let desired_keys = 10;
    let solution = dfs(&d, start, max_letters_per_key, desired_keys, &penalty_goals);
    println!();
    println!("Penalty Goals:");
    (1..26).for_each(|key_count| match penalty_goals.get(key_count) {
        None => {}
        Some(goal) => {
            println!("  {} : {}", key_count, goal);
        }
    });
    println!();
    match solution {
        None => {
            println!("No solution found");
        }
        Some(solution) => {
            println!("Solution found:");
            println!("  {}", solution);
        }
    }
    let duration = start_time.elapsed();
    println!();
    println!("Elapsed time: {:?}", duration);
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
