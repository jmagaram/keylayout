use std::fmt;

use crate::{keyboard::Keyboard, penalty::Penalty};

#[derive(Clone)]
pub struct Solution {
    keyboard: Keyboard,
    penalty: Penalty,
}

impl Solution {
    pub fn new(keyboard: Keyboard, penalty: Penalty) -> Solution {
        Solution { keyboard, penalty }
    }

    pub fn penalty(&self) -> Penalty {
        self.penalty
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.penalty, self.keyboard)
    }
}
