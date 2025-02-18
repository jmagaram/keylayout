use std::{iter, time::Duration};

use humantime::{format_duration, FormattedDuration};

pub trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

/// Calculates the factorial of `n`
pub fn factorial(n: u128) -> u128 {
    let mut result = 1;
    for i in 1..=n {
        result = result * i;
    }
    result
}

/// Counts the number of ways to choose `k` from `n`.
pub fn choose(n: u32, k: u32) -> u128 {
    let n = n as u128;
    let k = k as u128;
    let temp = ((n - k + 1)..=n).fold(1u128, |total, i| total * i);
    temp / factorial(k)
}

/// Returns the sequence of numbers with `count` bits set, starting from the
/// smallest value. This is useful for generating combinations of items from a
/// vec. For example, to calculate all unique combinations of 3 items,
/// generate the sequence of numbers with exactly 3 bits set, and then use those
/// bit positions to select items from the vec. Based on this algorithm :
/// https://www.geeksforgeeks.org/next-higher-number-with-same-number-of-set-bits
pub fn same_set_bits(count: u8) -> impl Iterator<Item = u64> {
    let count: u32 = count as u32;
    assert!(
        count >= 1 && count <= u32::BITS,
        "Expected the bit count to be {}..={}.",
        1,
        u32::BITS
    );
    let mut n: u64 = (1 << count) - 1;
    let expected_max = ((1 << count) - 1) << (u32::BITS - count);
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

/// Returns an iterator of the specific bit indexes set in the number `n`,
/// starting with the least significant bit.
pub fn set_bits(n: u32) -> impl Iterator<Item = usize> {
    (0..32)
        .take_while(move |b| 1 << b <= n)
        .filter_map(move |b| if 1 << b & n != 0 { Some(b) } else { None })
}

/// Returns an iterator of the specific bit indexes set in the number `n`,
/// starting with the most significant bit.
pub fn set_bits_decreasing(n: u32) -> impl Iterator<Item = usize> {
    set_bits(n.reverse_bits()).map(|b| 31 - b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_bits_test() {
        let data = [
            (127, vec![0, 1, 2, 3, 4, 5, 6]),
            (0, vec![]),
            (1, vec![0]),
            (3, vec![0, 1]),
            (8, vec![3]),
            (2, vec![1]),
            (12, vec![2, 3]),
            (
                u32::MAX,
                vec![
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                ],
            ),
            (0b10101010101010, vec![1, 3, 5, 7, 9, 11, 13]),
        ];
        for (n, expected_low_to_high) in data {
            let low_to_high = set_bits(n).collect::<Vec<usize>>();
            assert_eq!(expected_low_to_high, low_to_high, "for number {}", n);

            let mut high_to_low = set_bits_decreasing(n).collect::<Vec<usize>>();
            high_to_low.reverse();
            assert_eq!(expected_low_to_high, high_to_low);
        }
    }

    #[test]
    fn factorial_test() {
        let data = [(5, 120), (4, 24), (3, 6), (2, 2), (1, 1), (0, 1)];
        for (n, expected) in data {
            assert_eq!(factorial(n), expected);
        }
    }

    #[test]
    fn choose_test() {
        let data = [
            (20, 4, 4845),
            (1, 1, 1),
            (1, 0, 1),
            (23, 4, 8855),
            (6, 3, 20),
            (18, 7, 31824),
        ];
        for (n, k, expected) in data {
            assert_eq!(choose(n, k), expected);
        }
    }

    #[test]
    #[should_panic]
    fn same_set_bits_panic_if_count_more_than_32() {
        same_set_bits(33).count();
    }

    #[test]
    #[should_panic]
    fn same_set_bits_panic_if_count_is_0() {
        #[allow(unused)]
        same_set_bits(0).count();
    }

    #[test]
    fn same_set_bits_ends_with_top_bits_filled() {
        for bits in [1, 5, 9, 24, 32] {
            let expected_max = ((1 << bits) - 1) << (u32::BITS - bits as u32);
            let actual_max = same_set_bits(bits).last().unwrap();
            assert_eq!(actual_max, expected_max);
        }
    }

    #[test]
    fn same_set_bits_returns_numbers_with_same_count_of_bits() {
        for bits in [1, 5, 9, 12, 32] {
            assert!(
                same_set_bits(bits).all(|n| n.count_ones() == bits as u32),
                "Expected every number to have exactly {} bits.",
                bits
            );
        }
    }

    #[test]
    fn same_set_bits_returns_correct_count_of_results() {
        for bits in [1, 5, 26, 32] {
            let actual = same_set_bits(bits).count();
            let expected: usize = choose(32, bits as u32).try_into().unwrap();
            assert_eq!(actual, expected);
        }
    }
}
