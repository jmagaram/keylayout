use crate::{key::Key, letter::Letter, word::Word};
use std::fmt::{self, Display};

#[derive(Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct KeySet(Vec<Key>);

impl KeySet {
    const EMPTY: KeySet = KeySet(vec![]);

    fn with_unsorted(keys: Vec<Key>) -> KeySet {
        let mut keys = keys;
        keys.sort_unstable();
        KeySet(keys)
    }

    pub fn with_keys(keys: Vec<Key>) -> KeySet {
        Self::with_unsorted(keys)
    }

    pub fn with_pairs(pairs: Vec<Key>) -> KeySet {
        let mut result = KeySet::EMPTY;
        for p in pairs {
            debug_assert!(
                p.count_letters() == 2,
                "Expected each pair to have exactly 2 letters."
            );
            let i = p.min_letter().unwrap();
            let j = p.max_letter().unwrap();
            result = result.with_letters_on_same_key(i, j);
        }
        result
    }

    pub fn with_layout(layout: &str) -> KeySet {
        let keys = layout
            .split([',', ' '])
            .map(|letters| {
                Key::try_from(letters).expect("Expected each key to have valid letters and be separated by a single comma or space.")
            })
            .collect::<Vec<Key>>();
        Self::with_unsorted(keys)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn letter_count(&self) -> u8 {
        self.0.iter().map(|k| k.len()).sum()
    }

    fn find_key_index(&self, a: Letter) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .find_map(|(index, r)| match r.contains(a) {
                true => Some(index),
                false => None,
            })
    }

    pub fn with_word_differences(a: &Word, b: &Word) -> KeySet {
        a.different_pairs(&b).fold(KeySet::EMPTY, |total, (a, b)| {
            total.with_letters_on_same_key(a, b)
        })
    }

    pub fn with_letters_on_same_key(&self, a: Letter, b: Letter) -> KeySet {
        assert_ne!(a, b, "The letters must be different.");
        let a_index = self.find_key_index(a);
        let b_index = self.find_key_index(b);
        match (a_index, b_index) {
            (None, None) => {
                let key_ab = Key::EMPTY.add(a).add(b);
                let mut keys = self.0.clone();
                keys.push(key_ab);
                Self::with_unsorted(keys)
            }
            (Some(a_index), None) => {
                let mut keys = self.0.clone();
                keys[a_index] = keys[a_index].add(b);
                Self::with_unsorted(keys)
            }
            (None, Some(b_index)) => {
                let mut keys = self.0.clone();
                keys[b_index] = keys[b_index].add(a);
                Self::with_unsorted(keys)
            }
            (Some(a_index), Some(b_index)) => {
                if a_index == b_index {
                    let keys = self.0.clone();
                    Self::with_unsorted(keys)
                } else {
                    let merged_key = self.0[a_index].union(self.0[b_index]);
                    let keys = self
                        .0
                        .iter()
                        .enumerate()
                        .filter_map(|(index, k)| {
                            if index == a_index {
                                None
                            } else if index == b_index {
                                Some(merged_key)
                            } else {
                                Some(k.clone())
                            }
                        })
                        .collect::<Vec<Key>>();
                    Self::with_unsorted(keys)
                }
            }
        }
    }
}

impl Display for KeySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let data = [
            ("ab,cd", "ab cd"),
            ("cd,ab", "ab cd"),
            ("ab,bc", "abc"),
            ("ac,bc", "abc"),
            ("bc,ab", "abc"),
            ("ab,bc,cd", "abcd"),
            ("dc,cb,ba", "abcd"),
            ("ab,bc,xy,yz", "abc xyz"),
            ("zy,yx,cb,ba", "abc xyz"),
        ];
        for (insert, expected) in data {
            let actual = insert
                .split(',')
                .map(|pair| {
                    let a = pair
                        .chars()
                        .enumerate()
                        .find_map(|(index, c)| {
                            if index == 0 {
                                Some(Letter::new(c))
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    let b = pair
                        .chars()
                        .enumerate()
                        .find_map(|(index, c)| {
                            if index == 1 {
                                Some(Letter::new(c))
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    (a, b)
                })
                .fold(KeySet::EMPTY, |total, (a, b)| {
                    total.with_letters_on_same_key(a, b)
                })
                .to_string();
            assert_eq!(actual, expected, "source:{},expected:{}", insert, expected);
        }
    }

    #[test]
    fn word_differences() {
        let data = [
            ("hit", "hit", ""),
            ("hit", "hat", "ai"),
            ("hit", "bat", "ai bh"),
            ("hit", "mob", "bt hm io"),
            ("aaa", "bbb", "ab"),
            ("book", "beek", "eo"),
            ("bookii", "beekjj", "eo ij"),
            ("a", "b", "ab"),
            ("a", "a", ""),
            ("ab", "a", ""),
            ("a", "ab", ""),
            ("a", "aa", ""),
        ];
        for (word_a, word_b, expected) in data {
            let word_a = Word::new(word_a, 0.0);
            let word_b = Word::new(word_b, 0.0);
            let actual = KeySet::with_word_differences(&word_a, &word_b).to_string();
            assert_eq!(
                actual, expected,
                "word_a:{} word_b:{} expected:{}, actual:{}",
                word_a, word_b, expected, actual
            )
        }
    }

    #[test]
    fn equality() {
        assert!(KeySet::with_layout("abc def") == KeySet::with_layout("abc def"));
        assert!(KeySet::with_layout("abc") != KeySet::with_layout("abc def"));
    }
}
