use crate::{frequency::Frequency, set32::Set32};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::str::Chars;
use std::{fmt, str::FromStr};

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Word {
    word: String,
    frequency: Frequency,
    letter_set: Set32,
}

impl Ord for Word {
    fn cmp(&self, other: &Self) -> Ordering {
        self.word.cmp(&other.word)
    }
}

impl Eq for Word {}

impl Hash for Word {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state);
    }
}

impl Word {
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: usize = 40;

    pub fn with_details(word: String, frequency: Frequency, letter_set: Set32) -> Word {
        Word {
            word,
            frequency,
            letter_set,
        }
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    pub fn to_tuple(&self) -> (String, f32) {
        (self.word.to_owned(), self.frequency.to_f32())
    }

    pub fn cmp_by_frequency(a: &Word, b: &Word) -> Ordering {
        a.frequency.cmp(&b.frequency)
    }

    pub fn new(word: String) -> Word {
        let word = word.trim().to_string();
        debug_assert!(word.len() >= Word::MIN_LENGTH, "{}", word);
        debug_assert!(word.len() <= Word::MAX_LENGTH, "{}", word);
        Word {
            word,
            frequency: Frequency::ZERO,
            letter_set: Set32::EMPTY,
        }
    }

    pub fn chars(&self) -> Chars<'_> {
        self.word.chars()
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.word)
    }
}

impl std::convert::From<String> for Word {
    fn from(value: String) -> Self {
        Word::new(value)
    }
}

impl std::convert::From<&str> for Word {
    fn from(value: &str) -> Self {
        let w = String::from_str(value).expect("Could not convert the characters to a String");
        Word::new(w)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn from_string_test() {
        let source = String::from("abc");
        let word = Word::from(source);
        assert_eq!(word.to_string(), "abc");
    }

    #[test]
    fn from_str() {
        let source: Word = "abc".into();
        let word = Word::from(source);
        assert_eq!(word.to_string(), "abc");
    }

    #[test]
    fn display() {
        assert_eq!(Word::from("abc").to_string(), "abc");
    }

    #[test]
    fn ord() {
        let strs = ["a", "z", "b", "y", "c", "x", "d", "w", "p", "o", "n", "m"];
        let mut word_vec = strs
            .into_iter()
            .map(|w| Word::from(w))
            .collect::<Vec<Word>>();
        word_vec.sort();
        let combined = word_vec
            .into_iter()
            .map(|w| w.to_string())
            .collect::<Vec<String>>()
            .join("");
        assert_eq!(combined, "abcdmnopwxyz");
    }
}
