use crate::{dictionary::Dictionary, penalty::Penalty, util::choose, word::Word};
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{BufWriter, Write},
};
use thousands::Separable;

pub struct Conflicts(HashMap<String, HashSet<Word>>);

impl Conflicts {
    pub fn empty() -> Conflicts {
        Conflicts(HashMap::new())
    }

    pub fn calculate(&mut self, max_keys: u8, max_letters: u8, dictionary: &Dictionary) {
        let words = dictionary.words();
        let total_pairs = choose(words.len().try_into().unwrap(), 2);
        let mut processed = 0u128;
        for word_a_index in 0..words.len() - 1 {
            for word_b_index in word_a_index + 1..words.len() {
                processed = processed + 1;
                if processed.rem_euclid(10_000_000) == 0 {
                    println!(
                        "Processed: {} of {}",
                        processed.separate_with_underscores(),
                        total_pairs.separate_with_underscores()
                    )
                };
                let word_a = &words[word_a_index];
                let word_b = &words[word_b_index];
                let diff = word_a.letter_pair_difference(&word_b);
                if diff.len() > 0
                    && diff.len() <= max_keys as usize
                    && diff.letter_count() <= max_letters
                {
                    let diff_as_string = diff.to_string();
                    let words = self.0.get_mut(&diff_as_string);
                    match words {
                        None => {
                            let mut words = HashSet::new();
                            words.insert(word_a.clone());
                            words.insert(word_b.clone());
                            self.0.insert(diff_as_string, words);
                            let key_count = self.0.len();
                            if key_count.rem_euclid(100_000) == 0 {
                                println!("Unique pairs: {}", key_count.separate_with_underscores());
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
    }

    fn penalty(words: &HashSet<Word>) -> Penalty {
        let mut words = words.iter().collect::<Vec<&Word>>();
        words.sort_unstable_by(|a, b| b.frequency().cmp(&a.frequency()));
        let penalty_value = words
            .into_iter()
            .enumerate()
            .fold(0.0, |total, (index, w)| {
                let multiplier = index.min(4) as f32;
                let penalty = multiplier * w.frequency().to_f32();
                total + penalty
            });
        Penalty::new(penalty_value)
    }

    pub fn write_to_file(&self) {
        const FILE_NAME: &'static str = "./conflicts.csv";
        let write = OpenOptions::new().create(true).write(true).open(FILE_NAME);
        let mut writer = BufWriter::new(write.unwrap());
        writeln!(writer, "pairs,penalty").unwrap();
        self.0.iter().for_each(|(pairs, words)| {
            let penalty = Self::penalty(words);
            writeln!(writer, "{},{}", pairs, penalty.to_f32()).unwrap();
            println!("{},{}", pairs, penalty);
        });
        writer.flush().unwrap();
    }
}
