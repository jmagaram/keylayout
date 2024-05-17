use std::{collections::HashMap, fs::File, io::BufReader};

use crate::key::Key;
use crate::letter::Letter;
use crate::word::Word;

#[derive(Clone)]
pub struct Dictionary {
    words_highest_frequency_first: Vec<Word>,
    alphabet: Key,
}

impl Dictionary {
    pub fn new(words: &HashMap<String, f32>) -> Dictionary {
        let mut words = words
            .iter()
            .map(|(w, f)| Word::new(w.as_str(), *f))
            .collect::<Vec<Word>>();
        words.sort_by(|a, b| b.frequency().cmp(&a.frequency()));
        Dictionary::from_unique_sorted_words(words)
    }

    pub fn load() -> Dictionary {
        let items = Dictionary::load_json()
            .iter()
            .filter(|(k, _)| k.len() <= Word::MAX_WORD_LENGTH)
            .map(|(k, v)| (k.to_owned(), *v))
            .collect();
        Dictionary::new(&items)
    }

    pub fn from_unique_sorted_words(words: Vec<Word>) -> Dictionary {
        let alphabet = words.iter().flat_map(|w| w.letters()).collect::<Key>();
        Dictionary {
            words_highest_frequency_first: words,
            alphabet,
        }
    }

    pub fn replace_letters(&self, find: Key, replace_with: Letter) -> Dictionary {
        let words = self
            .words_highest_frequency_first
            .iter()
            .map(|w| w.replace_letters(find.letters(), replace_with))
            .collect::<Vec<Word>>();
        Self::from_unique_sorted_words(words)
    }

    pub fn filter_top_n_words(&self, count: usize) -> Dictionary {
        let words = self
            .words()
            .into_iter()
            .take(count)
            .map(|w| w.to_owned())
            .collect::<Vec<Word>>();
        Dictionary::from_unique_sorted_words(words)
    }

    pub fn words(&self) -> &Vec<Word> {
        let result = &self.words_highest_frequency_first;
        result
    }

    pub fn alphabet(&self) -> Key {
        self.alphabet
    }

    fn load_json() -> HashMap<String, f32> {
        const FILE_NAME: &'static str = "./words.json";
        let file = File::open(FILE_NAME).expect("file not found");
        let reader = BufReader::new(file);
        let word_frequencies: HashMap<String, f32> =
            serde_json::from_reader(reader).expect("read json properly");
        word_frequencies
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn standard_dictionary_has_proper_count_of_words() {
        let d = Dictionary::load();
        let expected = 307629 - 4; // take away words longer than 25 characters
        assert_eq!(d.words_highest_frequency_first.len(), expected,);
    }

    #[test]
    fn standard_dictionary_has_proper_letter_set() {
        let d = Dictionary::load();
        assert_eq!(d.alphabet.len(), 27,);
    }

    #[test]
    fn standard_dictionary_has_proper_frequency_sum() {
        let d = Dictionary::load();
        let expected = 0.96;
        let frequency_sum: f32 = d
            .words_highest_frequency_first
            .iter()
            .map(|w| w.frequency().to_f32())
            .sum();
        let is_close = (frequency_sum - expected).abs() < 0.01;
        assert!(is_close)
    }

    #[test]
    #[ignore]
    fn display_top_words() {
        let d = Dictionary::load();
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
