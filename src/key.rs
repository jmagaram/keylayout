use std::{
    fmt::{self, Debug},
    iter,
};

use crate::letter::Letter;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Key(u32);

impl Key {
    pub const EMPTY: Key = Key(0);
    pub const MAX_SIZE: u32 = Letter::ALPHABET_SIZE as u32;

    pub fn with_every_letter() -> Key {
        Key((1 << Letter::ALPHABET_SIZE) - 1)
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

    // https://www.geeksforgeeks.org/next-higher-number-with-same-number-of-set-bits
    fn same_ones_count(count: u32) -> impl Iterator<Item = u64> {
        assert!(count >= 1 && count <= Key::MAX_SIZE);
        let mut n: u64 = (1 << count) - 1;
        let max_bits = u32::BITS;
        let expected_max = ((1 << count) - 1) << (max_bits - count);
        let next = move || {
            let result = n;
            let right_one = 1 << n.trailing_zeros();
            let next_higher_one_bit = n + right_one;
            let right_ones_pattern = n ^ next_higher_one_bit;
            let right_ones_pattern = right_ones_pattern / right_one;
            let right_ones_pattern = right_ones_pattern >> 2;
            n = next_higher_one_bit | right_ones_pattern;
            match result <= expected_max {
                true => Some(result),
                false => None,
            }
        };
        let iterator = iter::from_fn(next);
        iterator
    }

    pub fn subsets_of_size(&self, size: u32) -> impl Iterator<Item = Key> {
        assert!(size <= Key::MAX_SIZE, "subset size is too big");
        let items = self.into_iter().collect::<Vec<Letter>>();
        let items_count = items.len();
        let max_exclusive = 1 << items_count;
        assert!(items_count >= size.try_into().unwrap());
        Key::same_ones_count(size)
            .take_while(move |i| *i < max_exclusive)
            .map(move |i| {
                Key(i as u32)
                    .into_iter()
                    .fold(Key::EMPTY, |total, i| total.add(items[i.to_usize()]))
            })
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

    trait Int32Wrapper {
        fn add_(&self, n: u32) -> Key;
        fn remove_(&self, n: u32) -> Key;
        fn contains_(&self, n: u32) -> bool;
    }

    impl Int32Wrapper for Key {
        fn add_(&self, n: u32) -> Key {
            Letter::try_from(n).map(|r| self.add(r)).unwrap()
        }

        fn remove_(&self, n: u32) -> Key {
            Letter::try_from(n).map(|r| self.remove(r)).unwrap()
        }

        fn contains_(&self, n: u32) -> bool {
            Letter::try_from(n).map(|r| self.contains(r)).unwrap()
        }
    }

    fn string_to_bits(s: &str) -> Key {
        let s = s.trim();
        if s == "" {
            Key::EMPTY
        } else {
            s.split(",")
                .map(|i| {
                    i.trim()
                        .parse::<u32>()
                        .expect("could not convert the string to a u32")
                })
                .collect::<Vec<u32>>()
                .into_iter()
                .fold(Key::EMPTY, |total, i| total.add_(i))
        }
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
    fn same_ones_count_ends_at_max() {
        for expected_ones in [1, 5, 9, 12, 32] {
            let max_bits = Key::MAX_SIZE;
            let expected_max = ((1 << expected_ones) - 1) << (max_bits - expected_ones);
            let actual_max = Key::same_ones_count(expected_ones)
                .last()
                .unwrap_or(u64::MAX);
            assert_eq!(actual_max, expected_max);
        }
    }

    #[test]
    fn same_ones_count_has_correct_values() {
        for expected_ones in [1, 5, 9, 12, 23, 31, 32] {
            let all_correct_ones = Key::same_ones_count(expected_ones).take(1000).all(|n| {
                let actual_ones = Key(n as u32).into_iter().count();
                actual_ones == (expected_ones as usize)
            });
            assert!(all_correct_ones)
        }
    }

    #[test]
    #[should_panic]
    fn subsets_of_size_panic_if_bigger_than_alphabet_size() {
        let key = Key::with_every_letter();
        key.subsets_of_size(Key::MAX_SIZE).take(1).count();
    }

    #[test]
    fn subsets_of_size() {
        fn test(items: &str) {
            let bits = string_to_bits(items);
            let ones_count = bits.into_iter().count() as u32; // fix

            // subsets have correct number of items (no duplicates)
            (1..ones_count).for_each(|subset_size| {
                let actual_size = bits.subsets_of_size(subset_size).count();
                let expected_count = util::choose(ones_count as u32, subset_size as u32);
                assert_eq!(actual_size, expected_count as usize);
            });

            // subsets items are unique
            (1..ones_count).for_each(|subset_size| {
                let set = bits
                    .subsets_of_size(subset_size.into())
                    .map(|b| b.to_string())
                    .collect::<std::collections::HashSet<String>>();
                let expected_count = util::choose(ones_count as u32, subset_size as u32);
                assert_eq!(set.len(), expected_count as usize);
            });

            // subsets items are all in the source bits
            (1..ones_count).for_each(|subset_size| {
                let all_valid_items = bits.subsets_of_size(subset_size.into()).all(move |subset| {
                    let m = subset.except(bits) == Key::EMPTY;
                    m
                });
                assert!(all_valid_items)
            });
        }
        let data = [
            "0,1,5,7",
            "2,4,10,30",
            "1,2,3,4,5,6,7,30",
            "1,2,3,4,5,6,7,8,9,10,12,13,14",
            "6,1,8,7,2,9",
            "1",
            "1,2",
        ];
        data.iter().for_each(|s| {
            test(&s);
        })
    }

    #[test]
    #[ignore]
    fn subsets_print_out() {
        todo!("do for all letters in set")
        // Key::fill(6)
        //     .remove_(3)
        //     .subsets_of_size(3)
        //     .for_each(|i| println!("{:?}", i.to_string()));
    }
}
