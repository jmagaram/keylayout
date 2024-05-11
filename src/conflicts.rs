use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use crate::{dictionary::Dictionary, letter_pair_set::LetterPairSet, penalty::Penalty, word::Word};

pub struct Conflicts(HashMap<LetterPairSet, Penalty>);

impl Conflicts {
    pub fn new(dictionary: &Dictionary) {
        let mut result = HashMap::<String, HashSet<Word>>::new();
        for word_length in 1..=Word::MAX_WORD_LENGTH {
            let words = dictionary
                .words()
                .iter()
                .filter(|w| w.len() as usize == word_length)
                .enumerate()
                .collect::<Vec<(usize, &Word)>>();
            for word_a_index in 0..words.len() - 1 {
                for word_b_index in word_a_index + 1..words.len() {
                    println!("{},{}", word_a_index, word_b_index);
                    let (_, word_a) = words[word_a_index];
                    let (_, word_b) = words[word_b_index];
                    let diff = word_a.difference(word_b);
                    if !diff.is_empty() {
                        let serialized = diff.to_string();
                        let item = result.get_mut(&serialized);
                        match item {
                            None => {
                                let mut set = HashSet::new();
                                set.insert(word_a.clone());
                                set.insert(word_b.clone());
                                result.insert(serialized, set);
                            }
                            Some(set) => {
                                set.insert(word_a.clone());
                                set.insert(word_b.clone());
                            }
                        }
                    }
                }
            }
        }
        const FILE_NAME: &'static str = "./conflicts.txt";
        let write = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(FILE_NAME);
        let mut writer = BufWriter::new(write.unwrap());
        for (k, set) in result {
            let set_as_string = set
                .iter()
                .map(|w| w.to_string())
                .collect::<Vec<String>>()
                .join(",");
            writeln!(writer, "{}:{}", k, set_as_string);
        }
        writer.flush();
    }
}
