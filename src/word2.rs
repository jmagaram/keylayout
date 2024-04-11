use crate::frequency::Frequency;
use crate::letter::Letter;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Eq)]
pub struct Word2 {
    letters: Vec<Letter>,
    frequency: Frequency,
}

impl Word2 {
    pub fn new(letters: Vec<Letter>, frequency: Frequency) -> Word2 {
        if letters.len() == 0 {
            panic!("A Word must have 1 or more letters in it.")
        }
        Word2 { letters, frequency }
    }

    pub fn letters(&self) -> &Vec<Letter> {
        &self.letters
    }

    pub fn frequency(&self) -> &Frequency {
        &self.frequency
    }
}

impl Hash for Word2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.letters.hash(state);
    }
}

impl TryFrom<&str> for Word2 {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.len() {
            0 => Err("A Word must have 1 or more letters in it."),
            _ => value
                .chars()
                .map(|c| Letter::try_from(c))
                .collect::<Result<Vec<Letter>, _>>()
                .map(|letters| Word2 {
                    letters: letters,
                    frequency: Frequency::ZERO,
                }),
        }
    }
}

impl std::fmt::Display for Word2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letters
            .iter()
            .map(|r| write!(f, "{}", r))
            .collect::<Result<(), _>>()
    }
}

impl PartialOrd for Word2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.letters.cmp(&other.letters))
    }
}

impl Ord for Word2 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.letters.cmp(&other.letters)
    }
}

impl PartialEq for Word2 {
    fn eq(&self, other: &Self) -> bool {
        self.letters == other.letters
    }
}

#[cfg(test)]
mod tests {

    use std::{cmp::Ordering, collections::HashSet};

    use super::*;

    fn aa() -> Letter {
        Letter::try_from('a').unwrap()
    }

    fn bb() -> Letter {
        Letter::try_from('b').unwrap()
    }

    fn cc() -> Letter {
        Letter::try_from('c').unwrap()
    }

    #[test]
    fn try_from_str_when_valid_characters() {
        for s in ["banana", "apple", "pear"] {
            assert_eq!(s.to_string(), Word2::try_from(s).unwrap().to_string());
        }
    }

    #[test]
    fn try_from_str_when_invalid_characters() {
        for s in ["45jal", "a%pple", "pe   ar", "   "] {
            assert!(Word2::try_from(s).is_err());
        }
    }

    #[test]
    fn try_from_str_when_no_characters_fails() {
        assert!(Word2::try_from("").is_err());
    }

    #[test]
    fn letters_test() {
        let word = Word2::try_from("abc").unwrap();
        let letters = word.letters();
        assert_eq!(aa(), letters[0]);
        assert_eq!(bb(), letters[1]);
        assert_eq!(cc(), letters[2]);
    }

    #[test]
    fn frequency_test() {
        let word = Word2::new(vec![aa()], Frequency::new(0.123));
        assert_eq!(word.frequency(), &Frequency::new(0.123));
    }

    #[test]
    fn new_test() {
        let word = Word2::new(vec![aa(), bb(), cc()], Frequency::new(0.5));
        let letters = word.letters();
        assert_eq!('a', letters[0].to_char());
        assert_eq!('b', letters[1].to_char());
        assert_eq!('c', letters[2].to_char());
        assert_eq!(Frequency::new(0.5), word.frequency);
    }

    #[test]
    #[should_panic]
    fn new_panics_if_letters_are_empty() {
        let _word = Word2::new(vec![], Frequency::new(0.5));
    }

    #[test]
    fn cmp_sorts_only_by_word() {
        let freq_small = Frequency::new(0.1);
        let freq_big = Frequency::new(0.8);
        fn with_freq(letters: &str, freq: Frequency) -> Word2 {
            let word = Word2::try_from(letters).unwrap();
            Word2::new(word.letters().to_owned(), freq)
        }
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
                let a_word = with_freq(a, freq_small);
                let b_word = with_freq(b, if freq_same { freq_small } else { freq_big });
                assert_eq!(a_word.cmp(&b_word), ordering);
                assert_eq!(b_word.cmp(&a_word), ordering.reverse());
                assert_eq!(
                    Word2::cmp(&a_word, &b_word),
                    String::cmp(&a_word.to_string(), &b_word.to_string())
                );
            }
        }
    }

    #[test]
    fn hash_ignores_frequency() {
        let a = Word2::new(vec![aa(), bb(), cc()], Frequency::new(0.3));
        let b = Word2::new(vec![aa(), bb(), cc()], Frequency::new(0.8));
        let c = Word2::new(vec![cc()], Frequency::new(0.3));
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(2, set.len());
    }

    // #[test]
    // fn ord() {
    //     let strs = ["a", "z", "b", "y", "c", "x", "d", "w", "p", "o", "n", "m"];
    //     let mut word_vec = strs
    //         .into_iter()
    //         .map(|w| Word2::from(w))
    //         .collect::<Vec<Word2>>();
    //     word_vec.sort();
    //     let combined = word_vec
    //         .into_iter()
    //         .map(|w| w.to_string())
    //         .collect::<Vec<String>>()
    //         .join("");
    //     assert_eq!(combined, "abcdmnopwxyz");
    // }
}
