use hashbrown::HashSet;

use crate::key::Key;

pub struct Prohibited(HashSet<Key>);

impl Prohibited {
    pub fn new() -> Prohibited {
        Prohibited(HashSet::new())
    }

    pub fn is_allowed(&self, other: Key) -> bool {
        !self.0.iter().any(|p| other.contains_all(&p))
    }

    pub fn add(&mut self, key: Key) {
        if key.is_empty() {
            panic!("Can not add an empty key to the list of prohibited keys.")
        }
        let subsets = self
            .0
            .iter()
            .filter(|i| key.contains_all(i) && key != **i)
            .map(|i| i.clone())
            .collect::<Vec<Key>>();
        let supersets = self
            .0
            .iter()
            .filter(|i| (**i).contains_all(&key) && key != **i)
            .map(|i| i.clone())
            .collect::<Vec<Key>>();
        for s in supersets {
            self.0.remove(&s);
        }
        if subsets.is_empty() {
            self.0.insert(key);
        }
    }

    pub fn add_many(&mut self, keys: impl Iterator<Item = Key>) {
        for k in keys {
            self.add(k);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn is_allowed() {
        let data = [
            ("ae,st", "ae", false),
            ("ae,st", "st", false),
            ("ae,st", "xy", true),
            ("ae,st", "a", true),
            ("ae,st", "s", true),
            ("ae,st", "t", true),
            ("ae,st", "aer", false),
            ("ae,st", "stu", false),
            ("ast", "ast", false),
            ("ast", "astx", false),
            ("ast", "as", true),
            ("ast", "str", true),
        ];
        for (prohibited, test_key, expect_is_key_allowed) in data {
            let mut p = Prohibited::new();
            p.add_many(prohibited.split(',').map(|pattern| Key::new(pattern)));
            let actual = p.is_allowed(Key::new(test_key));
            assert_eq!(
                actual, expect_is_key_allowed,
                "prohibited [{}] test [{}] expect_is_allowed {}",
                prohibited, test_key, expect_is_key_allowed
            );
        }
    }

    #[test]
    #[should_panic]
    fn if_add_empty_key_panic() {
        let mut p = Prohibited::new();
        p.add(Key::EMPTY);
    }

    #[test]
    fn duplicates_are_removed_when_add_smaller_set_first() {
        let mut p = Prohibited::new();
        p.add(Key::new("ab"));
        p.add(Key::new("abc"));
        assert_eq!(p.0.len(), 1);
        assert_eq!(Key::new("ab"), p.0.into_iter().next().unwrap());
    }

    #[test]
    fn duplicates_are_removed_when_add_bigger_set_first() {
        let mut p = Prohibited::new();
        p.add(Key::new("abc"));
        p.add(Key::new("ab"));
        assert_eq!(p.0.len(), 1);
        assert_eq!(Key::new("ab"), p.0.into_iter().next().unwrap());
    }
}
