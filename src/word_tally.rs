use crate::letter::Letter;

pub struct WordTally {
    count: u32,
    children: [Option<Box<WordTally>>; Letter::ALPHABET_SIZE],
}

impl WordTally {
    const MAX_WORD_LENGTH: usize = 35;

    pub fn new() -> WordTally {
        WordTally {
            count: 0,
            children: Default::default(),
        }
    }

    fn increment_helper(&mut self, word: &Vec<u8>, word_index: usize) -> u32 {
        if word_index as usize == word.len() {
            self.count = self.count + 1;
            self.count
        } else {
            let letter = word[word_index] as usize;
            let node = self
                .children
                .get_mut(letter)
                .expect("The letter was too big; must be smaller than the alphabet size.");
            match node {
                None => {
                    let mut node = Box::new(WordTally::new());
                    let result = node.increment_helper(word, word_index + 1);
                    self.children[letter] = Some(node);
                    result
                }
                Some(child) => child.increment_helper(word, word_index + 1),
            }
        }
    }

    pub fn increment(&mut self, word: &Vec<u8>) -> u32 {
        assert!(
            word.len() > 0,
            "Attempted to insert an empty word into the trie."
        );
        assert!(
            word.len() <= WordTally::MAX_WORD_LENGTH,
            "Attempted to insert a word that is too long."
        );
        self.increment_helper(word, 0)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic]
    fn panic_on_empty_word() {
        let mut root = WordTally::new();
        let spelling: Vec<u8> = vec![];
        let _result = root.increment(&spelling);
    }

    #[test]
    #[should_panic]
    fn panic_on_word_longer_than_max() {
        let mut root = WordTally::new();
        let spelling: Vec<u8> = (1..=WordTally::MAX_WORD_LENGTH + 1)
            .map(|_| 9)
            .collect::<Vec<u8>>();
        let _result = root.increment(&spelling);
    }

    #[test]
    fn accept_word_of_max_length() {
        let mut root = WordTally::new();
        let spelling: Vec<u8> = (1..=WordTally::MAX_WORD_LENGTH)
            .map(|_| 9)
            .collect::<Vec<u8>>();
        let _result = root.increment(&spelling);
    }

    #[test]
    #[should_panic]
    fn panic_if_letter_bigger_than_alphabet() {
        let mut root = WordTally::new();
        let max_letter = Letter::ALPHABET_SIZE as u8;
        let max_word = vec![max_letter, max_letter];
        root.increment(&max_word);
    }

    #[test]
    fn accept_max_letter_in_alphabet() {
        let mut root = WordTally::new();
        let max_letter = (Letter::ALPHABET_SIZE - 1) as u8;
        let max_word = vec![max_letter, max_letter];
        root.increment(&max_word);
        assert_eq!(2, root.increment(&max_word));
    }

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
        let mut root = WordTally::new();
        for (word, expected) in data {
            let letters = word
                .chars()
                .map(|r| Letter::new(r).to_u8())
                .collect::<Vec<u8>>();
            let actual = root.increment(&letters);
            assert_eq!(
                actual, expected,
                "inserted word '{}' and expected count '{}'",
                word, expected
            );
        }
    }
}
