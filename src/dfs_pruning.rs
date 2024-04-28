use crate::{
    dfs_pruning::keyboard_status::KeyboardStatus, dictionary::Dictionary, keyboard::Keyboard,
    partitions::Partitions, penalty::Penalty, penalty_goal::PenaltyGoals, prohibited::Prohibited,
};
use humantime::{format_duration, FormattedDuration};
use std::time::Duration;

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
        dictionary::Dictionary,
        keyboard::{Keyboard, Pruneable},
        penalty::Penalty,
        penalty_goal::PenaltyGoals,
        prohibited::Prohibited,
        solution::Solution,
    };

    #[derive(Clone)]
    pub enum KeyboardStatus {
        Ok(Solution),
        HasProhibitedLetters(Keyboard),
        PenaltyExceeded(Keyboard),
    }

    impl Pruneable for KeyboardStatus {
        fn should_prune(&self) -> bool {
            match self {
                KeyboardStatus::Ok(_) => false,
                _ => true,
            }
        }
    }

    impl KeyboardStatus {
        pub fn new(
            k: &Keyboard,
            dictionary: &Dictionary,
            prohibited: &Prohibited,
            goals: &PenaltyGoals,
        ) -> KeyboardStatus {
            if k.len() == 0 {
                KeyboardStatus::Ok(k.clone().to_solution(Penalty::ZERO, "".to_string()))
            } else {
                match k.has_prohibited_keys(prohibited) {
                    true => KeyboardStatus::HasProhibitedLetters(k.clone()),
                    false => {
                        // 27 - number of letters on keyboard
                        // af bq
                        // 27 + 2 - 4 = 25
                        let penalty_goal = goals
                            .get(27 + k.len() as u8 - k.letters().count_letters() as u8)
                            .unwrap_or(Penalty::MAX);
                        let k_filled = k.fill_missing(dictionary.alphabet());
                        let penalty = k_filled.penalty(&dictionary, penalty_goal);
                        if penalty <= penalty_goal {
                            let solution = k.clone().to_solution(penalty, "".to_string());
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
    use super::{keyboard_status::KeyboardStatus, DurationFormatter};
    use crate::{solution::Solution, tally::Tally};
    use hashbrown::HashMap;
    use std::{fmt, time::Instant};
    use thousands::Separable;

    pub struct Statistics {
        seen: u128,
        ok: Tally<usize>,
        penalty: Tally<usize>,
        letters: Tally<usize>,
        best: HashMap<usize, Solution>,
        started: Instant,
        has_new_best: bool,
    }

    impl Statistics {
        pub fn new() -> Statistics {
            Statistics {
                seen: 0,
                penalty: Tally::new(),
                letters: Tally::new(),
                ok: Tally::new(),
                best: HashMap::new(),
                started: Instant::now(),
                has_new_best: false,
            }
        }

        pub fn seen_per_second(&self) -> i32 {
            ((self.seen as f32) / (self.started.elapsed().as_secs_f32())) as i32
        }

        pub fn add(&mut self, status: &KeyboardStatus) {
            self.seen = self.seen + 1;
            self.has_new_best = false;
            match &status {
                &KeyboardStatus::Ok(solution) => {
                    let key_count = solution.keyboard().len();
                    self.ok.increment(key_count);
                    match self.best.get(&key_count) {
                        None => {
                            self.best.insert(key_count, solution.clone());
                            self.has_new_best = true;
                        }
                        Some(best) => {
                            if solution.penalty() < best.penalty() {
                                self.best.insert(key_count, solution.clone());
                                self.has_new_best = true;
                            }
                        }
                    }
                }
                &KeyboardStatus::HasProhibitedLetters(keyboard) => {
                    let key_count = keyboard.len();
                    self.letters.increment(key_count);
                }
                &KeyboardStatus::PenaltyExceeded(keyboard) => {
                    self.penalty.increment(keyboard.len());
                }
            }
        }

        pub fn seen_is_multiple_of(&self, n: u128) -> bool {
            self.seen.rem_euclid(n) == 0
        }

        pub fn has_new_best(&self) -> bool {
            self.has_new_best
        }
    }

    impl fmt::Display for Statistics {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(
                f,
                "===================================================================="
            )?;
            writeln!(f, "")?;
            writeln!(
                f,
                "Seen:    {} ({:0.}/sec)",
                self.seen.separate_with_underscores(),
                self.seen_per_second().separate_with_underscores()
            )?;
            writeln!(f, "Elapsed: {}", self.started.elapsed().round_to_seconds())?;
            let pct = |n: u32| (n as f32) / (self.seen as f32) * 100.0;
            writeln!(f, "")?;
            writeln!(
                f,
                "K    Penalty           Letters           Pruned            Ok"
            )?;
            (2usize..=10)
                .map(|key_count| {
                    let ok = self.ok.count(&key_count);
                    let ok_pct = pct(ok);
                    let penalty = self.penalty.count(&key_count);
                    let penalty_pct = pct(penalty);
                    let letters = self.letters.count(&key_count);
                    let letters_pct = pct(letters);
                    let pruned = letters + penalty;
                    let pruned_pct = pct(pruned);
                    let format = |n: u32, pct: f32| {
                        let result = format!("{} ({:.1}%)", n.separate_with_underscores(), pct);
                        format!("{:<18}", result)
                    };
                    writeln!(
                        f,
                        "{:<5}{}{}{}{}",
                        key_count,
                        format(penalty, penalty_pct),
                        format(letters, letters_pct),
                        format(pruned, pruned_pct),
                        format(ok, ok_pct),
                    )
                })
                .collect::<Result<(), _>>()?;
            writeln!(f, "")?;
            writeln!(f, "K    Best")?;
            (2usize..=10)
                .filter_map(|key_count| {
                    self.best
                        .get(&key_count)
                        .map(|solution| (key_count, solution))
                })
                .map(|(key_count, solution)| writeln!(f, "{:<3}  {}", key_count, solution))
                .collect::<Result<(), _>>()?;
            Ok(())
        }
    }
}

pub fn solve() {
    let d = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&d, 10);
    let goals = PenaltyGoals::none(d.alphabet())
        .with(26, Penalty::new(0.00006)) // 1 key with 2 letters
        .with(25, Penalty::new(0.000174)) // 1 key with 3 letters
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
        .with(11, Penalty::new(1.02109))
        .with_adjustment(11..=26, 5.0)
        .with(10, Penalty::new(0.0246));
    // .with(10, Penalty::MAX);
    let prune = |k: &Keyboard| KeyboardStatus::new(k, &d, &prohibited, &goals);
    // investigate the penalty scores, 10 or 27
    // only gettng to 10 if parts is 11
    // stopped at 0 doing nothing if ...? didn't prune anything
    let key_sizes = Partitions {
        sum: 27,
        parts: 11,
        min: 2,
        max: 3,
    };
    let solutions = Keyboard::with_dfs(d.alphabet(), &key_sizes, &prune);
    let mut statistics = statistics::Statistics::new();
    for s in solutions {
        statistics.add(&s);
        if statistics.seen_is_multiple_of(100_000) || statistics.has_new_best() {
            println!("{}", statistics);
        }
    }
}
