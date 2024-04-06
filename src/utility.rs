fn partitions(sum: u32, parts: u32, min: u32, max: u32) -> Vec<Vec<u32>> {
    match sum == 0 {
        true => vec![vec![]],
        false => (min..=max)
            .into_iter()
            .filter_map(|n| match n + (min * parts - 1) <= sum && n * parts >= sum {
                true => Some(n),
                false => None,
            })
            .into_iter()
            .flat_map(|digit| {
                let m = partitions(sum - digit, parts - 1, min, digit);
                let zz = m.into_iter().map(move |dd| {
                    let mut ddCopy = dd.clone();
                    ddCopy.push(digit);
                    ddCopy
                });
                zz
            })
            .collect::<Vec<Vec<u32>>>(),
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
        partitions(sum, parts, min, max)
            .into_iter()
            .for_each(|p| println!("{:?}", p))
    }
}
