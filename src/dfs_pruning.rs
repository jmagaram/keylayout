use crate::{
    dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, penalty_goal::PenaltyGoals,
    prohibited::Prohibited,
};
use humantime::{format_duration, FormattedDuration};
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};

trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

enum PruneReason {
    SomeKeyTooBig(u32),
    ProhibitedLetters,
    PenaltyTooBig(Penalty),
    NotEnoughKeys(usize),
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
    let prune_result = |k: &Keyboard| -> Result<(), PruneReason> {
        Ok(())
            .inspect(|_| {
                println!("Evaluating {}", k);
            })
            .and_then(|_| match k.len() < 10 {
                true => Err(PruneReason::NotEnoughKeys(k.len())),
                false => Ok(()),
            })
            .and_then(|_| match k.max_key_size() {
                Some(size) if size > max_key_size => Err(PruneReason::SomeKeyTooBig(size)),
                _ => Ok(()),
            })
            .and_then(|_| match k.has_prohibited_keys(&prohibited) {
                false => Ok(()),
                true => Err(PruneReason::ProhibitedLetters),
            })
            .and_then(|_| {
                let penalty_to_beat = penalty_goals.get(k.len() as u8).unwrap_or(Penalty::MAX);
                let actual_penalty = k.penalty(&d, penalty_to_beat);
                match actual_penalty > penalty_to_beat {
                    true => Err(PruneReason::PenaltyTooBig(actual_penalty)),
                    false => Ok(()),
                }
            })
    };
    let start = Keyboard::with_every_letter_on_own_key(d.alphabet());
    let prune = |k: &Keyboard| -> bool { prune_result(k).is_err() };
    let solutions = start
        .every_smaller_with(&prune)
        .filter(|k| k.len() == 10)
        .map(|k| {
            let penalty = k.penalty(&d, Penalty::MAX);
            k.to_solution(penalty, "".to_string())
        });
    for s in solutions {
        println!("{}", s);
    }
    println!("Elapsed time: {}", start_time.elapsed().round_to_seconds());
}
