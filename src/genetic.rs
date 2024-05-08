use std::borrow::Borrow;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    prohibited::Prohibited, single_key_penalties::SingleKeyPenalties, solution::Solution,
};

struct Genetic<'a> {
    best: Solution,
    current_generation: u32,
    die_threshold: Penalty,
    dictionary: &'a Dictionary,
    single_key_penalties: &'a SingleKeyPenalties,
}

impl<'a> Iterator for Genetic<'a> {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        let children = self
            .best
            .borrow()
            .keyboard()
            .every_swap()
            .iter()
            .flat_map(|k| {
                [
                    k.clone(),
                    k.swap_random_letters_n_times(2).unwrap(),
                    k.swap_random_letters_n_times(2).unwrap(),
                    k.swap_random_letters_n_times(2).unwrap(),
                    k.swap_random_letters_n_times(4).unwrap(),
                    k.swap_random_letters_n_times(4).unwrap(),
                    k.swap_random_letters_n_times(4).unwrap(),
                    k.swap_random_letters_n_times(8).unwrap(),
                    k.swap_random_letters_n_times(8).unwrap(),
                    k.swap_random_letters_n_times(8).unwrap(),
                    k.swap_random_letters_n_times(12).unwrap(),
                    k.swap_random_letters_n_times(12).unwrap(),
                    k.swap_random_letters_n_times(12).unwrap(),
                ]
            })
            .collect::<Vec<Keyboard>>();
        let best_child = children
            .into_par_iter()
            .map(|k| {
                let penalty = k.penalty_to_beat(
                    self.dictionary,
                    self.best.penalty(),
                    self.single_key_penalties,
                );
                k.to_solution(penalty, format!("gen:{}", self.current_generation))
            })
            .min_by(|a, b| a.penalty().cmp(&b.penalty()))
            .unwrap();
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
    pub dictionary: &'a Dictionary,
    pub die_threshold: Penalty,
    pub key_count: u8,
    pub prohibited: Prohibited,
    pub single_key_penalties: &'a SingleKeyPenalties,
}

/// Tries to find the best keyboard using a genetic algorithm. Runs forever.
pub fn find_best<'a>(args: FindBestArgs<'a>) -> impl Iterator<Item = Option<Solution>> + 'a {
    let alphabet_size = args.dictionary.alphabet().len();
    let key_size_max = (alphabet_size / args.key_count + 2).min(alphabet_size);
    let partition = Partitions {
        sum: alphabet_size,
        parts: args.key_count,
        min: 1,
        max: key_size_max,
    };
    let mut best: Option<Solution> = None;
    let results = std::iter::repeat_with(move || {
        let start = Keyboard::random(
            args.dictionary.alphabet(),
            partition.clone(),
            &args.prohibited,
        )
        .map(|k| {
            let penalty = k.penalty(args.dictionary, Penalty::MAX);
            k.to_solution(penalty, "".to_string())
        })
        .next()
        .unwrap();
        let dictionary = Dictionary::load();
        let solution = Genetic {
            best: start,
            current_generation: 1,
            die_threshold: args.die_threshold,
            single_key_penalties: args.single_key_penalties,
            dictionary: &dictionary,
        }
        .last();
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
