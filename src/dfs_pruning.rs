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
            d: &Dictionary,
            prohibited: &Prohibited,
            goals: &PenaltyGoals,
        ) -> KeyboardStatus {
            let k = k.fill_missing(d.alphabet());
            if k.len() == d.alphabet().len() as usize {
                KeyboardStatus::Ok(k.clone().to_solution(Penalty::ZERO, "".to_string()))
            } else {
                match k.has_prohibited_keys(prohibited) {
                    true => KeyboardStatus::HasProhibitedLetters(k.clone()),
                    false => match goals.get(k.len() as u8) {
                        None => KeyboardStatus::Ok(k.to_solution(Penalty::MAX, "".to_string())),
                        Some(penalty_goal) => {
                            let k_penalty = k.penalty(&d, penalty_goal);
                            if k_penalty <= penalty_goal {
                                let solution = k.clone().to_solution(k_penalty, "".to_string());
                                KeyboardStatus::Ok(solution)
                            } else {
                                KeyboardStatus::PenaltyExceeded(k.clone())
                            }
                        }
                    },
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

        pub fn total_at_key_count(&self, key_count: usize) -> u32 {
            self.penalty.count(&key_count)
                + self.letters.count(&key_count)
                + self.ok.count(&key_count)
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
                    self.letters.increment(keyboard.len());
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
                "==================================================================================================="
            )?;
            writeln!(f, "")?;
            writeln!(
                f,
                "Seen:    {} ({:0.}/sec)",
                self.seen.separate_with_underscores(),
                self.seen_per_second().separate_with_underscores()
            )?;
            writeln!(f, "Elapsed: {}", self.started.elapsed().round_to_seconds())?;
            let pct_total = |n: u32| (n as f32) / (self.seen as f32) * 100.0;
            writeln!(f, "")?;
            writeln!(
                f,
                "K    Penalty             Letters             Pruned              Ok                  Total"
            )?;
            let format = |n: u32, pct: f32| {
                let result = format!("{} ({:.0}%)", n.separate_with_underscores(), pct);
                format!("{:<20}", result)
            };
            (10usize..=27)
                .map(|key_count| {
                    let total_at_key_count = self.total_at_key_count(key_count);
                    let pct_of = |n: u32| {
                        if total_at_key_count == 0 {
                            0.0
                        } else {
                            (n as f32) / (total_at_key_count as f32) * 100.0
                        }
                    };
                    let ok = self.ok.count(&key_count);
                    let ok_pct = pct_of(ok);
                    let penalty = self.penalty.count(&key_count);
                    let penalty_pct = pct_of(penalty);
                    let letters = self.letters.count(&key_count);
                    let letters_pct = pct_of(letters);
                    let pruned = letters + penalty;
                    let pruned_pct = pct_of(pruned);
                    let total = self.total_at_key_count(key_count);
                    let total_pct = pct_total(total);
                    writeln!(
                        f,
                        "{:<5}{}{}{}{}{}",
                        key_count,
                        format(penalty, penalty_pct),
                        format(letters, letters_pct),
                        format(pruned, pruned_pct),
                        format(ok, ok_pct),
                        format(total, total_pct),
                    )
                })
                .collect::<Result<(), _>>()?;
            let penalty_total = self.penalty.count_all();
            let letters_total = self.letters.count_all();
            let pruned_total = penalty_total + letters_total;
            let ok_total = self.ok.count_all();
            writeln!(
                f,
                "{:<5}{}{}{}{}",
                "ALL",
                format(penalty_total, pct_total(penalty_total)),
                format(letters_total, pct_total(letters_total)),
                format(pruned_total, pct_total(pruned_total)),
                format(ok_total, pct_total(ok_total)),
            )?;
            writeln!(f, "")?;
            writeln!(f, "K    Best")?;
            (10usize..=27)
                .filter_map(|key_count| {
                    self.best
                        .get(&key_count)
                        .map(|solution| (key_count, solution))
                })
                .map(|(key_count, solution)| {
                    writeln!(
                        f,
                        "{:<3}  {}",
                        key_count,
                        solution.without_keys_with_one_letter()
                    )
                })
                .collect::<Result<(), _>>()?;
            Ok(())
        }
    }
}

pub fn solve() {
    let d = Dictionary::load();
    // top 20%
    let penalty_limits = [
        (11, 0.035212047),
        (12, 0.028836569),
        (13, 0.023573535),
        (14, 0.018724455),
        (15, 0.016393073),
        (16, 0.014338043),
        (17, 0.012302379),
        (18, 0.010197972),
        (19, 0.008511533),
        (20, 0.006951685),
        (21, 0.0055407956),
        (22, 0.0040391646),
        (23, 0.0029261562),
        (24, 0.0017350693),
        (25, 0.00082302664),
        (26, 0.0001239321),
    ];
    let mut goals = PenaltyGoals::none(d.alphabet());
    for (key_count, penalty) in penalty_limits {
        goals.with(key_count, Penalty::new(penalty));
    }
    goals.with(10, Penalty::new(0.03));
    let prohibited = Prohibited::with_top_n_letter_pairs(&d, 60);
    let prune = |k: &Keyboard| KeyboardStatus::new(k, &d, &prohibited, &goals);
    let key_sizes = Partitions {
        sum: 27,
        parts: 10,
        min: 2,
        max: 5,
    };
    let solutions = Keyboard::with_dfs(d.alphabet(), &key_sizes, &prune);
    let mut statistics = statistics::Statistics::new();
    for s in solutions {
        statistics.add(&s);
        if statistics.seen_is_multiple_of(10_000) || statistics.has_new_best() {
            println!("{}", statistics);
        }
        if statistics.seen_is_multiple_of(200_000) {
            println!("");
            println!("===================================================================================================");
            println!("");
            println!("{}", goals);
            println!("");
        }
    }
}

// let standard_penalties = [
//     (26, 0.00006),
//     (25, 0.000174),
//     (24, 0.000385),
//     (23, 0.0007),
//     (22, 0.0012),
//     (21, 0.001974),
//     (20, 0.002559),
//     (19, 0.003633),
//     (18, 0.004623),
//     (17, 0.005569),
//     (16, 0.007603),
//     (15, 0.009746),
//     (14, 0.013027),
//     (13, 0.016709),
//     (12, 0.02109),
//     (11, 0.05),
// ];
