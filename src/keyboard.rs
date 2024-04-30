#![allow(dead_code)]

use crate::{
    dictionary::Dictionary, key::Key, key_sizes_tree::KeySizesTree, letter::Letter,
    partitions::Partitions, penalty::Penalty, prohibited::Prohibited, solution::Solution,
    tally::Tally, word::Word,
};
use rand::Rng;
use std::{fmt, iter};

#[derive(Clone)]
pub struct Keyboard {
    keys: Vec<Key>,
    letter_to_key_index: [Option<usize>; Letter::ALPHABET_SIZE],
}

impl Keyboard {
    pub fn with_keys(keys: Vec<Key>) -> Keyboard {
        let mut letter_to_key_index: [Option<usize>; Letter::ALPHABET_SIZE] = Default::default();
        for (key_index, key) in keys.iter().enumerate() {
            for letter in key.letters() {
                debug_assert!(
                    letter_to_key_index[letter.to_usize_index()].is_none(),
                    "Some keys on the keyboard have duplicate letters."
                );
                letter_to_key_index[letter.to_usize_index()] = Some(key_index);
            }
        }
        Keyboard {
            keys,
            letter_to_key_index,
        }
    }

    pub fn empty() -> Keyboard {
        Keyboard::with_keys(vec![])
    }

    pub fn with_every_letter_on_own_key(alphabet: Key) -> Keyboard {
        let keys = alphabet
            .letters()
            .map(|r| Key::with_one_letter(r))
            .collect::<Vec<Key>>();
        Keyboard::with_keys(keys)
    }

    pub fn without_keys_with_one_letter(&self) -> Keyboard {
        let keys = self
            .keys
            .iter()
            .filter_map(|k| if k.len() > 1 { Some(*k) } else { None })
            .collect::<Vec<Key>>();
        Keyboard::with_keys(keys)
    }

    /// Generates a keyboard based on a sequence of letters delimited by spaces
    /// or commas. For example "abc,def,ghi" or "abc def ghi".
    pub fn with_layout(s: &str) -> Keyboard {
        let keys = s
            .split([',', ' '])
            .map(|letters| {
                Key::try_from(letters).expect("Expected each key to have valid letters and be separated by a single comma or space.")
            })
            .collect::<Vec<Key>>();
        Keyboard::with_keys(keys)
    }

    pub fn to_solution(self, penalty: Penalty, notes: String) -> Solution {
        Solution::new(self, penalty, notes)
    }

    pub fn has_prohibited_keys(&self, prohibited: &Prohibited) -> bool {
        self.keys.iter().any(|k| !prohibited.is_allowed(*k))
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn letters(&self) -> Key {
        self.keys
            .iter()
            .fold(Key::EMPTY, |total, i| total.union(*i))
    }

    pub fn max_key_size(&self) -> Option<u32> {
        self.keys.iter().map(|k| k.count_letters()).max()
    }

    pub fn min_key_size(&self) -> Option<u32> {
        self.keys.iter().map(|k| k.count_letters()).min()
    }

    pub fn key_sizes(&self) -> Tally<u32> {
        self.keys.iter().fold(Tally::new(), |mut total, i| {
            total.increment(i.len());
            total
        })
    }

    fn find_key(&self, letter: Letter) -> Option<Key> {
        let key_index = self.find_key_index(letter)?;
        let key = self.keys.get(key_index)?;
        Some(*key)
    }

    fn find_key_index(&self, letter: Letter) -> Option<usize> {
        self.letter_to_key_index[letter.to_usize_index()]
    }

    /// Returns the keys that need to be typed to enter a specific word. Each
    /// key is described by the letters on that key, and each key is separated
    /// by a comma. For example, to spell the word "the", the answer might be
    /// "tmn,ehx,ehx".
    pub fn spell(&self, word: &Word) -> String {
        let result = word
            .letters()
            .map(|letter| self.find_key(letter))
            .collect::<Option<Vec<Key>>>()
            .map(|keys| keys.iter().map(|k| k.to_string()).collect::<Vec<String>>())
            .map(|kk| kk.join(","));
        match result {
            None => panic!(
                "Could not spell the word {} because the keyboard is missing a necessary key.",
                word
            ),
            Some(spelling) => spelling,
        }
    }

    /// Converts the sequence of keys needed to be typed to enter a specific
    /// word serialized as a u128.
    fn spell_serialized(&self, word: &Word) -> u128 {
        let mut result: u128 = 0;
        for letter in word.letters() {
            match self.find_key_index(letter) {
                Some(index) => {
                    result = result << 5;
                    result = result | (index as u128 + 1);
                }
                None => panic!(
                    "Could not spell the word {} because the keyboard is missing the letter {}",
                    word, letter
                ),
            }
        }
        result
    }

    /// Returns an endless iteration of random keyboards given a specific
    /// alphabet, key sizes, and a prohibited list of letters that can not
    /// appear together on the same key.
    pub fn random<'a>(
        alphabet: Key,
        layout: &'a Partitions,
        prohibited: &'a Prohibited,
    ) -> impl Iterator<Item = Keyboard> + 'a {
        assert!(
            alphabet.len() == layout.sum,
            "The layout sum must be the exact same as the the number of letters in the alphabet."
        );
        let mut rng = rand::thread_rng();
        let layout_options = layout.calculate();
        iter::repeat_with(move || {
            let layout_index = rng.gen_range(0..layout_options.len());
            let layout = layout_options.get(layout_index).unwrap();
            let mut keys = vec![];
            let mut remain = alphabet;
            for key_size in layout {
                let try_take = 5;
                let key = std::iter::repeat_with(|| remain.random_subset(*key_size..=*key_size))
                    .take(try_take)
                    .find(|k| k.is_allowed(&prohibited));
                match key {
                    Some(key) => {
                        keys.push(key);
                        remain = remain.except(key);
                    }
                    None => {
                        // This can occur if the first say 8 keys all satisfy
                        // the prohibited key list, but there is no way to
                        // satisfy the prohibited keys with the remaining
                        // letters.
                        break;
                    }
                }
            }
            match keys.len() == layout.len() {
                true => Some(Keyboard::with_keys(keys)),
                false => None,
            }
        })
        .filter_map(|k| k)
    }

    pub fn swap_random_letters_n_times(&self, count: u32) -> Result<Keyboard, &'static str> {
        if count == 0 {
            Ok(self.clone())
        } else {
            let k = self.swap_random_letters()?;
            k.swap_random_letters_n_times(count - 1)
        }
    }

    /// Randomly swaps 2 letters on the keyboard. May fail if the keyboard only
    /// has 1 key.
    pub fn swap_random_letters(&self) -> Result<Keyboard, &'static str> {
        let total_keys = self.keys.len();
        if total_keys == 1 {
            Err("It is not possible to swap letters on a keyboard with only 1 key.")
        } else if total_keys == 0 {
            Err("It is not possible to swap letters on a keyboard with 0 keys.")
        } else {
            let mut rng = rand::thread_rng();
            let from_index = rng.gen_range(0..total_keys);
            let to_index = iter::repeat_with(move || rng.gen_range(0..total_keys))
                .find(|n| *n != from_index)
                .unwrap();
            let a_key = self.keys[from_index];
            let b_key = self.keys[to_index];
            let (new_a_key, new_b_key) = a_key.swap_random_letter(&b_key).unwrap();
            let new_keys = self
                .keys
                .iter()
                .map(|k| {
                    if *k == a_key {
                        new_a_key
                    } else if *k == b_key {
                        new_b_key
                    } else {
                        *k
                    }
                })
                .collect();
            Ok(Keyboard::with_keys(new_keys))
        }
    }

    /// Generates the sequence of all keyboards where every letter is swapped
    /// with every other letter.
    pub fn every_swap(&self) -> Vec<Keyboard> {
        if self.keys.len() < 2 {
            panic!("Can not swap keys on a keyboard with less than 2 keys on it.")
        }
        let mut result = vec![];
        for a_key_index in 0..=self.keys.len() - 2 {
            for b_key_index in a_key_index + 1..=(self.keys.len() - 1) {
                let a_key = self.keys[a_key_index];
                let b_key = self.keys[b_key_index];
                for a_letter in a_key.letters() {
                    for b_letter in b_key.letters() {
                        if a_letter < b_letter {
                            let a_key_after = a_key.remove(a_letter).add(b_letter);
                            let b_key_after = b_key.remove(b_letter).add(a_letter);
                            let letters = self
                                .keys
                                .iter()
                                .map(|k| {
                                    if *k == a_key {
                                        a_key_after
                                    } else if *k == b_key {
                                        b_key_after
                                    } else {
                                        *k
                                    }
                                })
                                .collect();
                            let keyboard = Keyboard::with_keys(letters);
                            result.push(keyboard);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn penalty_by_word<'a>(
        &'a self,
        dictionary: &'a Dictionary,
    ) -> impl Iterator<Item = (&Word, Penalty)> {
        let mut found = Tally::new();
        dictionary.words().iter().map(move |word| {
            let how_to_spell = self.spell_serialized(word);
            let serialized_count = found.increment(how_to_spell);
            let word_penalty = match serialized_count {
                1 => Penalty::ZERO,
                _ => Penalty::new(word.frequency().to_f32() * (serialized_count - 1).min(4) as f32),
            };
            (word, word_penalty)
        })
    }

    /// Calculate the total penalty for a keyboard based on the `dictionary` and
    /// a `to_beat` penalty. Calculation is short-circuited if the calculated
    /// penalty exceeds the `to_beat` value.
    pub fn penalty(&self, dictionary: &Dictionary, to_beat: Penalty) -> Penalty {
        let mut penalty = Penalty::ZERO;
        for (_, word_penalty) in self.penalty_by_word(dictionary) {
            penalty = penalty + word_penalty;
            if penalty > to_beat {
                break;
            }
        }
        penalty
    }

    /// Given a keyboard that lacks specific letters in the alphabet, fills in
    /// additional keys with each letter on its own key.
    pub fn fill_missing(&self, alphabet: Key) -> Keyboard {
        let add = alphabet.letters().filter_map(|r| match self.find_key(r) {
            None => Some(Key::with_one_letter(r)),
            Some(_) => None,
        });
        let new_keys = add.fold(self.keys.clone(), |mut total, i| {
            total.push(i);
            total
        });
        Keyboard::with_keys(new_keys)
    }

    pub fn add_key(&self, key: Key) -> Keyboard {
        let mut keys = self.keys.clone();
        keys.push(key);
        Keyboard::with_keys(keys)
    }

    /// Generates all possible keyboards in a depth-first manner, adding keys
    /// one at a time. Each keyboard is first mapped to a `Prunable` using the
    /// `prune` function. If the keyboard should be pruned, it is returned in
    /// the results and no further depth-first traversal happens in that branch.
    /// All intermediate results are returned. So if you're trying to make a
    /// keyboard of length 10, this function will return keyboards of length
    /// 1..=10.
    pub fn with_dfs<'a, F, G>(
        letters: Key,
        key_sizes: &Partitions,
        prune: &'a F,
    ) -> impl Iterator<Item = G> + 'a
    where
        F: (Fn(&Keyboard) -> G) + 'a,
        G: Pruneable + Sized + Clone + 'a,
    {
        assert_eq!(
            letters.len(),
            key_sizes.sum,
            "The total number of letters must equal the partition sum."
        );
        let tree = KeySizesTree::new(key_sizes);
        let start = Keyboard::empty();
        start.with_dfs_util(letters, tree, prune)
    }

    pub fn with_dfs_util<'a, F, G>(
        self,
        letters: Key,
        key_sizes: KeySizesTree,
        prune: &'a F,
    ) -> impl Iterator<Item = G> + 'a
    where
        F: (Fn(&Keyboard) -> G) + 'a,
        G: Pruneable + Sized + Clone + 'a,
    {
        let current = prune(&self);
        let current_take = if self.len() == 0 { 0 } else { 1 };
        if current.should_prune() || key_sizes.is_empty() {
            let result = std::iter::once(current).take(current_take);
            let result: Box<dyn Iterator<Item = G>> = Box::new(result);
            result
        } else {
            let result = key_sizes.next().flat_map(move |(key_size, key_sizes)| {
                let min_letter = letters.min_letter().unwrap();
                let remaining_letters = letters.remove(min_letter);
                let other_letters_for_key = remaining_letters.subsets_of_size(key_size - 1);
                let new_keys = other_letters_for_key.map(move |o| {
                    let new_key = o.add(min_letter);
                    let remaining_letters = letters.except(new_key);
                    (new_key, remaining_letters)
                });
                let kbd = self.clone();
                let keyboards = new_keys.flat_map(move |(new_key, letters)| {
                    let k = kbd.add_key(new_key);
                    let descendents = k.with_dfs_util(letters, key_sizes.clone(), prune);
                    descendents
                });
                keyboards
            });
            let current = std::iter::once(current.clone()).take(current_take);
            let result: Box<dyn Iterator<Item = G>> = Box::new(current.chain(result));
            result
        }
    }
}

pub trait Pruneable {
    fn should_prune(&self) -> bool;
}

impl fmt::Display for Keyboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = self
            .keys
            .iter()
            .map(|k| Key::to_string(k))
            .collect::<Vec<String>>();
        result.sort();
        let result = result.join(" ");
        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn new_panic_if_keys_with_duplicate_letters() {
        Keyboard::with_layout("abc,def,ghi,axy");
    }

    #[test]
    fn spell_test() {
        let k = Keyboard::with_layout("abc,def,ghi,jkl,mno,pqr,stu,vwx,yz'");
        let w = Word::try_from("word").unwrap();
        let actual = k.spell(&w);
        assert_eq!(actual, "vwx,mno,pqr,def");
    }

    #[test]
    fn spell_serialized_test() {
        let k = Keyboard::with_layout("abc,def,ghi,jkl,mno,pqr,stu,vwx,y'z");
        let data = [
            ("adg", "beh", true),
            ("adgj", "behk", true),
            ("a", "b", true),
            ("a", "c", true),
            ("a", "d", false),
            ("abc", "cba", true),
            ("abc", "cbaz", false),
            ("z", "y", true),
            ("zzm", "yzm", true),
            ("jmr", "jma", false),
            ("poad", "pobg", false),
            ("poad", "rmaf", true),
        ];
        for (w1, w2, expect_are_same) in data {
            let w1_spell = k.spell_serialized(&Word::try_from(w1).unwrap());
            let w2_spell = k.spell_serialized(&Word::try_from(w2).unwrap());
            assert_eq!(w1_spell == w2_spell, expect_are_same);
        }
    }

    #[test]
    #[should_panic]
    fn spell_panic_if_required_letter_not_on_keyboard() {
        let k = Keyboard::with_layout("abc,def,ghi");
        let w = Word::try_from("abcx").unwrap();
        k.spell(&w);
    }

    #[test]
    fn find_key_index_for_letter() {
        let data = [
            ("abc", 'a', Some(0)),
            ("abc", 'b', Some(0)),
            ("abc", 'c', Some(0)),
            ("abc", 'x', None),
            ("abc", 'b', Some(0)),
            ("abc,def", 'd', Some(1)),
            ("abc,def", 'e', Some(1)),
            ("abc,def", 'f', Some(1)),
            ("abc,def", 'x', None),
        ];
        for (layout, letter, expected_key_index) in data {
            let keyboard = Keyboard::with_layout(layout);
            let letter_to_find = Letter::new(letter);
            let actual = keyboard.find_key_index(letter_to_find);
            assert_eq!(actual, expected_key_index);
        }
    }

    #[test]
    #[ignore]
    fn spell_print_each_dictionary_word_out() {
        let d = Dictionary::load();
        let k = Keyboard::with_layout("abc,def,ghi,jkl,mnop,qrs,tuv,wxyz'");
        d.words().iter().take(20).for_each(|w| {
            let spelling = k.spell(&w);
            println!("{} : {}", w, spelling);
        })
    }

    #[test]
    fn penalty_score_is_correct() {
        let d = Dictionary::load();
        let k = Keyboard::with_layout("abc,def,ghi,jkl,mno,pqr,st,uv,wx,yz'");
        let actual: f32 = k.penalty(&d, Penalty::MAX).to_f32(); // why into does not work
        assert!(actual >= 0.0802 && actual <= 0.0804); // 0.0803
    }

    #[test]
    #[ignore]
    fn swap_random_letters() {
        let mut k = Keyboard::with_layout("abc,def,ghi");
        for _i in 1..10 {
            k = k.swap_random_letters().unwrap();
            println!("{}", k)
        }
    }

    #[test]
    fn letters() {
        let data = [("abc,def,ghi", "abcdefghi"), ("abc", "abc"), ("", "")];
        for (keyboard, expected) in data {
            let keyboard = Keyboard::with_layout(&keyboard);
            let actual = keyboard.letters();
            let expected = Key::new(expected);
            assert_eq!(actual, expected)
        }
    }

    #[test]
    #[ignore]
    fn every_swap() {
        let k = Keyboard::with_layout("abc,def,ghi,jkl,mno,pqr,stu,vw,xy,z'");
        k.every_swap().iter().for_each(|k| println!("{}", k));
        println!("Total swaps: {}", k.every_swap().iter().count());
    }

    #[test]
    fn has_prohibited_keys_true_if_any_prohibited() {
        let data = [
            ("abc,def,ghi", "de", true),
            ("abc,def,ghi", "gh", true),
            ("abc,def,ghi", "c", true),
            ("abc,def,ghi", "fg", false),
            ("abc,def,ghi", "ag", false),
            ("abc,def,ghi", "adg", false),
        ];
        for d in data {
            let (keyboard_layout, prohibited_items, expect_is_prohibited) = d;
            let k = Keyboard::with_layout(&keyboard_layout);
            let mut prohibited = Prohibited::new();
            prohibited.add_many(prohibited_items.split(",").map(|rr| Key::new(rr)));
            let actual = k.has_prohibited_keys(&prohibited);
            assert_eq!(
                actual, expect_is_prohibited,
                "for keyboard {} and prohibited {}",
                keyboard_layout, prohibited_items
            );
        }
    }

    #[test]
    fn with_layout_can_split_on_comma_or_space() {
        let a = Keyboard::with_layout("abc,def,ghi");
        let b = Keyboard::with_layout("abc def ghi");
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    #[ignore]
    fn create_file_of_penalty_per_word() {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create("output.txt").unwrap();
        writeln!(file, "index, word, penalty").unwrap();
        let d = Dictionary::load();
        let keyboard = Keyboard::with_layout("ot,gr,dh,su,im,bn,awz,cky',fjlx,epqv");
        for (word_index, (word, penalty)) in keyboard.penalty_by_word(&d).enumerate() {
            writeln!(file, "{},{},{}", word_index + 1, word, penalty.to_f32()).unwrap();
        }
    }

    #[test]
    #[ignore]
    fn display_penalty_for_specific_keyboard() {
        let dict = Dictionary::load();
        let layout = "ajxz' biky cglov dfpu emq h n r sw t";
        let keyboard = Keyboard::with_layout(layout);
        let penalty = keyboard.penalty(&dict, Penalty::MAX);
        let solution = keyboard.to_solution(penalty, "".to_string());
        println!("{}", solution);
    }

    #[test]
    #[ignore]
    fn random_with_display() {
        let dict = Dictionary::load();
        let layout = Partitions {
            sum: 27,
            parts: 10,
            min: 2,
            max: 4,
        };
        let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 50);
        for k in Keyboard::random(dict.alphabet(), &layout, &prohibited).take(20) {
            println!("{}", k);
        }
    }

    mod dfs_builder {
        use super::{Prohibited, Pruneable, Tally};
        use crate::keyboard::{
            tests::{Key, Partitions},
            Keyboard,
        };
        use core::fmt;
        use std::cell::RefCell;

        #[derive(Clone)]
        struct KeyboardStatus {
            keyboard: Keyboard,
            has_bad_letters: bool,
        }

        impl fmt::Display for KeyboardStatus {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let indent = std::iter::repeat("   ")
                    .take(self.keyboard.len())
                    .collect::<Vec<&str>>()
                    .join("");
                write!(
                    f,
                    "{}  '{}' {}",
                    indent,
                    self.keyboard.to_string(),
                    if self.has_bad_letters { "bad" } else { "" }
                )
            }
        }

        impl KeyboardStatus {
            fn new_ok(k: &Keyboard) -> KeyboardStatus {
                let has_bad_letters = false;
                KeyboardStatus {
                    keyboard: k.clone(),
                    has_bad_letters: has_bad_letters,
                }
            }

            fn evaluate(k: &Keyboard, disallow: Key) -> KeyboardStatus {
                let mut prohibited = Prohibited::new();
                prohibited.add(disallow);
                let has_bad_letters = k.has_prohibited_keys(&prohibited);
                KeyboardStatus {
                    keyboard: k.clone(),
                    has_bad_letters: has_bad_letters,
                }
            }
        }

        impl Pruneable for KeyboardStatus {
            fn should_prune(&self) -> bool {
                self.has_bad_letters
            }
        }

        #[test]
        fn with_dfs_creates_proper_number_of_keyboards() {
            let data = [(8, 3), (4, 2), (7, 3), (1, 1), (2, 2), (6, 3)];
            for (letter_count, key_count) in data {
                let key_sizes = Partitions {
                    sum: letter_count,
                    min: 1,
                    max: letter_count,
                    parts: key_count,
                };
                let expected = key_sizes.total_unique_keyboards();
                let alphabet = Key::with_first_n_letters(letter_count);
                let prune = |k: &Keyboard| KeyboardStatus::new_ok(k);
                let actual = Keyboard::with_dfs(alphabet, &key_sizes, &prune)
                    .filter(|k| k.keyboard.len() == key_count as usize)
                    .count();
                assert_eq!(expected, actual as u128);
            }
        }

        #[test]
        fn with_dfs_never_calls_prune_more_than_once_per_keyboard() {
            let data = [(8, 3), (8, 5), (4, 2), (7, 3), (1, 1), (2, 2), (6, 3)];
            for (letter_count, key_count) in data {
                let key_sizes = Partitions {
                    sum: letter_count,
                    min: 1,
                    max: letter_count,
                    parts: key_count,
                };
                let alphabet = Key::with_first_n_letters(letter_count);
                let prune_count: RefCell<Tally<String>> = RefCell::new(Tally::new());
                let prune = |k: &Keyboard| {
                    let mut tally = prune_count.borrow_mut();
                    if tally.increment(k.to_string()) > 1 {
                        panic!("Tried to evaluate the same keyboard more than once.");
                    }
                    KeyboardStatus::new_ok(k)
                };
                Keyboard::with_dfs(alphabet, &key_sizes, &prune).count();
            }
        }

        #[test]
        fn with_dfs_creates_unique_intermediate_keyboards() {
            let data = [(8, 5), (6, 2), (5, 1), (1, 1), (4, 2), (4, 3)];
            for (letter_count, key_count) in data {
                let alphabet = Key::with_first_n_letters(letter_count);
                let prune = |k: &Keyboard| KeyboardStatus::new_ok(k);
                (2..key_count)
                    .into_iter()
                    .map(|len| {
                        let key_sizes = Partitions {
                            sum: letter_count,
                            min: 1,
                            max: letter_count,
                            parts: key_count,
                        };
                        let mut tally = Tally::new();
                        for k in Keyboard::with_dfs(alphabet, &key_sizes, &prune)
                            .filter(|k| k.keyboard.len() == len as usize)
                        {
                            let count = tally.increment(k.keyboard.to_string());
                            assert!(count < 2, "Expected only unique keyboards of size {}", len);
                        }
                    })
                    .count();
            }
        }

        #[test]
        fn with_dfs_never_returns_empty_keyboard() {
            let data = [(8, 3), (6, 3), (1, 1), (4, 3), (2, 1)];
            for (letter_count, key_count) in data {
                let alphabet = Key::with_first_n_letters(letter_count);
                let prune = |k: &Keyboard| KeyboardStatus::new_ok(k);
                let key_sizes = Partitions {
                    sum: letter_count,
                    min: 1,
                    max: letter_count,
                    parts: key_count,
                };
                assert!(
                    Keyboard::with_dfs(alphabet, &key_sizes, &prune).all(|k| k.keyboard.len() > 0)
                );
            }
        }

        #[test]
        fn with_dfs_creates_unique_final_keyboards() {
            let data = [(8, 3), (5, 1), (4, 3), (4, 2)];
            for (letter_count, key_count) in data {
                let alphabet = Key::with_first_n_letters(letter_count);
                let prune = |k: &Keyboard| KeyboardStatus::new_ok(k);
                let key_sizes = Partitions {
                    sum: letter_count,
                    min: 1,
                    max: letter_count,
                    parts: key_count,
                };
                let mut tally = Tally::new();
                for k in Keyboard::with_dfs(alphabet, &key_sizes, &prune)
                    .filter(|k| k.keyboard.len() == key_count as usize)
                {
                    let count = tally.increment(k.keyboard.to_string());
                    assert!(
                        count < 2,
                        "Expected only unique keyboards of size {}",
                        key_count
                    );
                }
            }
        }

        #[test]
        fn with_dfs_pruned_keyboard_is_last() {
            let data = [(6, 4, "ab"), (6, 4, "cd"), (6, 4, "ad"), (5, 3, "ab")];
            for (letter_count, key_count, prohibited) in data {
                let key_sizes = Partitions {
                    sum: letter_count,
                    min: 1,
                    max: letter_count,
                    parts: key_count,
                };
                let prohibited = Key::new(prohibited);
                let prune = |k: &Keyboard| KeyboardStatus::evaluate(k, prohibited);
                let alphabet = Key::with_first_n_letters(letter_count);
                assert!(Keyboard::with_dfs(alphabet, &key_sizes, &prune).all(|k| {
                    k.keyboard.keys.iter().enumerate().all(|(index, key)| {
                        if key.contains_all(&prohibited) {
                            index == (k.keyboard.keys.len() - 1)
                        } else {
                            true
                        }
                    })
                }));
            }
        }

        #[test]
        #[ignore]
        fn with_dfs_print_keyboards() {
            let data = [(5, 3)];
            for (letter_count, key_count) in data {
                let key_sizes = Partitions {
                    sum: letter_count,
                    min: 1,
                    max: letter_count,
                    parts: key_count,
                };
                let alphabet = Key::with_first_n_letters(letter_count);
                let prune = |k: &Keyboard| KeyboardStatus::new_ok(k);
                let actual = Keyboard::with_dfs(alphabet, &key_sizes, &prune);
                for k in actual {
                    println!("{}", k);
                }
            }
        }
    }
}
