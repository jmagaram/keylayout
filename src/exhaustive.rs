use core::fmt;
use std::{cmp::Ordering, collections::HashMap};

use crate::{
    dictionary::Dictionary,
    keyboard::Keyboard,
    partitions::Partitions,
    penalty::Penalty,
    solution::{self, Solution},
    tally::Tally,
};

#[derive(Debug)]
struct PenaltyGoals(HashMap<usize, Penalty>);

impl PenaltyGoals {
    pub fn none() -> PenaltyGoals {
        PenaltyGoals(HashMap::new())
    }

    pub fn as_hash_map(&self) -> HashMap<usize, Penalty> {
        self.0.clone()
    }

    pub fn tighten_all(&self, factor: f32) -> PenaltyGoals {
        let scores = self
            .0
            .iter()
            .map(|(k, v)| (*k, Penalty::new(v.to_f32() * factor)));
        PenaltyGoals(HashMap::from_iter(scores))
    }

    pub fn with_specific_penalty(&self, key_count: usize, penalty: Penalty) -> PenaltyGoals {
        let mut m = self.0.clone();
        m.insert(key_count, penalty);
        PenaltyGoals(m)
    }

    pub fn from_keyboard(keyboard: &Keyboard, dictionary: &Dictionary) -> PenaltyGoals {
        // let penalties_by_key_count = (2..=5)
        let penalties_by_key_count = (1..=keyboard.key_count())
            .flat_map(|n| keyboard.subsets_of_keys(n))
            .map(|k| k.fill_missing(dictionary.alphabet()))
            .fold(HashMap::new(), |mut total, k| {
                let p = k.penalty(&dictionary, Penalty::MAX);
                match total.get_mut(&k.key_count()) {
                    None => {
                        total.insert(k.key_count(), vec![p]);
                    }
                    Some(penalties) => {
                        penalties.push(p);
                    }
                }
                total
            });
        let worst_penalties = penalties_by_key_count
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .max_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Greater)),
                )
            })
            .filter_map(|(k, v)| v.map(|p| (k, p)))
            .collect::<HashMap<usize, Penalty>>();
        PenaltyGoals(worst_penalties)
    }
}

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
            let solution = k.with_penalty_and_notes(penalty, format!("#{}", index));
            println!("{} > {}", index, solution);
            best = Some(solution);
        }
        if index.rem_euclid(10000) == 0 {
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
    penalty_goals: &HashMap<usize, Penalty>,
) -> Option<Solution> {
    println!("{}", keyboard);
    let penalty_goal = *penalty_goals
        .get(&keyboard.key_count())
        .unwrap_or(&Penalty::MAX);
    let penalty = keyboard.penalty(&dictionary, penalty_goal);
    if penalty < penalty_goal {
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
    let penalty_goals = PenaltyGoals::none()
        .with_specific_penalty(10, Penalty::new(0.5))
        .as_hash_map();
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

pub fn run_dfs() {
    let d = Dictionary::load();
    let start = Keyboard::new_every_letter_on_own_key(d.alphabet());

    let penalty_goals = PenaltyGoals::none()
        .with_specific_penalty(10, Penalty::new(0.024))
        .with_specific_penalty(25, best_n_key(24).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(24, best_n_key(23).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(23, best_n_key(22).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(22, best_n_key(21).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(21, best_n_key(20).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(20, best_n_key(19).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(19, best_n_key(18).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(18, best_n_key(17).map_or(Penalty::MAX, |s| s.penalty()))
        .with_specific_penalty(17, best_n_key(16).map_or(Penalty::MAX, |s| s.penalty()))
        .as_hash_map();
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

    use crate::key::Key;

    use super::*;

    #[test]
    #[ignore]
    fn penalty_goals() {
        let d = Dictionary::load();
        let k = Keyboard::new_from_layout("akw,bn,cejq,dfx',gm,hiv,lyz,ot,pr,su");
        let g = PenaltyGoals::from_keyboard(&k, &d);
        for (size, penalty) in g.0.iter() {
            println!("PENALTY GOAL: {}, {}", size, penalty);
        }
    }

    #[test]
    #[ignore]
    fn try_dfs() {
        let d = Dictionary::load();
        let start = Keyboard::new_every_letter_on_own_key(d.alphabet());
        let penalty_goals = {
            let data = [(2usize, 0.5f32), (3usize, 0.5f32), (4usize, 0.5f32)]
                .map(|(key_size, penalty)| (key_size, Penalty::new(penalty)));
            HashMap::from(data)
        };
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
