use std::{borrow::Borrow, fmt};

use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    prohibited::Prohibited, solution::Solution,
};

pub struct Genetic<'a> {
    best: Solution,
    dictionary: &'a Dictionary,
    current_generation: u32,
    die_threshold: Penalty,
    keyboards_seen: u32,
}

impl<'a> Iterator for Genetic<'a> {
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
                let solution = k.to_solution(
                    penalty,
                    format!(
                        "gen:{} kbds:{}",
                        self.current_generation, self.keyboards_seen
                    ),
                );
                Some(solution)
            })
            .min_by(|a, b| a.penalty().cmp(&b.penalty()))?;
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

pub struct FindBestArgs<'a> {
    pub start: Solution,
    pub dictionary: &'a Dictionary,
    pub die_threshold: Penalty,
}

impl<'a> fmt::Display for FindBestArgs<'a> {
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

impl<'a> FindBestArgs<'a> {
    pub fn start(&self) -> Genetic<'a> {
        Genetic {
            best: self.start.clone(),
            dictionary: self.dictionary,
            current_generation: 1,
            die_threshold: self.die_threshold,
            keyboards_seen: 0,
        }
    }
}

/// Tries to find the best keyboard using a genetic algorithm. Runs forever.
pub fn find_best<'a>(
    dict: &'a Dictionary,
    key_count: u32,
    die_threshold: Penalty,
) -> impl Iterator<Item = Option<Solution>> + 'a {
    let prohibited = Prohibited::new();
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
        let start = Keyboard::random(dict.alphabet(), &partition, &prohibited)
            .map(|k| {
                let penalty = k.penalty(&dict, Penalty::MAX);
                k.to_solution(penalty, "".to_string())
            })
            .next()
            .unwrap();
        let args: FindBestArgs = FindBestArgs {
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

#[cfg(test)]
mod tests {
    use crate::{keyboard::Keyboard, partitions::Partitions, penalty::Penalty};

    use super::*;

    #[test]
    #[ignore]
    fn try_genetic() {
        let dict = Dictionary::load();
        let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 10);
        let partition = Partitions {
            sum: 27,
            parts: 10,
            min: 2,
            max: 4,
        };
        let start = Keyboard::random(dict.alphabet(), &partition, &prohibited)
            .take(1)
            .map(|k| {
                let penalty = k.penalty(&dict, Penalty::MAX);
                k.to_solution(penalty, "initial state".to_string())
            })
            .last()
            .unwrap();
        let args: FindBestArgs = FindBestArgs {
            dictionary: &dict,
            start,
            die_threshold: Penalty::new(0.0001),
        };
        for s in args.start() {
            println!("{}", s)
        }
    }
}
