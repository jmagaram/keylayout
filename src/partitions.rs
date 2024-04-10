use crate::permutable::Permutable;

pub struct Partitions {
    pub sum: u32,
    pub parts: u32,
    pub min: u32,
    pub max: u32,
}

impl Permutable<u32> for Partitions {
    fn is_empty(&self) -> bool {
        self.sum == 0
    }

    fn parts(&self) -> Vec<(u32, Self)> {
        (self.min..=self.max)
            .into_iter()
            .filter_map(|n| {
                match n + (self.min * self.parts - 1) <= self.sum && n * self.parts >= self.sum {
                    true => Some(n),
                    false => None,
                }
            })
            .map(|digit| {
                (
                    digit,
                    Partitions {
                        sum: self.sum - digit,
                        parts: self.parts - 1,
                        min: self.min,
                        max: digit,
                    },
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn partitions_case() {
        let p = Partitions {
            sum: 27,
            parts: 2,
            min: 3,
            max: 20,
        }
        .permute();
        assert!(p.len() > 4);
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
                .permute()
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
                        .permute()
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
        .permute()
        .into_iter()
        .for_each(|p| println!("{:?}", p))
    }
}
