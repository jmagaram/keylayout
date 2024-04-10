use std::{collections::HashMap, hash::Hash};

use crate::permutable::Permutable;

pub struct ItemCount<T>(HashMap<T, u32>);

impl<T> ItemCount<T> {}

pub fn with_u32_groups(items: &Vec<u32>) -> ItemCount<u32> {
    let mut map: HashMap<u32, u32> = HashMap::new();
    for size in items {
        match map.get(size) {
            None => {
                map.insert(*size, 1);
            }
            Some(count) => {
                map.insert(*size, count + 1);
            }
        }
    }
    ItemCount(map)
}

impl<T> Permutable<T> for ItemCount<T>
where
    T: Clone + PartialEq + Eq + Hash,
{
    fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    fn parts(&self) -> Vec<(T, Self)> {
        let mut result = vec![];
        for i in self.0.iter() {
            let (k, count) = i;
            match count {
                0 => (),
                1 => {
                    let mut copy = self.0.clone();
                    copy.remove(k);
                    let distribution = ItemCount(copy);
                    let state = (k.to_owned(), distribution);
                    result.push(state)
                }
                count => {
                    let mut copy = self.0.clone();
                    copy.insert(k.to_owned(), count - 1);
                    let distribution = ItemCount(copy);
                    let state = (k.to_owned(), distribution);
                    result.push(state)
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const A: &str = "a";
    const B: &str = "b";
    const C: &str = "c";

    fn make_map(a: u32, b: u32, c: u32) -> HashMap<&'static str, u32> {
        let mut map = HashMap::new();
        if a > 0 {
            map.insert(A, a);
        };
        if b > 0 {
            map.insert(B, b);
        };
        if c > 0 {
            map.insert(C, c);
        };
        map
    }

    #[test]
    #[ignore]
    fn print_permutation_sample() {
        let map = make_map(3, 1, 1);
        let f = ItemCount(map);
        let results = f.permute();
        println!("=== Permuations by frequency count ===");
        results.iter().for_each(|v| {
            println!("{:?}", v);
        })
    }

    #[test]
    fn when_many_items_permutation_count_is_correct() {
        let map = make_map(2, 1, 1);
        let f = ItemCount(map);
        assert_eq!(12, f.permute().len());
    }

    #[test]
    fn when_1_item() {
        let map = make_map(3, 0, 0);
        let f = ItemCount(map);
        assert_eq!(1, f.permute().len());
    }

    #[test]
    fn when_2_item() {
        let map = make_map(1, 1, 0);
        let f = ItemCount(map);
        assert_eq!(2, f.permute().len());
    }
}
