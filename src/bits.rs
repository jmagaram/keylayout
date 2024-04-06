use std::{iter, u32};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Bits(u32);

impl Bits {
    const EMPTY: Bits = Bits(0);
    const MAX_BITS: u32 = 30; // using this or u32::BITS consistently?
    const MAX_BIT_VALUE: u32 = 31;

    pub fn set_lowest(count: u32) -> Bits {
        debug_assert!(count <= Self::MAX_BITS);
        Bits((1 << count) - 1)
    }

    pub fn set_bit(&self, bit: u32) -> Bits {
        debug_assert!(bit <= Self::MAX_BIT_VALUE);
        Bits(self.0 | 1 << bit)
    }

    pub fn except(&self, other: Bits) -> Bits {
        Bits(self.0 & !other.0)
    }

    pub fn except_bit(&self, bit: u32) -> Bits {
        debug_assert!(bit <= Self::MAX_BIT_VALUE);
        Bits(self.0 & !(1 << bit))
    }

    pub fn union(&self, other: Bits) -> Bits {
        Bits(self.0 | other.0)
    }

    pub fn highest_bit(&self) -> Option<u32> {
        match self.0.leading_zeros() {
            u32::BITS => None,
            n => Some(u32::BITS - n - 1),
        }
    }

    pub fn ones(&self) -> impl Iterator<Item = u32> {
        let mut current = self.0;
        let next = move || match current {
            0 => None,
            _ => {
                let trailing_zeros = current.trailing_zeros();
                current = current ^ (1 << (trailing_zeros));
                Some(trailing_zeros)
            }
        };
        let iterator = iter::from_fn(next);
        iterator
    }

    fn same_ones_count(count: u32) -> impl Iterator<Item = i64> {
        debug_assert!(count >= 1 && count <= 32);
        let mut n: i64 = (1 << count) - 1;
        let max_bits = u32::BITS;
        let expected_max = ((1 << count) - 1) << (max_bits - count);
        let next = move || {
            let result = n;
            let right_one = n & (-n);
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

    // https://www.geeksforgeeks.org/next-higher-number-with-same-number-of-set-bits
    fn subsets_of_size(&self, size: u32) -> impl Iterator<Item = Bits> {
        debug_assert!(size <= Self::MAX_BITS, "subset size is too big");
        debug_assert!(size > 0, "the size of subset must be bigger than 0");
        let items = self.ones().collect::<Vec<u32>>();
        let items_count = items.len();
        let max_exclusive = 1 << items_count;
        debug_assert!(items_count >= size as usize);
        let r = Bits::same_ones_count(size)
            .take_while(move |i| *i < max_exclusive)
            .map(move |i| {
                let res = Bits(i as u32).ones().fold(Bits::EMPTY, |total, i| {
                    let aa = items[i as usize];
                    total.set_bit(aa)
                });
                res
            });
        r
    }

    fn to_string(&self) -> String {
        let result = self
            .ones()
            .into_iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(",");
        format!("[{}]", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_to_bits(s: &str) -> Bits {
        let s = s.trim();
        if s == "" {
            Bits::EMPTY
        } else {
            s.split(",")
                .map(|i| {
                    i.trim()
                        .parse::<u32>()
                        .expect("could not convert the string to a u32")
                })
                .collect::<Vec<u32>>()
                .into_iter()
                .fold(Bits::EMPTY, |total, i| total.set_bit(i))
        }
    }

    fn assert_has_set_bits(target: &Bits, expected: Vec<u32>) -> () {
        let actual = target.ones().into_iter().collect::<Vec<u32>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn equals_operator() {
        assert_eq!(Bits::EMPTY == Bits::EMPTY, true);
    }

    #[test]
    fn not_equals_operator() {
        assert_eq!(Bits::EMPTY != Bits::EMPTY, false);
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(Bits::EMPTY < Bits::EMPTY, false);
    }

    #[test]
    fn set_lowest_when_zero() {
        assert_eq!(Bits::set_lowest(0), Bits::EMPTY);
    }

    #[test]
    fn set_lowest_when_not_zero() {
        for bit_count in 1u32..10u32 {
            let target = Bits::set_lowest(bit_count);
            let actual = target.ones().into_iter().collect::<Vec<u32>>();
            let expected: Vec<u32> = (0..=bit_count - 1).into_iter().collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn set_bit() {
        let zero = Bits::EMPTY;
        assert_has_set_bits(&zero, vec![]);
        assert_has_set_bits(&zero.set_bit(2), vec![2]);
        assert_has_set_bits(&zero.set_bit(2).set_bit(5), vec![2, 5]);
        assert_has_set_bits(&zero.set_bit(30).set_bit(8).set_bit(13), vec![8, 13, 30]);
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
    fn except_bit() {
        let data = [
            ("1", 1, ""),
            ("1,2", 2, "1"),
            ("1,2,10", 10, "1,2"),
            ("5,6,7", 1, "5,6,7"),
            ("", 8, ""),
        ];
        fn test(start: &str, bit: u32, expected: &str) -> () {
            let start = string_to_bits(start);
            let actual = start.except_bit(bit);
            let expected = string_to_bits(expected);
            assert_eq!(actual, expected);
        }
        data.into_iter().for_each(|(a, b, c)| test(a, b, c));
    }

    #[test]
    fn to_string_test() {
        let data = [
            ("", "[]"),
            ("1", "[1]"),
            ("1,2,3", "[1,2,3]"),
            ("5,4,3", "[3,4,5]"),
            ("15,4,0", "[0,4,15]"),
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
    fn highest_bit() {
        let data = [
            ("1,2,3", 3),
            ("0", 0),
            ("1", 1),
            ("2", 2),
            ("5,6,7", 7),
            ("23,1,8,3,4", 23),
            ("30", 30),
            ("", u32::MAX),
        ];
        fn test(start: &str, expected: u32) -> () {
            let start = string_to_bits(start);
            let actual = start.highest_bit();
            assert_eq!(
                actual,
                if expected == u32::MAX {
                    None
                } else {
                    Some(expected)
                }
            );
        }
        data.into_iter()
            .for_each(|(input, expected)| test(input, expected));
    }

    #[test]
    fn same_ones_count_ends_at_max() {
        for expected_ones in [1, 5, 9, 12, 32] {
            let max_bits = 32;
            let expected_max = ((1 << expected_ones) - 1) << (max_bits - expected_ones);
            let actual_max = Bits::same_ones_count(expected_ones).last().unwrap_or(-1);
            assert_eq!(actual_max, expected_max);
        }
    }

    #[test]
    fn with_ones_count_test() {
        for expected_ones in [1, 5, 9, 12, 23, 31, 32] {
            let all_correct_ones = Bits::same_ones_count(expected_ones).take(1000).all(|n| {
                let actual_ones = Bits(n as u32).ones().count();
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
    fn subsets_of_size() {
        fn test(items: &str) {
            let bits = string_to_bits(items);
            let ones_count = bits.ones().count();

            // subsets have correct number of items (no duplicates)
            (1..ones_count).for_each(|subset_size| {
                let actual_size = bits.subsets_of_size(subset_size as u32).count();
                let expected_count = choose_count(ones_count as u32, subset_size as u32);
                assert_eq!(actual_size, expected_count as usize);
            });

            // subsets items are unique
            (1..ones_count).for_each(|subset_size| {
                let set = bits
                    .subsets_of_size(subset_size as u32)
                    .map(|b| b.to_string())
                    .collect::<std::collections::HashSet<String>>();
                let expected_count = choose_count(ones_count as u32, subset_size as u32);
                assert_eq!(set.len(), expected_count as usize);
            });

            // subsets items are all in the source bits
            (1..ones_count).for_each(|subset_size| {
                let all_valid_items = bits.subsets_of_size(subset_size as u32).all(move |subset| {
                    let m = subset.except(bits) == Bits::EMPTY;
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
        Bits::set_lowest(6)
            .except_bit(3)
            .subsets_of_size(3)
            .for_each(|i| println!("{:?}", i.to_string()));
    }
}
