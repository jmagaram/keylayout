use crate::frequency::Frequency;
use crate::key::Key;
use crate::key_set::KeySet;
use crate::letter::Letter;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Eq, Debug, Clone)]
pub struct Word {
    letters: u128,
    frequency: Frequency,
}

impl Word {
    pub const MAX_WORD_LENGTH: usize = 25;

    pub fn new(word: &str, frequency: f32) -> Word {
        Word::try_from((word, frequency)).unwrap()
    }

    pub fn len(&self) -> u8 {
        ((128u8 - (self.letters.leading_zeros() as u8)) + 4) / 5
    }

    pub fn letters(&self) -> impl Iterator<Item = Letter> {
        let word_len = self.len();
        let mut current = self.letters;
        let top_letter: u128 = 0b11111 << ((word_len - 1) * 5);
        let without_top_letter = (1 << (word_len - 1) * 5) - 1;
        std::iter::from_fn(move || {
            if current == 0 {
                None
            } else {
                let letter_val = ((current & top_letter) >> ((word_len - 1) * 5)) - 1;
                let letter = Letter::try_from(letter_val).unwrap();
                current = current & without_top_letter;
                current = current << 5;
                Some(letter)
            }
        })
    }

    pub fn different_pairs<'a>(
        &'a self,
        other: &Word,
    ) -> impl Iterator<Item = (Letter, Letter)> + 'a {
        let take_while = self.len() == other.len();
        self.letters()
            .zip(other.letters())
            .filter_map(|(a, b)| match a == b {
                true => None,
                false => Some((a, b)),
            })
            .take_while(move |_| take_while)
    }

    pub fn letter_pair_difference(&self, other: &Word) -> KeySet {
        KeySet::with_word_differences(self, other)
    }

    pub fn overlap(&self, other: &Word, placeholder: char) -> Option<String> {
        if self.len() == other.len() {
            Some(
                self.to_string()
                    .chars()
                    .zip(other.to_string().chars())
                    .map(|(a, b)| {
                        if a == b {
                            a.to_string()
                        } else {
                            placeholder.to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(""),
            )
        } else {
            None
        }
    }

    pub fn replace_letters(&self, find: impl Iterator<Item = Letter>, letter: Letter) -> Word {
        let find_key = Key::from_iter(find);
        let new_letters = self
            .letters()
            .map(|r| match find_key.contains(r) {
                true => letter,
                false => r,
            })
            .fold(String::new(), |mut total, i| {
                total.push(i.to_char());
                total
            });
        Self::new(new_letters.as_str(), self.frequency.to_f32())
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
        Word::try_from((value, 0.0))
    }
}

impl TryFrom<(&str, f32)> for Word {
    type Error = &'static str;

    fn try_from(value: (&str, f32)) -> Result<Self, Self::Error> {
        let (word, frequency) = value;
        if word.len() == 0 || word.len() > Word::MAX_WORD_LENGTH {
            Err("The Word has an invalid number of letters.")
        } else {
            let letters =
                word.chars()
                    .fold(Ok(0u128), |total, i| match (total, Letter::try_from(i)) {
                        (Ok(total), Ok(letter)) => {
                            Ok((total << 5) | (letter.to_u8_index() as u128 + 1))
                        }
                        _ => Err("ooops"),
                    })?;
            let frequency = Frequency::try_from(frequency)?;
            Ok(Word { letters, frequency })
        }
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letters()
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

    use super::*;
    use std::{cmp::Ordering, collections::HashSet};

    #[test]
    fn longest_word_fits_in_u128() {
        let max_bits_per_letter = Letter::ALPHABET_SIZE.ilog2() + 1;
        let max_bits = Word::MAX_WORD_LENGTH * (max_bits_per_letter as usize);
        assert!(max_bits <= 128)
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
            let word = Word::new(s, f);
            let letters_as_string = word
                .letters()
                .map(|r| Letter::to_string(&r))
                .collect::<Vec<String>>()
                .join("");
            assert_eq!(s, letters_as_string);
            assert_eq!(f, word.frequency().to_f32());
        }
    }

    #[test]
    fn try_from_tuple_when_invalid() {
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
            let word = Word::try_from((s, f));
            assert!(word.is_err());
        }
    }

    #[test]
    fn try_from_str_when_too_long() {
        assert!(Word::try_from("abcdefghijklmnopqrstuvwxy").is_ok());
        assert!(Word::try_from("abcdefghijklmnopqrstuvwxyz").is_err());
    }

    #[test]
    fn len_for_all_word_lengths() {
        for letter in Letter::ALPHABET {
            for size in 1..=Word::MAX_WORD_LENGTH {
                let word_string =
                    std::iter::repeat(letter)
                        .take(size)
                        .fold(String::new(), |mut total, i| {
                            total.push(i);
                            total
                        });
                let word = Word::new(&word_string, 0.0); // annoying to unwrap!
                assert_eq!(
                    word.len(),
                    size as u8,
                    "checking size of '{}' repeated {} times",
                    word_string,
                    size
                );
            }
        }
    }

    #[test]
    fn letters_count_is_correct() {
        for letter in Letter::ALPHABET {
            for size in 1..=Word::MAX_WORD_LENGTH {
                let word_string =
                    std::iter::repeat(letter)
                        .take(size)
                        .fold(String::new(), |mut total, i| {
                            total.push(i);
                            total
                        });
                let word = Word::new(&word_string, 0.0);
                let result = word.letters().count();
                assert_eq!(
                    result, size,
                    "checking length of '{}' repeated {} times",
                    word_string, size
                );
            }
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
    fn cmp_sorts_only_by_word() {
        let data = [
            ("a", "b", Ordering::Less),
            ("a", "bcd", Ordering::Less),
            ("abc", "abcd", Ordering::Less),
            ("a", "a", Ordering::Equal),
            ("abc", "abc", Ordering::Equal),
            ("aaaaa", "z", Ordering::Greater),
        ];
        for (a, b, ordering) in data {
            for freq_same in [false, true] {
                let a_word = Word::new(a, 0.2);
                let b_word = Word::new(b, if freq_same { 0.2 } else { 0.8 });
                assert_eq!(
                    a_word.cmp(&b_word),
                    ordering,
                    "comparing '{}' and '{}'",
                    a,
                    b
                );
                assert_eq!(
                    b_word.cmp(&a_word),
                    ordering.reverse(),
                    "comparing '{}' and '{}'",
                    a,
                    b
                );
            }
        }
    }

    #[test]
    fn hash_ignores_frequency() {
        let a = Word::new("abc", 0.3);
        let b = Word::new("abc", 0.8);
        let c = Word::new("c", 0.3);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(2, set.len());
    }

    #[test]
    fn overlap() {
        let data = [
            ("book", "beek", '_', Some("b__k")),
            ("bit", "bat", '_', Some("b_t")),
            ("moist", "moose", '_', Some("mo_s_")),
            ("book", "the", '_', None),
            ("a", "a", '_', Some("a")),
            ("the", "the", '_', Some("the")),
            ("the", "and", '_', Some("___")),
            ("the", "happy", '_', None),
        ];
        for (a, b, placeholder, expect) in data {
            let word_a = Word::new(a, 0.0);
            let word_b = Word::new(b, 0.0);
            let actual = word_a.overlap(&word_b, placeholder);
            let expected = expect.map(|s| s.to_string());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn replace_letters_keeps_same_frequency() {
        let source = Word::new("abc", 3.54);
        let result = source.replace_letters(Key::new("b").letters(), Letter::new('c'));
        assert_eq!(source.frequency(), result.frequency());
    }

    #[test]
    fn replace_letters() {
        let data = [
            ("abc", "ab", 'x', "xxc"),
            ("abc", "b", 'x', "axc"),
            ("abc", "abc", 'x', "xxx"),
            ("abc", "abcd", 'x', "xxx"),
            ("abc", "q", 'x', "abc"),
            ("abc", "", 'x', "abc"),
        ];
        for (source, find, replace_with, expected) in data {
            let source_word = Word::try_from(source).unwrap();
            let find_iter = find.chars().map(|r| Letter::new(r));
            let replace_with_letter = Letter::new(replace_with);
            let result = source_word.replace_letters(find_iter, replace_with_letter);
            assert_eq!(
                result.to_string(),
                expected.to_string(),
                "{},{},{}",
                source,
                find,
                replace_with
            )
        }
    }
}
