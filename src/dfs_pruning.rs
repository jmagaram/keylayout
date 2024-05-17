use crate::{
    dfs_pruning::keyboard_status::KeyboardStatus,
    dictionary::Dictionary,
    keyboard::Keyboard,
    partitions::Partitions,
    penalty::Penalty,
    penalty_goal::{self, PenaltyGoals, ProhibitedPairs},
    prohibited::Prohibited,
};
use core::fmt;
use dialoguer::{Input, Select};
use thousands::Separable;

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
    use super::keyboard_status::KeyboardStatus;
    use crate::{solution::Solution, tally::Tally, util::DurationFormatter};
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

pub struct SolveArgs {
    dictionary_size: Option<usize>,
    prohibited_pairs: u8,
    max_key_size: u8,
    penalty_based_on_samples_with: penalty_goal::ProhibitedPairs,
    penalty_top_percent: u8,
    penalty_goal_from_key_count: u8,
    penalty_goal_until_key_count: u8,
    penalty_multiplier: f32,
    penalty_for_10_keys: f32,
    display_progress_every: u128,
}

impl fmt::Display for SolveArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Dictionary size: {}",
            self.dictionary_size
                .map(|i| i.separate_with_underscores())
                .unwrap_or("full".to_string())
        )?;
        writeln!(f, "Prohibited pairs: {}", self.prohibited_pairs)?;
        writeln!(f, "Maximum key size: {}", self.max_key_size)?;
        writeln!(f, "Penalty pruning:")?;
        writeln!(
            f,
            "  Samples have prohibited pairs: {}",
            self.penalty_based_on_samples_with
        )?;
        writeln!(
            f,
            "  Threshold is top percent: {}",
            self.penalty_top_percent
        )?;
        writeln!(f, "  From key count: {}", self.penalty_goal_from_key_count)?;
        writeln!(
            f,
            "  Until key count: {}",
            self.penalty_goal_until_key_count
        )?;
        writeln!(f, "  Multiplier: {}", self.penalty_multiplier)?;
        writeln!(f, "  10 key goal: {}", self.penalty_for_10_keys)?;
        Ok(())
    }
}

impl SolveArgs {
    pub fn preconfigured() -> SolveArgs {
        SolveArgs {
            dictionary_size: Some(120_000),
            display_progress_every: 10_000,
            max_key_size: 5,
            penalty_based_on_samples_with: ProhibitedPairs::Num60,
            penalty_for_10_keys: 0.030,
            penalty_goal_from_key_count: 11,
            penalty_goal_until_key_count: 24,
            penalty_multiplier: 1.0,
            penalty_top_percent: 20,
            prohibited_pairs: 70,
        }
    }

    pub fn new_from_prompts() -> SolveArgs {
        let dictionary_size_index = Select::new()
            .with_prompt("Dictionary size")
            .item("Entire (307_000)")
            .item("Significant (120_000 words)")
            .item("Small (90_000)")
            .item("Very small (25_000")
            .item("Tiny (5_000")
            .default(0)
            .interact()
            .unwrap();
        let dictionary_size = match dictionary_size_index {
            0 => None,
            1 => Some(120_000),
            2 => Some(90_000),
            3 => Some(25_000),
            4 => Some(5_000),
            _ => panic!("Do not know how to handle that input for dictionary size."),
        };
        let prohibited_pairs = Input::<u8>::new()
            .with_prompt("Prohibited letter pairs")
            .default(40)
            .interact_text()
            .unwrap();
        let max_key_size = Input::<u8>::new()
            .with_prompt("Maximum letters per key")
            .default(5)
            .interact_text()
            .unwrap();
        let penalty_goals_based_on_index = Select::new()
            .with_prompt("Penalty goals based on which random samples")
            .item("No prohibited pairs")
            .item("20 prohibited pairs")
            .item("40 prohibited pairs")
            .item("60 prohibited pairs")
            .item("80 prohibited pairs")
            .default(0)
            .interact()
            .unwrap();
        let penalty_goals_based_on = match penalty_goals_based_on_index {
            0 => ProhibitedPairs::Zero,
            1 => ProhibitedPairs::Num20,
            2 => ProhibitedPairs::Num40,
            3 => ProhibitedPairs::Num60,
            4 => ProhibitedPairs::Num80,
            _ => {
                panic!("Unexpected penalty goal index");
            }
        };
        let penalty_goal_from_key_count = Input::<u8>::new()
            .with_prompt("Penalty goal from key count")
            .default(11)
            .interact_text()
            .unwrap();
        let penalty_goal_until_key_count = Input::<u8>::new()
            .with_prompt("Penalty goal to key count")
            .default(24)
            .interact_text()
            .unwrap();
        let penalty_goal_top_percent = Input::<u8>::new()
            .with_prompt("Penalties must in the top percent of random samples")
            .default(25)
            .interact_text()
            .unwrap();
        let penalty_goal_adjustment = Input::<f32>::new()
            .with_prompt("Penalty limit multiplier")
            .default(1.0)
            .interact_text()
            .unwrap();
        let penalty_goal_for_10_keys = Input::<f32>::new()
            .with_prompt("Penalty goal for 10 key solution")
            .default(0.03)
            .interact_text()
            .unwrap();
        let display_progress_every = Input::<u128>::new()
            .with_prompt("Print status every n keyboards evaluated?")
            .default(10_000)
            .interact_text()
            .unwrap();
        SolveArgs {
            dictionary_size,
            display_progress_every,
            max_key_size,
            penalty_based_on_samples_with: penalty_goals_based_on,
            penalty_top_percent: penalty_goal_top_percent,
            penalty_multiplier: penalty_goal_adjustment,
            penalty_for_10_keys: penalty_goal_for_10_keys,
            penalty_goal_from_key_count,
            penalty_goal_until_key_count,
            prohibited_pairs,
        }
    }
}

pub fn solve(args: &SolveArgs) {
    let d = match args.dictionary_size {
        None => Dictionary::load(),
        Some(count) => Dictionary::load().filter_top_n_words(count),
    };
    let mut goals = PenaltyGoals::none(d.alphabet());
    goals.use_random_samples(
        args.penalty_goal_from_key_count..=args.penalty_goal_until_key_count,
        args.penalty_based_on_samples_with,
        (args.penalty_top_percent as f32) / 100.0,
        args.penalty_multiplier,
    );
    goals.with(10, Penalty::new(args.penalty_for_10_keys));
    let prohibited = Prohibited::with_top_n_letter_pairs(&d, args.prohibited_pairs as usize);
    let prune = |k: &Keyboard| KeyboardStatus::new(k, &d, &prohibited, &goals);
    let key_sizes = Partitions {
        sum: 27,
        parts: 10,
        min: 1,
        max: args.max_key_size,
    };
    let solutions = Keyboard::with_dfs(d.alphabet(), &key_sizes, &prune);
    let mut statistics = statistics::Statistics::new();
    for s in solutions {
        statistics.add(&s);
        if statistics.seen_is_multiple_of(args.display_progress_every * 10) {
            println!("{}", statistics);
            println!("");
            println!("{}", args);
            println!("");
            println!("{}", goals);
            println!("");
        } else if statistics.seen_is_multiple_of(args.display_progress_every)
            || statistics.has_new_best()
        {
            println!("{}", statistics);
            println!();
        }
    }
}
