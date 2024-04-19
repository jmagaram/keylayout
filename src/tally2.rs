#![allow(dead_code)]

use std::collections::HashMap;

// #[derive(Clone)]
pub struct ShortVecU8Hasher {
    state: u64,
}

impl std::hash::Hasher for ShortVecU8Hasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes.iter().take(4) {
            self.state = self.state.rotate_left(8) ^ u64::from(byte);
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

pub struct BuildShortVecU8Hasher;

impl std::hash::BuildHasher for BuildShortVecU8Hasher {
    type Hasher = ShortVecU8Hasher;

    fn build_hasher(&self) -> ShortVecU8Hasher {
        ShortVecU8Hasher { state: 0 }
    }
}

// pub struct Tally2(HashMap<Vec<u8>, u32, BuildShortVecU8Hasher>);
pub struct Tally2(fnv_rs::FnvHashMap<Vec<u8>, u32>);

impl Tally2 {
    pub fn new() -> Tally2 {
        // let map = HashMap::with_hasher(BuildShortVecU8Hasher);
        let map = fnv_rs::FnvHashMap::default();
        Tally2(map)
    }

    pub fn increment(&mut self, item: Vec<u8>) -> u32 {
        match self.0.get_mut(&item) {
            None => {
                self.0.insert(item, 1);
                1
            }
            Some(count) => {
                let new_count = *count + 1;
                *count = new_count;
                new_count
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::letter::Letter;

    use super::Tally2;

    #[test]
    fn returns_number_of_same_words_inserted() {
        let data = [
            ("a", 1),
            ("ab", 1),
            ("abc", 1),
            ("the", 1),
            ("the", 2),
            ("the", 3),
            ("no", 1),
            ("their", 1),
            ("their", 2),
            ("their", 3),
            ("no", 2),
            ("not", 1),
            ("note", 1),
            ("notes", 1),
            ("note", 2),
            ("not", 2),
            ("no", 3),
            ("experiment", 1),
            ("experiment", 2),
        ];
        let mut root = Tally2::new();
        for (word, expected) in data {
            let letters = word
                .chars()
                .map(|r| Letter::new(r).to_u8())
                .collect::<Vec<u8>>();
            let actual = root.increment(letters);
            assert_eq!(
                actual, expected,
                "inserted word '{}' and expected count '{}'",
                word, expected
            );
        }
    }
}
