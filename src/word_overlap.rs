use crate::{dictionary::Dictionary, key_set::KeySet, util::choose, word::Word};
use std::collections::{HashMap, HashSet};
use thousands::Separable;

pub type WordIndex = u32;

pub struct WordOverlap {
    words: Vec<Word>,
    pairs: HashMap<KeySet, HashSet<WordIndex>>,
    empty: HashSet<WordIndex>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CsvRow {
    pairs: String,
    word: String,
}

impl WordOverlap {
    pub fn calculate(dictionary: &Dictionary, max_pairs: u8) -> WordOverlap {
        let mut pairs: HashMap<KeySet, HashSet<WordIndex>> = HashMap::new();
        let words = dictionary.words().clone();
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
                    && diff.len() <= max_pairs as usize
                    && diff.letter_count() <= max_pairs * 2
                {
                    let words = pairs.get_mut(&diff);
                    match words {
                        None => {
                            let mut words = HashSet::new();
                            words.insert(word_a_index);
                            words.insert(word_b_index);
                            pairs.insert(diff, words);
                        }
                        Some(words) => {
                            words.insert(word_a_index);
                            words.insert(word_b_index);
                        }
                    }
                }
            }
        }
        WordOverlap {
            words,
            pairs,
            empty: HashSet::new(),
        }
    }

    pub fn word_from_index(&self, index: WordIndex) -> Option<&Word> {
        self.words.get(index as usize)
    }

    pub fn words_for_pairs(&self, pairs: &KeySet) -> &HashSet<WordIndex> {
        self.pairs.get(pairs).unwrap_or(&self.empty)
    }

    pub fn print(&self) {
        self.pairs.iter().for_each(|(pairs, words)| {
            println!("{}", pairs);
            words.iter().for_each(|word_index| {
                println!(
                    "  {}",
                    self.word_from_index(*word_index).unwrap().to_string()
                )
            })
        })
    }

    pub fn save_to_csv(&self, file_name: &str) -> Result<(), csv::Error> {
        let mut wtr = csv::Writer::from_path(file_name)?;
        let rows = self.pairs.iter().flat_map(|(pairs, words)| {
            let pairs = pairs.to_string();
            words.iter().map(move |word_index| {
                let word = self.word_from_index(*word_index).unwrap().to_string();
                CsvRow {
                    pairs: pairs.clone(),
                    word,
                }
            })
        });
        let _write_result = rows
            .map(|r| wtr.serialize(r))
            .collect::<Result<(), csv::Error>>()?;
        wtr.flush().unwrap();
        Ok(())
    }

    pub fn load_from_csv(dictionary: &Dictionary, file_name: &str) -> WordOverlap {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_name)
            .unwrap();
        let words = dictionary.words().clone();
        let mut words_to_index: HashMap<String, u32> = HashMap::new();
        for i in 0..words.len() {
            words_to_index.insert(words.get(i).map(|w| w.to_string()).unwrap(), i as u32);
        }
        let mut pairs: HashMap<KeySet, HashSet<WordIndex>> = HashMap::new();
        let mut progress: u64 = 0;
        for row in rdr.deserialize::<CsvRow>() {
            let row = row.unwrap();
            progress = progress + 1;
            if progress.rem_euclid(10_000) == 0 {
                println!("{}", progress.separate_with_underscores())
            }
            if let Some(word_index) = words_to_index.get(&row.word) {
                let keys = KeySet::with_layout(&row.pairs);
                match pairs.get_mut(&keys) {
                    Some(set) => {
                        set.insert(*word_index);
                    }
                    None => {
                        let mut set = HashSet::new();
                        set.insert(*word_index);
                        pairs.insert(keys, set);
                    }
                }
            }
        }
        WordOverlap {
            words,
            pairs,
            empty: HashSet::new(),
        }
    }
}
