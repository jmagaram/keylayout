use std::iter;

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

// pub trait Permutable<T>
// where
//     Self: Sized,
// {
//     fn is_empty(&self) -> bool;
//     fn parts(&self) -> Box<dyn Iterator<Item = (T, Self)>>;
//     fn permute(&self, first_time_called: bool) -> Box<dyn Iterator<Item = Vec<T>>>
//     where
//         T: Copy + Clone,
//     {
//         match (self.is_empty(), first_time_called) {
//             (true, true) => {
//                 let result = iter::empty();
//                 let result_boxed: Box<dyn Iterator<Item = Vec<T>>> = Box::new(result);
//                 result_boxed
//             }
//             (true, false) => {
//                 let result = iter::once(vec![]);
//                 let result_boxed: Box<dyn Iterator<Item = Vec<T>>> = Box::new(result);
//                 result_boxed
//             }
//             (false, _) => {
//                 let result = self.parts().into_iter().flat_map(|part| {
//                     let (item, rest) = part;
//                     let result = rest.permute(false).map(move |r| {
//                         let mut results_copy = r.to_owned();
//                         results_copy.push(item);
//                         results_copy
//                     });
//                     result
//                 });
//                 let result_boxed: Box<dyn Iterator<Item = Vec<T>>> = Box::new(result);
//                 result_boxed
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn permuter() {
    //     fn combos(items: Vec<i32>) -> Option<Box<dyn Iterator<Item = (i32, Vec<i32>)>>> {
    //         match items.len() {
    //             0 => None,
    //             len => {
    //                 let states = items.iter().enumerate().map(|(index, item)| {
    //                     let head = items[index];
    //                     let rest = if index == len - 1 {
    //                         vec![]
    //                     } else {
    //                         items[index + 1..].to_vec()
    //                     };
    //                     (head, rest)
    //                 });
    //                 Some(Box::new(states))
    //             }
    //         }
    //     }
    // }

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
