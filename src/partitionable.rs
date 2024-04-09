fn permutations<STATE, T>(
    first_time_called: bool,
    state: &STATE,
    is_empty: fn(&STATE) -> bool,
    parts: fn(&STATE) -> Vec<(T, STATE)>,
) -> Vec<Vec<T>>
where
    T: Copy,
{
    match (is_empty(state), first_time_called) {
        (true, true) => vec![],
        (true, false) => vec![vec![]],
        (false, _) => {
            let mut results = vec![];
            for (item, rest) in parts(state) {
                for children in permutations(false, &rest, is_empty, parts) {
                    let mut results_copy = children.to_vec();
                    results_copy.push(item);
                    results.push(results_copy);
                }
            }
            results
        }
    }
}

pub trait Partitionable<T>
where
    Self: Sized,
{
    fn is_empty(&self) -> bool;
    fn parts(&self) -> Vec<(T, Self)>;

    fn permutations(&self) -> Vec<Vec<T>>
    where
        T: Copy,
    {
        permutations(true, self, Self::is_empty, Self::parts)
    }
}

impl<T> Partitionable<T> for Vec<T>
where
    T: PartialEq + Copy,
{
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn parts(&self) -> Vec<(T, Self)> {
        self.iter()
            .map(|current_item| {
                let remain: Vec<T> = self
                    .iter()
                    .filter(move |s| **s != *current_item)
                    .map(|s| *s)
                    .collect();
                (*current_item, remain)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use std::fmt::Display;

    use super::*;

    fn combo_to_string<T>(items: &Vec<T>) -> String
    where
        T: Display,
    {
        let contents = items
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(",");
        format!("[{}]", contents)
    }

    #[test]
    fn permutations_when_empty() {
        let source: Vec<u32> = vec![];
        let perms = source.permutations();
        assert_eq!(0, perms.len());
    }

    #[test]
    fn permutations_when_one_item() {
        let source: Vec<i32> = vec![99];
        let perms: Vec<String> = source
            .permutations()
            .iter()
            .map(|c| combo_to_string(c))
            .collect();
        assert_eq!(1, perms.len());
        assert_eq!("[99]", perms.get(0).unwrap());
    }

    #[test]
    fn permutations_when_many() {
        let nums = [1, 2, 3].to_vec();
        let perms: Vec<String> = nums
            .permutations()
            .iter()
            .map(|c| combo_to_string(c))
            .collect();
        assert_eq!(6, perms.len());
        [
            "[1,2,3]", "[1,3,2]", "[2,1,3]", "[2,3,1]", "[3,1,2]", "[3,2,1]",
        ]
        .iter()
        .for_each(|s| {
            assert!(perms.contains(&(s.to_string())));
        })
    }
}
