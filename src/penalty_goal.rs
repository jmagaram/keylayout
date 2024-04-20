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
    pub fn empty(alphabet: Key) -> PenaltyGoals {
        PenaltyGoals {
            goals: BTreeMap::new(),
            alphabet,
        }
    }

    pub fn with_specific_penalty(&self, key_count: u8, penalty: Penalty) -> PenaltyGoals {
        let mut result = self.clone();
        result.goals.insert(key_count, penalty);
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
            key_counts.clone().max().unwrap() as usize <= self.alphabet.count(),
            "The maximum key count must be less than or equal to the size of the alphabet."
        );
        let partitions = key_counts.map(move |key_count| Partitions {
            sum: self.alphabet.count() as u32,
            parts: key_count as u32,
            min: 1,
            max: self.alphabet.count() as u32,
        });
        let mut result = self.clone();
        for p in partitions {
            println!("Calculating sample for {}", p.parts);
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

    pub fn penalty_goal(&self, key_count: u8) -> Option<Penalty> {
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
            .map(|(key_count, penalty)| format!("{} {}", key_count, penalty))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}", ordered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn display_property() {
        let d = Dictionary::load();
        let p = PenaltyGoals::empty(d.alphabet()).with_random_sampling(1..=10, 100, 10, &d);
        println!("Penalties: {}", p);
    }
}
