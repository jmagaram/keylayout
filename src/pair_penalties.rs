use crate::{dictionary::Dictionary, key_set::KeySet, penalty::Penalty, util::choose, word::Word};
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{BufWriter, Write},
};
use thousands::Separable;

pub struct PairPenalties(HashMap<KeySet, Penalty>);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CsvOutput {
    pairs: String,
    penalty: f32,
}

impl PairPenalties {
    const FILE_NAME: &'static str = "./pair_penalties.csv";

    fn load_from_csv(file_name: &str) -> Result<Vec<CsvOutput>, csv::Error> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_name)
            .unwrap();
        rdr.deserialize().collect::<Result<Vec<CsvOutput>, _>>()
    }

    pub fn penalty(&self, key_set: &KeySet) -> Penalty {
        self.0.get(key_set).map(|p| *p).unwrap_or(Penalty::ZERO)
    }

    pub fn load() -> PairPenalties {
        Self::load_from(Self::FILE_NAME)
    }

    pub fn load_from(file_name: &str) -> PairPenalties {
        let map_items = Self::load_from_csv(file_name)
            .unwrap()
            .into_iter()
            .map(|c| {
                let penalty = Penalty::new(c.penalty);
                let key_set = KeySet::with_layout(c.pairs.as_str());
                (key_set, penalty)
            });
        let map = HashMap::from_iter(map_items);
        PairPenalties(map)
    }
}

pub struct MakePairPenalties {
    pub dictionary: Dictionary,
    pub max_keys: u8,
    pub max_letters: u8,
    pub file_name: String,
}

impl MakePairPenalties {
    pub fn calculate(&self) {
        let mut result: HashMap<KeySet, HashSet<u32>> = HashMap::new();
        let words = self.dictionary.words();
        let words_len: u32 = words.len().try_into().unwrap();
        let total_pairs = choose(words.len().try_into().unwrap(), 2);
        let mut processed = 0u128;
        for word_a_index in 0..words_len - 1 {
            for word_b_index in word_a_index + 1..words_len {
                processed = processed + 1;
                if processed.rem_euclid(10_000_000) == 0 {
                    println!(
                        "Processed: {} of {}",
                        processed.separate_with_underscores(),
                        total_pairs.separate_with_underscores()
                    )
                };
                let word_a = &words[word_a_index as usize];
                let word_b = &words[word_b_index as usize];
                let diff = word_a.letter_pair_difference(&word_b);
                if diff.len() > 0
                    && diff.len() <= self.max_keys as usize
                    && diff.letter_count() <= self.max_letters
                {
                    let words = result.get_mut(&diff);
                    match words {
                        None => {
                            let mut words = HashSet::new();
                            words.insert(word_a_index);
                            words.insert(word_b_index);
                            result.insert(diff, words);
                        }
                        Some(words) => {
                            words.insert(word_a_index);
                            words.insert(word_b_index);
                        }
                    }
                }
            }
        }
        self.write_to_file(result);
    }

    fn write_to_file(&self, results: HashMap<KeySet, HashSet<u32>>) {
        let write = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.file_name.as_str());
        let mut writer = BufWriter::new(write.unwrap());
        writeln!(writer, "pairs,penalty").unwrap();
        results.into_iter().for_each(|(pairs, words)| {
            let penalty = Self::penalty(self.dictionary.words(), &words);
            writeln!(writer, "{},{}", pairs, penalty.to_f32()).unwrap();
            println!("{},{}", pairs, penalty);
        });
        writer.flush().unwrap();
    }

    fn penalty(words: &Vec<Word>, word_indexes: &HashSet<u32>) -> Penalty {
        let mut penalty_total = 0.0;
        let max_word_len = words.iter().map(|w| w.len()).max().unwrap();
        for word_len in 1..=max_word_len {
            let mut words = word_indexes
                .iter()
                .map(|index| words[*index as usize].clone())
                .filter(|w| w.len() == word_len)
                .collect::<Vec<Word>>();
            words.sort_unstable_by(|a, b| b.frequency().cmp(&a.frequency()));
            let penalty_value = words
                .into_iter()
                .enumerate()
                .fold(0.0, |total, (index, w)| {
                    let multiplier = index.min(4) as f32;
                    let penalty = multiplier * w.frequency().to_f32();
                    total + penalty
                });
            penalty_total = penalty_total + penalty_value;
        }
        Penalty::new(penalty_total)
    }
}
