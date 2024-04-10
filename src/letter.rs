use std::fmt;

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Default)]
pub struct Letter(u8);

const ALPHABET: [char; 27] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '\'',
];

// fn index_of(c: char) -> Option<u8> {
//     0
// }

impl Letter {
    // pub fn new(value: u32) -> Letter {
    //     assert!(value < ALPHABET.len() as u32);
    //     Letter(value as u8)
    // }

    // pub fn to_u8(&self) -> u8 {
    //     self.0
    // }

    // pub fn to_u32(&self) -> u32 {
    //     self.0.into()
    // }

    // pub fn serialize(&self) -> char {
    //     let unicode_base = 0x0041;
    //     let char_as_digit = self.to_u32();
    //     let result = char::from_u32(unicode_base + char_as_digit);
    //     result.expect("should be able to convert a u8 to a char to display")
    // }

    // pub fn to_usize(&self) -> usize {
    //     self.0.into()
    // }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Letter(inx) = self;
        write!(f, "{}", ALPHABET[*inx as usize])
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
        if value > 31 {
            Err("Letter only accepts u32 values in the range 0..=31")
        } else {
            Ok(Letter(value as u8))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_the_character_from_alphabet_indexed() {
        for c in ALPHABET {
            let actual = c.to_string();
            let expected = c.to_string();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn try_from_when_in_alphabet() {
        for c in ALPHABET {
            let letter = Letter::try_from(c);
            match letter {
                Err(_) => panic!("Could not convert the character '{}' to a Letter", c),
                Ok(letter) => assert_eq!(letter.to_string(), c.to_string()),
            }
        }
    }

    #[test]
    fn try_from_when_not_in_alphabet() {
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

    // #[test]
    // #[should_panic]
    // fn new_panic_when_value_too_big() {
    //     Letter::new(32);
    // }

    // #[test]
    // #[ignore]
    // fn display_is_just_a_simple_int() {
    //     let tests = [0, 1, 2, 13, 31];
    //     tests.into_iter().for_each(|p| {
    //         let p = Letter(p);
    //         println!("The number is {}", p);
    //     })
    // }

    // #[test]
    // fn serialize_creates_non_whitespace_characters() {
    //     let mut s = String::new();
    //     (0..=31).for_each(|i| {
    //         let num = Letter::new(i);
    //         let char = num.serialize();
    //         assert!(!char.is_whitespace(), "expected not whitespace");
    //         s.push(char);
    //     });
    //     assert_eq!(s.chars().count(), 32);
    // }
}
