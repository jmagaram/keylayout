#[derive(Debug, Clone)]
pub struct Partitions {
    pub sum: u32,
    pub parts: u32,
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, Copy)]
pub enum Goal {
    Combinations,
    Permutations,
}

impl Partitions {
    fn empty() -> Partitions {
        Partitions {
            sum: 0,
            min: 0,
            max: 0,
            parts: 0,
        }
    }

    fn has_solution(&self) -> bool {
        self.min * self.parts <= self.sum && self.max * self.parts >= self.sum
    }

    pub fn flatten(&self, goal: Goal) -> Box<dyn Iterator<Item = Vec<u32>>> {
        if self.sum == 0 {
            let result = std::iter::empty();
            let boxed_result: Box<dyn Iterator<Item = Vec<u32>>> = Box::new(result);
            boxed_result
        } else if self.parts == 1 {
            let result = std::iter::once(vec![self.sum]);
            let boxed_result: Box<dyn Iterator<Item = Vec<u32>>> = Box::new(result);
            boxed_result
        } else {
            let results = self.clone().iterate(goal).flat_map(move |(n, rest)| {
                rest.flatten(goal.clone()).map(move |r| {
                    let mut w = r.clone();
                    w.push(n);
                    w
                })
            });
            let boxed_result: Box<dyn Iterator<Item = Vec<u32>>> = Box::new(results);
            boxed_result
        }
    }

    pub fn total_unique_keyboards(&self) -> u128 {
        use crate::tally::*;
        self.flatten(Goal::Combinations)
            .map(|x| x.iter().map(|n| *n as u8).collect::<Vec<u8>>())
            .map(|x| Tally::from_iter(x.into_iter()).unique_keyboards())
            .sum()
    }

    fn subtract(&self, n: u32, goal: Goal) -> Option<Partitions> {
        if n >= self.sum {
            None
        } else {
            let result = Partitions {
                sum: self.sum - n,
                min: match goal {
                    Goal::Combinations => self.min.max(n),
                    Goal::Permutations => self.min,
                },
                max: self.max,
                parts: self.parts - 1,
            };
            match result.has_solution() {
                true => Some(result),
                false => None,
            }
        }
    }

    fn iterate<'a>(self, goal: Goal) -> Box<dyn Iterator<Item = (u32, Partitions)> + 'a> {
        assert!(self.has_solution(), "Can't partition the sum.");
        if self.sum == 0 && self.parts == 0 {
            let result = std::iter::empty::<(u32, Partitions)>();
            let boxed_result: Box<dyn Iterator<Item = (u32, Partitions)>> = Box::new(result);
            boxed_result
        } else if self.sum > 0 && self.parts == 1 {
            let result = std::iter::once::<(u32, Partitions)>((self.sum, Self::empty()));
            let boxed_result: Box<dyn Iterator<Item = (u32, Partitions)>> = Box::new(result);
            boxed_result
        } else {
            let result = (self.min..=self.max)
                .filter_map(move |i| self.subtract(i, goal.clone()).map(|p| (i, p)));
            let boxed_result: Box<dyn Iterator<Item = (u32, Partitions)>> = Box::new(result);
            boxed_result
        }
    }

    pub fn permutations<'a>(self) -> Box<dyn Iterator<Item = (u32, Partitions)> + 'a> {
        self.iterate(Goal::Permutations)
    }

    pub fn combinations<'a>(self) -> Box<dyn Iterator<Item = (u32, Partitions)> + 'a> {
        self.iterate(Goal::Combinations)
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
                .flatten(Goal::Combinations)
                .count(),
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
                        let target = Partitions {
                            sum,
                            parts,
                            min,
                            max,
                        };
                        if target.has_solution() {
                            let all_correct_sum = target
                                .flatten(Goal::Combinations)
                                .into_iter()
                                .map(|digits| digits.into_iter().fold(0, |total, i| total + i))
                                .all(|r| r == sum);
                            assert!(all_correct_sum)
                        }
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
    fn print_a_sample() {
        let sum = 8;
        let parts = 3;
        let min = 1;
        let max = sum;
        let kind = Goal::Combinations;
        Partitions {
            sum,
            parts,
            min,
            max,
        }
        .flatten(kind)
        .into_iter()
        .for_each(|p| println!("{:?}", p))
    }
}
