use std::fmt;

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Default)]
pub struct U6(u8);

impl U6 {
    pub const MAX: U6 = U6(31);
    pub const MIN: U6 = U6(0);

    pub fn new(value: u32) -> U6 {
        debug_assert!(value <= 31);
        U6(value as u8)
    }
}

impl fmt::Display for U6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let U6(n) = self;
        let suffix = '\u{2807}';
        write!(f, "{}{}", n, suffix)
    }
}

impl std::convert::From<usize> for U6 {
    fn from(value: usize) -> Self {
        debug_assert!(value <= 31);
        U6((value & 31) as u8)
    }
}

impl std::convert::From<u32> for U6 {
    fn from(value: u32) -> Self {
        debug_assert!(value <= 31);
        U6((value & 31) as u8)
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
            let p = U6(p);
            println!("The number is {}", p);
        })
    }
}
