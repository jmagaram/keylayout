use crate::dictionary;
use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    solution::Solution, tally::Tally,
};
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

/// Finds the best keyboard with `count` keys by exhaustively examining every keyboard.
pub fn find_best_n_key(count: u32, dictionary: &Dictionary) -> Option<Solution> {
    let alphabet = dictionary.alphabet();
    let start_time = Instant::now();
    let max = ((alphabet.count_letters() / count) + 4).min(alphabet.count_letters());
    let key_sizes = Partitions {
        sum: alphabet.count_letters(),
        parts: count,
        min: 1,
        max,
    }
    .calculate();
    let keyboards = key_sizes.iter().flat_map(|key_sizes| {
        let arrangements: Tally<u32> = Tally::from(key_sizes);
        alphabet
            .distribute(arrangements)
            .map(|keys| Keyboard::with_keys(keys))
    });
    let mut best: Option<Solution> = None;
    for (index, k) in keyboards.enumerate() {
        let best_penalty = best.as_ref().map(|b| b.penalty()).unwrap_or(Penalty::MAX);
        let penalty = k.penalty(&dictionary, best_penalty);
        if penalty < best_penalty {
            let solution = k.to_solution(
                penalty,
                format!(
                    "{} keys, kbd {}, {}",
                    count,
                    index.separate_with_underscores(),
                    start_time.elapsed().round_to_seconds()
                ),
            );
            println!("{}", solution);
            best = Some(solution);
        }
        if index > 0 && index.rem_euclid(1_000_000) == 0 {
            println!(
                "> seen {} keyboards with {} keys, {}",
                index.separate_with_underscores(),
                count,
                start_time.elapsed().round_to_seconds()
            );
        }
    }
    best
}
