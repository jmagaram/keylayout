use std::{collections::HashMap, fmt, iter};

use rand::Rng;

use crate::{dictionary::Dictionary, key::Key, letter::Letter, penalty::Penalty, word::Word};

#[derive(Clone)]
pub struct Keyboard {
    keys: Vec<Key>,
}

impl Keyboard {
    pub fn new(keys: Vec<Key>) -> Keyboard {
        debug_assert!(
            Keyboard::has_unique_letters(&keys),
            "Some keys on the keyboard have duplicate letters."
        );
        Keyboard { keys }
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    pub fn max_key_size(&self) -> Option<u32> {
        self.keys.iter().map(|k| k.count_items()).max()
    }

    fn has_unique_letters(keys: &Vec<Key>) -> bool {
        let count_letters_on_each_key = keys
            .iter()
            .map(|k| k.count_items())
            .fold(0, |total, i| total + i);
        let count_letters_when_union_each_key = keys
            .iter()
            .fold(Key::EMPTY, |total, i| total.union(*i))
            .count_items();
        count_letters_on_each_key == count_letters_when_union_each_key
    }

    // abc,def,ghh
    pub fn with_layout(s: &str) -> Keyboard {
        let keys = s
            .split(",")
            .map(|letters| {
                let m = Key::try_from(letters).unwrap(); // fix this!
                m
            })
            .collect::<Vec<Key>>();
        Keyboard::new(keys)
    }

    pub fn format(&self, d: &Dictionary) -> String {
        let keys: Vec<String> = self.keys.iter().map(|k| k.to_string()).collect();
        let joined = keys.join(" ");
        format!("| {} |", joined)
    }

    fn find_key_for_letter(&self, letter: Letter) -> Option<Key> {
        let keys = &self.keys;
        let m = keys.iter().find(|k| {
            let q = k.contains(letter);
            q
        });
        let qqq = m.map(|k| k.clone());
        qqq
    }

    pub fn spell(&self, word: &Word) -> String {
        let result = word
            .letters()
            .into_iter()
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
            Ok(Keyboard { keys: new_keys })
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
                            let keyboard = Keyboard::new(letters);
                            result.push(keyboard);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn every_combine_two_keys(&self) -> Vec<Keyboard> {
        if self.keys.len() <= 1 {
            panic!("It is not possible to combine keys on the keyboard since it only has {} keys right now.", self.keys.len());
        }
        let mut results = vec![];
        for a_index in 0..=self.keys.len() - 2 {
            for b_index in a_index + 1..=self.keys.len() - 1 {
                let combined_key = self.keys[a_index].union(self.keys[b_index]);
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
                let new_keyboard = Keyboard::new(new_keys);
                results.push(new_keyboard);
            }
        }
        results
    }

    pub fn penalty(&self, dictionary: &Dictionary, to_beat: Penalty) -> Penalty {
        let mut found = HashMap::new();
        let mut penalty = Penalty::ZERO;
        for word in dictionary.words() {
            let how_to_spell = self.spell(word);
            let word_penalty = match found.get(&how_to_spell) {
                None => {
                    found.insert(how_to_spell.to_string(), 1);
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

#[cfg(test)]
mod tests {

    use crate::{frequency::Frequency, util};

    use super::*;

    fn make_dictionary(words: Vec<&str>) -> Dictionary {
        let words: Vec<(String, f32)> = words
            .iter()
            .map(|w| (w.to_string(), Frequency::random().to_f32()))
            .collect();
        let map = HashMap::from_iter(words);
        Dictionary::new(map)
    }

    #[test]
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
    #[should_panic]
    fn spell_panic_if_required_letter_not_on_keyboard() {
        let k = Keyboard::with_layout("abc,def,ghi");
        let w = Word::try_from("abcx").unwrap();
        k.spell(&w);
    }

    #[test]
    #[ignore]
    fn spell_print_each_dictionary_word_out() {
        let d = Dictionary::load_large_dictionary();
        let k = Keyboard::with_layout("abc,def,ghi,jkl,mnop,qrs,tuv,wxyz'");
        d.words().iter().take(20).for_each(|w| {
            let spelling = k.spell(&w);
            println!("{} : {}", w, spelling);
        })
    }

    #[test]
    fn penalty_score_is_correct() {
        let d = Dictionary::load_large_dictionary();
        let k = Keyboard::with_layout("abc,def,ghi,jkl,mno,pqr,st,uv,wx,yz'");
        let actual: f32 = k.penalty(&d, Penalty::MAX).to_f32(); // why into does not work
        assert!(actual >= 0.0802 && actual <= 0.0804); // 0.0803
    }

    #[test]
    #[ignore]
    fn swap_random_letters() {
        let mut k = Keyboard::with_layout("abc,def,ghi");
        for i in 1..10 {
            k = k.swap_random_letters().unwrap();
            println!("{}", k)
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
    #[ignore]
    fn every_combine_two_keys() {
        let k = Keyboard::with_layout("a,b,c,d,efg,hi");
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
            let k = Keyboard::with_layout(d);
            let actual_count = k.every_combine_two_keys().len();
            let expected = util::choose(k.keys.len() as u32, 2);
            assert_eq!(actual_count, expected as usize);
        }
    }
}
