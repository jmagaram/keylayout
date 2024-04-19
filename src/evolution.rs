use std::{borrow::Borrow, cmp::Ordering};

use crate::{dictionary::Dictionary, key::Key, penalty::Penalty, solution::Solution};

pub struct Evolve<'a> {
    best: Solution,
    dictionary: &'a Dictionary,
    prohibited: Vec<Key>,
    current_generation: u32,
}

pub struct EvolveArgs<'a> {
    start: Solution,
    dictionary: &'a Dictionary,
    prohibited: Vec<Key>,
}

impl<'a> EvolveArgs<'a> {
    pub fn start(&self) -> Evolve<'a> {
        Evolve {
            best: self.start.clone(),
            dictionary: self.dictionary,
            current_generation: 1,
            prohibited: self.prohibited.clone(),
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
                    k.swap_random_letters_n_times(8).unwrap(),
                    k.swap_random_letters_n_times(12).unwrap(),
                    k.swap_random_letters_n_times(16).unwrap(),
                ]
            })
            .filter(|k| false == k.contains_on_any_key(&self.prohibited))
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
            });
        match best_child {
            None => {
                self.current_generation = self.current_generation + 1;
                Some(self.best.clone())
            }
            Some(best_child) => {
                if best_child.penalty() < self.best.penalty() {
                    self.best = best_child.clone();
                }
                self.current_generation = self.current_generation + 1;
                Some(best_child)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{english, keyboard::Keyboard, partitions::Partitions, penalty::Penalty};

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
            prohibited: english::top_penalties(0, 0),
        };
        for s in args.start().take(10) {
            println!("{}", s)
        }
    }
}
