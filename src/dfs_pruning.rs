use crate::{
    dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, penalty_goal::PenaltyGoals,
    prohibited::Prohibited, solution::Solution,
};
use core::fmt;
use humantime::{format_duration, FormattedDuration};
use rand::{thread_rng, Rng};
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
    SomeKeyTooBig(u32),
    ProhibitedLetters(Keyboard),
    PenaltyTooBig(Solution),
    NotEnoughKeys(usize),
}

impl fmt::Display for PruneReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PruneReason::SomeKeyTooBig(size) => write!(f, "Key too big: {}", size),
            PruneReason::ProhibitedLetters(k) => write!(f, "Prohibited letters: {}", k),
            PruneReason::PenaltyTooBig(solution) => {
                write!(f, "Penalty exceeded: {} ", solution)
            }
            PruneReason::NotEnoughKeys(key_count) => write!(f, "Not enough keys: {}", key_count),
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
            .and_then(|k| match k.len() < 10 {
                true => Err(PruneReason::NotEnoughKeys(k.len())),
                false => Ok(k),
            })
            .and_then(|k| match k.max_key_size() {
                Some(size) if size > max_key_size => Err(PruneReason::SomeKeyTooBig(size)),
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
        let mut evaluated: u128 = 0;
        loop {
            let prune_result = rx.recv().unwrap();
            evaluated = evaluated + 1;
            if evaluated.rem_euclid(10_000) == 0 {
                println!("Evaluated {}", evaluated.separate_with_underscores());
            }
            match prune_result {
                Ok(k) => {
                    if thread_rng().gen_range(1..1_000) == 1 {
                        println!("{}", k);
                    }
                }
                Err(err) => match err {
                    PruneReason::PenaltyTooBig(solution) => {
                        if thread_rng().gen_range(1..10_000) == 1 {
                            println!("{}", solution);
                        }
                    }
                    _ => {}
                },
            }
        }
    });
    for s in solutions {
        println!("{}", s);
    }
    println!("Elapsed time: {}", start_time.elapsed().round_to_seconds());
}
