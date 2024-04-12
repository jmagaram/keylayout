use std::{cmp::Ordering, fmt, ops::Add};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Default)]
pub struct Frequency(f32);

impl Ord for Frequency {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl Eq for Frequency {}

impl Frequency {
    fn validate(value: f32) -> Result<Frequency, &'static str> {
        if value.is_nan() {
            Err("Frequency values can not be NAN.")
        } else if value.is_infinite() {
            Err("Frequency values must be finite.")
        } else if value.is_sign_negative() {
            Err("Frequency values must be zero or positive.")
        } else {
            Ok(Frequency(value))
        }
    }

    pub fn new(value: f32) -> Frequency {
        Frequency::validate(value).unwrap()
    }

    pub fn random() -> Frequency {
        Frequency::validate(rand::random()).unwrap()
    }

    pub fn to_f32(&self) -> f32 {
        self.0
    }

    pub const ZERO: Frequency = Frequency(0.0);
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Frequency(freq) = self;
        let suffix = '\u{1D41F}';
        write!(f, "{:.3}{}", freq, suffix)
    }
}

impl TryFrom<f32> for Frequency {
    type Error = &'static str;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Frequency::validate(value)
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
    fn try_from_when_valid() {
        let data = [0.0, 0.1, 99.9];
        for d in data {
            let actual = Frequency::try_from(d).unwrap();
            assert_eq!(actual.to_f32(), d);
        }
    }

    #[test]
    fn try_from_when_out_of_range() {
        let data = [-1.0, f32::INFINITY, f32::NAN, f32::NEG_INFINITY];
        for d in data {
            let actual = Frequency::try_from(d);
            assert!(actual.is_err())
        }
    }

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
