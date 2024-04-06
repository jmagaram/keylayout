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
                .into_iter()
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
    fn perf_im() {
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
