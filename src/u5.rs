use std::fmt;

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Default)]
pub struct U5(u8);

impl U5 {
    pub fn new(value: u32) -> U5 {
        assert!(value < 32);
        U5(value as u8)
    }

    pub fn to_u32(&self) -> u32 {
        self.0.into()
    }

    pub fn to_u8(&self) -> u8 {
        self.0
    }

    pub fn to_usize(&self) -> usize {
        self.0.into()
    }

    pub fn serialize(&self) -> char {
        char::from_u32(self.to_u32()).expect("should be able to convert a u8 to a char")
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
    fn serialize_can_convert_all_values_to_char() {
        let mut s = String::new();
        (0..=31).for_each(|i| {
            let num = U5::new(i);
            let char = num.serialize();
            s.push(char);
        });
        assert_eq!(s.len(), 32);
    }
}
