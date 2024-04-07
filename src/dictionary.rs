use std::collections::HashSet;
use std::{collections::HashMap, fs::File, io::BufReader};

use crate::set32::Set32;
use crate::{frequency::Frequency, word::Word};

pub struct Dictionary {
    words_highest_frequency_first: Vec<Word>,
    frequency_sum: Frequency,
    letter_to_u6: HashMap<char, u32>,
    u6_to_letter: Vec<char>,
    letter_index_set: Set32,
}

// let make: unit => dictionary
// let makeFrom: Seq.t<(word, frequency)> => dictionary
// let wordsByFreq: dictionary => array<(word, frequency)>
// let lettersByFreq: dictionary => array<character>
// let topWords: (dictionary, int) => dictionary
// let letters: dictionary => Seq.t<character>
// let random: (~characters: string, ~length: int) => dictionary

impl Dictionary {
    const FILE_NAME: &'static str = "./src/words.json";

    // pub fn summarize_letters(
    //     words: impl Iterator<Item = String>,
    // ) -> (Set32, HashMap<char, u32>, Vec<char>) {
    //     let mut letter_set = HashSet::new();
    //     words.flat_map(|w| w.chars()).for_each(|c| {
    //         letter_set.insert(c);
    //     });
    //     let mut letter_to_num = HashMap::new();
    //     let mut num_to_letter = Vec::new();
    //     letter_set.iter().enumerate().for_each(|(index, letter)| {
    //         let index = u32::try_from(index).unwrap();
    //         letter_to_num.insert(*letter, index);
    //         num_to_letter.push(*letter);
    //     });
    //     let set = Set32::fill(letter_set.len().try_into().unwrap());
    //     (set, letter_to_num, num_to_letter)
    // }

    fn new(words: HashMap<String, f32>) -> Dictionary {
        let (_letter_set, letter_index_set, letter_to_u6, u6_to_letter) = {
            // create set of unique letters
            let mut letter_set = HashSet::new();
            words.iter().flat_map(|(s, f)| s.chars()).for_each(|c| {
                letter_set.insert(c);
            });

            // map each letter to an index and vice versa
            let mut letter_to_u6 = HashMap::new();
            let mut u6_to_letter = Vec::new();
            letter_set.iter().enumerate().for_each(|(i, c)| {
                let index = u32::try_from(i).unwrap(); // switch to u6
                letter_to_u6.insert(*c, index);
                u6_to_letter.push(*c);
            });

            // make a Set32 of the letters in all words
            let letter_index_set = Set32::fill(letter_set.len().try_into().unwrap());

            // return calculated values
            (letter_set, letter_index_set, letter_to_u6, u6_to_letter)
        };
        let (words_highest_frequency_first, frequency_sum) = {
            let mut words_highest_frequency_first = Vec::new();
            let mut frequency_sum = Frequency::ZERO;
            words.iter().for_each(|(s, f)| {
                let word_frequency = Frequency::new(*f);
                let letter_set_in_word = s.chars().fold(Set32::EMPTY, |set, c| {
                    let char_as_u6 = letter_to_u6
                        .get(&c)
                        .expect("the letter could not be converted to a u6");
                    set.add(*char_as_u6)
                });
                let word = Word::with_details(s.to_owned(), word_frequency, letter_set_in_word);
                frequency_sum = frequency_sum + word_frequency;
                words_highest_frequency_first.push(word)
            });
            words_highest_frequency_first.sort_by(|a, b| Word::cmp_by_frequency(b, a));
            (words_highest_frequency_first, frequency_sum)
        };
        Dictionary {
            words_highest_frequency_first,
            frequency_sum,
            letter_to_u6,
            u6_to_letter,
            letter_index_set,
        }
    }

    pub fn letter_for_u6(&self, inx: u32) -> char {
        let result = self.u6_to_letter[inx as usize]; // fix
        result
    }

    pub fn u6_for_letter(&self, char: char) -> u32 {
        let result = self.letter_to_u6.get(&char).unwrap();
        *result
    }

    fn load_json() -> HashMap<String, f32> {
        let file = File::open(Dictionary::FILE_NAME).expect("file not found");
        let reader = BufReader::new(file);
        let word_frequencies: HashMap<String, f32> =
            serde_json::from_reader(reader).expect("read json properly");
        word_frequencies
    }

    pub fn load() -> Dictionary {
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
        let d = Dictionary::load();
        println!("{}", d.frequency_sum);
        let expected = 307629;
        assert_eq!(d.words_highest_frequency_first.len(), expected);
        assert_eq!(d.letter_to_u6.len(), 27);
        assert_eq!(d.u6_to_letter.len(), 27);
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
        assert_eq!(d.u6_to_letter.len(), 11);
        assert_eq!(d.letter_to_u6.len(), 11);

        // mapping back and forth is consistent
        for i in 0..11 {
            let char1 = d.letter_for_u6(i);
            let inx = d.u6_for_letter(char1);
            let char2 = d.letter_for_u6(inx);
            assert_eq!(char1, char2);
        }
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
