use std::{fmt, ops::Add};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Frequency(pub f32);

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Frequency(freq) = self;
        let suffix = '\u{1D41F}';
        write!(f, "{:.3}{}", freq, suffix)
    }
}

impl Add for Frequency {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn display_property() {
        let tests = [0.002, 0.0235, 0.0, 0.6, 0.0285, 0.132];
        tests.into_iter().for_each(|p| {
            let p = Frequency(p);
            println!("The frequency is {}", p);
        })
    }
}
