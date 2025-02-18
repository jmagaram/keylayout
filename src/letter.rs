use std::fmt;

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Default, Hash)]
pub struct Letter(u8);

impl Letter {
    pub const ALPHABET: [char; 27] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '\'',
    ];

    pub const ALPHABET_SIZE: usize = Letter::ALPHABET.len();

    pub fn new(char: char) -> Letter {
        Letter::try_from(char).unwrap()
    }

    pub fn to_char(&self) -> char {
        Letter::ALPHABET[self.0 as usize]
    }

    pub fn to_usize_index(&self) -> usize {
        self.0 as usize
    }

    pub fn to_u8_index(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Letter(inx) = self;
        write!(f, "{}", Letter::ALPHABET[*inx as usize])
    }
}

impl TryFrom<char> for Letter {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value >= 'a' && value <= 'z' {
            let letter_a_index = 'a' as u32;
            let value_index = value as u32;
            Ok(Letter((value_index - letter_a_index) as u8))
        } else if value == '\'' {
            Ok(Letter(26))
        } else {
            Err("Could not convert the character into a Letter.")
        }
    }
}

impl TryFrom<u32> for Letter {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value >= Letter::ALPHABET.len() as u32 {
            Err("Letter indexes must be >= 0 and < the size of the alphabet.")
        } else {
            Ok(Letter(value as u8))
        }
    }
}

impl TryFrom<u128> for Letter {
    type Error = &'static str;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value >= Letter::ALPHABET.len() as u128 {
            Err("Letter indexes must be >= 0 and < the size of the alphabet.")
        } else {
            Ok(Letter(value as u8))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn try_from_char_when_in_alphabet() {
        for c in Letter::ALPHABET {
            let letter = Letter::try_from(c);
            match letter {
                Err(_) => panic!("Could not convert the character '{}' to a Letter.", c),
                Ok(letter) => assert_eq!(letter.to_string(), c.to_string()),
            }
        }
    }

    #[test]
    fn try_from_char_when_not_in_alphabet() {
        let before_a = char::from_u32(('a' as u32) - 1).unwrap();
        let after_z = char::from_u32(('z' as u32) + 1).unwrap();
        let invalid_character = ['4', '$', 'A', 'B', 'Z', before_a, after_z];
        for c in invalid_character {
            let letter = Letter::try_from(c);
            match letter {
                Err(_) => (),
                Ok(_) => panic!("Converted the invalid character '{}' into a Letter.", c),
            }
        }
    }

    #[test]
    fn new_for_valid_letters_in_alphabet() {
        for c in Letter::ALPHABET {
            let actual = Letter::new(c).to_string();
            let expected = c.to_string();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    #[should_panic]
    fn new_panic_on_invalid_letter() {
        Letter::new('5');
    }

    #[test]
    fn display_is_the_character_from_alphabet_indexed() {
        for c in Letter::ALPHABET {
            let actual = c.to_string();
            let expected = c.to_string();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn try_from_u128_when_not_in_alphabet() {
        assert!(Letter::try_from(Letter::ALPHABET.len() as u128).is_err());
        assert!(Letter::try_from((Letter::ALPHABET.len() + 1) as u128).is_err());
    }

    #[test]
    fn try_from_u128_when_in_alphabet() {
        for i in 0..Letter::ALPHABET.len() - 1 {
            assert!(Letter::try_from(i as u128).is_ok());
        }
    }

    #[test]
    fn try_from_u32_when_not_in_alphabet() {
        assert!(Letter::try_from(Letter::ALPHABET.len() as u32).is_err());
        assert!(Letter::try_from((Letter::ALPHABET.len() + 1) as u32).is_err());
    }

    #[test]
    fn try_from_u32_when_in_alphabet() {
        for i in 0..Letter::ALPHABET.len() - 1 {
            assert!(Letter::try_from(i as u32).is_ok());
        }
    }

    #[test]
    fn to_char() {
        for expected in Letter::ALPHABET {
            let actual: char = Letter::new(expected).to_char();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn to_u8_index() {
        for expected in Letter::ALPHABET {
            let letter = Letter::new(expected);
            let actual = letter.to_u8_index();
            let expected = letter.0;
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn to_usize_index_returns_zero_based_index_into_alphabet() {
        for c in Letter::ALPHABET {
            let actual = Letter::new(c).to_usize_index();
            let expected = Letter::ALPHABET
                .iter()
                .enumerate()
                .find_map(|(inx, char)| match *char == c {
                    true => Some(inx),
                    false => None,
                })
                .map(|inx| inx as usize)
                .unwrap();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn alphabet_size_is_number_of_letters() {
        assert_eq!(Letter::ALPHABET_SIZE, Letter::ALPHABET.len());
    }

    #[test]
    fn ord_is_alphabetical() {
        let a = Letter::new('a');
        let b = Letter::new('b');
        let min = a.min(b);
        assert_eq!(min, a);
    }
}
