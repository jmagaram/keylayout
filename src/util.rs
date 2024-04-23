use std::iter;

/// Calculates the factorial of `n`
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
/// smallest value. This is useful for generating combinations of items from a
/// vec. For example, to calculate all unique combinations of 3 items,
/// generate the sequence of numbers with exactly 3 bits set, and then use those
/// bit positions to select items from the vec. Based on this algorithm :
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

/// Returns an iterator of the specific bit indexes set in the number `n`,
/// starting with the lowest significant bit.
pub fn set_bits(n: u32) -> impl Iterator<Item = usize> {
    (0..31)
        .take_while(move |b| 1 << b <= n)
        .filter_map(move |b| if 1 << b & n != 0 { Some(b) } else { None })
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
            (0b10101010101010, vec![1, 3, 5, 7, 9, 11, 13]),
            (
                0b1010101010101010101010101010,
                vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27],
            ),
        ];
        for (n, expected) in data {
            let actual = set_bits(n).collect::<Vec<usize>>();
            assert_eq!(expected, actual, "for number {}", n);
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
