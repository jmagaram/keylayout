use std::fmt;

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Default)]
pub struct U5(u8);

impl U5 {
    pub fn new(value: u32) -> U5 {
        assert!(value < 32);
        U5(value as u8)
    }

    pub fn to_u8(&self) -> u8 {
        self.0
    }

    pub fn to_u32(&self) -> u32 {
        self.0.into()
    }

    pub fn serialize(&self) -> char {
        let unicode_base = 0x0041;
        let char_as_digit = self.to_u32();
        let result = char::from_u32(unicode_base + char_as_digit);
        result.expect("should be able to convert a u8 to a char to display")
    }

    pub fn to_usize(&self) -> usize {
        self.0.into()
    }
}

impl fmt::Display for U5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let U5(n) = self;
        let _suffix = '\u{2807}';
        write!(f, "{}", n)
    }
}

impl std::convert::From<u32> for U5 {
    fn from(value: u32) -> Self {
        assert!(value < 32);
        U5(value as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn new_panic_when_value_too_big() {
        U5::new(32);
    }

    #[test]
    #[ignore]
    fn display_is_just_a_simple_int() {
        let tests = [0, 1, 2, 13, 31];
        tests.into_iter().for_each(|p| {
            let p = U5(p);
            println!("The number is {}", p);
        })
    }

    #[test]
    fn serialize_creates_non_whitespace_characters() {
        let mut s = String::new();
        (0..=31).for_each(|i| {
            let num = U5::new(i);
            let char = num.serialize();
            assert!(!char.is_whitespace(), "expected not whitespace");
            s.push(char);
        });
        assert_eq!(s.chars().count(), 32);
    }
}
