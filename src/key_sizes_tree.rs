use crate::{partitions::Partitions, tally::Tally};
use std::collections::HashSet;

/// Represents all possible key size layouts for a given number of keys, number
/// of letters, and max and min key sizes. Note that a layout of 1,2,3 is
/// considered different than 3,2,1; order matters.
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

    pub fn next(&self) -> Vec<(u32, KeySizesTree)> {
        let unique = self
            .0
            .iter()
            .flat_map(|xx| xx.first())
            .map(|x| *x)
            .collect::<HashSet<u32>>();
        unique
            .iter()
            .map(|x| {
                let children = KeySizesTree(
                    self.0
                        .iter()
                        .filter(move |yy| yy.first().map_or(false, |y| *y == *x))
                        .map(|yy| {
                            yy.as_slice()
                                .get(1..)
                                .map(|xx| xx.to_vec())
                                .unwrap_or(vec![])
                        })
                        .collect::<Vec<Vec<u32>>>(),
                );
                (*x, children)
            })
            .collect::<Vec<(u32, KeySizesTree)>>()
    }

    pub fn print(&self, depth: usize) {
        let indent = std::iter::repeat(".").take(depth).collect::<String>();
        for (root, children) in self.next() {
            println!("{}{}", indent, root);
            children.print(depth + 1);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore]
    fn test_print_out() {
        let p = Partitions {
            sum: 27,
            parts: 10,
            min: 2,
            max: 4,
        };
        let target = KeySizesTree::new(&p);
        target.print(0);
    }
}
