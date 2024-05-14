use crate::{dictionary::Dictionary, penalty::Penalty, word::Word};
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{BufWriter, Write},
};
use thousands::Separable;

pub struct Conflicts(HashMap<String, TopWords>);

struct TopWords {
    words: HashSet<Word>,
    penalty: Penalty,
}

impl TopWords {
    fn empty() -> TopWords {
        TopWords {
            words: HashSet::new(),
            penalty: Penalty::ZERO,
        }
    }

    fn add_word(&mut self, word: Word) {
        if self.words.insert(word) {
            if self.words.len() >= 4 {
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

    fn final_penalty(&self) -> Penalty {
        let words = self.words.iter().collect::<Vec<&Word>>();
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
        let mut result = HashMap::<String, TopWords>::new();
        let max_keys = 3;
        let max_letters = 4;
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
                            let mut words = TopWords::empty();
                            words.add_word(word_a.clone());
                            words.add_word(word_b.clone());
                            result.insert(diff_as_string, words);
                        }
                        Some(words) => {
                            words.add_word(word_a.clone());
                            words.add_word(word_b.clone());
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
            let penalty = v.final_penalty();
            writeln!(writer, "{},{}", k, penalty).unwrap();
            println!("{},{}", k, penalty);
        });
        writer.flush().unwrap();
    }
}
