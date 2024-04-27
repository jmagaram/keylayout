use std::{fmt, iter};

use rand::Rng;

use crate::{
    dictionary::Dictionary, key::Key, letter::Letter, partitions::Partitions, penalty::Penalty,
    prohibited::Prohibited, solution::Solution, tally::Tally, word::Word,
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

    pub fn with_no_keys() -> Keyboard {
        Keyboard::with_keys(vec![])
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

    /// Generates every keyboard, including the current one, that results from
    /// recursively combining keys in a depth-first manner. So if you start with
    /// a 15 key keyboard, this returns the current keyboard plus all possible
    /// 14, 13, 12, ... size keyboards down to the case where every letter is on
    /// a single key. Duplicates do not occur.
    pub fn every_smaller(&self) -> impl Iterator<Item = Keyboard> {
        let explorer = DfsExplorer {
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
        let explorer = DfsExplorer {
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
    /// one at a time. Any keyboard where the `prune` function returns false is
    /// removed from the output, and prevents further depth-first building.
    pub fn with_dfs_builder<'a, F>(
        self,
        letters: Key,
        key_sizes: Partitions,
        prune: &'a F,
    ) -> impl Iterator<Item = Keyboard> + 'a
    where
        F: (Fn(&Keyboard) -> bool) + 'a,
    {
        assert_eq!(
            letters.len(),
            key_sizes.sum,
            "The total number of letters must equal the partition sum."
        );
        if letters.len() != key_sizes.sum {}
        key_sizes
            .calculate()
            .into_iter()
            .map(|key_sizes| Tally::from(key_sizes))
            .flat_map(|t| t.combinations())
            .flat_map(move |key_sizes| {
                Keyboard::dfs_builder_utility(self.clone(), letters, key_sizes.to_vec(), prune)
            })
    }

    fn dfs_builder_utility<'a, F>(
        self,
        letters: Key,
        key_sizes: Vec<u32>,
        prune: &'a F,
    ) -> impl Iterator<Item = Keyboard> + 'a
    where
        F: (Fn(&Keyboard) -> bool) + 'a,
    {
        if prune(&self) {
            let result = std::iter::empty();
            let result: Box<dyn Iterator<Item = Keyboard>> = Box::new(result);
            result
        } else {
            if key_sizes.len() == 0 {
                let result = std::iter::once(self.clone());
                let result: Box<dyn Iterator<Item = Keyboard>> = Box::new(result);
                result
            } else {
                let (key_size, key_sizes) = key_sizes.split_first().unwrap();
                let key_sizes = key_sizes.to_vec();
                let min_letter = letters.min_letter().unwrap();
                let remaining_letters = letters.remove(min_letter);
                let other_letters_for_key = remaining_letters.subsets_of_size(key_size - 1);
                let new_keys = other_letters_for_key.map(move |o| {
                    let new_key = o.add(min_letter);
                    let remaining_letters = letters.except(new_key);
                    (new_key, remaining_letters)
                });
                let keyboards = new_keys.flat_map(move |(new_key, letters)| {
                    let k = self.add_key(new_key);
                    k.dfs_builder_utility(letters, key_sizes.to_vec(), prune)
                });
                let result: Box<dyn Iterator<Item = Keyboard>> = Box::new(keyboards);
                result
            }
        }
    }
}

#[derive(Clone)]
struct DfsExplorer {
    keyboard: Keyboard,
    index: usize,
}

impl DfsExplorer {
    pub fn dfs<'a>(self) -> Box<dyn Iterator<Item = DfsExplorer> + 'a> {
        self.dfs_with(&|_k| false)
    }

    pub fn dfs_with<'a, F>(self, prune: &'a F) -> Box<dyn Iterator<Item = DfsExplorer> + 'a>
    where
        F: (Fn(&Keyboard) -> bool) + 'a,
    {
        match prune(&self.keyboard) {
            true => {
                let result = std::iter::empty();
                let boxed_result: Box<dyn Iterator<Item = DfsExplorer>> = Box::new(result);
                boxed_result
            }
            false => {
                if self.keyboard.keys.len() == 1 {
                    let result = std::iter::once(self.clone());
                    let boxed_result: Box<dyn Iterator<Item = DfsExplorer>> = Box::new(result);
                    boxed_result
                } else {
                    let children = self.next();
                    let current = std::iter::once(self);
                    let descendents = children
                        .into_iter()
                        .filter(move |k| false == prune(&k.keyboard))
                        .flat_map(move |child| child.dfs_with(prune));
                    let boxed_result: Box<dyn Iterator<Item = DfsExplorer>> =
                        Box::new(current.chain(descendents));
                    boxed_result
                }
            }
        }
    }

    pub fn next(&self) -> Vec<DfsExplorer> {
        if self.keyboard.len() <= 1 {
            vec![]
        } else {
            let can_combine = |a: Key, b: Key| -> bool { a.max_letter() < b.min_letter() };
            let indexes = (self.index..=self.keyboard.len() - 2)
                .flat_map(|i| (i + 1..=self.keyboard.len() - 1).map(move |j| (i, j)))
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
                DfsExplorer { keyboard, index: i }
            });
            parts.collect::<Vec<DfsExplorer>>()
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

    use crate::prohibited;

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
    fn every_smaller_goes_depth_first() {
        let start = Keyboard::with_every_letter_on_own_key(Key::with_every_letter());
        let first_10_key = start.every_smaller().find(|k| k.len() == 10);
        match first_10_key {
            Some(ten_key) => assert_eq!(10, ten_key.len()),
            None => panic!("Could not get a 10 key"),
        }
    }

    #[test]
    fn every_smaller_can_prune_root() {
        let k = Keyboard::with_layout("a,b,c,d,e");
        let prune = |k: &Keyboard| k.len() == 5;
        let actual = k.every_smaller_with(&prune).count();
        assert_eq!(0, actual);
    }

    #[test]
    fn every_smaller_can_prune_base_case_of_single_key() {
        let k = Keyboard::with_layout("a,b,c,d,e");
        let prune = |k: &Keyboard| k.len() == 1;
        let base_case_count = k
            .every_smaller_with(&prune)
            .filter(|k| k.len() == 1)
            .count();
        assert_eq!(base_case_count, 0);
    }

    #[test]
    fn every_smaller_can_prune() {
        let k = Keyboard::with_layout("a,b,c,d,e");
        let prune = |k: &Keyboard| k.max_key_size().map(|ks| ks > 2).unwrap_or(false);
        let count_big_keys = k
            .every_smaller_with(&prune)
            .filter(|k| k.max_key_size().map(|ks| ks > 2).unwrap_or(true))
            .count();
        assert_eq!(count_big_keys, 0);
    }

    #[test]
    fn with_dfs_builder_creates_proper_number_of_keyboards() {
        let empty = Keyboard::with_no_keys();
        let key_sizes = Partitions {
            sum: 8,
            min: 1,
            max: 8,
            parts: 3,
        };
        let expected = key_sizes.total_unique_keyboards();
        let alphabet = Key::with_first_n_letters(8);
        let prune = |_k: &Keyboard| false;
        let actual = empty.with_dfs_builder(alphabet, key_sizes, &prune).count();
        assert_eq!(expected, actual as u128);
    }

    #[test]
    fn with_dfs_builder_creates_unique_keyboards() {
        let empty = Keyboard::with_no_keys();
        let key_sizes = Partitions {
            sum: 8,
            min: 1,
            max: 8,
            parts: 3,
        };
        let mut tally = Tally::new();
        let alphabet = Key::with_first_n_letters(8);
        let prune = |_k: &Keyboard| false;
        for k in empty.with_dfs_builder(alphabet, key_sizes, &prune) {
            let count = tally.increment(k.to_string());
            assert!(count < 2);
        }
    }

    #[test]
    fn with_dfs_builder_print() {
        let empty = Keyboard::with_no_keys();
        let key_sizes = Partitions {
            sum: 5,
            min: 1,
            max: 5,
            parts: 3,
        };
        let mut prohibited = Prohibited::new();
        prohibited.add_many([Key::new("cd")].into_iter());
        let alphabet = Key::with_first_n_letters(5);
        let prune = |k: &Keyboard| k.has_prohibited_keys(&prohibited);
        for k in empty.with_dfs_builder(alphabet, key_sizes, &prune) {
            println!("{}", k)
        }
    }

    #[test]
    #[ignore]
    fn every_smaller_print() {
        for k in Keyboard::with_layout("a,b,c,d").every_smaller() {
            println!("{}", k)
        }
    }
}
