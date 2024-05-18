#![allow(dead_code)]
#![allow(unused_variables)]

use std::cell::RefCell;

use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, letter::Letter, penalty::Penalty,
    prohibited::Prohibited, solution::Solution,
};

pub struct Args {
    pub max_key_size: usize,
    pub prohibited_pairs: usize,
}

impl Args {
    pub fn solve(&self) {
        let dictionary = Dictionary::load();
        let alphabet = dictionary.alphabet().letters().collect::<Vec<Letter>>();
        let best: RefCell<Option<Solution>> = RefCell::new(None);
        let prohibited = Prohibited::with_top_n_letter_pairs(&dictionary, self.prohibited_pairs);
        let dfs = Dfs {
            dictionary: &dictionary,
            alphabet: &alphabet,
            alphabet_index: 0,
            keys: vec![],
            max_key_size: self.max_key_size,
            best: &best,
            prohibited: &prohibited,
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
    prohibited: &'a Prohibited,
}

impl<'a> Dfs<'a> {
    pub fn go(&self) {
        let letter = self.alphabet.get(self.alphabet_index);
        match letter {
            None => {
                let keyboard = Keyboard::with_keys(self.keys.clone());
                let best_penalty = self
                    .best
                    .borrow()
                    .as_ref()
                    .map(|b| b.penalty())
                    .unwrap_or(Penalty::MAX);
                let penalty = keyboard.penalty(&self.dictionary, best_penalty);
                if penalty < best_penalty {
                    let solution = keyboard.to_solution(penalty, "".into());
                    println!("{}", solution);
                    self.best.replace(Some(solution));
                }
            }
            Some(letter) => {
                for i in 0..=self.keys.len().min(9) {
                    let insert_key = i == self.keys.len();
                    let keys = match insert_key {
                        true => {
                            let mut keys = self.keys.clone();
                            keys.push(Key::with_one_letter(*letter));
                            Some(keys)
                        }
                        false => {
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
