use std::fmt;

// let make = s => Word(s)
// let toString = (Word(s)) => s
// let characters = (Word(w)) => w->String.split("")->Seq.fromArray->Seq.map(Character.make)
// let lastCharacter = (Word(w)) => Character(w->String.charAt(w->String.length - 1))
// let excludeLetter = (Word(w), Character(c)) => {
//   let result = w->String.replaceAll(c, "")
//   switch result->String.length {
//   | 0 => None
//   | _ => Some(Word(result))
//   }
// }
// let random = (~minLength, ~maxLength, ~characters) => {

#[derive(PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct Word(String);

impl Word {}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Word(word) = self;
        write!(f, "\"{}\"", word)
    }
}

impl std::convert::From<String> for Word {
    fn from(value: String) -> Self {
        Word(value.clone())
    }
}

impl std::convert::From<&str> for Word {
    fn from(value: &str) -> Self {
        Word(String::from(value))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn from_str() {
        let word: Word = "abc".into();
        assert_eq!(word.to_string(), "\"abc\"");
    }

    #[test]
    fn from_string_test() {
        let source = String::from("abc");
        let word = Word::from(source);
        assert_eq!(word.to_string(), "\"abc\"");
    }

    #[test]
    fn display_trait() {
        assert_eq!(Word::from("abc").to_string(), "\"abc\"");
    }

    #[test]
    fn ord() {
        let words = ["abc", "the", "box", "cat", "zebra", "banana"];
        let mut sorted = words
            .into_iter()
            .map(|w| Word::from(w))
            .collect::<Vec<Word>>();
        sorted.sort();
        let combined = sorted
            .into_iter()
            .map(|w| w.to_string())
            .collect::<Vec<String>>()
            .join(",");
        assert_eq!(combined, "abc,banana,box,cat,the,zebra");
    }
}
