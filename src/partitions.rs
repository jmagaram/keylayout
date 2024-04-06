pub struct Partitions {
    sum: u32,
    parts: u32,
    min: u32,
    max: u32,
}

impl Partitions {
    fn partitions(&self) -> Vec<Vec<u32>> {
        match self.sum == 0 {
            true => vec![vec![]],
            false => (self.min..=self.max)
                .into_iter()
                .filter_map(|n| {
                    match n + (self.min * self.parts - 1) <= self.sum && n * self.parts >= self.sum
                    {
                        true => Some(n),
                        false => None,
                    }
                })
                .flat_map(|digit| {
                    let solutions = Partitions {
                        sum: self.sum - digit,
                        parts: self.parts - 1,
                        min: self.min,
                        max: digit,
                    }
                    .partitions()
                    .into_iter()
                    .map(move |digits| {
                        let mut digits_copy = digits.clone();
                        digits_copy.push(digit);
                        digits_copy
                    });
                    solutions
                })
                .collect::<Vec<Vec<u32>>>(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
                .partitions()
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
                            sum: sum,
                            parts: parts,
                            min: min,
                            max: max,
                        }
                        .partitions()
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
    fn print() {
        let sum = 10;
        let parts = 3;
        let min = 1;
        let max = sum;
        Partitions {
            sum: sum,
            parts: parts,
            min: min,
            max: max,
        }
        .partitions()
        .into_iter()
        .for_each(|p| println!("{:?}", p))
    }
}
