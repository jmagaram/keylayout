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
    pub fn new(value: f32) -> Frequency {
        debug_assert!(value >= 0.0);
        debug_assert!(!value.is_infinite());
        debug_assert!(!value.is_nan());
        Frequency(value)
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

impl std::convert::From<f32> for Frequency {
    fn from(value: f32) -> Self {
        Frequency::new(value)
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
