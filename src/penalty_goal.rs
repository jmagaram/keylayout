#![allow(dead_code)]

use core::fmt;
use std::{collections::BTreeMap, ops::RangeInclusive};

use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
};

#[derive(Debug, Clone)]
pub struct PenaltyGoals {
    goals: BTreeMap<u8, Penalty>,
    alphabet: Key,
}

impl PenaltyGoals {
    pub fn none(alphabet: Key) -> PenaltyGoals {
        PenaltyGoals {
            goals: BTreeMap::new(),
            alphabet,
        }
    }

    pub fn with_specific(&self, key_count: u8, penalty: Penalty) -> PenaltyGoals {
        let mut result = self.clone();
        result.goals.insert(key_count, penalty);
        result
    }

    pub fn with_adjustment(&self, key_counts: RangeInclusive<u8>, multiplier: f32) -> PenaltyGoals {
        assert!(
            multiplier > 0.0,
            "The multiplier must be greater that zero."
        );
        assert!(
            key_counts.clone().min().unwrap() > 0,
            "The minimum key count is 1."
        );
        assert!(
            key_counts.clone().max().unwrap() as usize <= self.alphabet.count_letters() as usize,
            "The maximum key count must be less than or equal to the size of the alphabet."
        );
        let mut result = self.clone();
        for key_count in key_counts {
            match result.goals.get(&key_count) {
                None => (),
                Some(previous) => {
                    let penalty = Penalty::new(previous.to_f32() * multiplier);
                    result.goals.insert(key_count, penalty);
                }
            }
        }
        result
    }

    pub fn with_random_sampling(
        &self,
        key_counts: RangeInclusive<u8>,
        sample_size: usize,
        take_index: usize,
        dictionary: &Dictionary,
    ) -> PenaltyGoals {
        assert!(sample_size > 0, "Expected sample_size>0.");
        assert!(
            take_index < sample_size,
            "Expected take_index < sample_size."
        );
        assert!(
            key_counts.clone().min().unwrap() > 0,
            "The minimum key count is 1."
        );
        assert!(
            key_counts.clone().max().unwrap() as usize <= self.alphabet.count_letters() as usize,
            "The maximum key count must be less than or equal to the size of the alphabet."
        );
        let alphabet_size = self.alphabet.count_letters() as usize;
        let partitions = key_counts.map(move |key_count| {
            let max = ((alphabet_size / (key_count as usize)) + 2).min(alphabet_size);
            Partitions {
                sum: alphabet_size as u32,
                parts: key_count as u32,
                min: 1,
                max: max as u32,
            }
        });
        let mut result = self.clone();
        for p in partitions {
            println!(
                "Calculating random sample of size {} for keyboard with {} keys...",
                sample_size, p.parts
            );
            let mut penalties = Keyboard::random(self.alphabet, &p)
                .take(sample_size)
                .map(|k| k.penalty(&dictionary, Penalty::MAX))
                .collect::<Vec<Penalty>>();
            penalties.sort_by(|a, b| a.partial_cmp(&b).unwrap());
            let penalty = penalties.iter().skip(take_index).next().unwrap();
            result.goals.insert(p.parts as u8, penalty.clone());
        }
        result
    }

    pub fn get(&self, key_count: u8) -> Option<Penalty> {
        assert!(
            key_count > 0,
            "The minimum number of keys on a keyboard is 1."
        );
        self.goals.get(&key_count).map(|p| *p)
    }
}

impl fmt::Display for PenaltyGoals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ordered = self
            .goals
            .iter()
            .map(|(key_count, penalty)| format!("{}:{}", key_count, penalty))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}", ordered)
    }
}

pub fn calculate_for_standard() {
    let d = Dictionary::load();
    let p = PenaltyGoals::none(d.alphabet()).with_random_sampling(11..=26, 10000, 0, &d);
    println!("Best penalty goals with 10000 random samples");
    println!("{}", p);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn display_property() {
        let d = Dictionary::load();
        let p = PenaltyGoals::none(d.alphabet()).with_random_sampling(1..=10, 100, 10, &d);
        println!("Penalties: {}", p);
    }
}
