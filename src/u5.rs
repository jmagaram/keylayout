use std::fmt;

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Default)]
pub struct U5(u8);

impl U5 {
    pub const MAX: U5 = U5(31);
    pub const MIN: U5 = U5(0);
    pub const MAX_VALUES: U5 = U5(32);

    pub fn new(value: u32) -> U5 {
        assert!(value <= 31);
        U5(value as u8)
    }

    pub fn to_u32(&self) -> u32 {
        self.0.into()
    }

    pub fn to_usize(&self) -> usize {
        self.0.into()
    }

    pub fn to_char(&self) -> char {
        char::from_u32(self.0 as u32).unwrap()
    }
}

impl fmt::Display for U5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let U5(n) = self;
        let _suffix = '\u{2807}';
        write!(f, "{}", n)
    }
}

impl std::convert::From<usize> for U5 {
    fn from(value: usize) -> Self {
        assert!(value <= 31);
        U5((value & 31) as u8)
    }
}

impl std::convert::From<u32> for U5 {
    fn from(value: u32) -> Self {
        assert!(value <= 31);
        U5((value & 31) as u8)
    }
}

impl std::convert::From<i32> for U5 {
    fn from(value: i32) -> Self {
        assert!(value >= 0 && value <= 31);
        U5((value & 31) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn display_property() {
        let tests = [0, 1, 2, 13, 31];
        tests.into_iter().for_each(|p| {
            let p = U5(p);
            println!("The number is {}", p);
        })
    }

    #[test]
    fn to_char_works_for_all() {
        let mut s = String::new();
        (0..=31).for_each(|i| {
            let num = U5::new(i);
            let char = num.to_char();
            s.push(char);
        });
        assert_eq!(s.len(), 32);
    }
}
