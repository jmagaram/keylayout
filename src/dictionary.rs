use std::{collections::HashMap, fs::File, io::BufReader};

use crate::frequency::Frequency;

pub struct Dictionary {
    frequencies: HashMap<String, Frequency>,
    words_ordered_by_frequency: Vec<(String, Frequency)>,
    frequency_sum: Frequency,
}

// let make: unit => dictionary
// let makeFrom: Seq.t<(word, frequency)> => dictionary
// let wordsByFreq: dictionary => array<(word, frequency)>
// let lettersByFreq: dictionary => array<character>
// let topWords: (dictionary, int) => dictionary
// let letters: dictionary => Seq.t<character>
// let random: (~characters: string, ~length: int) => dictionary

impl Dictionary {
    const FILE_NAME: &str = "./src/words.json";

    fn load_all_words() -> HashMap<String, f32> {
        let file = File::open(Dictionary::FILE_NAME).expect("file not found");
        let reader = BufReader::new(file);
        let word_frequencies: HashMap<String, f32> =
            serde_json::from_reader(reader).expect("read json properly");
        word_frequencies
    }

    pub fn load() -> Dictionary {
        let frequencies = Dictionary::load_all_words()
            .into_iter()
            .map(|(k, v)| (k, Frequency(v)))
            .collect::<HashMap<String, Frequency>>();
        let mut words_ordered_by_frequency = frequencies
            .clone()
            .into_iter()
            .collect::<Vec<(String, Frequency)>>();
        words_ordered_by_frequency.sort_by(|a, b| {
            let (_a_word, a_freq) = a;
            let (_b_word, b_freq) = b;
            b_freq
                .partial_cmp(a_freq)
                .expect("expected every frequency to be comparable to any other.")
        });
        let frequency_sum = words_ordered_by_frequency
            .clone()
            .into_iter()
            .map(|(_word, freq)| freq)
            .fold(Frequency(0.0), |total, i| total + i);
        Dictionary {
            words_ordered_by_frequency,
            frequencies,
            frequency_sum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_dictionary_has_proper_count_of_words() {
        let d = Dictionary::load();
        let expected = 307629;
        assert_eq!(d.words_ordered_by_frequency.len(), expected);
        assert_eq!(d.frequencies.len(), expected);
    }

    #[test]
    #[ignore]
    fn display_top_words() {
        let d = Dictionary::load();
        d.words_ordered_by_frequency
            .into_iter()
            .take(200)
            .for_each(|f| {
                println!("{0:?}", f);
            });
        let word_count = d.frequencies.len();
        println!("total words {}", word_count);
    }
}
