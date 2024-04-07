use std::{fmt, str::FromStr};

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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    word: String,
}

fn make(word: String) -> Word {
    Word { word }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.word)
    }
}

impl std::convert::From<String> for Word {
    fn from(value: String) -> Self {
        make(value)
    }
}

impl std::convert::From<&str> for Word {
    fn from(value: &str) -> Self {
        let w = String::from_str(value).expect("Could not convert the characters to a String");
        make(w)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn from_string_test() {
        let source = String::from("abc");
        let word = Word::from(source);
        assert_eq!(word.to_string(), "abc");
    }

    #[test]
    fn from_str() {
        let source: Word = "abc".into();
        let word = Word::from(source);
        assert_eq!(word.to_string(), "abc");
    }

    #[test]
    fn display() {
        assert_eq!(Word::from("abc").to_string(), "abc");
    }

    #[test]
    fn ord() {
        let strs = ["a", "z", "b", "y", "c", "x", "d", "w", "p", "o", "n", "m"];
        let mut word_vec = strs
            .into_iter()
            .map(|w| Word::from(w))
            .collect::<Vec<Word>>();
        word_vec.sort();
        let combined = word_vec
            .into_iter()
            .map(|w| w.to_string())
            .collect::<Vec<String>>()
            .join("");
        assert_eq!(combined, "abcdmnopwxyz");
    }
}
