use std::iter;

pub fn factorial(n: u128) -> u128 {
    (1..=n).product()
}

/// Counts the number of ways to choose `k` from `n`.
pub fn choose(n: u32, k: u32) -> u128 {
    let n = n as u128;
    let k = k as u128;
    factorial(n) / factorial(n - k) / factorial(k)
}

/// Returns the sequence of numbers with `count` bits set, starting from the
/// smallest value. Based on this algorithm :
/// https://www.geeksforgeeks.org/next-higher-number-with-same-number-of-set-bits
pub fn same_set_bits(count: u32) -> impl Iterator<Item = u64> {
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

struct BitEnumerator {
    value: u32,
    current_bit_index: usize,
}

impl Iterator for BitEnumerator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value == 0 {
            None
        } else {
            while (self.value & 1) != 1 {
                self.value = self.value >> 1;
                self.current_bit_index = self.current_bit_index + 1;
            }
            self.value = self.value >> 1;
            self.current_bit_index = self.current_bit_index + 1;
            Some(self.current_bit_index - 1)
        }
    }
}

pub fn set_bits(n: u32) -> impl Iterator<Item = usize> {
    BitEnumerator {
        value: n,
        current_bit_index: 0,
    }
    .into_iter()
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
            (12, vec![2, 3]),
        ];
        for (n, expected) in data {
            let actual = set_bits(n).collect::<Vec<usize>>();
            assert_eq!(expected, actual);
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
            let expected_max = ((1 << bits) - 1) << (u32::BITS - bits);
            let actual_max = same_set_bits(bits).last().unwrap();
            assert_eq!(actual_max, expected_max);
        }
    }

    #[test]
    fn same_set_bits_returns_numbers_with_same_count_of_bits() {
        for bits in [1, 5, 9, 12, 32] {
            assert!(
                same_set_bits(bits).all(|n| n.count_ones() == bits),
                "Expected every number to have exactly {} bits.",
                bits
            );
        }
    }

    #[test]
    fn same_set_bits_returns_correct_count_of_results() {
        for bits in [1, 5, 26, 32] {
            let actual = same_set_bits(bits).count();
            let expected: usize = choose(32, bits).try_into().unwrap();
            assert_eq!(actual, expected);
        }
    }
}
