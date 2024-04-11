use std::fmt;

use crate::frequency::{self, Frequency};
use crate::letter::Letter;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct Word2 {
    letters: Vec<Letter>,
    frequency: Frequency,
}

impl Word2 {
    pub fn new(letters: Vec<Letter>, frequency: Frequency) -> Word2 {
        Word2 { letters, frequency }
    }

    pub fn letters(&self) -> &Vec<Letter> {
        &self.letters
    }

    pub fn frequency(&self) -> &Frequency {
        &self.frequency
    }
}

impl TryFrom<&str> for Word2 {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .chars()
            .map(|c| Letter::try_from(c))
            .collect::<Result<Vec<Letter>, _>>()
            .map(|letters| Word2 {
                letters: letters,
                frequency: Frequency::ZERO,
            })
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

// fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     let Letter(inx) = self;
//     write!(f, "{}", Letter::ALPHABET[*inx as usize])
// }
// impl Ord for Word2 {
//     fn cmp(&self, other: &Self) -> Ordering {
//         Ordering::Greater
//     }
// }

// impl Eq for Word2 {}

// impl Hash for Word2 {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.word.hash(state);
//     }
// }

// impl Word2 {
//     const MIN_LENGTH: usize = 1;
//     const MAX_LENGTH: usize = 40;

//     pub fn with_details(word: String, frequency: Frequency) -> Word2 {
//         Word2 { word, frequency }
//     }

//     pub fn with_random_frequency(word: String) -> Word2 {
//         Word2 {
//             word,
//             frequency: Frequency::random(),
//         }
//     }

//     pub fn with_str(word: &str) -> Word2 {
//         Word2::with_details(word.to_string(), Frequency::random())
//     }

//     pub fn frequency(&self) -> Frequency {
//         self.frequency
//     }

//     pub fn to_tuple(&self) -> (String, f32) {
//         (self.word.to_owned(), self.frequency.to_f32())
//     }

//     pub fn cmp_by_frequency(a: &Word2, b: &Word2) -> Ordering {
//         a.frequency.cmp(&b.frequency)
//     }

//     pub fn new(word: String) -> Word2 {
//         let word = word.trim().to_string();
//         assert!(word.len() >= Word2::MIN_LENGTH, "{}", word);
//         assert!(word.len() <= Word2::MAX_LENGTH, "{}", word);
//         Word2 {
//             word,
//             frequency: Frequency::random(),
//         }
//     }

//     pub fn chars(&self) -> Chars<'_> {
//         self.word.chars()
//     }
// }

// impl fmt::Display for Word2 {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.word)
//     }
// }

// impl std::convert::From<String> for Word2 {
//     fn from(value: String) -> Self {
//         Word2::new(value)
//     }
// }

// impl std::convert::From<&str> for Word2 {
//     fn from(value: &str) -> Self {
//         let w = String::from_str(value).expect("Could not convert the characters to a String");
//         Word2::new(w)
//     }
// }

#[cfg(test)]
mod tests {

    use std::cmp::Ordering;

    use crate::word::Word;

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
        for s in ["banana", "apple", "pear", ""] {
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
    fn cmp_sorts_by_word() {
        let data = [
            ("a", "b", Ordering::Less),
            ("a", "bcd", Ordering::Less),
            ("abc", "abcd", Ordering::Less),
            ("a", "a", Ordering::Equal),
            ("abc", "abc", Ordering::Equal),
            ("aaaaa", "z", Ordering::Less),
        ];
        for (a, b, ordering) in data {
            let a_word = Word2::try_from(a).unwrap();
            let b_word = Word2::try_from(b).unwrap();
            assert_eq!(a_word.cmp(&b_word), ordering);
            assert_eq!(b_word.cmp(&a_word), ordering.reverse());
            assert_eq!(
                Word2::cmp(&a_word, &b_word),
                String::cmp(&a_word.to_string(), &b_word.to_string())
            );
        }
    }

    #[test]
    fn cmp_sorts_by_frequency_if_words_are_same() {
        let a = Word2::new(vec![aa(), bb(), cc()], Frequency::new(0.2));
        let b = Word2::new(vec![aa(), bb(), cc()], Frequency::new(0.5));
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&a), Ordering::Greater);
    }

    #[test]
    fn cmp_ignores_frequency_if_words_are_different() {
        let a = Word2::new(vec![aa(), bb()], Frequency::new(0.9));
        let b = Word2::new(vec![cc()], Frequency::new(0.1));
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&a), Ordering::Greater);
    }
    // #[test]
    // fn from_string_test() {
    //     let source = String::from("abc");
    //     let word = Word2::from(source);
    //     assert_eq!(word.to_string(), "abc");
    // }

    // #[test]
    // fn from_str() {
    //     let source: Word2 = "abc".into();
    //     let word = Word2::from(source);
    //     assert_eq!(word.to_string(), "abc");
    // }

    // #[test]
    // fn display() {
    //     assert_eq!(Word2::from("abc").to_string(), "abc");
    // }

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
