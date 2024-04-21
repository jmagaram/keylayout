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
    keyboards_seen: u32,
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
            keyboards_seen: 0,
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
                    k.swap_random_letters_n_times(2).unwrap(),
                    k.swap_random_letters_n_times(4).unwrap(),
                    k.swap_random_letters_n_times(4).unwrap(),
                    k.swap_random_letters_n_times(4).unwrap(),
                    k.swap_random_letters_n_times(8).unwrap(),
                    k.swap_random_letters_n_times(8).unwrap(),
                    k.swap_random_letters_n_times(8).unwrap(),
                    k.swap_random_letters_n_times(12).unwrap(),
                ]
            })
            .scan(self.best.borrow(), |best, k| {
                let penalty = k.penalty(&self.dictionary, best.penalty());
                self.keyboards_seen = self.keyboards_seen + 1;
                let solution = k.with_penalty_and_notes(
                    penalty,
                    format!(
                        "gen:{} kbds:{}",
                        self.current_generation, self.keyboards_seen
                    ),
                );
                Some(solution)
            })
            .min_by(|a, b| a.penalty().partial_cmp(&b.penalty()).unwrap())?;
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

pub fn find_best<'a>(
    dict: &'a Dictionary,
    key_count: u32,
    die_threshold: Penalty,
) -> impl Iterator<Item = Option<Solution>> + 'a {
    let bad_pairs_count = 50;
    let bad_pairs = english::top_penalties(bad_pairs_count, 0);
    let alphabet_size = dict.alphabet().count_letters();
    let key_size_max = (alphabet_size / key_count + 2).min(alphabet_size);
    let partition = Partitions {
        sum: alphabet_size,
        parts: key_count,
        min: 1,
        max: key_size_max,
    };
    let mut best: Option<Solution> = None;
    let results = std::iter::repeat_with(move || {
        let start = Keyboard::random(dict.alphabet(), &partition)
            .filter(|k| false == k.contains_on_any_key(&bad_pairs))
            .map(|k| {
                let penalty = k.penalty(&dict, Penalty::MAX);
                k.with_penalty(penalty)
            })
            .next()
            .unwrap();
        let args: EvolveArgs = EvolveArgs {
            dictionary: &dict,
            start,
            die_threshold,
        };
        let solution = args.start().last();
        match (solution, &best) {
            (Some(solution), None) => best = Some(solution),
            (Some(solution), Some(current_best)) => {
                if solution.penalty() < current_best.penalty() {
                    best = Some(solution)
                }
            }
            _ => {}
        }
        best.clone()
    });
    results
}

pub fn evolve_one_random_keyboard() -> Option<Solution> {
    let bad_pairs = 60;
    let start_penalty = Penalty::new(0.035);
    let die_threshold = 0.00005;
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
        .filter(|k| k.penalty(&dict, start_penalty) < start_penalty)
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
    let mut last = None;
    for s in args.start() {
        println!("  {}", s);
        last = Some(s);
    }
    last
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
