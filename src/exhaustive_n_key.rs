use crate::partitions::Partitions;
use crate::{dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, solution::Solution};
use humantime::{format_duration, FormattedDuration};
use std::time::Duration;
use std::time::Instant;
use thousands::Separable;

trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

pub fn find_best_n_key(key_count: u32, dictionary: &Dictionary) -> Option<Solution> {
    let start_time = Instant::now();
    let mut best: Option<Solution> = None;
    let prune = |_k: &Keyboard| -> bool { false };
    let key_sizes = Partitions {
        sum: dictionary.alphabet().len(),
        parts: key_count,
        min: 1,
        max: key_count,
    };
    let keyboards = Keyboard::with_dfs_builder(dictionary.alphabet(), key_sizes, &prune);
    for (index, k) in keyboards.enumerate() {
        let best_penalty = best.as_ref().map(|b| b.penalty()).unwrap_or(Penalty::MAX);
        let penalty = k.penalty(&dictionary, best_penalty);
        if penalty < best_penalty {
            let solution = k.to_solution(
                penalty,
                format!(
                    "{} keys, kbd {}, {}",
                    key_count,
                    index.separate_with_underscores(),
                    start_time.elapsed().round_to_seconds()
                ),
            );
            println!("{}", solution);
            best = Some(solution);
        }
        if index > 0 && index.rem_euclid(10_000) == 0 {
            println!(
                "> seen {} keyboards with {} keys, {}",
                index.separate_with_underscores(),
                key_count,
                start_time.elapsed().round_to_seconds()
            );
        }
    }
    best
}
