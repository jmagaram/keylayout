use crate::tally::{KeyLayout, Tally};

#[derive(Debug, Clone)]
pub struct Partitions {
    pub sum: u32,
    pub parts: u32,
    pub min: u32,
    pub max: u32,
}

impl Partitions {
    fn go(
        sum: u32,
        remaining_parts: u32,
        max: u32,
        min_value: u32,
        max_value: u32,
    ) -> Vec<Vec<u32>> {
        if sum == 0 && remaining_parts == 0 {
            return vec![vec![]];
        }
        if sum == 0 || remaining_parts == 0 {
            return vec![];
        }
        let mut result = Vec::new();
        for i in min_value..=max.min(sum).min(max_value) {
            let sub_partitions =
                Partitions::go(sum - i, remaining_parts - 1, i, min_value, max_value);
            for mut sub_partition in sub_partitions {
                sub_partition.push(i);
                result.push(sub_partition);
            }
        }
        result
    }

    pub fn total_unique_keyboards(&self) -> u128 {
        self.calculate()
            .iter()
            .map(|groups| {
                let as_key_sizes = groups.iter().map(|key_size| *key_size as u8);
                Tally::from_iter(as_key_sizes).unique_keyboards()
            })
            .sum()
    }

    pub fn calculate(&self) -> Vec<Vec<u32>> {
        Partitions::go(self.sum, self.parts, self.sum, self.min, self.max)
    }

    fn has_solution(&self) -> bool {
        self.min * self.parts <= self.sum && self.max * self.parts >= self.sum
    }

    pub fn calculate_tree<'a>(self) -> Box<dyn Iterator<Item = (u32, Partitions)> + 'a> {
        assert!(self.has_solution(), "Can not partition the sum.");
        if self.sum == 0 && self.parts == 0 {
            let result = std::iter::empty::<(u32, Partitions)>();
            let boxed_result: Box<dyn Iterator<Item = (u32, Partitions)>> = Box::new(result);
            boxed_result
        } else if self.sum > 0 && self.parts == 1 {
            let result = std::iter::once::<(u32, Partitions)>((
                self.sum,
                Partitions {
                    sum: 0,
                    parts: 0,
                    min: 0,
                    max: 0,
                },
            ));
            let boxed_result: Box<dyn Iterator<Item = (u32, Partitions)>> = Box::new(result);
            boxed_result
        } else {
            let result = (self.min..=self.max)
                .filter(move |i| {
                    if *i > self.sum {
                        false
                    } else {
                        let next = Partitions {
                            sum: self.sum - i,
                            min: self.min,
                            max: self.max,
                            parts: self.parts - 1,
                        };
                        next.has_solution()
                    }
                    // *i * self.parts <= self.sum && *i + self.max * (self.parts - 1) >= self.sum
                })
                .map(move |i| {
                    (
                        i,
                        Partitions {
                            sum: self.sum - i,
                            parts: self.parts - 1,
                            min: self.min,
                            // min: self.min.max(i),
                            max: self.max,
                        },
                    )
                });
            let boxed_result: Box<dyn Iterator<Item = (u32, Partitions)>> = Box::new(result);
            boxed_result
        }
    }

    pub fn print_recursive(&self, depth: usize) {
        let indent = std::iter::repeat("...").take(depth).collect::<String>();
        self.clone().calculate_tree().for_each(|(item, rest)| {
            println!("{}{}", indent, item);
            rest.print_recursive(depth + 1);
        });
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
            (13, 1, 3, 14, 1),
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
    fn total_unique_keyboards() {
        // 5 letters in 2 groups
        // Only groups are 2,3 and 1,4
        let p = Partitions {
            sum: 5,
            min: 1,
            max: 5,
            parts: 2,
        };
        assert_eq!(p.total_unique_keyboards(), 15);
    }

    #[test]
    #[ignore]
    fn print_recursive() {
        let sum = 3;
        let parts = 2;
        let min = 1;
        let max = 3;
        let partitions = Partitions {
            sum,
            parts,
            min,
            max,
        };
        partitions.print_recursive(0);
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
