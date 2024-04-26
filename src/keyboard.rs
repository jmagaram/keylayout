use std::{fmt, iter};

use hashbrown::HashMap;
use rand::Rng;

use crate::{
    dictionary::Dictionary, key::Key, letter::Letter, partitions::Partitions, penalty::Penalty,
    prohibited::Prohibited, solution::Solution, tally::Tally, util, word::Word,
};

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

    pub fn with_every_letter_on_own_key(alphabet: Key) -> Keyboard {
        let keys = alphabet
            .letters()
            .map(|r| Key::with_one_letter(r))
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

    pub fn key_count(&self) -> usize {
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

    fn find_key_for_letter(&self, letter: Letter) -> Option<Key> {
        let key_index = self.find_key_index_for_letter(letter)?;
        let key = self.keys.get(key_index)?;
        Some(*key)
    }

    fn find_key_index_for_letter(&self, letter: Letter) -> Option<usize> {
        self.letter_to_key_index[letter.to_usize_index()]
    }

    /// Returns the keys that need to be typed to enter a specific word. Each
    /// key is described by the letters on that key, and each key is separated
    /// by a comma. For example, to spell the word "the", the answer might be
    /// "tmn,ehx,ehx".
    pub fn spell(&self, word: &Word) -> String {
        let result = word
            .letters()
            .map(|letter| self.find_key_for_letter(letter))
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
            match self.find_key_index_for_letter(letter) {
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

    pub fn contains_on_any_key(&self, other: &Vec<Key>) -> bool {
        self.keys
            .iter()
            .any(|k| other.iter().any(|o| k.contains_all(o)))
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
            alphabet.count_letters() == layout.sum,
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

    // pub fn random_with_key_sizes(alphabet: Key, prohibited: Prohibited, key_sizes: Tally<u8>) {}

    // pub fn every<'a>(alphabet: Key, layout: &'a Partitions) -> impl Iterator<Item = Keyboard> + 'a {
    //     assert!(
    //         alphabet.count_letters() == layout.sum,
    //         "The layout must have the same number of letters as the alphabet."
    //     );
    //     let key_sizes = layout.calculate();
    //     let alphabet = alphabet.clone();
    //     key_sizes.into_iter().flat_map(move |s| {
    //         let arrangements: Tally<u32> = Tally::from(s);
    //         alphabet
    //             .clone()
    //             .distribute(arrangements)
    //             .map(|keys| Keyboard::with_keys(keys))
    //     })
    // }

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

    /// Generates every keyboard, including the current one, that results from
    /// recursively combining keys in a depth-first manner. So if you start with
    /// a 15 key keyboard, this returns the current keyboard plus all possible
    /// 14, 13, 12, ... size keyboards down to the case where every letter is on
    /// a single key. Duplicates do not occur.
    pub fn every_smaller(&self) -> impl Iterator<Item = Keyboard> {
        let explorer = KeyCombiner {
            keyboard: self.clone(),
            index: 0,
        };
        explorer.dfs().map(|k| k.keyboard)
    }

    /// Generates every keyboard, including the current one, that results from
    /// recursively combining keys in a depth-first manner. So if you start with
    /// a 15 key keyboard, this returns the current keyboard plus all possible
    /// 14, 13, 12, ... size keyboards down to the case where every letter is on
    /// a single key. Duplicates do not occur. The `prune` function stops the
    /// depth-first traversal, making it possible to stop searching based on
    /// maximum key size or the penalty score.
    pub fn every_smaller_with<'a, F>(self, prune: &'a F) -> impl Iterator<Item = Keyboard> + 'a
    where
        F: (Fn(&Keyboard) -> bool) + 'a,
    {
        let explorer = KeyCombiner {
            keyboard: self.clone(),
            index: 0,
        };
        explorer.dfs_with(prune).map(|k| k.keyboard)
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

    /// Given a specific keyboard, generates all possible keyboards resulting from taking `size` keys.
    pub fn subsets_of_keys<'a>(&'a self, size: usize) -> impl Iterator<Item = Keyboard> + 'a {
        assert!(
            size <= self.key_count(),
            "Can not create subset keyboards with that many keys; more than on the original keyboard."
        );
        let minimum: u64 = (1u64 << size) - 1;
        let maximum: u64 = minimum << (self.key_count() - size);
        util::same_set_bits(size as u32)
            .filter(move |n| *n <= maximum)
            .map(|n| {
                let keys = util::set_bits(n as u32).fold(vec![], |mut total, i| {
                    total.push(self.keys[i]);
                    total
                });
                Keyboard::with_keys(keys)
            })
    }

    /// Given a keyboard that lacks specific letters in the alphabet, fills in
    /// additional keys with each letter on its own key.
    pub fn fill_missing(&self, alphabet: Key) -> Keyboard {
        let add = alphabet
            .letters()
            .filter_map(|r| match self.find_key_for_letter(r) {
                None => Some(Key::with_one_letter(r)),
                Some(_) => None,
            });
        let new_keys = add.fold(self.keys.clone(), |mut total, i| {
            total.push(i);
            total
        });
        Keyboard::with_keys(new_keys)
    }
}

#[derive(Clone)]
struct KeyCombiner {
    keyboard: Keyboard,
    index: usize,
}

impl KeyCombiner {
    pub fn dfs<'a>(self) -> Box<dyn Iterator<Item = KeyCombiner> + 'a> {
        self.dfs_with(&|_k| false)
    }

    pub fn dfs_with<'a, F>(self, prune: &'a F) -> Box<dyn Iterator<Item = KeyCombiner> + 'a>
    where
        F: (Fn(&Keyboard) -> bool) + 'a,
    {
        match prune(&self.keyboard) {
            true => {
                let result = std::iter::empty();
                let boxed_result: Box<dyn Iterator<Item = KeyCombiner>> = Box::new(result);
                boxed_result
            }
            false => {
                if self.keyboard.keys.len() == 1 {
                    let result = std::iter::once(self.clone());
                    let boxed_result: Box<dyn Iterator<Item = KeyCombiner>> = Box::new(result);
                    boxed_result
                } else {
                    let children = self.next();
                    let current = std::iter::once(self);
                    let descendents = children
                        .into_iter()
                        .filter(move |k| false == prune(&k.keyboard))
                        .flat_map(move |child| child.dfs_with(prune));
                    let boxed_result: Box<dyn Iterator<Item = KeyCombiner>> =
                        Box::new(current.chain(descendents));
                    boxed_result
                }
            }
        }
    }

    pub fn next(&self) -> Vec<KeyCombiner> {
        if self.keyboard.key_count() <= 1 {
            vec![]
        } else {
            let can_combine = |a: Key, b: Key| -> bool { a.max_letter() < b.min_letter() };
            let indexes = (self.index as i32..=self.keyboard.key_count() as i32 - 2)
                .flat_map(|i| (i + 1..=self.keyboard.key_count() as i32 - 1).map(move |j| (i, j)))
                .filter_map(|(i, j)| {
                    if i < 0 {
                        None
                    } else {
                        Some((i as usize, j as usize))
                    }
                })
                .filter(|(i, j)| {
                    let i_key = self.keyboard.keys[*i];
                    let j_key = self.keyboard.keys[*j];
                    can_combine(i_key, j_key)
                });
            let parts = indexes.map(|(i, j)| {
                let items = self
                    .keyboard
                    .keys
                    .iter()
                    .enumerate()
                    .into_iter()
                    .flat_map(move |(index, item)| {
                        if index == i {
                            let combined_key = self.keyboard.keys[i].union(self.keyboard.keys[j]);
                            Some(combined_key)
                        } else if index == j {
                            None
                        } else {
                            Some(item.clone())
                        }
                    })
                    .collect::<Vec<Key>>();
                let keyboard = Keyboard::with_keys(items);
                KeyCombiner { keyboard, index: i }
            });
            parts.collect::<Vec<KeyCombiner>>()
        }
    }
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

    use crate::util;

    use super::*;

    #[test]
    fn subsets_of_keys_when_one_key() {
        let source = Keyboard::with_layout("abc");
        let result = source.subsets_of_keys(1).collect::<Vec<Keyboard>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_string(), "abc");
    }

    #[test]
    fn subsets_of_keys_test() {
        let source = Keyboard::with_layout("abc,def,ghi,pqr");
        let result = source.subsets_of_keys(2);
        assert_eq!(6, result.count());
    }

    #[test]
    #[should_panic]
    fn subsets_of_keys_panic_if_too_many() {
        let source = Keyboard::with_layout("abc,def,ghi,pqr");
        source.subsets_of_keys(5).count();
    }

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
            let actual = keyboard.find_key_index_for_letter(letter_to_find);
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
    fn contains_on_any_key_test() {
        let data = [
            ("abc", "a,b,c", true),
            ("abc", "a", true),
            ("abc", "b", true),
            ("abc", "c", true),
            ("abc", "cd", false),
            ("abc", "ad", false),
            ("abc", "x", false),
            ("abc,def", "ac", true),
            ("abc,def", "df", true),
            ("abc,def", "cd", false),
            ("abc,def", "c", true),
            ("abc,def", "x", false),
        ];
        for (keyboard, other, expected) in data {
            let k = Keyboard::with_layout(keyboard);
            let contains = other
                .split(",")
                .map(|p| Key::try_from(p).unwrap())
                .collect::<Vec<Key>>();
            let actual = k.contains_on_any_key(&contains);
            assert_eq!(
                actual, expected,
                "KBD: {}   OTHER: {} EXPECT: {}",
                keyboard, other, expected
            );
        }
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

    #[test]
    fn every_smaller_if_empty_return_self() {
        let start = Keyboard::with_keys(vec![]);
        let result = start.every_smaller().collect::<Vec<Keyboard>>();
        assert_eq!(1, result.len());
        assert_eq!("".to_string(), result[0].to_string());
    }

    #[test]
    fn every_smaller_if_one_key_return_self() {
        let start = Keyboard::with_keys(vec![Key::new("abc")]);
        let result = start.every_smaller().collect::<Vec<Keyboard>>();
        assert_eq!(1, result.len());
        assert_eq!("abc".to_string(), result[0].to_string());
    }

    #[test]
    fn every_smaller_returns_correct_number_of_keyboards() {
        let data = [3, 4, 5, 7];
        for keys in data {
            let start = Keyboard::with_every_letter_on_own_key(Key::with_first_n_letters(keys));
            let expected: u128 = (1..=keys)
                .map(|key_count| Partitions {
                    sum: keys,
                    parts: key_count,
                    min: 1,
                    max: keys,
                })
                .map(|p| p.total_unique_keyboards())
                .sum();
            let actual = start.every_smaller().count() as u128;
            assert_eq!(expected, actual, "for keys {}", keys);
        }
    }

    #[test]
    fn every_smaller_returns_unique_keyboards() {
        let data = [3, 4, 5, 7];
        for keys in data {
            let start = Keyboard::with_every_letter_on_own_key(Key::with_first_n_letters(keys));
            let mut tally = Tally::new();
            start.every_smaller().for_each(move |k| {
                let count = tally.increment(k.to_string());
                assert_eq!(1, count)
            });
        }
    }

    #[test]
    fn every_smaller_can_prune_root() {
        let k = Keyboard::with_every_letter_on_own_key(Key::with_first_n_letters(5));
        let prune = |k: &Keyboard| k.key_count() == 5;
        let actual = k.every_smaller_with(&prune).count();
        assert_eq!(0, actual);
    }

    #[test]
    fn every_smaller_can_prune_base_case_of_single_key() {
        let k = Keyboard::with_every_letter_on_own_key(Key::with_first_n_letters(5));
        let prune = |k: &Keyboard| k.key_count() == 1;
        let base_case_count = k
            .every_smaller_with(&prune)
            .filter(|k| k.key_count() == 1)
            .count();
        assert_eq!(base_case_count, 0);
    }

    #[test]
    fn every_smaller_can_prune() {
        let letter_count = 4;
        let k = Keyboard::with_every_letter_on_own_key(Key::with_first_n_letters(letter_count));
        let prune = |k: &Keyboard| k.max_key_size().map(|ks| ks > 2).unwrap_or(false);
        let count_big_keys = k
            .every_smaller_with(&prune)
            .filter(|k| k.max_key_size().map(|ks| ks > 2).unwrap_or(true))
            .count();
        assert_eq!(count_big_keys, 0);
    }

    #[test]
    #[ignore]
    fn every_smaller_print() {
        let letters = 4;
        // let prune = |k: &Keyboard| k.max_key_size().map(|size| size > 3).unwrap_or(false);
        for k in Keyboard::with_every_letter_on_own_key(Key::with_first_n_letters(letters))
            .every_smaller()
        {
            println!("{}", k)
        }
    }
}
