use std::{
    fmt::{self, Debug},
    iter,
};

use crate::{letter::Letter, util};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Key(u32);

impl Key {
    pub const EMPTY: Key = Key(0);
    pub const MAX_SIZE: u32 = Letter::ALPHABET_SIZE as u32;

    pub fn with_every_letter() -> Key {
        Key((1 << Letter::ALPHABET_SIZE) - 1)
    }

    pub fn with_first_n_letters(count: u32) -> Key {
        if (count as usize) > Letter::ALPHABET_SIZE {
            panic!(
                "A Key can have at most {} letters, but you asked for {}.",
                Letter::ALPHABET_SIZE,
                count,
            );
        }
        Key((1 << count) - 1)
    }

    pub fn with_one_letter(r: Letter) -> Key {
        Key(1 << r.to_u8())
    }

    pub fn add(&self, r: Letter) -> Key {
        Key(self.0 | 1 << r.to_u8())
    }

    pub fn remove(&self, r: Letter) -> Key {
        Key(self.0 & !(1 << r.to_u8()))
    }

    pub fn contains(&self, r: Letter) -> bool {
        self.0 & (1 << r.to_u8()) != 0
    }

    pub fn count_items(&self) -> u32 {
        self.into_iter().count() as u32 // fix!
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn except(&self, other: Key) -> Key {
        Key(self.0 & !other.0)
    }

    pub fn union(&self, other: Key) -> Key {
        Key(self.0 | other.0)
    }

    pub fn intersect(&self, other: Key) -> Key {
        Key(self.0 & other.0)
    }

    pub fn max_letter(&self) -> Option<Letter> {
        match self.0.leading_zeros() {
            u32::BITS => None,
            n => Letter::try_from(u32::BITS - n - 1).ok(),
        }
    }

    pub fn min_letter(&self) -> Option<Letter> {
        match self.0.trailing_zeros() {
            32 => None,
            n => Letter::try_from(n).ok(),
        }
    }

    pub fn subsets_of_size(&self, size: u32) -> impl Iterator<Item = Key> {
        assert!(
            size > 0 && size <= Key::MAX_SIZE,
            "Expected the subset size to be 1..={} (the maximum letters in the alphabet.",
            Key::MAX_SIZE
        );
        let letters = self.into_iter().collect::<Vec<Letter>>();
        let letters_count = letters.len();
        let max_exclusive = 1 << letters_count;
        assert!(
            letters_count >= size.try_into().unwrap(),
            "Expected the subset size ({}) to be <= the number of letters on the key ({}).",
            size,
            letters_count
        );
        util::same_set_bits(size)
            .take_while(move |i| *i < max_exclusive)
            .map(move |i| {
                Key(i as u32)
                    .into_iter()
                    .fold(Key::EMPTY, |total, i| total.add(letters[i.to_usize()]))
            })
    }
}

impl FromIterator<Letter> for Key {
    fn from_iter<I: IntoIterator<Item = Letter>>(iter: I) -> Self {
        let mut c = Key::EMPTY;
        for i in iter {
            c = c.add(i);
        }
        c
    }
}

impl TryFrom<&str> for Key {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .chars()
            .map(Letter::try_from)
            .try_fold(Key::EMPTY, |total, r| r.map(|letter| total.add(letter)))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        for c in self.into_iter() {
            result.push(c.to_char());
        }
        write!(f, "{}", result)
    }
}

// iter not into_iter?
impl Iterator for Key {
    type Item = Letter;

    fn next(&mut self) -> Option<Letter> {
        match self.0 {
            0 => None,
            _ => {
                let trailing_zeros = self.0.trailing_zeros();
                self.0 = self.0 ^ (1 << (trailing_zeros));
                Letter::try_from(trailing_zeros).ok()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{letter::Letter, util};

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
    fn dd() -> Letter {
        Letter::try_from('d').unwrap()
    }

    #[test]
    fn try_from_str_when_valid_test() {
        for s in ["abc", "a", "abcdez\'", "xyz", ""] {
            let key: Key = s.try_into().unwrap();
            assert_eq!(key.to_string(), s.to_string());
        }
    }

    #[test]
    fn try_from_str_when_invalid_test() {
        for s in ["098", "a#4", "   "] {
            let key: Result<Key, _> = s.try_into();
            assert!(key.is_err());
        }
    }

    #[test]
    fn to_string_test() {
        let data = ["abc", "", "mnop'"];
        for s in data {
            let actual = Key::try_from(s).unwrap().to_string();
            assert_eq!(s.to_string(), actual);
        }
    }

    #[test]
    fn with_every_letter_test() {
        let target = Key::with_every_letter();
        assert_eq!(
            target.to_string(),
            "abcdefghijklmnopqrstuvwxyz'".to_string()
        );
    }

    #[test]
    fn with_first_n_letters_test() {
        let data = [(0, ""), (1, "a"), (27, "abcdefghijklmnopqrstuvwxyz'")];
        for (count, expected) in data {
            let target = Key::with_first_n_letters(count);
            assert_eq!(expected, target.to_string())
        }
    }

    #[test]
    #[should_panic]
    fn with_first_n_letters_panic_if_more_than_alphabet() {
        Key::with_first_n_letters(28);
    }

    #[test]
    fn max_letters_is_alphabet_size() {
        assert_eq!(Letter::ALPHABET_SIZE, Key::MAX_SIZE as usize);
    }

    #[test]
    fn into_iterator_test() {
        let data = ["", "abc", "a", "xyz", "az'"];
        for d in data {
            let k: Vec<String> = Key::try_from(d)
                .unwrap()
                .into_iter()
                .map(|r| r.to_string())
                .collect();
            let result = k.join("");
            assert_eq!(d, result);
        }
    }

    #[test]
    fn add_test() {
        assert_eq!(
            Key::EMPTY
                .add(aa())
                .add(bb())
                .add(cc())
                .add(dd())
                .to_string(),
            "abcd"
        );
    }

    #[test]
    fn except_test() {
        let data = [
            ("", "", ""),
            ("a", "a", ""),
            ("abc", "bc", "a"),
            ("abc", "x", "abc"),
            ("abcdefg", "afg", "bcde"),
            ("", "abc", ""),
            ("a", "abc", ""),
        ];
        for (start, other, expected) in data {
            let start = Key::try_from(start).unwrap();
            let except = Key::try_from(other).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.except(except), expected);
        }
    }

    #[test]
    fn remove_test() {
        let data = [
            ("", 'a', ""),
            ("a", 'a', ""),
            ("abc", 'b', "ac"),
            ("abc", 'x', "abc"),
            ("abcdefg", 'e', "abcdfg"),
            ("", 'a', ""),
        ];
        for (start, to_remove, expected) in data {
            let start = Key::try_from(start).unwrap();
            let except = Letter::try_from(to_remove).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.remove(except), expected);
        }
    }

    #[test]
    fn contains_test() {
        let data = [
            ("", 'a', false),
            ("a", 'a', true),
            ("a", 'b', false),
            ("abcd", 'b', true),
            ("abcd", 'x', false),
        ];
        for (start, find, expected) in data {
            let start = Key::try_from(start).unwrap();
            let other = Letter::try_from(find).unwrap();
            assert_eq!(start.contains(other), expected);
        }
    }

    #[test]
    fn count_items_test() {
        let data = [(""), ("a"), ("abcde"), ("abcdefghijklmnopqrstuvwxyz'")];
        for start in data {
            let start = Key::try_from(start).unwrap();
            assert_eq!(start.count_items() as usize, start.to_string().len());
        }
    }

    #[test]
    fn with_one_letter_test() {
        assert_eq!("a", Key::with_one_letter(aa()).to_string());
        assert_eq!("b", Key::with_one_letter(bb()).to_string());
        assert_eq!("c", Key::with_one_letter(cc()).to_string());
    }

    #[test]
    fn union_test() {
        let data = [
            ("", "", ""),
            ("a", "a", "a"),
            ("a", "", "a"),
            ("abc", "x", "abcx"),
            ("abcdefg", "afg", "abcdefg"),
            ("abc", "xyz", "abcxyz"),
        ];
        for (start, other, expected) in data {
            let start = Key::try_from(start).unwrap();
            let other = Key::try_from(other).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.union(other), expected);
        }
    }

    #[test]
    fn intersect_test() {
        let data = [
            ("", "", ""),
            ("a", "a", "a"),
            ("a", "", ""),
            ("abc", "x", ""),
            ("abcdefg", "afg", "afg"),
            ("abc", "xyz", ""),
            ("abcd", "cdef", "cd"),
        ];
        for (start, other, expected) in data {
            let start = Key::try_from(start).unwrap();
            let other = Key::try_from(other).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.intersect(other), expected);
        }
    }

    #[test]
    fn max_letter_test() {
        let data = [("a", 'a'), ("abc", 'c'), ("cba", 'c')];
        for (start, expected) in data {
            let start = Key::try_from(start).unwrap();
            let expected = Letter::try_from(expected).unwrap();
            assert_eq!(start.max_letter(), Some(expected));
        }
        assert!(Key::EMPTY.max_letter().is_none());
    }

    #[test]
    fn min_letter_test() {
        let data = [("a", 'a'), ("abc", 'a'), ("cba", 'a'), ("xyfwfg", 'f')];
        for (start, expected) in data {
            let start = Key::try_from(start).unwrap();
            let expected = Letter::try_from(expected).unwrap();
            assert_eq!(start.min_letter(), Some(expected));
        }
        assert!(Key::EMPTY.min_letter().is_none());
    }

    #[test]
    #[should_panic]
    fn subsets_of_size_panic_if_bigger_than_alphabet_size() {
        let key = Key::with_every_letter();
        key.subsets_of_size(Key::MAX_SIZE + 1).take(1).count();
    }

    #[test]
    #[should_panic]
    fn subsets_of_size_panic_if_zero() {
        let key = Key::with_every_letter();
        key.subsets_of_size(0).take(1).count();
    }

    #[test]
    fn subsets_of_size_test() {
        fn test(items: &str) {
            let key = Key::try_from(items).unwrap();
            let ones_count = key.into_iter().count() as u32; // fix

            // subsets have correct number of items (no duplicates)
            (1..ones_count).for_each(|subset_size| {
                let actual_size = key.subsets_of_size(subset_size).count();
                let expected_count = util::choose(ones_count as u32, subset_size as u32);
                assert_eq!(actual_size, expected_count as usize);
            });

            // subsets items are unique
            (1..ones_count).for_each(|subset_size| {
                let set = key
                    .subsets_of_size(subset_size.into())
                    .map(|b| b.to_string())
                    .collect::<std::collections::HashSet<String>>();
                let expected_count = util::choose(ones_count as u32, subset_size as u32);
                assert_eq!(set.len(), expected_count as usize);
            });

            // subsets items are all in the source bits
            (1..ones_count).for_each(|subset_size| {
                let all_valid_items = key.subsets_of_size(subset_size.into()).all(move |subset| {
                    let m = subset.except(key) == Key::EMPTY;
                    m
                });
                assert!(all_valid_items)
            });
        }

        let data = ["abdfj", "abcdefghmpq", "apqrx", "cdexyz", "", "f"];
        for s in data {
            test(&s);
        }
    }

    #[test]
    fn from_iterator_of_letter() {
        let result = Key::from_iter(
            ['a', 'b', 'c', 'd', 'e', 'f']
                .iter()
                .map(|c| Letter::try_from(*c).unwrap()),
        );
        assert_eq!("abcdef", result.to_string());
    }
}
