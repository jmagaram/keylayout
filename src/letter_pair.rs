use std::fmt;

use crate::letter::Letter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LetterPair {
    a: Letter,
    b: Letter,
}

impl LetterPair {
    pub fn new(a: Letter, b: Letter) -> LetterPair {
        (a, b).try_into().unwrap()
    }
}

impl fmt::Display for LetterPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.a, self.b)
    }
}

impl TryFrom<(Letter, Letter)> for LetterPair {
    type Error = &'static str;

    fn try_from(value: (Letter, Letter)) -> Result<Self, Self::Error> {
        let (a, b) = value;
        match a == b {
            true => Err("The letters in a LetterPair must be different."),
            false => {
                let pair = match a < b {
                    true => LetterPair { a, b },
                    false => LetterPair { a: b, b: a },
                };
                Ok(pair)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic]
    fn new_panic_if_same_letter() {
        let _p = LetterPair::new(Letter::new('a'), Letter::new('a'));
    }

    #[test]
    fn display_concats_the_letters_in_alphabetical_order() {
        let p1 = LetterPair::new(Letter::new('x'), Letter::new('y'));
        let p2 = LetterPair::new(Letter::new('y'), Letter::new('x'));
        assert_eq!("xy", p1.to_string());
        assert_eq!("xy", p2.to_string());
    }

    #[test]
    fn eq_ignores_order_of_letters() {
        let p1 = LetterPair::new(Letter::new('x'), Letter::new('y'));
        let p2 = LetterPair::new(Letter::new('y'), Letter::new('x'));
        assert!(p1 == p2);
    }

    #[test]
    fn try_from_return_error_if_same_letters() {
        assert!(LetterPair::try_from((Letter::new('a'), Letter::new('a'))).is_err());
    }

    #[test]
    fn try_from_return_ok_if_not_same_letter() {
        let p = LetterPair::try_from((Letter::new('a'), Letter::new('b')));
        assert!(p.is_ok());
        assert_eq!(p.unwrap().to_string(), "ab");
    }
}
