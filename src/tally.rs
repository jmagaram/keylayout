use hashbrown::HashMap; // much faster than built-in HashMap
use std::hash::Hash;

#[derive(Clone, Default, Debug)]
pub struct Tally<T>(HashMap<T, u32>);

impl<T> Tally<T>
where
    T: Hash + Eq,
{
    pub fn new() -> Tally<T> {
        Tally(HashMap::new())
    }

    fn increment_by(&mut self, item: T, n: u32) -> u32 {
        match self.0.get_mut(&item) {
            None => {
                if n > 0 {
                    self.0.insert(item, n);
                    n
                } else {
                    0
                }
            }
            Some(count) => {
                let new_count = *count + n;
                *count = new_count;
                new_count
            }
        }
    }

    pub fn increment(&mut self, item: T) -> u32 {
        self.increment_by(item, 1)
    }

    pub fn count(&self, item: &T) -> u32 {
        match self.0.get(item) {
            None => 0,
            Some(count) => count.clone(),
        }
    }

    fn decrement_by(&mut self, item: T, n: u32) -> u32 {
        match self.0.get_mut(&item) {
            None => {
                if n > 0 {
                    panic!("Attempted to decrement the tally below zero.");
                } else {
                    0
                }
            }
            Some(count) => {
                if n > *count {
                    panic!("Attempted to decrement the tally below zero.");
                } else if n < *count {
                    let new_count = *count - n;
                    *count = new_count;
                    new_count
                } else {
                    self.0.remove(&item);
                    0
                }
            }
        }
    }

    pub fn decrement(&mut self, item: T) -> u32 {
        self.decrement_by(item, 1)
    }

    /// Generates all unique ways the items in the Tally can be distributed. For
    /// example, if the tally shows 2A and 1B, the unique combinations are AAB,
    /// ABA, and BAA.
    pub fn combinations(&self) -> Vec<Vec<T>>
    where
        T: Clone + Hash + Eq,
    {
        if self.0.len() == 0 {
            vec![vec![]]
        } else {
            self.0
                .iter()
                .flat_map(|pair| {
                    let (item, _) = pair;
                    let mut items = self.clone();
                    items.decrement(item.clone());
                    items.combinations().into_iter().map(|mut p| {
                        p.push(item.clone());
                        p
                    })
                })
                .collect::<Vec<Vec<T>>>()
        }
    }
}

impl<K, const N: usize> From<[K; N]> for Tally<K>
where
    K: Hash + Eq + Clone,
{
    fn from(value: [K; N]) -> Self {
        value.iter().fold(Tally::<K>::new(), |mut total, i| {
            total.increment(i.clone());
            total
        })
    }
}

impl From<Vec<u32>> for Tally<u32> {
    fn from(value: Vec<u32>) -> Self {
        value.iter().fold(Tally::<u32>::new(), |mut total, i| {
            total.increment(*i);
            total
        })
    }
}

impl From<&Vec<u32>> for Tally<u32> {
    fn from(value: &Vec<u32>) -> Self {
        value.iter().fold(Tally::<u32>::new(), |mut total, i| {
            total.increment(*i);
            total
        })
    }
}

impl<K> FromIterator<K> for Tally<K>
where
    K: Hash + Eq + Clone,
{
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        iter.into_iter().fold(Tally::<K>::new(), |mut total, i| {
            total.increment(i.clone());
            total
        })
    }
}

impl<K, const N: usize> From<[(K, u32); N]> for Tally<K>
where
    K: Hash + Eq + Clone,
{
    fn from(value: [(K, u32); N]) -> Self {
        value
            .iter()
            .filter(|(_, v)| *v > 0)
            .fold(Tally::<K>::new(), |mut total, (k, v)| {
                total.increment_by(k.clone(), *v);
                total
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashSet;

    #[test]
    fn from_iterator_of_items() {
        let tally = Tally::from_iter(["a", "a", "b", "b", "a"]);
        assert_eq!(tally.count(&"a"), 3);
        assert_eq!(tally.count(&"b"), 2);
    }

    #[test]
    fn from_tuples_array_sums_duplicate_items() {
        let tally = Tally::<&str>::from([("a", 3), ("b", 2), ("a", 1)]);
        println!("{:?}", tally);
        assert_eq!(tally.count(&"a"), 4);
    }

    #[test]
    fn from_tuples_array_ignores_items_with_zero_occurrences() {
        let tally = Tally::<&str>::from([("a", 3), ("b", 0), ("c", 5)]);
        assert!(!tally.0.contains_key(&"b"));
    }

    #[test]
    fn count() {
        let tally = Tally::<&str>::from([("a", 3), ("b", 0), ("c", 5)]);
        assert_eq!(3, tally.count(&"a"));
        assert_eq!(5, tally.count(&"c"));
    }

    #[test]
    fn count_is_zero_if_item_missing() {
        let tally = Tally::<&str>::from([("a", 3), ("b", 0), ("c", 5)]);
        assert_eq!(0, tally.count(&"banana"));
    }

    #[test]
    fn increment_by_returns_count() {
        let mut tally = Tally::<&str>::from([("a", 1)]);
        assert_eq!(2, tally.increment_by(&"a", 1));
        assert_eq!(5, tally.increment_by(&"a", 3));
        assert_eq!(5, tally.increment_by(&"a", 0));
    }

    #[test]
    fn increment_by() {
        let mut tally = Tally::<&str>::from([("a", 3), ("b", 0), ("c", 5)]);
        tally.increment_by(&"b", 2);
        assert_eq!(2, tally.count(&"b"));
        tally.increment_by(&"b", 0);
        assert_eq!(2, tally.count(&"b"));
    }

    #[test]
    fn increment_by_zero_has_no_effect() {
        let mut tally = Tally::<&str>::new();
        tally.increment_by(&"a", 0);
        assert_eq!(0, tally.0.len());
    }

    #[test]
    fn increment_by_zero_returns_same_count() {
        let mut tally = Tally::<&str>::from([("a", 1)]);
        assert_eq!(1, tally.increment_by(&"a", 0));
    }

    #[test]
    fn decrement_by() {
        let mut tally = Tally::<&str>::from([("a", 5)]);
        tally.decrement_by(&"a", 2);
        assert_eq!(3, tally.count(&"a"));
    }

    #[test]
    fn decrement_by_returns_new_count() {
        let mut tally = Tally::<&str>::from([("a", 5)]);
        assert_eq!(3, tally.decrement_by(&"a", 2));
        assert_eq!(2, tally.decrement_by(&"a", 1));
    }

    #[test]
    fn decrement_by_zero_returns_same_count() {
        let mut t1 = Tally::<&str>::from([("a", 5)]);
        assert_eq!(5, t1.decrement_by(&"a", 0));

        let mut t2 = Tally::<&str>::from([("a", 5)]);
        assert_eq!(0, t2.decrement_by(&"b", 0));
    }

    #[test]
    #[should_panic]
    fn decrement_by_count_larger_than_current_panics() {
        let letter = "a";
        let mut tally = Tally::<&str>::from([(letter, 5)]);
        tally.decrement_by(letter, 100000);
    }

    #[test]
    fn decrement_by_removes_item_if_zero() {
        let letter = "a";
        let mut tally = Tally::<&str>::from([(letter, 5)]);
        tally.decrement_by(letter, 5);
        assert_eq!(0, tally.0.len());
    }

    #[test]
    #[ignore]
    fn combinations_display() {
        let tally = Tally::<&str>::from([("a", 2), ("b", 1)]);
        let result = tally.combinations();
        for r in result {
            println!("{}", r.join(","));
        }
    }

    #[test]
    fn combinations_of_empty_is_one_empty() {
        let tally = Tally::<&str>::new();
        let result = tally.combinations();
        assert_eq!(1, result.len());
        assert!(result[0].len() == 0);
    }

    #[test]
    fn combinations_count_is_correct() {
        let data = [
            (1, 1, 1, 6),
            (4, 3, 2, 1260),
            (3, 3, 2, 560),
            (7, 1, 4, 3960),
            (1, 0, 0, 1),
            (1, 1, 0, 2),
            (1, 0, 0, 1),
        ];
        for (a, b, c, expected) in data {
            let tally = Tally::<&str>::from([("a", a), ("b", b), ("c", c)]);
            let result = tally.combinations();

            // total count is correct
            assert_eq!(expected, result.len());

            // each is unique
            let unique = result
                .iter()
                .map(|x| x.join(","))
                .collect::<HashSet<String>>();
            assert_eq!(unique.len(), result.len());
        }
    }
}
