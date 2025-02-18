use std::fmt;

use crate::{keyboard::Keyboard, penalty::Penalty};

#[derive(Clone)]
pub struct Solution {
    keyboard: Keyboard,
    penalty: Penalty,
    notes: String,
}

impl Solution {
    pub fn new(keyboard: Keyboard, penalty: Penalty, notes: String) -> Solution {
        Solution {
            keyboard,
            penalty,
            notes,
        }
    }

    pub fn penalty(&self) -> Penalty {
        self.penalty
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn notes(&self) -> String {
        self.notes.to_string()
    }

    pub fn without_keys_with_one_letter(&self) -> Solution {
        Solution {
            keyboard: self.keyboard.without_keys_with_one_letter(),
            notes: self.notes.clone(),
            penalty: self.penalty,
        }
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let penalty = if self.penalty == Penalty::MAX {
            "-------".to_string()
        } else {
            format!("{}", self.penalty)
        };
        if self.notes.len() > 0 {
            write!(f, "{} {} | {}", penalty, self.keyboard, self.notes)
        } else {
            write!(f, "{} {} ", penalty, self.keyboard)
        }
    }
}
