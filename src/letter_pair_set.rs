use std::fmt;

use hashbrown::HashSet;

use crate::letter_pair::LetterPair;

#[derive(Hash)]
pub struct LetterPairSet(Vec<LetterPair>);

impl LetterPairSet {
    pub fn new(pairs: impl Iterator<Item = LetterPair>) -> LetterPairSet {
        let pairs = pairs.collect::<HashSet<LetterPair>>();
        let mut pairs = pairs.into_iter().collect::<Vec<LetterPair>>();
        pairs.sort();
        LetterPairSet(pairs)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for LetterPairSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts = self
            .0
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{}", parts)
    }
}
