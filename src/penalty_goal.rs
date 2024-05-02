#![allow(dead_code)]

use core::fmt;
use std::{collections::BTreeMap, ops::RangeInclusive};

use crate::{key::Key, penalty::Penalty, solution_samples};

#[derive(Clone, Copy)]
pub enum ProhibitedPairs {
    Zero,
    Num20,
    Num40,
    Num60,
    Num80,
}

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

    pub fn with(&mut self, key_count: u8, penalty: Penalty) {
        self.goals.insert(key_count, penalty);
    }

    pub fn remove(&mut self, key_counts: RangeInclusive<u8>) {
        assert!(
            key_counts.clone().min().unwrap() > 0,
            "The minimum key count is 1."
        );
        assert!(
            key_counts.clone().max().unwrap() as usize <= self.alphabet.len() as usize,
            "The maximum key count must be less than or equal to the size of the alphabet."
        );
        for key_count in key_counts {
            self.goals.remove(&key_count);
        }
    }

    pub fn use_random_samples(
        &mut self,
        key_counts: RangeInclusive<u8>,
        prohibited_pairs: ProhibitedPairs,
        top_pct: f32,
        adjustment: f32,
    ) {
        let file_name = match prohibited_pairs {
            ProhibitedPairs::Zero => "kbd_penalties_5000_samples_0_pairs.csv",
            ProhibitedPairs::Num20 => "kbd_penalties_5000_samples_20_pairs.csv",
            ProhibitedPairs::Num40 => "kbd_penalties_5000_samples_40_pairs.csv",
            ProhibitedPairs::Num60 => "kbd_penalties_5000_samples_60_pairs.csv",
            ProhibitedPairs::Num80 => "kbd_penalties_5000_samples_80_pairs.csv",
        };
        let mut penalty_samples = solution_samples::CsvOutput::load_from_csv(file_name).unwrap();
        penalty_samples.sort_by(|a, b| a.penalty().cmp(&b.penalty()));
        for key_count in key_counts {
            let total_with_key_count = penalty_samples
                .iter()
                .filter(|k| k.keys() == key_count as u32)
                .count();
            let index_num = (top_pct * total_with_key_count as f32) as usize;
            let penalty_goal = penalty_samples
                .iter()
                .filter(|i| i.keys() == key_count as u32)
                .nth(index_num)
                .unwrap()
                .penalty();
            let adjusted_penalty = Penalty::new(penalty_goal.to_f32() * adjustment);
            self.goals.insert(key_count, adjusted_penalty);
        }
    }

    pub fn with_adjustment(&mut self, key_counts: RangeInclusive<u8>, multiplier: f32) {
        assert!(
            multiplier > 0.0,
            "The multiplier must be greater that zero."
        );
        assert!(
            key_counts.clone().min().unwrap() > 0,
            "The minimum key count is 1."
        );
        assert!(
            key_counts.clone().max().unwrap() as usize <= self.alphabet.len() as usize,
            "The maximum key count must be less than or equal to the size of the alphabet."
        );
        for key_count in key_counts {
            match self.goals.get(&key_count) {
                None => (),
                Some(previous) => {
                    let penalty = Penalty::new(previous.to_f32() * multiplier);
                    self.goals.insert(key_count, penalty);
                }
            }
        }
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
        writeln!(f, "Penalty goals")?;
        writeln!(f, "")?;
        let ordered = self
            .goals
            .iter()
            .map(|(key_count, penalty)| format!("{:<2} {:.4}", key_count, penalty))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", ordered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
