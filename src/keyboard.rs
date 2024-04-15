use std::{collections::HashMap, fmt, iter};

use rand::Rng;

use crate::{
    dictionary::Dictionary, key::Key, letter::Letter, penalty::Penalty, permute::PermuteSeed,
    solution::Solution, word::Word,
};

// fix this!
#[derive(Clone)]
pub struct Keyboard {
    keys: Vec<Key>,
    letter_to_key_index: [Option<usize>; Letter::ALPHABET_SIZE],
}

impl Keyboard {
    pub fn new_from_keys(keys: Vec<Key>) -> Keyboard {
        let mut letter_to_key_index: [Option<usize>; Letter::ALPHABET_SIZE] = Default::default();
        for (key_index, key) in keys.iter().enumerate() {
            for letter in *key {
                debug_assert!(
                    letter_to_key_index[letter.to_usize()].is_none(),
                    "Some keys on the keyboard have duplicate letters."
                );
                letter_to_key_index[letter.to_usize()] = Some(key_index);
            }
        }
        Keyboard {
            keys,
            letter_to_key_index,
        }
    }

    // abc,def,ghh
    pub fn new_from_layout(s: &str) -> Keyboard {
        let keys = s
            .split(",")
            .map(|letters| {
                let m = Key::try_from(letters).unwrap();
                m
            })
            .collect::<Vec<Key>>();
        Keyboard::new_from_keys(keys)
    }

    pub fn with_penalty(self, penalty: Penalty) -> Solution {
        Solution::new(self, penalty)
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    pub fn max_key_size(&self) -> Option<u32> {
        self.keys.iter().map(|k| k.count_items()).max()
    }

    fn find_key_index_for_letter(&self, letter: Letter) -> Option<usize> {
        self.letter_to_key_index[letter.to_usize()]
    }

    fn find_key_for_letter(&self, letter: Letter) -> Option<Key> {
        let key_index = self.find_key_index_for_letter(letter)?;
        let key = self.keys.get(key_index)?;
        Some(*key)
    }

    pub fn spell_serialized(&self, word: &Word) -> String {
        let mut result = String::new();
        for letter in word.letters() {
            match self.find_key_index_for_letter(*letter) {
                None => panic!(
                    "Could not spell the word {} because the keyboard is missing the letter {}",
                    word, letter
                ),
                Some(index) => {
                    const BASE_CHAR: u32 = 'A' as u32;
                    let char = char::from_u32((index as u32 + BASE_CHAR) as u32).unwrap();
                    result.push(char);
                }
            }
        }
        result
    }

    pub fn spell(&self, word: &Word) -> String {
        let result = word
            .letters()
            .iter()
            .map(|letter| self.find_key_for_letter(*letter))
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
            let a_letter_to_swap = a_key.random_letter().unwrap();
            let b_letter_to_swap = b_key.random_letter().unwrap();
            let new_a_key = a_key.remove(a_letter_to_swap).add(b_letter_to_swap);
            let new_b_key = b_key.remove(b_letter_to_swap).add(a_letter_to_swap);
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
            Ok(Keyboard::new_from_keys(new_keys))
        }
    }

    pub fn every_swap(&self) -> Vec<Keyboard> {
        if self.keys.len() < 2 {
            panic!("Can not swap keys on a keyboard with less than 2 keys on it.")
        }
        let mut result = vec![];
        for a_key_index in 0..=self.keys.len() - 2 {
            for b_key_index in a_key_index + 1..=(self.keys.len() - 1) {
                let a_key = self.keys[a_key_index];
                let b_key = self.keys[b_key_index];
                for a_letter in a_key {
                    for b_letter in b_key {
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
                            let keyboard = Keyboard::new_from_keys(letters);
                            result.push(keyboard);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn every_combine_two_keys_filter(&self, prohibited_pairs: &Vec<Key>) -> Vec<Keyboard> {
        if self.keys.len() <= 1 {
            panic!("It is not possible to combine keys on the keyboard since it only has {} keys right now.", self.keys.len());
        }
        let mut results = vec![];
        for a_index in 0..=self.keys.len() - 2 {
            for b_index in a_index + 1..=self.keys.len() - 1 {
                let combined_key = self.keys[a_index].union(self.keys[b_index]);
                if prohibited_pairs
                    .iter()
                    .all(move |k| k.intersect(combined_key).count_items() <= 1)
                {
                    let new_keys: Vec<Key> = self
                        .keys
                        .iter()
                        .enumerate()
                        .filter_map(|(index, k)| {
                            if index == a_index {
                                Some(combined_key)
                            } else if index == b_index {
                                None
                            } else {
                                Some(*k)
                            }
                        })
                        .collect();
                    let new_keyboard = Keyboard::new_from_keys(new_keys);
                    results.push(new_keyboard);
                }
            }
        }
        results
    }

    pub fn every_combine_two_keys(&self) -> Vec<Keyboard> {
        self.every_combine_two_keys_filter(&vec![])
    }

    pub fn penalty(&self, dictionary: &Dictionary, to_beat: Penalty) -> Penalty {
        let mut found = HashMap::new();
        let mut penalty = Penalty::ZERO;
        for word in dictionary.words() {
            let how_to_spell = self.spell_serialized(word);
            let word_penalty = match found.get(&how_to_spell) {
                None => {
                    found.insert(how_to_spell, 1);
                    Penalty::ZERO
                }
                Some(seen) => {
                    let seen = *seen;
                    found.insert(how_to_spell, seen + 1);
                    Penalty::new(word.frequency().to_f32() * seen.min(4) as f32)
                }
            };
            penalty = penalty + word_penalty;
            if penalty >= to_beat {
                break;
            }
        }
        penalty
    }
}

impl fmt::Display for Keyboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self
            .keys
            .iter()
            .map(|k| Key::to_string(k))
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{}", result)
    }
}

struct KeyboardGenerator<'a> {
    groups: &'a Vec<u32>,
    group_index: usize,
    letters: Key,
}

impl<'a> PermuteSeed<'a, Key> for KeyboardGenerator<'a> {
    fn next(&self) -> Vec<(Key, Self)> {
        if self.group_index == self.groups.len() {
            vec![]
        } else {
            let key_size = self.groups[self.group_index];
            if self.letters.count_items() < key_size {
                panic!(
                    "Can't generate a key of size {} since there are no remaining letters.",
                    key_size
                )
            }
            let first_letter = self.letters.max_letter().unwrap();
            let remaining_letters = self.letters.remove(first_letter);
            let remaining_work = match remaining_letters.count_items() {
                0 => {
                    let key = self.letters;
                    vec![(
                        key,
                        KeyboardGenerator {
                            group_index: self.group_index + 1,
                            letters: Key::EMPTY,
                            ..*self
                        },
                    )]
                }
                _ => match key_size {
                    1 => {
                        let key = Key::with_one_letter(first_letter);
                        let remaining_letters = self.letters.except(key);
                        let seed = KeyboardGenerator {
                            group_index: self.group_index + 1,
                            letters: remaining_letters,
                            ..*self
                        };
                        vec![(key, seed)]
                    }
                    _ => {
                        let result = remaining_letters
                            .subsets_of_size(key_size - 1)
                            .map(|k| {
                                let key = k.add(first_letter);
                                let remaining_letters = self.letters.except(key);
                                let seed = KeyboardGenerator {
                                    group_index: self.group_index + 1,
                                    letters: remaining_letters,
                                    ..*self
                                };
                                (key, seed)
                            })
                            .collect::<Vec<(Key, KeyboardGenerator<'a>)>>();
                        result
                    }
                },
            };
            remaining_work
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{permutable::Permutable, util};

    use super::*;

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn new_panic_if_keys_with_duplicate_letters() {
        Keyboard::new_from_layout("abc,def,ghi,axy");
    }

    #[test]
    fn spell_test() {
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mno,pqr,stu,vwx,yz'");
        let w = Word::try_from("word").unwrap();
        let actual = k.spell(&w);
        assert_eq!(actual, "vwx,mno,pqr,def");
    }

    #[test]
    #[should_panic]
    fn spell_panic_if_required_letter_not_on_keyboard() {
        let k = Keyboard::new_from_layout("abc,def,ghi");
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
            let keyboard = Keyboard::new_from_layout(layout);
            let letter_to_find = Letter::try_from(letter).unwrap();
            let actual = keyboard.find_key_index_for_letter(letter_to_find);
            assert_eq!(actual, expected_key_index);
        }
    }

    #[test]
    #[ignore]
    fn spell_print_each_dictionary_word_out() {
        let d = Dictionary::load();
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mnop,qrs,tuv,wxyz'");
        d.words().iter().take(20).for_each(|w| {
            let spelling = k.spell(&w);
            println!("{} : {}", w, spelling);
        })
    }

    #[test]
    fn penalty_score_is_correct() {
        let d = Dictionary::load();
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mno,pqr,st,uv,wx,yz'");
        let actual: f32 = k.penalty(&d, Penalty::MAX).to_f32(); // why into does not work
        assert!(actual >= 0.0802 && actual <= 0.0804); // 0.0803
    }

    #[test]
    #[ignore]
    fn swap_random_letters() {
        let mut k = Keyboard::new_from_layout("abc,def,ghi");
        for _i in 1..10 {
            k = k.swap_random_letters().unwrap();
            println!("{}", k)
        }
    }

    #[test]
    #[ignore]
    fn every_swap() {
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mno,pqr,stu,vw,xy,z'");
        k.every_swap().iter().for_each(|k| println!("{}", k));
        println!("Total swaps: {}", k.every_swap().iter().count());
    }

    #[test]
    #[ignore]
    fn every_combine_two_keys() {
        let k = Keyboard::new_from_layout("a,b,c,d,efg,hi");
        k.every_combine_two_keys()
            .iter()
            .for_each(|k| println!("{}", k));
    }

    #[test]
    fn every_combine_two_keys_generates_correct_number_of_answers() {
        let data = [
            "a,b",
            "a,b,c,d",
            "a,b,c,d,e,f,g,h,i,j,k,l,m",
            "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,'",
        ];
        for d in data {
            let k = Keyboard::new_from_layout(d);
            let actual_count = k.every_combine_two_keys().len();
            let expected = util::choose(k.keys.len() as u32, 2);
            assert_eq!(actual_count, expected as usize);
        }
    }

    #[test]
    #[ignore]
    fn output_letters_to_scoring() {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create("output.txt").unwrap();
        writeln!(file, "This is a line of text written to a file.").unwrap();

        let letters = "pt,ly,bn,sz,em,gr,afj,ikwx,cdu',hoqv";
        let d = Dictionary::load();
        let total_words = d.words().len();
        for i in 1..total_words {
            let d = d.with_top_n_words(i);
            let keyboard = Keyboard::new_from_layout(letters);
            let penalty = keyboard.penalty(&d, Penalty::MAX);
            println!("{},{}", i, penalty.to_f32());
            writeln!(file, "{},{}", i, penalty.to_f32()).unwrap();
        }
    }

    #[test]
    fn keyboard_generator() {
        let letter_count = 5;
        let letters = Key::with_first_n_letters(letter_count);
        let layouts = crate::partitions::Partitions {
            parts: 2,
            sum: letter_count,
            min: 1,
            max: letter_count,
        }
        .permute();
        for layout in layouts {
            let source = KeyboardGenerator {
                letters,
                group_index: 0,
                groups: &layout,
            };
            for k in source.permute().map(|ks| Keyboard::new_from_keys(ks)) {
                println!("{0}", k)
            }
        }
    }
}
