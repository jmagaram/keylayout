use std::{
    fmt::{self},
    iter, u32,
};

use crate::u6::U6;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Set32(u32);

impl Set32 {
    pub const EMPTY: Set32 = Set32(0);
    pub const MAX_SIZE: u32 = 32;
    pub const MAX_ITEM_VALUE: U6 = U6::MAX;
    pub const MIN_ITEM_VALUE: U6 = U6::MIN;

    pub fn fill(count: u32) -> Set32 {
        assert!(count <= Self::MAX_SIZE);
        let bits = match count {
            32 => !0,
            count => (1 << count) - 1,
        };
        Set32(bits)
    }

    pub fn add(&self, bit: U6) -> Set32 {
        assert!(bit <= Self::MAX_ITEM_VALUE);
        Set32(self.0 | 1 << bit.to_u32())
    }

    pub fn singleton(bit: U6) -> Set32 {
        assert!(bit <= Self::MAX_ITEM_VALUE);
        Set32(1 << bit.to_u32())
    }

    pub fn contains(&self, bit: U6) -> bool {
        assert!(bit <= Self::MAX_ITEM_VALUE);
        self.0 & (1 << bit.to_u32()) != 0
    }

    pub fn count_items(&self) -> u32 {
        self.into_iter().count() as u32 // fix!
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn difference(&self, other: Set32) -> Set32 {
        Set32(self.0 & !other.0)
    }

    pub fn remove(&self, bit: U6) -> Set32 {
        assert!(bit <= Self::MAX_ITEM_VALUE);
        Set32(self.0 & !(1 << bit.to_u32()))
    }

    pub fn union(&self, other: Set32) -> Set32 {
        Set32(self.0 | other.0)
    }

    pub fn intersect(&self, other: Set32) -> Set32 {
        Set32(self.0 & other.0)
    }

    pub fn max_item(&self) -> Option<U6> {
        match self.0.leading_zeros() {
            u32::BITS => None,
            n => Some((u32::BITS - n - 1).into()),
        }
    }

    pub fn min_item(&self) -> Option<U6> {
        match self.0.trailing_zeros() {
            32 => None,
            n => Some(n.into()),
        }
    }

    // https://www.geeksforgeeks.org/next-higher-number-with-same-number-of-set-bits
    fn same_ones_count(count: u32) -> impl Iterator<Item = u64> {
        assert!(count >= 1 && count <= 32);
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

    pub fn subsets_of_size(&self, size: u32) -> impl Iterator<Item = Set32> {
        assert!(size <= Self::MAX_SIZE, "subset size is too big");
        let items = self.into_iter().collect::<Vec<U6>>();
        let items_count = items.len();
        let max_exclusive = 1 << items_count;
        assert!(items_count >= size.try_into().unwrap()); // fix
        Set32::same_ones_count(size)
            .take_while(move |i| *i < max_exclusive)
            .map(move |i| {
                Set32(i as u32)
                    .into_iter()
                    .fold(Set32::EMPTY, |total, i| total.add(items[i.to_usize()]))
            })
    }
}

impl fmt::Display for Set32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self
            .into_iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let left = "{";
        let right = "}";
        write!(f, "{}{}{}", left, digits, right)
    }
}

impl Iterator for Set32 {
    type Item = U6;

    fn next(&mut self) -> Option<U6> {
        match self.0 {
            0 => None,
            _ => {
                let trailing_zeros = self.0.trailing_zeros();
                self.0 = self.0 ^ (1 << (trailing_zeros));
                Some(trailing_zeros.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    trait Int32Wrapper {
        fn add_(&self, n: u32) -> Set32;
        fn remove_(&self, n: u32) -> Set32;
        fn contains_(&self, n: u32) -> bool;
    }

    impl Int32Wrapper for Set32 {
        fn add_(&self, n: u32) -> Set32 {
            self.add(n.into())
        }

        fn remove_(&self, n: u32) -> Set32 {
            self.remove(n.into())
        }

        fn contains_(&self, n: u32) -> bool {
            self.contains(n.into())
        }
    }

    fn string_to_bits(s: &str) -> Set32 {
        let s = s.trim();
        if s == "" {
            Set32::EMPTY
        } else {
            s.split(",")
                .map(|i| {
                    i.trim()
                        .parse::<u32>()
                        .expect("could not convert the string to a u32")
                })
                .collect::<Vec<u32>>()
                .into_iter()
                .fold(Set32::EMPTY, |total, i| total.add_(i))
        }
    }

    #[test]
    #[ignore]
    fn display_trait() {
        let data = ["1,2,3", "1", "15,25,2", "0"];
        data.into_iter().for_each(|d| {
            let bit = string_to_bits(d);
            println!("The characters are {}", bit)
        })
    }

    #[test]
    fn into_iterator() {
        assert_eq!(Set32::EMPTY.into_iter().count(), 0);
        assert_eq!(Set32::EMPTY.add_(1).add_(2).into_iter().count(), 2);
        assert_eq!(
            Set32::EMPTY
                .add_(5)
                .add_(3)
                .into_iter()
                .fold(0u32, |total, i| total + i.to_u32()),
            8,
        );
    }

    #[test]
    fn fill_when_zero() {
        assert_eq!(Set32::fill(0), Set32::EMPTY);
    }

    #[test]
    fn fill_when_max() {
        assert_eq!(
            Set32::fill(32).to_string(),
            "{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31}"
        );
    }

    #[test]
    fn fill_when_not_zero() {
        assert_eq!(Set32::fill(1).to_string(), "{0}");
        assert_eq!(Set32::fill(2).to_string(), "{0,1}");
    }

    #[test]
    fn add() {
        let zero = Set32::EMPTY;
        assert_eq!(zero.to_string(), "{}");
        assert_eq!(zero.add_(3).add_(8).to_string(), "{3,8}");
        assert_eq!(zero.add_(31).to_string(), "{31}");
    }

    #[test]
    #[should_panic]
    fn add_panic_if_invalid_index() {
        Set32::EMPTY.add_(32);
    }

    #[test]
    fn difference() {
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
            let actual = start.difference(other);
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
        assert_eq!(Set32::EMPTY.contains_(3), false);
        assert_eq!(Set32::EMPTY.add_(1).contains_(2), false);
        assert_eq!(Set32::EMPTY.add_(1).add_(2).contains_(1), true);
    }

    #[test]
    fn count_items_test() {
        assert_eq!(Set32::EMPTY.count_items(), 0);
        assert_eq!(Set32::EMPTY.add_(1).count_items(), 1);
        assert_eq!(Set32::EMPTY.add_(1).add_(2).count_items(), 2);
        assert_eq!(Set32::fill(32).count_items(), 32);
    }

    #[test]
    fn singleton_test() {
        assert_eq!(Set32::singleton(0.into()).to_string(), "{0}");
        assert_eq!(Set32::singleton(31.into()).to_string(), "{31}");
        assert_eq!(Set32::singleton(5.into()).to_string(), "{5}");
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
    fn max_item() {
        assert_eq!(Set32::EMPTY.max_item(), None);
        assert_eq!(Set32::EMPTY.add_(0).max_item(), Some(0.into()));
        assert_eq!(Set32::EMPTY.add_(0).add_(1).max_item(), Some(1.into()));
        assert_eq!(Set32::EMPTY.add_(5).max_item(), Some(5.into()));
        assert_eq!(
            Set32::EMPTY.add_(5).add_(17).add_(3).max_item(),
            Some(17.into())
        );
        assert_eq!(Set32::EMPTY.add_(31).max_item(), Some(31.into()));
        assert_eq!(Set32::EMPTY.add_(31).add_(5).max_item(), Some(31.into()));
        assert_eq!(Set32::fill(32).max_item(), Some(31.into()));
    }

    #[test]
    fn min_item() {
        assert_eq!(Set32::EMPTY.min_item(), None);
        assert_eq!(Set32::EMPTY.add_(0).min_item(), Some(0.into()));
        assert_eq!(Set32::EMPTY.add_(0).add_(1).min_item(), Some(0.into()));
        assert_eq!(Set32::EMPTY.add_(5).min_item(), Some(5.into()));
        assert_eq!(
            Set32::EMPTY.add_(5).add_(17).add_(3).min_item(),
            Some(3.into())
        );
        assert_eq!(Set32::EMPTY.add_(31).min_item(), Some(31.into()));
    }

    #[test]
    fn same_ones_count_ends_at_max() {
        for expected_ones in [1, 5, 9, 12, 32] {
            let max_bits = 32;
            let expected_max = ((1 << expected_ones) - 1) << (max_bits - expected_ones);
            let actual_max = Set32::same_ones_count(expected_ones)
                .last()
                .unwrap_or(u64::MAX);
            assert_eq!(actual_max, expected_max);
        }
    }

    #[test]
    fn with_ones_count_test() {
        for expected_ones in [1, 5, 9, 12, 23, 31, 32] {
            let all_correct_ones = Set32::same_ones_count(expected_ones).take(1000).all(|n| {
                let actual_ones = Set32(n as u32).into_iter().count();
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
    fn subsets_of_max_size_panic() {
        let set = Set32::fill(2);
        let results = set.subsets_of_size(3);
    }

    #[test]
    fn subsets_of_max_size() {
        let set = Set32::fill(32);
        let results = set.subsets_of_size(32);
        assert_eq!(results.count(), 1);
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
                    let m = subset.difference(bits) == Set32::EMPTY;
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
        Set32::fill(6)
            .remove_(3)
            .subsets_of_size(3)
            .for_each(|i| println!("{:?}", i.to_string()));
    }
}
