use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use thousands::Separable;

use crate::{dictionary::Dictionary, letter_pair_set::LetterPairSet, penalty::Penalty, word::Word};

pub struct Conflicts(HashMap<LetterPairSet, Penalty>);

// score is...
//    0 for most popular word
//    freq * min (4, place in line)

impl Conflicts {
    pub fn new(dictionary: &Dictionary) {
        let mut result = HashMap::<String, HashSet<Word>>::new();
        let mut hash_set_keys: usize = 0;
        let mut hash_set_values: usize = 0;
        let min_word_length = 1;
        let max_word_length = Word::MAX_WORD_LENGTH;
        let min_diff_pairs = 1;
        let max_diff_pairs = 3;
        for word_length in min_word_length..=max_word_length {
            let words = dictionary
                .words()
                .iter()
                .filter(|w| w.len() as usize == word_length)
                .enumerate()
                .collect::<Vec<(usize, &Word)>>();
            for word_a_index in 0..words.len() - 1 {
                for word_b_index in word_a_index + 1..words.len() {
                    if hash_set_keys.rem_euclid(100_000) == 0 {
                        println!("");
                        println!("word len {}", word_length);
                        println!("keys   {}", hash_set_keys.separate_with_underscores());
                        println!("values {}", hash_set_values.separate_with_underscores());
                    }
                    let (_, word_a) = words[word_a_index];
                    let (_, word_b) = words[word_b_index];
                    let diff = word_a.difference(word_b);
                    let diff_as_string = diff.to_string();
                    if diff.len() >= min_diff_pairs && diff.len() <= max_diff_pairs {
                        let words = result.get_mut(&diff_as_string);
                        match words {
                            None => {
                                let mut words = HashSet::new();
                                words.insert(word_a.clone());
                                words.insert(word_b.clone());
                                result.insert(diff_as_string, words);
                                hash_set_keys = hash_set_keys + 1;
                                hash_set_values = hash_set_values + 2;
                            }
                            Some(words) => {
                                let previous_count = words.len();
                                words.insert(word_a.clone());
                                words.insert(word_b.clone());
                                let current_count = words.len();
                                hash_set_values = hash_set_values + current_count - previous_count;
                            }
                        }
                    }
                }
            }
        }
        // const FILE_NAME: &'static str = "./conflicts.txt";
        // let write = OpenOptions::new()
        //     .write(true)
        //     .truncate(true)
        //     .open(FILE_NAME);
        // let mut writer = BufWriter::new(write.unwrap());
        // for (diff, words) in result {
        //     let set_as_string = words
        //         .iter()
        //         .map(|w| w.to_string())
        //         .collect::<Vec<String>>()
        //         .join(",");
        //     writeln!(writer, "{}:{}", diff, set_as_string);
        // }
        // writer.flush();
    }
}
