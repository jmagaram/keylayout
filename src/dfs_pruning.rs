use crate::{
    dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, penalty_goal::PenaltyGoals,
    prohibited::Prohibited,
};
use humantime::{format_duration, FormattedDuration};
use std::time::{Duration, Instant};

trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

pub fn solve() {
    let start_time = Instant::now();
    let d = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&d, 50);
    let max_key_size = 4;
    let penalty_goals = PenaltyGoals::none(d.alphabet())
        .with(26, Penalty::new(0.00006))
        .with(25, Penalty::new(0.000174))
        .with(24, Penalty::new(0.000385))
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
        .with_adjustment(12..=25, 1.2)
        .with(10, Penalty::new(0.0246));
    let prune = |k: &Keyboard| -> bool {
        let key_count = k.key_count() as u8;
        let penalty_exceeds_threshold = || {
            let penalty_to_beat = penalty_goals.get(key_count).unwrap_or(Penalty::MAX);
            let actual_penalty = k.penalty(&d, penalty_to_beat);
            actual_penalty > penalty_to_beat
        };
        let has_prohibited_keys = || k.has_prohibited_keys(&prohibited);
        let some_key_too_big = || k.max_key_size().map_or(false, |size| size > max_key_size);
        key_count < 10 || some_key_too_big() || has_prohibited_keys() || penalty_exceeds_threshold()
    };
    let start = Keyboard::with_every_letter_on_own_key(d.alphabet());
    let solutions = start
        .every_smaller_with(&prune)
        .filter(|k| k.key_count() == 10)
        .map(|k| {
            let penalty = k.penalty(&d, Penalty::MAX);
            k.to_solution(penalty, "".to_string())
        });
    for s in solutions {
        println!("{}", s);
    }
    println!("Elapsed time: {}", start_time.elapsed().round_to_seconds());
}
