use std::{fmt, ops::Add};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Penalty(f32);

impl fmt::Display for Penalty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Penalty(penalty) = self;
        let suffix = '\u{1D561}';
        write!(f, "{:.3}{}", penalty, suffix)
    }
}

impl Penalty {
    pub fn new(value: f32) -> Penalty {
        Penalty(value)
    }

    pub fn to_f32(&self) -> f32 {
        self.0
    }

    pub const ZERO: Penalty = Penalty(0.0);
}

impl std::convert::From<f32> for Penalty {
    fn from(value: f32) -> Self {
        Penalty::new(value)
    }
}

impl Add for Penalty {
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
        let tests = [0.002, 0.0235, 0.0, 1.43, 0.0285, 6.0];
        tests.into_iter().for_each(|p| {
            let p = Penalty(p);
            println!("The penalty is {}", p);
        })
    }
}
