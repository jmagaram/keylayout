use std::{fmt, ops::Add};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Default)]
pub struct Penalty(f32);

impl fmt::Display for Penalty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Penalty(penalty) = self;
        let suffix = '\u{1D561}';
        write!(f, "{:.4}{}", penalty, suffix)
    }
}

impl Ord for Penalty {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl Eq for Penalty {}

impl Penalty {
    pub fn new(value: f32) -> Penalty {
        Penalty::try_from(value).unwrap()
    }

    pub fn to_f32(&self) -> f32 {
        self.0
    }

    pub const ZERO: Penalty = Penalty(0.0);
    pub const MAX: Penalty = Penalty(std::f32::MAX);
}

impl TryFrom<f32> for Penalty {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            Err("Penalty cannot be infinite or NAN.")
        } else if value.is_sign_negative() {
            Err("Penalty must be zero or positive.")
        } else {
            Ok(Penalty(value))
        }
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
    fn try_from_when_valid() {
        let data = [0.0, 0.1, 99.9];
        for d in data {
            let actual = Penalty::try_from(d).unwrap();
            assert_eq!(actual.to_f32(), d);
        }
    }

    #[test]
    fn try_from_when_invalid() {
        let data = [-0.0, -1.0, f32::INFINITY, f32::NAN, f32::NEG_INFINITY];
        for d in data {
            let actual = Penalty::try_from(d);
            assert!(actual.is_err())
        }
    }

    #[test]
    #[ignore]
    fn display() {
        let tests = [0.002, 0.0235, 0.0, 1.43, 0.0285, 6.0];
        tests.into_iter().for_each(|p| {
            let p = Penalty(p);
            println!("The penalty is {}", p);
        })
    }
}
