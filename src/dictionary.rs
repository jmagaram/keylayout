use std::collections::HashSet;
use std::{collections::HashMap, fs::File, io::BufReader};

use crate::key::Key;
use crate::letter::Letter;
use crate::{frequency::Frequency, word::Word};

pub struct Dictionary {
    words_highest_frequency_first: Vec<Word>,
    frequency_sum: Frequency,
    alphabet: Key,
}

impl Dictionary {
    const FILE_NAME: &'static str = "./src/words.json";

    // take a str or String or &str?
    pub fn create(words: Vec<(String, f32)>) -> Dictionary {
        let mut unique_words = words
            .into_iter()
            .map(|(w, f)| Word::new(w.as_str(), f))
            .collect::<Result<Vec<Word>, _>>()
            .unwrap()
            .into_iter()
            .collect::<HashSet<Word>>()
            .into_iter()
            .collect::<Vec<Word>>();

        unique_words.sort_by(|a, b| b.frequency().cmp(&a.frequency()));

        let alphabet = unique_words
            .iter()
            .flat_map(|w| w.letters())
            .map(|r| r.clone())
            .collect::<Key>();

        let frequency_sum = unique_words
            .iter()
            .map(|w| w.frequency())
            .fold(Frequency::ZERO, |total, i| total + *i);
        Dictionary {
            words_highest_frequency_first: unique_words,
            frequency_sum,
            alphabet,
        }
    }

    // do not understand iter vs into_iter
    // String or str?
    // pass &HashMap anyways
    pub fn new(words: HashMap<String, f32>) -> Dictionary {
        Dictionary::create(words.into_iter().collect())
    }

    // easier to convert a vec<letters> to a &str
    pub fn with_top_n_words(&self, count: usize) -> Dictionary {
        let r: Vec<(_, _)> = self
            .words()
            .into_iter()
            .take(count)
            .map(|w| {
                let c = w
                    .letters()
                    .iter()
                    .map(|r| r.to_char())
                    .collect::<Vec<char>>()
                    .iter()
                    .collect::<String>();
                (c, w.frequency().to_f32())
            })
            .collect();
        Dictionary::create(r)
    }

    pub fn words(&self) -> &Vec<Word> {
        let result = &self.words_highest_frequency_first;
        result
    }

    pub fn alphabet(&self) -> Key {
        self.alphabet
    }

    fn load_json() -> HashMap<String, f32> {
        let file = File::open(Dictionary::FILE_NAME).expect("file not found");
        let reader = BufReader::new(file);
        let word_frequencies: HashMap<String, f32> =
            serde_json::from_reader(reader).expect("read json properly");
        word_frequencies
    }

    pub fn load_large_dictionary() -> Dictionary {
        let items = Dictionary::load_json()
            .iter()
            .map(|(k, v)| (k.to_owned(), *v))
            .collect();
        Dictionary::new(items)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn standard_dictionary_has_proper_count_of_words() {
        let d = Dictionary::load_large_dictionary();
        let expected = 307629;
        assert_eq!(d.words_highest_frequency_first.len(), expected,);
    }

    #[test]
    fn standard_dictionary_has_proper_letter_set() {
        let d = Dictionary::load_large_dictionary();
        assert_eq!(d.alphabet.count(), 27,);
    }

    #[test]
    fn standard_dictionary_has_proper_frequency_sum() {
        let d = Dictionary::load_large_dictionary();
        let expected = 0.96;
        let is_close = (d.frequency_sum.to_f32() - expected).abs() < 0.01;
        assert!(is_close)
    }

    #[test]
    #[ignore]
    fn display_top_words() {
        let d = Dictionary::load_large_dictionary();
        d.words_highest_frequency_first
            .iter()
            .take(200)
            .for_each(|w| {
                println!("{:?}", w);
            });
        let word_count = d.words_highest_frequency_first.len();
        println!("total words {}", word_count);
    }
}
