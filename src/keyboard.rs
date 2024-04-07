use std::collections::HashMap;

use crate::{dictionary::Dictionary, penalty::Penalty, set32::Set32, word::Word};

pub struct Keyboard {
    keys: Vec<Set32>,
}

impl Keyboard {
    pub fn new(keys: Vec<Set32>) -> Keyboard {
        Keyboard { keys }
    }

    fn spell(&self, dictionary: &Dictionary, word: &Word) -> String {
        let mut spell = String::new();
        word.chars().for_each(|c| {
            let u6 = dictionary.u6_for_letter(c);
            spell.push(u6.to_char());
            spell.push(',');
        });
        spell
    }

    pub fn penalty(&self, dictionary: &Dictionary) -> Penalty {
        let mut found = HashMap::new();
        let result = &dictionary
            .words()
            .iter()
            .map(move |word| {
                let how_to_spell = self.spell(dictionary, word);
                match found.get(&how_to_spell) {
                    None => {
                        found.insert(how_to_spell.to_string(), 1);
                        Penalty::ZERO
                    }
                    Some(seen) => {
                        let seen = *seen;
                        found.insert(how_to_spell, seen + 1);
                        Penalty::new(word.frequency().to_f32() * seen.min(4) as f32)
                    }
                }
            })
            .fold(Penalty::ZERO, |total, i| total + i);
        *result
    }
}
