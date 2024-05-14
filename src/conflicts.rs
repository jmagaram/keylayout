use crate::{dictionary::Dictionary, frequency::Frequency, penalty::Penalty, word::Word};
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{BufWriter, Write},
};
use thousands::Separable;

pub struct Conflicts(HashMap<String, FrequentWords>);

struct FrequentWords {
    words: HashSet<Word>,
    penalty: Penalty,
}

impl FrequentWords {
    fn empty() -> FrequentWords {
        FrequentWords {
            words: HashSet::new(),
            penalty: Penalty::ZERO,
        }
    }

    fn insert(&mut self, word: Word) {
        if self.words.insert(word) {
            if self.words.len() > 4 {
                let least_frequent = self
                    .words
                    .clone()
                    .into_iter()
                    .min_by(|a, b| a.frequency().cmp(&b.frequency()))
                    .unwrap();
                self.penalty =
                    Penalty::new(self.penalty.to_f32() + least_frequent.frequency().to_f32() * 4.0);
                self.words.remove(&least_frequent);
            }
        }
    }

    fn penalty(&self) -> Penalty {
        let mut words = self.words.iter().collect::<Vec<&Word>>();
        words.sort_unstable_by(|a, b| b.frequency().cmp(a.frequency()));
        let partial: f32 = words
            .into_iter()
            .enumerate()
            .map(|(index, w)| w.frequency().to_f32() * (index as f32))
            .sum();
        let total = self.penalty.to_f32() + partial;
        Penalty::new(total)
    }
}

impl Conflicts {
    pub fn new(dictionary: &Dictionary) -> Conflicts {
        let mut result = HashMap::<String, FrequentWords>::new();
        let max_keys = 3;
        let max_letters = 6;
        let words = dictionary.words();
        for word_a_index in 0..words.len() - 1 {
            println!("{}", word_a_index.separate_with_underscores());
            for word_b_index in word_a_index + 1..words.len() {
                let word_a = &words[word_a_index];
                let word_b = &words[word_b_index];
                let diff = word_a.letter_pair_difference(&word_b);
                if diff.len() <= max_keys && diff.letter_count() <= max_letters {
                    let diff_as_string = diff.to_string();
                    let words = result.get_mut(&diff_as_string);
                    match words {
                        None => {
                            println!("{}", diff_as_string);
                            let mut words = FrequentWords::empty();
                            words.insert(word_a.clone());
                            words.insert(word_b.clone());
                            result.insert(diff_as_string, words);
                            let key_count = result.len();
                            if key_count.rem_euclid(10_000) == 0 {
                                println!("Keys: {}", key_count.separate_with_underscores());
                            }
                        }
                        Some(words) => {
                            words.insert(word_a.clone());
                            words.insert(word_b.clone());
                        }
                    }
                }
            }
        }
        Conflicts(result)
    }

    pub fn write_to_file(&self) {
        const FILE_NAME: &'static str = "./conflicts.txt";
        let write = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(FILE_NAME);
        let mut writer = BufWriter::new(write.unwrap());
        self.0.iter().for_each(|(k, v)| {
            let penalty = v.penalty();
            writeln!(writer, "{},{}", k, penalty.to_f32()).unwrap();
            println!("{},{}", k, penalty);
        });
        writer.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn frequent_words() {
        let mut target = FrequentWords::empty();
        assert_eq!(target.penalty(), Penalty::ZERO);

        let data = [
            ("abc", 9.0, 0.0),
            ("abc", 9.0, 0.0),
            ("big", 6.0, 6.0),
            ("big", 6.0, 6.0),
            ("happy", 4.0, 14.0),
            ("clown", 3.0, 23.0),
        ];
        for (word, freq, expected) in data {
            target.insert(Word::new(word, freq));
            assert_eq!(
                target.penalty(),
                Penalty::new(expected),
                "word: {}, frequency: {}, expected: {}",
                word,
                freq,
                expected
            );
        }
        // target.insert(Word::new("abc", 9.0));
        // target.insert(Word::new("abc", 9.0));
        // assert_eq!(target.penalty(), Penalty::ZERO); // only 1 word, high frequency

        // target.insert(Word::new("def", 2.0));
        // target.insert(Word::new("def", 2.0));
        // assert_eq!(target.penalty(), Penalty::new(2.0)); // two words

        // target.insert(Word::new("def", 2.0));
        // target.insert(Word::new("def", 2.0));
        // assert_eq!(target.penalty(), Penalty::new(2.0)); // two words
    }
}
