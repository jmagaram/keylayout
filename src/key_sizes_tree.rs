use crate::{partitions::Partitions, tally::Tally};
use std::collections::HashSet;

/// Represents all possible key size layouts for a given number of keys, number
/// of letters, and max and min key sizes. Note that a layout of 1,2,3 is
/// considered different than 3,2,1.
#[derive(Clone)]
pub struct KeySizesTree(Vec<Vec<u32>>);

impl KeySizesTree {
    pub fn new(p: &Partitions) -> KeySizesTree {
        KeySizesTree(
            p.calculate()
                .iter()
                .flat_map(|p| Tally::from(p).combinations())
                .collect::<Vec<Vec<u32>>>(),
        )
    }

    pub fn is_empty(&self) -> bool {
        let only_zeros = |items: &Vec<u32>| items.iter().all(|i| *i == 0);
        self.0.is_empty() || self.0.iter().all(only_zeros)
    }

    pub fn next(self) -> impl Iterator<Item = (u32, KeySizesTree)> {
        self.0
            .iter()
            .flat_map(|xx| xx.first())
            .map(|x| *x)
            .collect::<HashSet<u32>>()
            .into_iter()
            .map(move |x| {
                let children = KeySizesTree(
                    self.0
                        .iter()
                        .filter(move |yy| yy.first().map_or(false, |y| *y == x))
                        .map(|yy| {
                            yy.as_slice()
                                .get(1..)
                                .map(|xx| xx.to_vec())
                                .unwrap_or(vec![])
                        })
                        .collect::<Vec<Vec<u32>>>(),
                );
                (x, children)
            })
    }

    pub fn print(&self, depth: usize) {
        let indent = std::iter::repeat(".").take(depth).collect::<String>();
        for (root, children) in self.clone().next() {
            println!("{}{}", indent, root);
            children.print(depth + 1);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn is_empty_true_if_no_items() {
        let target = KeySizesTree(vec![]);
        assert!(target.is_empty());
    }

    #[test]
    fn is_empty_true_if_only_empty_items() {
        let target = KeySizesTree(vec![vec![], vec![]]);
        assert!(target.is_empty());
    }

    #[test]
    fn is_empty_true_if_only_items_with_zeros() {
        let target = KeySizesTree(vec![vec![0, 0], vec![0, 0]]);
        assert!(target.is_empty());
    }

    #[test]
    fn is_empty_false() {
        assert!(false == KeySizesTree(vec![vec![1, 2]]).is_empty());
        assert!(false == KeySizesTree(vec![vec![1]]).is_empty());
        assert!(false == KeySizesTree(vec![vec![1], vec![2]]).is_empty());
    }

    #[test]
    #[ignore]
    fn test_print_out() {
        let p = Partitions {
            sum: 10,
            parts: 3,
            min: 2,
            max: 5,
        };
        let target = KeySizesTree::new(&p);
        target.print(0);
    }
}
