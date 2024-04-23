use crate::frequency::Frequency;
use crate::letter::Letter;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Eq, Debug, Clone)]
pub struct Word {
    letters: Vec<Letter>,
    frequency: Frequency,
}

impl Word {
    pub const MAX_WORD_LENGTH: usize = 25; // must fit in u128

    pub fn new(word: &str, frequency: f32) -> Result<Word, &'static str> {
        let letters = {
            let vec = word
                .chars()
                .map(Letter::try_from)
                .collect::<Result<Vec<Letter>, _>>()?;
            if vec.len() == 0 {
                Err("A Word must have 1 or more letters in it.")
            } else if vec.len() > Word::MAX_WORD_LENGTH {
                Err("A Word can not have more than 25 letters.")
            } else {
                Ok(vec)
            }
        }?;
        let frequency = Frequency::try_from(frequency)?;
        Ok(Word { letters, frequency })
    }

    pub fn letters(&self) -> &Vec<Letter> {
        &self.letters
    }

    pub fn frequency(&self) -> &Frequency {
        &self.frequency
    }
}

impl Hash for Word {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.letters.hash(state);
    }
}

impl TryFrom<&str> for Word {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Word::new(value, 0.0)
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letters
            .iter()
            .map(|r| write!(f, "{}", r))
            .collect::<Result<(), _>>()
    }
}

impl PartialOrd for Word {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.letters.cmp(&other.letters))
    }
}

impl Ord for Word {
    fn cmp(&self, other: &Self) -> Ordering {
        self.letters.cmp(&other.letters)
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.letters == other.letters
    }
}

#[cfg(test)]
mod tests {

    use std::{cmp::Ordering, collections::HashSet};

    use super::*;

    #[test]
    fn try_from_str_when_valid_characters() {
        for s in ["banana", "apple", "pear"] {
            assert_eq!(s.to_string(), Word::try_from(s).unwrap().to_string());
        }
    }

    #[test]
    fn try_from_str_when_invalid_characters() {
        for s in ["45jal", "a%pple", "pe   ar", "   "] {
            assert!(Word::try_from(s).is_err());
        }
    }

    #[test]
    fn try_from_str_when_no_characters_fails() {
        assert!(Word::try_from("").is_err());
    }

    #[test]
    fn new_when_valid() {
        let data = [
            ("abc", 0.3),
            ("happy", 0.1),
            ("abcdefghijklmnopqrstuvwxy", 0.0),
            ("bcdefghijklmnopqrstuvwxyz", 0.0),
            ("there's", 0.0),
        ];
        for (s, f) in data {
            let word = Word::new(s, f).unwrap();
            let letters_as_string = word
                .letters()
                .iter()
                .map(|r| Letter::to_string(r))
                .collect::<Vec<String>>()
                .join("");
            assert_eq!(s, letters_as_string);
            assert_eq!(f, word.frequency().to_f32());
        }
    }

    #[test]
    fn new_when_invalid() {
        let data = [
            ("", 0.3),
            ("h4jf", 0.1),
            ("fdkw**fj'", 0.2),
            ("abc", -0.3),
            ("abc", f32::INFINITY),
            ("abc", f32::NAN),
            ("abc", f32::NEG_INFINITY),
            ("5436", f32::NEG_INFINITY),
        ];
        for (s, f) in data {
            let word = Word::new(s, f);
            assert!(word.is_err());
        }
    }

    #[test]
    fn new_when_too_long() {
        assert!(Word::new("abcdefghijklmnopqrstuvwxy", 0.5).is_ok());
        assert!(Word::new("abcdefghijklmnopqrstuvwxyz", 0.5).is_err());
    }

    #[test]
    fn cmp_sorts_only_by_word() {
        let data = [
            ("a", "b", Ordering::Less),
            ("a", "bcd", Ordering::Less),
            ("abc", "abcd", Ordering::Less),
            ("a", "a", Ordering::Equal),
            ("abc", "abc", Ordering::Equal),
            ("aaaaa", "z", Ordering::Less),
        ];
        for (a, b, ordering) in data {
            for freq_same in [false, true] {
                let a_word = Word::new(a, 0.2).unwrap();
                let b_word = Word::new(b, if freq_same { 0.2 } else { 0.8 }).unwrap();
                assert_eq!(a_word.cmp(&b_word), ordering);
                assert_eq!(b_word.cmp(&a_word), ordering.reverse());
                assert_eq!(
                    Word::cmp(&a_word, &b_word),
                    String::cmp(&a_word.to_string(), &b_word.to_string())
                );
            }
        }
    }

    #[test]
    fn hash_ignores_frequency() {
        let a = Word::new("abc", 0.3).unwrap();
        let b = Word::new("abc", 0.8).unwrap();
        let c = Word::new("c", 0.3).unwrap();
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(2, set.len());
    }
}
