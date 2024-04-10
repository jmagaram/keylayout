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
    use crate::letter::Letter;

    use super::*;
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
    fn try_from_str_when_valid() {
        for s in ["abc", "a", "abcdez\'", "xyz", ""] {
            let key: Key = s.try_into().unwrap();
            assert_eq!(key.to_string(), s.to_string());
        }
    }

    #[test]
    fn try_from_str_when_invalid() {
        for s in ["098", "a#4", "   "] {
            let key: Result<Key, _> = s.try_into();
            assert!(key.is_err());
        }
    }

    #[test]
    fn display() {
        let data = ["abc", "", "mnop'"];
        for s in data {
            let actual = Key::try_from(s).unwrap().to_string();
            assert_eq!(s.to_string(), actual);
        }
    }

    #[test]
    fn with_every_letter() {
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
    fn into_iterator() {
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
    fn add() {
        let zero = Key::EMPTY;
        assert_eq!(zero.to_string(), "{}");
        assert_eq!(zero.add_(3).add_(8).to_string(), "{3,8}");
        assert_eq!(zero.add_(31).to_string(), "{31}");
    }

    #[test]
    #[should_panic]
    fn add_panic_if_invalid_index() {
        Key::EMPTY.add_(32);
    }

    #[test]
    fn except() {
        let data = [
            ("", "", ""),
            ("1", "1", ""),
            ("1,2,3", "2,3", "1"),
            ("5,6,7", "1", "5,6,7"),
            ("23,1,8,3,4", "8,1", "3,4,23"),
        ];
        fn test(start: &str, except: &str, expected: &str) -> () {
            let start = string_to_bits(start);
            let other = string_to_bits(except);
            let actual = start.except(other);
            let expected = string_to_bits(expected);
            assert_eq!(actual, expected);
        }
        data.into_iter().for_each(|(a, b, c)| test(a, b, c));
    }

    #[test]
    fn remove_test() {
        fn test(bits: &str, item: u32, expected: &str) {
            assert_eq!(string_to_bits(bits).remove_(item).to_string(), expected);
        }
        test("", 0, "{}");
        test("", 1, "{}");
        test("", 31, "{}");
        test("0", 0, "{}");
        test("0", 1, "{0}");
        test("0,1,2", 1, "{0,2}");
        test("0,1,2,3,4,5", 5, "{0,1,2,3,4}");
        test("0,1,2,3,4,5,31", 31, "{0,1,2,3,4,5}");
        test("0,1,2,3,4,5,31", 0, "{1,2,3,4,5,31}");
    }

    #[test]
    fn to_string_test() {
        let data = [
            ("", "{}"),
            ("1", "{1}"),
            ("1,2,3", "{1,2,3}"),
            ("5,4,3", "{3,4,5}"),
            ("15,4,0", "{0,4,15}"),
        ];
        fn test(start: &str, expected: &str) -> () {
            let start = string_to_bits(start);
            let actual = start.to_string();
            assert_eq!(actual, expected);
        }
        data.into_iter()
            .for_each(|(start, expected)| test(start, expected));
    }

    #[test]
    fn contains() {
        assert_eq!(Key::EMPTY.contains_(3), false);
        assert_eq!(Key::EMPTY.add_(1).contains_(2), false);
        assert_eq!(Key::EMPTY.add_(1).add_(2).contains_(1), true);
    }

    #[test]
    fn count_items_test() {
        assert_eq!(Key::EMPTY.count_items(), 0);
        assert_eq!(Key::EMPTY.add_(1).count_items(), 1);
        assert_eq!(Key::EMPTY.add_(1).add_(2).count_items(), 2);
        todo!("add case for every letter")
    }

    #[test]
    fn with_one_letter_test() {
        assert_eq!(
            Key::with_one_letter(Letter::try_from(0).unwrap()).to_string(),
            "{0}"
        );
        assert_eq!(
            Key::with_one_letter(Letter::try_from(31).unwrap()).to_string(),
            "{31}"
        );
        assert_eq!(
            Key::with_one_letter(Letter::try_from(5).unwrap()).to_string(),
            "{5}"
        );
    }

    #[test]
    fn union() {
        let data = [
            ("", "", ""),
            ("1", "1", "1"),
            ("1,2,3", "2,3", "1,2,3"),
            ("5,6,7", "1,2,3", "1,2,3,5,6,7"),
            ("2", "1", "1,2"),
            ("", "5", "5"),
        ];
        fn test(start: &str, except: &str, expected: &str) -> () {
            let start = string_to_bits(start);
            let other = string_to_bits(except);
            let actual = start.union(other);
            let expected = string_to_bits(expected);
            assert_eq!(actual, expected);
        }
        data.into_iter().for_each(|(a, b, c)| test(a, b, c));
    }

    #[test]
    fn intersect() {
        let data = [
            ("", "", ""),
            ("1", "1", "1"),
            ("1,2,3", "2,3", "2,3"),
            ("5,6,7", "1,2,3", ""),
            ("1,2,3,4,5", "1,2,3,4,5", "1,2,3,4,5"),
            ("0,31", "5,31", "31"),
            ("2", "1,2", "2"),
            ("", "5", ""),
        ];
        fn test(start: &str, except: &str, expected: &str) -> () {
            let start = string_to_bits(start);
            let other = string_to_bits(except);
            let actual = start.intersect(other);
            let expected = string_to_bits(expected);
            assert_eq!(actual, expected);
        }
        data.into_iter().for_each(|(a, b, c)| test(a, b, c));
    }

    #[test]
    fn max_letter() {
        fn k(n: u32) -> Letter {
            Letter::try_from(n).unwrap()
        }
        assert_eq!(Key::EMPTY.max_letter(), None);
        assert_eq!(Key::EMPTY.add_(0).max_letter(), Some(k(0)));
        assert_eq!(Key::EMPTY.add_(0).add_(1).max_letter(), Some(k(1)));
        assert_eq!(Key::EMPTY.add_(5).max_letter(), Some(k(5)));
        assert_eq!(
            Key::EMPTY.add_(5).add_(17).add_(3).max_letter(),
            Some(k(17))
        );
        assert_eq!(Key::EMPTY.add_(31).max_letter(), Some(k(31)));
        assert_eq!(Key::EMPTY.add_(31).add_(5).max_letter(), Some(k(31)));
        todo!("with every letter")
    }

    #[test]
    fn min_letter() {
        fn k(n: u32) -> Letter {
            Letter::try_from(n).unwrap()
        }
        assert_eq!(Key::EMPTY.min_letter(), None);
        assert_eq!(Key::EMPTY.add_(0).min_letter(), Some(k(0)));
        assert_eq!(Key::EMPTY.add_(0).add_(1).min_letter(), Some(k(0)));
        assert_eq!(Key::EMPTY.add_(5).min_letter(), Some(k(5)));
        assert_eq!(Key::EMPTY.add_(5).add_(17).add_(3).min_letter(), Some(k(3)));
        assert_eq!(Key::EMPTY.add_(31).min_letter(), Some(k(31)));
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
    fn with_ones_count_test() {
        for expected_ones in [1, 5, 9, 12, 23, 31, 32] {
            let all_correct_ones = Key::same_ones_count(expected_ones).take(1000).all(|n| {
                let actual_ones = Key(n as u32).into_iter().count();
                actual_ones == (expected_ones as usize)
            });
            assert!(all_correct_ones)
        }
    }

    fn choose_count(n: u32, k: u32) -> u128 {
        let n = n as u128;
        let k = k as u128;
        fn factorial(n: u128) -> u128 {
            (1..=n).product()
        }
        factorial(n) / factorial(n - k) / factorial(k)
    }

    #[test]
    #[should_panic]
    fn subsets_of_size_bigger_than_alphabet_should_panic() {
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
                let expected_count = choose_count(ones_count as u32, subset_size as u32);
                assert_eq!(actual_size, expected_count as usize);
            });

            // subsets items are unique
            (1..ones_count).for_each(|subset_size| {
                let set = bits
                    .subsets_of_size(subset_size.into())
                    .map(|b| b.to_string())
                    .collect::<std::collections::HashSet<String>>();
                let expected_count = choose_count(ones_count as u32, subset_size as u32);
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
