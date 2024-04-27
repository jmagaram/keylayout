#![allow(dead_code)]

use core::fmt;

use crate::{
    dictionary::Dictionary,
    key::Key,
    keyboard::Keyboard,
    letter::Letter,
    penalty::{self, Penalty},
    penalty_goal::PenaltyGoals,
    prohibited::Prohibited,
    solution::Solution,
};

#[derive(Clone, Copy)]
struct SizeLimitedKey {
    key: Key,
    max_size: usize,
}

impl SizeLimitedKey {
    pub fn new(size: usize) -> SizeLimitedKey {
        SizeLimitedKey {
            key: Key::EMPTY,
            max_size: size,
        }
    }

    pub fn has_space(&self, letter: Letter) -> bool {
        (self.key.len() as usize) < self.max_size
            && self.key.max_letter().map_or(true, |r| r < letter)
    }

    pub fn insert_letter(&self, letter: Letter) -> SizeLimitedKey {
        SizeLimitedKey {
            key: self.key.add(letter),
            ..*self
        }
    }
}

#[derive(Clone)]
struct KeyboardInProcess(Vec<SizeLimitedKey>);

impl KeyboardInProcess {
    pub fn new() -> KeyboardInProcess {
        let items = [3, 3, 3, 3, 3, 3, 3, 2, 2, 2]
            .map(|size| SizeLimitedKey::new(size as usize))
            .to_vec();
        KeyboardInProcess(items)
    }

    pub fn slot_indexes(&self, letter: Letter) -> Vec<usize> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(index, s)| match s.has_space(letter) {
                true => Some(index),
                false => None,
            })
            .collect::<Vec<usize>>()
    }

    pub fn insert_in_slot(&self, slot: usize, letter: Letter) -> KeyboardInProcess {
        let keys = self
            .0
            .iter()
            .enumerate()
            .map(|(index, k)| {
                if index == slot {
                    k.insert_letter(letter)
                } else {
                    k.clone()
                }
            })
            .collect::<Vec<SizeLimitedKey>>();
        KeyboardInProcess(keys)
    }

    pub fn as_keyboard(&self) -> Keyboard {
        Keyboard::with_keys(self.0.iter().map(|s| s.key).collect::<Vec<Key>>())
    }

    pub fn fill_in(
        &self,
        letters: Vec<Letter>,
        penalty_goals: &PenaltyGoals,
        dict: &Dictionary,
        prohibited: &Prohibited,
        count: &mut u64,
    ) -> Option<Solution> {
        *count = *count + 1;
        // println!("{}", count);
        let k = self.as_keyboard();
        println!("{} {}", count, k);
        let has_prohibited = k.has_prohibited_keys(prohibited);
        if has_prohibited {
            None
        } else {
            let to_beat = penalty_goals
                .get(self.0.len() as u8)
                .unwrap_or(Penalty::MAX);
            let k = k.fill_missing(dict.alphabet());
            let p = k.penalty(dict, to_beat);
            if p > to_beat {
                None
            } else {
                if letters.len() == 0 {
                    Some(k.to_solution(p, "".to_string()))
                } else {
                    let (first_letter, rest_letters) = letters.split_first().unwrap();
                    let rest_letters = rest_letters.to_vec();
                    let first_letter_inserted = self
                        .slot_indexes(*first_letter)
                        .iter()
                        .map(|inx| self.insert_in_slot(*inx, *first_letter))
                        .collect::<Vec<KeyboardInProcess>>();
                    first_letter_inserted
                        .iter()
                        .map(|k| {
                            k.fill_in(rest_letters.clone(), penalty_goals, dict, prohibited, count)
                        })
                        .find_map(|s| s)
                }
            }
        }
    }
}

impl fmt::Display for KeyboardInProcess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let k = Keyboard::with_keys(self.0.iter().map(|s| s.key).collect::<Vec<Key>>());
        write!(f, "{}", k.to_string())
    }
}

pub fn solver() {
    let letters = "eariotnslcudpmhgbfywvkxzjq'"
        .chars()
        .map(|c| Letter::new(c))
        .collect::<Vec<Letter>>();
    let kbd = KeyboardInProcess::new();
    let dict = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 55);
    let penalty_goals = PenaltyGoals::none(dict.alphabet())
        .with(26, Penalty::new(0.00006))
        .with(25, Penalty::new(0.000174))
        .with(24, Penalty::new(0.000385))
        .with(23, Penalty::new(0.0007))
        .with(22, Penalty::new(0.0012))
        .with(21, Penalty::new(0.001974))
        .with(20, Penalty::new(0.002559))
        .with(19, Penalty::new(0.003633))
        .with(18, Penalty::new(0.004623))
        .with(17, Penalty::new(0.005569))
        .with(16, Penalty::new(0.007603))
        .with(15, Penalty::new(0.009746))
        .with(14, Penalty::new(0.013027))
        .with(13, Penalty::new(0.016709))
        .with(12, Penalty::new(0.02109))
        .with_adjustment(11..=23, 0.75)
        .with(10, Penalty::new(0.0246));
    let mut count: u64 = 0;
    let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 45);
    let solution = kbd.fill_in(letters, &penalty_goals, &dict, &prohibited, &mut count);
    match solution {
        None => println!("Not found"),
        Some(solution) => println!("{}", solution),
    }
}

#[cfg(test)]
mod tests {

    use crate::{dictionary::Dictionary, penalty_goal::PenaltyGoals};

    use super::*;

    // #[test]
    // fn solve_everything() {
    //     let letters = "eariotnslcudpmhgbfywvkxzjq'"
    //         .chars()
    //         .map(|c| Letter::new(c))
    //         .collect::<Vec<Letter>>();
    //     let kbd = KeyboardInProcess::new();
    //     let dict = Dictionary::load();
    //     let penalty_goals = PenaltyGoals::none(dict.alphabet())
    //         .with(26, Penalty::new(0.00006))
    //         .with(25, Penalty::new(0.000174))
    //         .with(24, Penalty::new(0.000385))
    //         .with(23, Penalty::new(0.0007))
    //         .with(22, Penalty::new(0.0012))
    //         .with(21, Penalty::new(0.001974))
    //         .with(20, Penalty::new(0.002559))
    //         .with(19, Penalty::new(0.003633))
    //         .with(18, Penalty::new(0.004623))
    //         .with(17, Penalty::new(0.005569))
    //         .with(16, Penalty::new(0.007603))
    //         .with(15, Penalty::new(0.009746))
    //         .with(14, Penalty::new(0.013027))
    //         .with(13, Penalty::new(0.016709))
    //         .with(12, Penalty::new(0.02109))
    //         .with_adjustment(11..=23, 0.75)
    //         .with(10, Penalty::new(0.050));
    //     let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 45);
    //     let mut count: u64 = 0;
    //     let solution = kbd.fill_in(letters, &&penalty_goals, &dict, &prohibited, &mut count);
    //     match solution {
    //         None => println!("Not found"),
    //         Some(solution) => println!("{}", solution),
    //     }
    // }
}
