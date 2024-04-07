pub struct Partitions {
    pub sum: u32,
    pub parts: u32,
    pub min: u32,
    pub max: u32,
}

impl Partitions {
    pub fn calculate(&self) -> Vec<Vec<u32>> {
        debug_assert!(self.max >= self.min);
        debug_assert!(self.sum > 0);
        debug_assert!(self.parts > 0);
        fn go(sum: u32, parts: u32, min: u32, max: u32) -> Vec<Vec<u32>> {
            match sum == 0 {
                true => vec![vec![]],
                false => (min..=max)
                    .into_iter()
                    .filter_map(|n| match n + (min * parts - 1) <= sum && n * parts >= sum {
                        true => Some(n),
                        false => None,
                    })
                    .flat_map(|digit| {
                        let solutions =
                            go(sum - digit, parts - 1, min, digit)
                                .into_iter()
                                .map(move |digits| {
                                    let mut digits_copy = digits.to_vec();
                                    digits_copy.push(digit);
                                    digits_copy
                                });
                        solutions
                    })
                    .collect::<Vec<Vec<u32>>>(),
            }
        }
        go(self.sum, self.parts, self.min, self.max)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic]
    fn panic_if_max_less_than_min() {
        Partitions {
            sum: 100,
            parts: 1,
            min: 5,
            max: 4,
        }
        .calculate();
    }

    #[test]
    #[should_panic]
    fn panic_if_parts_is_zero() {
        Partitions {
            sum: 100,
            parts: 0,
            min: 1,
            max: 100,
        }
        .calculate();
    }

    #[test]
    #[should_panic]
    fn panic_if_sum_is_zero() {
        Partitions {
            sum: 0,
            parts: 1,
            min: 1,
            max: 10,
        }
        .calculate();
    }

    #[test]
    fn partitions_have_proper_count() {
        let data = [
            (1, 1, 1, 1, 1),
            (10, 3, 1, 10, 8),
            (27, 10, 1, 27, 267),
            (3, 2, 1, 3, 1),
            (4, 2, 1, 3, 2),
        ];
        for (sum, parts, min, max, expected) in data {
            assert_eq!(
                Partitions {
                    sum: sum,
                    parts: parts,
                    min: min,
                    max: max
                }
                .calculate()
                .len(),
                expected
            );
        }
    }

    #[test]
    fn partitions_total_to_sum() {
        for sum in 1..=20 {
            for parts in 1..=sum {
                for min in 1..=sum {
                    for max in min..=sum {
                        let all_correct_sum = Partitions {
                            sum,
                            parts,
                            min,
                            max,
                        }
                        .calculate()
                        .into_iter()
                        .map(|digits| digits.into_iter().fold(0, |total, i| total + i))
                        .all(|r| r == sum);
                        assert!(all_correct_sum)
                    }
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn print_a_sample() {
        let sum = 10;
        let parts = 3;
        let min = 1;
        let max = sum;
        Partitions {
            sum,
            parts,
            min,
            max,
        }
        .calculate()
        .into_iter()
        .for_each(|p| println!("{:?}", p))
    }
}
