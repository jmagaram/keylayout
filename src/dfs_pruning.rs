use crate::{
    dfs_pruning::keyboard_status::KeyboardStatus, dictionary::Dictionary, keyboard::Keyboard,
    partitions::Partitions, penalty::Penalty, penalty_goal::PenaltyGoals, prohibited::Prohibited,
};
use core::fmt;
use humantime::{format_duration, FormattedDuration};

use std::{sync::mpsc, thread, time::Duration};
use thousands::Separable;

trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

pub mod keyboard_status {
    use std::fmt;

    use crate::{
        dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, penalty_goal::PenaltyGoals,
        prohibited::Prohibited, solution::Solution,
    };

    #[derive(Clone)]
    pub enum KeyboardStatus {
        Ok(Solution),
        HasProhibitedLetters(Keyboard),
        PenaltyExceeded(Keyboard),
    }

    impl KeyboardStatus {
        pub fn is_ok(&self) -> bool {
            match self {
                KeyboardStatus::Ok(_) => true,
                _ => false,
            }
        }

        pub fn evaluate(
            k: &Keyboard,
            dictionary: &Dictionary,
            prohibited: &Prohibited,
            goals: &PenaltyGoals,
        ) -> KeyboardStatus {
            if k.len() < 2 {
                KeyboardStatus::Ok(k.clone().to_solution(Penalty::ZERO, "".to_string()))
            } else {
                match k.has_prohibited_keys(prohibited) {
                    true => KeyboardStatus::HasProhibitedLetters(k.clone()),
                    false => {
                        let penalty_goal = goals.get(27 - k.len() as u8).unwrap_or(Penalty::MAX);
                        let k_filled = k.fill_missing(dictionary.alphabet());
                        let penalty = k_filled.penalty(&dictionary, penalty_goal);
                        let solution = k.clone().to_solution(penalty, "".to_string());
                        if penalty <= penalty_goal {
                            KeyboardStatus::Ok(solution)
                        } else {
                            KeyboardStatus::PenaltyExceeded(k.clone())
                        }
                    }
                }
            }
        }
    }

    impl fmt::Display for KeyboardStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                KeyboardStatus::Ok(s) => {
                    write!(f, "ok:      {}", s)
                }
                KeyboardStatus::HasProhibitedLetters(k) => {
                    write!(f, "letters: {}", k)
                }
                KeyboardStatus::PenaltyExceeded(k) => {
                    write!(f, "penalty: {}", k)
                }
            }
        }
    }
}

pub mod statistics {
    use std::fmt;

    use thousands::Separable;

    use super::keyboard_status::KeyboardStatus;

    pub struct Statistics {
        seen: u128,
    }

    impl Statistics {
        pub fn new() -> Statistics {
            Statistics { seen: 0 }
        }

        pub fn add(&mut self, _status: &KeyboardStatus) {
            self.seen = self.seen + 1;
        }

        pub fn seen_is_multiple_of(&self, n: u128) -> bool {
            self.seen.rem_euclid(n) == 0
        }
    }

    impl fmt::Display for Statistics {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "Seen: {}", self.seen.separate_with_underscores())
        }
    }
}

pub fn solve() {
    let d = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&d, 80);
    let (tx, rx) = mpsc::channel::<KeyboardStatus>();
    let goals = PenaltyGoals::none(d.alphabet())
        .with(26, Penalty::new(0.00006))
        .with(25, Penalty::new(0.000174))
        .with(24, Penalty::new(0.000385))
        .with(23, Penalty::new(0.0007))
        .with(22, Penalty::new(0.0012))
        .with(21, Penalty::new(0.001974))
        .with(20, Penalty::new(0.002559))
        .with(19, Penalty::new(0.003633))
        .with(18, Penalty::new(0.004623))
        .with(17, Penalty::new(0.005569))
        .with(16, Penalty::new(0.007603))
        .with(15, Penalty::new(0.009746))
        .with(14, Penalty::new(0.013027))
        .with(13, Penalty::new(0.016709))
        .with(12, Penalty::new(0.02109))
        // .with_adjustment(11..=23, 5.0)
        .with(10, Penalty::new(0.0246));
    let prune = |k: &Keyboard| -> bool {
        let result = keyboard_status::KeyboardStatus::evaluate(k, &d, &prohibited, &goals);
        let result_is_ok = result.is_ok();
        let should_prune = !result_is_ok;
        tx.send(result).unwrap();
        should_prune
    };
    let key_sizes = Partitions {
        sum: 27,
        parts: 10,
        min: 2,
        max: 3,
    };
    let inspect = |_k: &Keyboard| {};
    let solutions =
        Keyboard::with_dfs_builder(d.alphabet(), key_sizes, &prune, &inspect).map(|k| {
            let penalty = k.penalty(&d, Penalty::MAX);
            k.to_solution(penalty, "".to_string())
        });
    let _join_handle = thread::spawn(move || {
        let mut statistics = statistics::Statistics::new();
        loop {
            let keyboard_status = rx.recv();
            match keyboard_status {
                Ok(keyboard_status) => {
                    statistics.add(&keyboard_status);
                    if statistics.seen_is_multiple_of(10) {
                        println!("{}", keyboard_status);
                    }
                }
                Err(err) => {
                    println!("{}", err)
                }
            }
        }
    });
    for s in solutions {
        println!("SOLVED {}", s);
    }
    println!("=== DONE ===");
}

// struct KeyCountTotals {
//     ok: u32,
//     prohibited_letters: u32,
//     penalty: u32,
// }

// impl KeyCountTotals {
//     pub fn new() -> KeyCountTotals {
//         KeyCountTotals {
//             ok: 0,
//             penalty: 0,
//             prohibited_letters: 0,
//         }
//     }
// }

// struct Statistics {
//     start_time: Instant,
//     seen: u32,
//     by_key_count: HashMap<usize, KeyCountTotals>,
//     recent_ok: Option<Keyboard>,
// }

// impl Statistics {
//     pub fn new() -> Statistics {
//         Statistics {
//             start_time: Instant::now(),
//             by_key_count: HashMap::new(),
//             recent_ok: None,
//             seen: 0,
//         }
//     }

//     pub fn prohibited_total(&self) -> u32 {
//         self.by_key_count
//             .values()
//             .map(|v| v.prohibited_letters)
//             .sum()
//     }

//     pub fn penalty_total(&self) -> u32 {
//         self.by_key_count.values().map(|v| v.penalty).sum()
//     }

// impl fmt::Display for Statistics {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let seen_per_second = ((self.seen as f32) / self.start_time.elapsed().as_secs_f32()) as i32;
//         fn write_num(
//             f: &mut fmt::Formatter<'_>,
//             caption: &str,
//             num: u32,
//             total: u32,
//         ) -> Result<(), std::fmt::Error> {
//             writeln!(
//                 f,
//                 "{} {} ({:.0}%)",
//                 caption,
//                 num.separate_with_underscores(),
//                 100.0 * (num as f32) / (total as f32)
//             )
//         }
//         writeln!(
//             f,
//             "Keyboards:    {} ({}/sec)",
//             self.seen.separate_with_underscores(),
//             seen_per_second.separate_with_underscores()
//         )?;
//         writeln!(
//             f,
//             "Recent:       {}",
//             self.recent_ok
//                 .clone()
//                 .map_or("(none)".to_string(), |k| k.to_string())
//         )?;
//         writeln!(
//             f,
//             "Elapsed:      {}",
//             self.start_time.elapsed().round_to_seconds()
//         )?;
//         write_num(f, "Prohibited:  ", self.prohibited_total(), self.seen)?;
//         write_num(f, "Penalty:     ", self.penalty_total(), self.seen)?;
//         (2usize..=10)
//             .filter_map(|key_count| {
//                 self.by_key_count
//                     .get(&key_count)
//                     .map(|i| (key_count, i.penalty))
//             })
//             .map(|(key_count, prune_count)| {
//                 writeln!(
//                     f,
//                     "                 {} : {} ({:.1}%)",
//                     key_count,
//                     prune_count.separate_with_underscores(),
//                     100.0 * (prune_count as f32) / (self.seen as f32)
//                 )
//             })
//             .collect::<Result<(), _>>()?;
//         Ok(())
//     }
// }
