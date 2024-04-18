use std::{fmt, iter};

use rand::Rng;

use crate::{
    dictionary::Dictionary, key::Key, letter::Letter, partitions::Partitions, penalty::Penalty,
    solution::Solution, tally::Tally, word::Word,
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
                Key::try_from(letters).expect("Expected each key to be separated by a comma.")
            })
            .collect::<Vec<Key>>();
        Keyboard::new_from_keys(keys)
    }

    pub fn with_penalty(self, penalty: Penalty) -> Solution {
        Solution::new(self, penalty, "".to_string())
    }

    pub fn with_penalty_and_notes(self, penalty: Penalty, notes: String) -> Solution {
        Solution::new(self, penalty, notes)
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    pub fn max_key_size(&self) -> Option<u32> {
        self.keys.iter().map(|k| k.count_letters()).max()
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

    pub fn random(alphabet: Key, layout: &Partitions) -> impl Iterator<Item = Keyboard> {
        let mut rng = rand::thread_rng();
        let layout_options = layout.calculate();
        iter::repeat_with(move || {
            let layout_index = rng.gen_range(0..layout_options.len());
            let layout = layout_options.get(layout_index).unwrap();
            let keys = alphabet.random_subsets(layout).collect::<Vec<Key>>();
            let keyboard = Keyboard::new_from_keys(keys);
            keyboard
        })
    }

    pub fn swap_random_letters_n_times(k: Keyboard, count: u32) -> Result<Keyboard, &'static str> {
        if count == 0 {
            Ok(k)
        } else {
            let k = k.swap_random_letters()?;
            Keyboard::swap_random_letters_n_times(k, count - 1)
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
                    .all(move |k| k.intersect(combined_key).count_letters() <= 1)
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

    pub fn penalty_by_word<'a>(
        &'a self,
        dictionary: &'a Dictionary,
    ) -> impl Iterator<Item = (&Word, Penalty)> {
        let mut found = Tally::new();
        dictionary.words().iter().map(move |word| {
            let how_to_spell = self.spell_serialized(word);
            let word_penalty = match found.count(&how_to_spell) {
                0 => {
                    found.increment(how_to_spell);
                    Penalty::ZERO
                }
                seen => {
                    found.increment(how_to_spell);
                    Penalty::new(word.frequency().to_f32() * seen.min(4) as f32)
                }
            };
            (word, word_penalty)
        })
    }

    pub fn penalty(&self, dictionary: &Dictionary, to_beat: Penalty) -> Penalty {
        let mut penalty = Penalty::ZERO;
        for (_, word_penalty) in self.penalty_by_word(dictionary) {
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
    fn create_file_of_penalty_per_word() {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create("output.txt").unwrap();
        writeln!(file, "index, word, penalty").unwrap();
        let d = Dictionary::load();
        let keyboard = Keyboard::new_from_layout("ot,gr,dh,su,im,bn,awz,cky',fjlx,epqv");
        for (word_index, (word, penalty)) in keyboard.penalty_by_word(&d).enumerate() {
            writeln!(file, "{},{},{}", word_index + 1, word, penalty.to_f32()).unwrap();
        }
    }

    #[test]
    #[ignore]
    fn random_keyboard_print_out() {
        let partition = Partitions {
            sum: 27,
            parts: 10,
            min: 2,
            max: 5,
        };
        let dict = Dictionary::load();
        let keyboards = Keyboard::random(dict.alphabet(), &partition);
        for k in keyboards.take(50) {
            println!("{}", k)
        }
    }
}
