use std::collections::HashMap;

use crate::{
    dictionary::Dictionary,
    penalty::{self, Penalty},
    set32::Set32,
    u5::U5,
    word::{self, Word},
};

pub struct Keyboard {
    keys: Vec<Set32>,
}

impl Keyboard {
    pub fn new(keys: Vec<Set32>) -> Keyboard {
        Keyboard { keys }
    }

    // abc,def,ghh
    pub fn with_layout(d: &Dictionary, s: &str) -> Keyboard {
        let keys = s
            .split(",")
            .map(|letters| {
                letters.chars().fold(Set32::EMPTY, |set, c| {
                    let k = d.letter_to_u5(c);
                    match k {
                        None => set,
                        Some(k) => set.add(*k),
                    }
                })
            })
            .collect::<Vec<Set32>>();
        Keyboard::new(keys)
    }

    pub fn format(&self, d: &Dictionary) -> String {
        let keys: Vec<String> = self
            .keys
            .iter()
            .map(|k| {
                let s = k.fold(String::new(), |mut total, i| {
                    let char = d.u5_to_letter(i);
                    total.push(char);
                    total
                });
                s
            })
            .collect();
        let joined = keys.join(" ");
        format!("| {} |", joined)
    }

    fn find_key_for_letter(&self, char: U5) -> Option<U5> {
        self.keys
            .iter()
            .enumerate()
            .find_map(|(inx, val)| match val.contains(char) {
                true => Some(U5::from(inx)),
                false => None,
            })
    }

    // fix cascading errors with if let etc.
    pub fn spell(&self, dictionary: &Dictionary, word: &Word) -> String {
        let mut result = String::new();
        word.chars().for_each(|c| {
            let u5 = dictionary.letter_to_u5(c);
            match u5 {
                None => {
                    panic!("Can't type the word because a letter is absent from the dictionary.")
                }
                Some(u5) => {
                    let key = self.find_key_for_letter(*u5);
                    match key {
                        None => {
                        panic!("Can not type the word \"{}\" because the keyboard is missing the letter '{}'",word,c);
                        }
                        Some(key_inx) => {
                            let serialize_as = key_inx.serialize();
                            result.push(serialize_as);
                        }
                    }
                }
            }
        });
        result
    }

    pub fn penalty(&self, dictionary: &Dictionary, to_beat: Penalty) -> Penalty {
        let mut found = HashMap::new();
        let mut penalty = Penalty::ZERO;
        for word in dictionary.words() {
            let how_to_spell = self.spell(dictionary, word);
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

#[cfg(test)]
mod tests {

    use crate::{frequency::Frequency, u5::U5};

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
    fn spell_uses_u5_index_of_key() {
        let d = make_dictionary(vec!["apple", "word", "banana"]);
        let k = Keyboard::with_layout(&d, "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz'");

        let w = Word::with_str("word");
        let letter1_key = U5::new(7).serialize();
        let letter2_key = U5::new(4).serialize();
        let letter3_key = U5::new(5).serialize();
        let letter4_key = U5::new(1).serialize();

        let mut expected = String::new();
        expected.push(letter1_key);
        expected.push(letter2_key);
        expected.push(letter3_key);
        expected.push(letter4_key);

        let actual = k.spell(&d, &w);

        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic]
    fn spell_panic_if_letter_not_in_dictionary() {
        let d = make_dictionary(vec!["the", "book"]);
        let k = Keyboard::with_layout(&d, "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz'");
        let w = Word::with_details("theocrat".to_string(), Frequency::ZERO);
        k.spell(&d, &w);
    }

    #[test]
    #[should_panic]
    fn spell_panic_if_required_letter_not_on_keyboard() {
        let d = make_dictionary(vec!["the", "book"]);
        let k = Keyboard::with_layout(&d, "the,boo");
        let w = Word::with_details("book".to_string(), Frequency::ZERO);
        k.spell(&d, &w);
    }

    #[test]
    fn spell_is_same_length_as_original_word() {
        let d = Dictionary::load_large_dictionary();
        let k = Keyboard::with_layout(&d, "abc,def,ghi,jkl,mnop,qrs,tuv,wxyz'");
        d.words().iter().for_each(|w| {
            let spelling = k.spell(&d, w);
            assert_eq!(spelling.len(), w.to_string().len());
        })
    }

    #[test]
    #[ignore]
    fn spell_print_each_dictionary_word_out() {
        let d = Dictionary::load_large_dictionary();
        let k = Keyboard::with_layout(&d, "abc,def,ghi,jkl,mnop,qrs,tuv,wxyz'");
        d.words().iter().for_each(|w| {
            let spelling = k.spell(&d, w);
            println!("{} as {}", w, spelling);
        })
    }

    #[test]
    fn penalty_score_is_correct() {
        let d = Dictionary::load_large_dictionary();
        let k = Keyboard::with_layout(&d, "abc,def,ghi,jkl,mno,pqr,st,uv,wx,yz'");
        let actual: f32 = k.penalty(&d).to_f32(); // why into does not work
        assert!(actual >= 0.0802 && actual <= 0.0804); // 0.0803
    }
}
