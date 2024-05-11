use crate::letter_pair::LetterPair;
use hashbrown::HashSet;
use std::fmt;

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

#[cfg(test)]
mod tests {

    use crate::letter::Letter;

    use super::*;

    #[test]
    fn display_is_empty_string_if_no_pairs() {
        let actual = LetterPairSet::new(vec![].into_iter());
        assert_eq!("", actual.to_string());
    }

    #[test]
    fn duplicates_are_removed() {
        let pair = LetterPair::new(Letter::new('a'), Letter::new('b'));
        let set = LetterPairSet::new(vec![pair.clone(), pair.clone(), pair.clone()].into_iter());
        assert_eq!("ab", set.to_string());
    }

    #[test]
    fn display_test() {
        let data = [
            ("ab,cd,ef", "ab,cd,ef"),
            ("ef,cd,ab", "ab,cd,ef"),
            ("fe,dc,ba", "ab,cd,ef"),
            ("ab", "ab"),
            ("ba", "ab"),
        ];
        for (input, expected) in data {
            let pairs = input.split(',').map(|pr| {
                let a = Letter::new(pr.chars().into_iter().nth(0).unwrap());
                let b = Letter::new(pr.chars().into_iter().nth(1).unwrap());
                LetterPair::new(a, b)
            });
            let set = LetterPairSet::new(pairs);
            let actual = set.to_string();
            assert_eq!(actual, expected);
        }
    }
}
