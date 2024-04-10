use std::collections::HashSet;
use std::{collections::HashMap, fs::File, io::BufReader};

use crate::set32::Set32;
use crate::u5::U5;
use crate::{frequency::Frequency, word::Word};

pub struct Dictionary {
    words_highest_frequency_first: Vec<Word>,
    frequency_sum: Frequency,
    letter_to_u5: HashMap<char, U5>,
    u5_to_letter: Vec<char>,
    letter_index_set: Set32,
}

impl Dictionary {
    const FILE_NAME: &'static str = "./src/words.json";

    pub fn new(words: HashMap<String, f32>) -> Dictionary {
        let (_letter_set, letter_index_set, letter_to_u5, u5_to_letter) = {
            // create set of unique letters
            let mut letter_set = HashSet::new();
            words.iter().flat_map(|(s, _f)| s.chars()).for_each(|c| {
                letter_set.insert(c);
            });

            // map each letter to an index and vice versa
            let mut letter_to_u5 = HashMap::new();
            let mut u5_to_letter = Vec::new();
            letter_set.iter().enumerate().for_each(|(i, c)| {
                let index = u32::try_from(i).unwrap(); // switch to u5
                letter_to_u5.insert(*c, U5::new(index));
                u5_to_letter.push(*c);
            });

            // make a Set32 of the letters in all words
            let letter_index_set = Set32::fill(letter_set.len().try_into().unwrap());

            // return calculated values
            (letter_set, letter_index_set, letter_to_u5, u5_to_letter)
        };
        let (words_highest_frequency_first, frequency_sum) = {
            let mut words_highest_frequency_first = Vec::new();
            let mut frequency_sum = Frequency::ZERO;
            words.iter().for_each(|(s, f)| {
                let word_frequency = Frequency::new(*f);
                let _letter_set_in_word = s.chars().fold(Set32::EMPTY, |set, c| {
                    let char_as_u5 = letter_to_u5
                        .get(&c)
                        .expect("the letter could not be converted to a u5");
                    set.add(*char_as_u5)
                });
                let word = Word::with_details(s.to_owned(), word_frequency);
                frequency_sum = frequency_sum + word_frequency;
                words_highest_frequency_first.push(word)
            });
            words_highest_frequency_first.sort_by(|a, b| Word::cmp_by_frequency(b, a));
            (words_highest_frequency_first, frequency_sum)
        };
        Dictionary {
            words_highest_frequency_first,
            frequency_sum,
            letter_to_u5,
            u5_to_letter,
            letter_index_set,
        }
    }

    pub fn with_top_n_words(&self, count: usize) -> Dictionary {
        let mut map = HashMap::new();
        self.words_highest_frequency_first
            .iter()
            .take(count)
            .map(|w| w.to_tuple())
            .for_each(|(s, f)| {
                map.insert(s, f);
            });
        Dictionary::new(map)
    }

    pub fn u5_to_letter(&self, inx: U5) -> char {
        let result = self.u5_to_letter[inx.to_usize()]; // fix
        result
    }

    pub fn letter_to_u5(&self, char: char) -> Option<&U5> {
        self.letter_to_u5.get(&char)
    }

    pub fn words(&self) -> &Vec<Word> {
        let result = &self.words_highest_frequency_first;
        result
    }

    pub fn letters(&self) -> Set32 {
        self.letter_index_set
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
        println!("{}", d.frequency_sum);
        let expected = 307629;
        assert_eq!(d.words_highest_frequency_first.len(), expected);
        assert_eq!(d.letter_to_u5.len(), 27);
        assert_eq!(d.u5_to_letter.len(), 27);
        assert_eq!(d.letter_index_set.count(), 27);
        assert!(d.frequency_sum >= Frequency::new(0.95) && d.frequency_sum <= Frequency::new(0.97));
    }

    #[test]
    fn letter_map_works_properly() {
        let words = vec!["apple", "banana", "charlie", "bob"];
        let source: HashMap<_, _> = words.iter().map(|s| (s.to_string(), 0.0)).collect();
        let d = Dictionary::new(source);

        // correct size of the letter set: aplebnchrio
        assert_eq!(d.letter_index_set.count(), 11);

        // internal sizes correct
        assert_eq!(d.u5_to_letter.len(), 11);
        assert_eq!(d.letter_to_u5.len(), 11);

        // mapping back and forth is consistent
        for i in 0u32..11 {
            let char1 = d.u5_to_letter(i.into());
            let inx = d.letter_to_u5(char1);
            match inx {
                None => panic!("could not find the u5 representation of a letter"),
                Some(inx) => {
                    let char2 = d.u5_to_letter(*inx);
                    assert_eq!(char1, char2);
                }
            }
        }
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
