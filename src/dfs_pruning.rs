use crate::{
    dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, penalty_goal::PenaltyGoals,
    prohibited::Prohibited, solution::Solution, tally::Tally,
};
use core::fmt;
use humantime::{format_duration, FormattedDuration};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use thousands::Separable;

trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

#[derive(Clone)]
enum PruneReason {
    SomeKeyTooBig(Keyboard, u32),
    ProhibitedLetters(Keyboard),
    PenaltyTooBig(Solution),
    NotEnoughKeys(Keyboard, usize),
}

struct ProgressStatistics {
    start_time: Instant,
    seen: u128,
    some_key_too_big: u128,
    prohibited_letters: u128,
    penalty_too_big: u128,
    penalty_too_big_key_count: Tally<usize>,
    not_enough_keys: u128,
    recent_good: Option<Keyboard>,
}

impl ProgressStatistics {
    pub fn new() -> ProgressStatistics {
        ProgressStatistics {
            start_time: Instant::now(),
            seen: 0,
            some_key_too_big: 0,
            prohibited_letters: 0,
            penalty_too_big: 0,
            not_enough_keys: 0,
            penalty_too_big_key_count: Tally::new(),
            recent_good: None,
        }
    }

    pub fn add(&mut self, r: Result<Keyboard, PruneReason>) {
        self.seen = self.seen + 1;
        match r {
            Ok(k) => self.recent_good = Some(k),
            Err(err) => match err {
                PruneReason::ProhibitedLetters(_k) => {
                    self.prohibited_letters = self.prohibited_letters + 1;
                }
                PruneReason::PenaltyTooBig(solution) => {
                    self.penalty_too_big = self.penalty_too_big + 1;
                    self.penalty_too_big_key_count
                        .increment(solution.keyboard().len());
                }
                PruneReason::SomeKeyTooBig(_k, _size) => {
                    self.some_key_too_big = self.some_key_too_big + 1;
                }
                PruneReason::NotEnoughKeys(_keyboard, _size) => {
                    self.not_enough_keys = self.not_enough_keys + 1;
                }
            },
        }
    }
}

impl fmt::Display for ProgressStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seen_per_second = ((self.seen as f32) / self.start_time.elapsed().as_secs_f32()) as i32;
        writeln!(
            f,
            "Keyboards: {} ({}/sec)",
            self.seen.separate_with_underscores(),
            seen_per_second.separate_with_underscores()
        )?;
        writeln!(
            f,
            "Recent: {}",
            self.recent_good
                .clone()
                .map_or("(none)".to_string(), |k| k.to_string())
        )?;
        writeln!(
            f,
            "Prohibited keys: {} ({:.0}%)",
            self.prohibited_letters.separate_with_underscores(),
            100.0 * (self.prohibited_letters as f32) / (self.seen as f32)
        )?;
        writeln!(
            f,
            "Penalty too big: {} ({:.0}%)",
            self.penalty_too_big.separate_with_underscores(),
            100.0 * (self.penalty_too_big as f32) / (self.seen as f32)
        )?;
        let m = (1usize..27).filter_map(|key_count| {
            let count = self.penalty_too_big_key_count.count(&key_count);
            match count == 0 {
                true => None,
                false => Some((key_count, count)),
            }
        });
        m.map(|(key_count, prune_count)| {
            writeln!(
                f,
                "  {} : {} ({:.0}%)",
                key_count,
                prune_count.separate_with_underscores(),
                100.0 * (prune_count as f32) / (self.seen as f32)
            )
        })
        .collect::<Result<(), _>>()?;
        writeln!(
            f,
            "Elapsed: {}",
            self.start_time.elapsed().round_to_seconds()
        )?;
        Ok(())
    }
}

impl fmt::Display for PruneReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PruneReason::SomeKeyTooBig(_, size) => write!(f, "Key too big: {}", size),
            PruneReason::ProhibitedLetters(k) => write!(f, "Prohibited letters: {}", k),
            PruneReason::PenaltyTooBig(solution) => {
                write!(f, "Penalty exceeded: {} ", solution)
            }
            PruneReason::NotEnoughKeys(_, key_count) => write!(f, "Not enough keys: {}", key_count),
        }
    }
}

pub fn solve() {
    let start_time = Instant::now();
    let d = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&d, 40);
    let max_key_size = 4;
    let penalty_goals = PenaltyGoals::none(d.alphabet())
        // .with(26, Penalty::new(0.00006))
        // .with(25, Penalty::new(0.000174))
        // .with(24, Penalty::new(0.000385))
        .with(23, Penalty::new(0.0007))
        .with(22, Penalty::new(0.0012))
        .with(21, Penalty::new(0.001985))
        .with(20, Penalty::new(0.0003152))
        .with(19, Penalty::new(0.0037))
        .with(18, Penalty::new(0.004739))
        .with(17, Penalty::new(0.005092))
        .with(16, Penalty::new(0.00825))
        .with(15, Penalty::new(0.009746))
        .with(14, Penalty::new(0.013445))
        .with(13, Penalty::new(0.016709))
        .with(12, Penalty::new(0.02109))
        // .with_adjustment(12..=25, 0.8)
        .with(10, Penalty::new(0.0246));
    let prune_result = |k: &Keyboard| -> Result<Keyboard, PruneReason> {
        Ok(k.clone())
            .and_then(|k| {
                let len = &k.len();
                match k.len() < 10 {
                    true => Err(PruneReason::NotEnoughKeys(k, *len)),
                    false => Ok(k),
                }
            })
            .and_then(|k| match k.max_key_size() {
                Some(size) if size > max_key_size => Err(PruneReason::SomeKeyTooBig(k, size)),
                _ => Ok(k),
            })
            .and_then(|k| match k.has_prohibited_keys(&prohibited) {
                false => Ok(k),
                true => Err(PruneReason::ProhibitedLetters(k)),
            })
            .and_then(|k| {
                let penalty_to_beat = penalty_goals.get(k.len() as u8).unwrap_or(Penalty::MAX);
                let actual_penalty = k.penalty(&d, penalty_to_beat);
                match actual_penalty > penalty_to_beat {
                    true => Err(PruneReason::PenaltyTooBig(
                        k.to_solution(actual_penalty, "".to_string()),
                    )),
                    false => Ok(k),
                }
            })
    };
    let start = Keyboard::with_every_letter_on_own_key(d.alphabet());
    let (tx, rx) = mpsc::channel::<Result<Keyboard, PruneReason>>();
    let prune = |k: &Keyboard| -> bool {
        let result = prune_result(k);
        let should_prune = result.is_err();
        tx.send(result).unwrap();
        should_prune
    };
    let solutions = start
        .every_smaller_with(&prune)
        .filter(|k| k.len() == 10)
        .map(|k| {
            let penalty = k.penalty(&d, Penalty::MAX);
            k.to_solution(penalty, "".to_string())
        });
    let _join_handle = thread::spawn(move || {
        let mut progress_stats = ProgressStatistics::new();
        loop {
            let prune_result = rx.recv().unwrap();
            progress_stats.add(prune_result);
            if progress_stats.seen.rem_euclid(50_000) == 0 {
                println!("{}", progress_stats);
            }
        }
    });
    for s in solutions {
        println!("{}", s);
    }
    println!("Elapsed time: {}", start_time.elapsed().round_to_seconds());
}
