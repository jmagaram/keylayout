use std::collections::HashMap;

use crate::{dictionary::Dictionary, key::Key, letter::Letter, penalty::Penalty, word::Word};

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
    // fromstr?
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

#[cfg(test)]
mod tests {

    use crate::frequency::Frequency;

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
}
