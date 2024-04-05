#[derive(PartialEq, PartialOrd, Debug)]
pub struct Bits(u32);

use std::iter;

impl Bits {
    const EMPTY: Bits = Bits(0);
    const MAX_BITS: u32 = 30;
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

    // fn increment_with_same_ones_count(&self) -> Bits {
    //     match x {
    //         EMPTY => 0,
    //         _ => {
    //             let rightOne = x & (-x);
    //             let nextHigherOneBit = x + rightOne;
    //             let rightOnesPattern = x ^ nextHigherOneBit;
    //             let rightOnesPattern = rightOnesPattern / rightOne;
    //             let rightOnesPattern = rightOnesPattern >> 2;
    //             let next = nextHigherOneBit | rightOnesPattern;
    //             next
    //         }
    //     }
    // }
    // pub fn subsets_of_size(size: u32) -> Iterator<Item = BitSet> {
    //     debug_assert!(size < usize::BITS);
    //     match size {
    //         0 => BitSet(0),
    //         size => BitSet(1 << size & usize::MAX),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits_to_string(b: &Bits) -> String {
        b.ones()
            .into_iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }

    fn string_to_vec(s: String) -> Vec<u32> {
        s.split(",")
            .map(|i| {
                i.parse::<u32>()
                    .expect("could not convert the string to a u32")
            })
            .collect::<Vec<u32>>()
    }

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
}

// let empty: t
// let isEmpty: t => bool
// let fromInt: int => t
// let every: size => t
// let only: index => t
// let union: (t, t) => t
// let remove: (t, t) => t
// let removeBiggest: t => option<(t, t)>
// let combinations: (t, size) => Seq.t<t>
// let toString: t => string
// let compare: (t, t) => float
// let ones: t => Seq.t<index>

// EMPTY
// MAX_BITS
// MAX_BIT_VALUE

// set_lowest
// set_bit
// except
// except_bit
// union
// highest_bit
// ones
//
// increment same bit count
