use std::iter;
use std::{collections::HashMap, hash::Hash};

pub fn choose(n: u32, k: u32) -> u128 {
    let n = n as u128;
    let k = k as u128;
    fn factorial(n: u128) -> u128 {
        (1..=n).product()
    }
    factorial(n) / factorial(n - k) / factorial(k)
}

// https://www.geeksforgeeks.org/next-higher-number-with-same-number-of-set-bits
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

pub fn permute_by_frequency<T>(items: HashMap<T, u32>) -> Vec<Vec<T>>
where
    T: Clone + Hash + Eq,
{
    debug_assert!(
        items.values().all(|i| *i > 0),
        "Every item is expected to have a frequency of 1 or more."
    );
    if items.len() == 0 {
        vec![vec![]]
    } else {
        items
            .iter()
            .flat_map(|pair| {
                let (item, count) = pair;
                let mut items = items.clone();
                if *count == 1 {
                    items.remove(&item);
                } else {
                    items.insert(item.clone(), count - 1);
                }
                permute_by_frequency(items).into_iter().map(|mut p| {
                    p.push(item.clone());
                    p
                })
            })
            .collect::<Vec<Vec<T>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod permute_by_frequency_tests {
        use std::collections::{HashMap, HashSet};

        use crate::util::permute_by_frequency;

        fn make_map(a: u32, b: u32, c: u32) -> HashMap<String, u32> {
            let mut map = HashMap::new();
            if a > 0 {
                map.insert("A".to_string(), a);
            };
            if b > 0 {
                map.insert("B".to_string(), b);
            };
            if c > 0 {
                map.insert("C".to_string(), c);
            };
            map
        }

        #[test]
        #[ignore]
        fn display_permutations() {
            let data = [(2, 1, 0)];
            for (a, b, c) in data {
                let m = make_map(a, b, c);
                let ff = permute_by_frequency(m);
                let q = ff.iter().map(|c| c.join(","));
                println!("");
                for i in q {
                    println!("{}", i)
                }
            }
        }

        #[test]
        fn count_is_correct() {
            let data = [
                (1, 1, 1, 6),
                (4, 3, 2, 1260),
                (3, 3, 2, 560),
                (7, 1, 4, 3960),
                (1, 0, 0, 1),
                (1, 1, 0, 2),
            ];
            for (a, b, c, expected) in data {
                let map = make_map(a, b, c);
                let f = permute_by_frequency(map);

                // total count is correct
                assert_eq!(expected, f.len());

                // each is unique
                let unique = f.iter().map(|x| x.join(",")).collect::<HashSet<String>>();
                assert_eq!(unique.len(), f.len());
            }
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
        for bits in [1, 5, 9, 12, 24, 32] {
            let expected_max = ((1 << bits) - 1) << (u32::BITS - bits);
            let actual_max = same_set_bits(bits).last().unwrap();
            assert_eq!(actual_max, expected_max);
        }
    }

    #[test]
    fn same_set_bits_returns_numbers_with_same_count_of_bits() {
        for bits in [1, 5, 9, 12, 23, 26, 32] {
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
