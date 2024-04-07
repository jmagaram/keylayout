use std::collections::{HashMap, HashSet};

use crate::set32::Set32;

struct LetterMap {
    letter_to_num: HashMap<char, u32>,
    num_to_letter: Vec<char>,
    letter_index_set: Set32,
}

impl LetterMap {
    pub fn new(words: Vec<String>) -> Self {
        let mut letter_set = HashSet::new();
        words.iter().flat_map(|w| w.chars()).for_each(|c| {
            letter_set.insert(c);
        });
        let mut letter_to_num = HashMap::new();
        let mut num_to_letter = Vec::new();
        letter_set.iter().enumerate().for_each(|(index, letter)| {
            let index = u32::try_from(index).unwrap();
            letter_to_num.insert(*letter, index);
            num_to_letter.push(*letter);
        });
        let set = Set32::fill(letter_set.len().try_into().unwrap());
        Self {
            letter_to_num,
            num_to_letter,
            letter_index_set: set,
        }
    }

    pub fn letter_index_set(&self) -> Set32 {
        self.letter_index_set
    }

    pub fn letter_for(&self, inx: u32) -> char {
        self.num_to_letter[usize::try_from(inx).unwrap()]
    }

    pub fn index_for(&self, char: char) -> u32 {
        let inx = self.letter_to_num.get(&char).unwrap();
        *inx
    }
}

impl From<Vec<&str>> for LetterMap {
    fn from(value: Vec<&str>) -> Self {
        let vec = value.iter().map(|w| w.to_string()).collect::<Vec<String>>();
        LetterMap::new(vec)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_initializes_all_values_properly() {
        let words = vec!["apple", "banana", "charlie", "bob"];
        let source = LetterMap::from(words);

        // correct size of the letter set: aplebnchrio
        assert_eq!(source.letter_index_set().count(), 11);

        // internal sizes correct
        assert_eq!(source.num_to_letter.len(), 11);
        assert_eq!(source.letter_to_num.len(), 11);

        // mapping back and forth is consistent
        for i in 0..11 {
            let char1 = source.letter_for(i);
            let inx = source.index_for(char1);
            let char2 = source.letter_for(inx);
            assert_eq!(char1, char2);
        }
    }
}
