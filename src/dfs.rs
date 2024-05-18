#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    time::Instant,
};

use rand::seq::SliceRandom;
use rand::thread_rng;
use thousands::Separable;

use crate::{
    dictionary::Dictionary,
    key::Key,
    key_set::KeySet,
    keyboard::Keyboard,
    letter::Letter,
    penalty::Penalty,
    prohibited::Prohibited,
    solution::Solution,
    tally::Tally,
    util::DurationFormatter,
    word::Word,
    word_overlap::{self, WordOverlap},
};

pub struct Args {
    pub max_key_size: usize,
    pub prohibited_pairs: usize,
    pub ten_key_prune_threshold: Penalty,
    pub prune_factor: f32,
    pub prune_from_key_count: usize,
    pub prune_to_key_count: usize,
}

impl Args {
    pub fn solve(&self) {
        let dictionary = Dictionary::load();
        let mut alphabet = dictionary.alphabet().letters().collect::<Vec<Letter>>();
        alphabet.shuffle(&mut thread_rng());
        let best: RefCell<Option<Solution>> = RefCell::new(None);
        let seen = Cell::new(0);
        let pruned = Cell::new(0);
        let prohibited = Prohibited::with_top_n_letter_pairs(&dictionary, self.prohibited_pairs);
        let word_overlap = WordOverlap::load_from_csv(&dictionary, "./word_overlaps.csv");
        let seen_at: RefCell<Tally<usize>> = RefCell::new(Tally::new());
        let dfs = Dfs {
            dictionary: &dictionary,
            alphabet: &alphabet,
            alphabet_index: 0,
            keys: vec![],
            max_key_size: self.max_key_size,
            best: &best,
            prohibited: &prohibited,
            overlaps: &word_overlap,
            ten_key_threshold: self.ten_key_prune_threshold,
            prune_factor: self.prune_factor,
            prune_from_key_count: self.prune_from_key_count,
            prune_to_key_count: self.prune_to_key_count,
            seen: &seen,
            pruned: &pruned,
            started_at: Instant::now(),
            seen_at: &seen_at,
        };
        dfs.go();
    }
}

struct Dfs<'a> {
    dictionary: &'a Dictionary,
    alphabet: &'a Vec<Letter>,
    alphabet_index: usize,
    keys: Vec<Key>,
    max_key_size: usize,
    best: &'a RefCell<Option<Solution>>,
    seen: &'a Cell<u128>,
    seen_at: &'a RefCell<Tally<usize>>,
    pruned: &'a Cell<u128>,
    prohibited: &'a Prohibited,
    overlaps: &'a WordOverlap,
    ten_key_threshold: Penalty,
    prune_factor: f32,
    prune_from_key_count: usize,
    prune_to_key_count: usize,
    started_at: Instant,
}

impl<'a> Dfs<'a> {
    pub fn go(&self) {
        self.seen.replace(self.seen.get() + 1);
        self.seen_at.borrow_mut().increment(self.alphabet_index);
        if self.seen.get().rem_euclid(1000) == 0 {
            let seen_per_second =
                (self.seen.get() as f32) / self.started_at.elapsed().as_secs_f32();
            println!("");
            println!("Pruned: {}", self.pruned.get().separate_with_underscores());
            println!(
                "Seen:   {} ({}/sec)",
                self.seen.get().separate_with_underscores(),
                (seen_per_second as u32).separate_with_underscores()
            );
            (1..=27).for_each(|key_count| {
                let total = self.seen_at.borrow().count(&key_count);
                if total > 0 {
                    println!("  {}: {}", key_count, total.separate_with_underscores());
                }
            });
            println!("Elapsed: {}", self.started_at.elapsed().round_to_seconds());
            if let Some(best) = self.best.borrow().as_ref() {
                println!("Best: {}", best);
            }
        }
        let k = Keyboard::with_keys(self.keys.clone()).fill_missing(self.dictionary.alphabet());
        let letter = self.alphabet.get(self.alphabet_index);
        match letter {
            None => {
                let best_penalty = self
                    .best
                    .borrow()
                    .as_ref()
                    .map(|b| b.penalty())
                    .unwrap_or(Penalty::MAX);
                let penalty = k.penalty(&self.dictionary, best_penalty);
                if penalty < best_penalty {
                    let solution = k.to_solution(penalty, "".into());
                    self.best.replace(Some(solution));
                }
            }
            Some(letter) => {
                let prune = {
                    if k.len() >= self.prune_from_key_count && k.len() <= self.prune_to_key_count {
                        let pairs = self
                            .keys
                            .iter()
                            .filter(|k| k.len() >= 2)
                            .flat_map(|k| k.subsets_of_size(2))
                            .collect::<Vec<Key>>();
                        let key_sets = {
                            let one_pair = pairs.iter().map(|p| KeySet::with_pairs(vec![*p]));
                            let two_pairs_indexes = (0..pairs.len() - 1)
                                .flat_map(|i| ((i + 1)..pairs.len()).map(move |j| (i, j)));
                            let two_pairs = two_pairs_indexes
                                .map(|(i, j)| KeySet::with_pairs(vec![pairs[i], pairs[j]]));
                            one_pair.chain(two_pairs)
                        };
                        let mut words = key_sets
                            .map(|ks| self.overlaps.words_for_pairs(&ks))
                            .fold(HashSet::<u32>::new(), |mut total, i| {
                                total.extend(i);
                                total
                            })
                            .iter()
                            .filter_map(|inx| {
                                self.overlaps.word_from_index(*inx).map(|w| w.clone())
                            })
                            .collect::<Vec<Word>>();
                        words.sort_unstable_by(|a, b| b.frequency().cmp(a.frequency()));
                        let estimate_dictionary = Dictionary::from_unique_sorted_words(words);
                        let estimate = k.penalty(&estimate_dictionary, Penalty::MAX);
                        let factor = self.prune_factor.powi(k.len() as i32 - 10);
                        let threshold = Penalty::new(self.ten_key_threshold.to_f32() * factor);
                        let should_prune = estimate > threshold;
                        if should_prune {
                            self.pruned.replace(self.pruned.get() + 1);
                        }
                        should_prune
                    } else {
                        false
                    }
                };
                if !prune {
                    for i in 0..=self.keys.len().min(9) {
                        let insert_key = i == self.keys.len();
                        let keys = match insert_key {
                            true => {
                                let mut keys = self.keys.clone();
                                keys.push(Key::with_one_letter(*letter));
                                Some(keys)
                            }
                            false => {
                                if (self.keys[i].len() as usize) < self.max_key_size {
                                    let mut keys = self.keys.clone();
                                    let key = keys[i].add(*letter);
                                    let key_is_prohibited = key.is_prohibited(&self.prohibited);
                                    match key_is_prohibited {
                                        true => None,
                                        false => {
                                            keys[i] = key;
                                            Some(keys)
                                        }
                                    }
                                } else {
                                    None
                                }
                            }
                        };
                        if let Some(keys) = keys {
                            let next = Dfs {
                                keys,
                                alphabet_index: self.alphabet_index + 1,
                                ..*self
                            };
                            next.go();
                        }
                    }
                }
            }
        }
    }
}
