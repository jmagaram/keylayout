use std::{borrow::Borrow, cmp::Ordering, fmt};

use crate::{
    dictionary::Dictionary, english, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    solution::Solution,
};

pub struct Evolve<'a> {
    best: Solution,
    dictionary: &'a Dictionary,
    current_generation: u32,
    die_threshold: Penalty,
}

pub struct EvolveArgs<'a> {
    pub start: Solution,
    pub dictionary: &'a Dictionary,
    pub die_threshold: Penalty,
}

impl<'a> fmt::Display for EvolveArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Words:{} Die:{} Start:{}",
            self.dictionary.words().len(),
            self.die_threshold,
            self.start.penalty()
        )
    }
}

impl<'a> EvolveArgs<'a> {
    pub fn start(&self) -> Evolve<'a> {
        Evolve {
            best: self.start.clone(),
            dictionary: self.dictionary,
            current_generation: 1,
            die_threshold: self.die_threshold,
        }
    }
}

impl<'a> Iterator for Evolve<'a> {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        let best_child = self
            .best
            .borrow()
            .keyboard()
            .every_swap()
            .iter()
            .flat_map(|k| {
                [
                    k.clone(),
                    k.swap_random_letters_n_times(1).unwrap(),
                    k.swap_random_letters_n_times(2).unwrap(),
                    k.swap_random_letters_n_times(4).unwrap(),
                    k.swap_random_letters_n_times(8).unwrap(),
                ]
            })
            .scan(self.best.borrow(), |best, k| {
                let penalty = k.penalty(&self.dictionary, best.penalty());
                let solution =
                    k.with_penalty_and_notes(penalty, format!("gen:{}", self.current_generation));
                Some(solution)
            })
            .min_by(|a, b| {
                if a.penalty() < b.penalty() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })?;
        let sufficient_progress = (self.best.penalty().to_f32() - best_child.penalty().to_f32())
            > self.die_threshold.to_f32();
        if sufficient_progress {
            self.best = best_child.clone();
            self.current_generation = self.current_generation + 1;
            Some(best_child)
        } else {
            None
        }
    }
}

pub fn evolve_one_random_keyboard() {
    let bad_pairs = 60;
    let die_threshold = 0.00001;
    let dict = Dictionary::load();
    let partition = Partitions {
        sum: 27,
        parts: 10,
        min: 2,
        max: 4,
    };
    let bad_pairs = english::top_penalties(bad_pairs, 0);
    let start = Keyboard::random(dict.alphabet(), &partition)
        .filter(|k| false == k.contains_on_any_key(&bad_pairs))
        .take(1)
        .map(|k| {
            let penalty = k.penalty(&dict, Penalty::MAX);
            k.with_penalty_and_notes(penalty, "initial state".to_string())
        })
        .last()
        .unwrap();
    let args: EvolveArgs = EvolveArgs {
        dictionary: &dict,
        start,
        die_threshold: Penalty::new(die_threshold),
    };
    println!("");
    println!("{}", args);
    println!("");
    for s in args.start() {
        println!("  {}", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::{keyboard::Keyboard, partitions::Partitions, penalty::Penalty};

    use super::*;

    #[test]
    #[ignore]
    fn try_genetic() {
        let dict = Dictionary::load();
        let partition = Partitions {
            sum: 27,
            parts: 10,
            min: 2,
            max: 4,
        };
        let start = Keyboard::random(dict.alphabet(), &partition)
            .take(1)
            .map(|k| {
                let penalty = k.penalty(&dict, Penalty::MAX);
                k.with_penalty_and_notes(penalty, "initial state".to_string())
            })
            .last()
            .unwrap();
        let args: EvolveArgs = EvolveArgs {
            dictionary: &dict,
            start,
            die_threshold: Penalty::new(0.0001),
        };
        for s in args.start() {
            println!("{}", s)
        }
    }
}
