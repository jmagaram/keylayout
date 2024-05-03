use crate::keyboard::Pruneable;
use crate::partitions::Partitions;
use crate::{dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, solution::Solution};
use dialoguer::{Input, Select};
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

impl Pruneable for Keyboard {
    fn should_prune(&self) -> bool {
        false
    }
}

pub struct Args {
    dictionary_size: Option<usize>,
    key_count: u8,
    min_key_size: u8,
    max_key_size: u8,
}

impl Args {
    pub fn new_from_prompts() -> Args {
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
        let key_count = Input::<u8>::new()
            .with_prompt("Total number of keys")
            .default(10)
            .interact_text()
            .unwrap();
        let min_key_size = Input::<u8>::new()
            .with_prompt("Minimum letters per key")
            .default(2)
            .interact_text()
            .unwrap();
        let max_key_size = Input::<u8>::new()
            .with_prompt("Maximum letters per key")
            .default(5)
            .interact_text()
            .unwrap();
        Args {
            dictionary_size,
            key_count,
            min_key_size,
            max_key_size,
        }
    }
}

pub fn find_best_n_key(args: Args) -> Option<Solution> {
    let dictionary = match args.dictionary_size {
        None => Dictionary::load(),
        Some(size) => Dictionary::load().filter_top_n_words(size),
    };
    let start_time = Instant::now();
    let mut best: Option<Solution> = None;
    let prune = |k: &Keyboard| -> Keyboard { k.clone() };
    let key_sizes = Partitions {
        sum: dictionary.alphabet().len(),
        parts: args.key_count,
        min: args.min_key_size,
        max: args.max_key_size,
    };
    let total_keyboards = key_sizes.total_unique_keyboards();
    let keyboards = Keyboard::with_dfs(dictionary.alphabet(), &key_sizes, &prune);
    for (index, k) in keyboards
        .filter(|k| k.len() == args.key_count as usize)
        .enumerate()
    {
        let best_penalty = best.as_ref().map(|b| b.penalty()).unwrap_or(Penalty::MAX);
        let penalty = k.penalty(&dictionary, best_penalty);
        if penalty < best_penalty {
            let solution = k.to_solution(
                penalty,
                format!(
                    "{} keys, kbd {} of {}, {}",
                    args.key_count,
                    index.separate_with_underscores(),
                    total_keyboards.separate_with_underscores(),
                    start_time.elapsed().round_to_seconds()
                ),
            );
            println!("{}", solution);
            best = Some(solution);
        }
        if index > 0 && index.rem_euclid(100_000) == 0 {
            println!(
                "--- seen {}/{} keyboards in {}",
                index.separate_with_underscores(),
                total_keyboards.separate_with_underscores(),
                start_time.elapsed().round_to_seconds()
            );
            if let Some(ref solution) = best {
                println!("{}", solution);
            }
        }
    }
    println!("Elapsed: {}", start_time.elapsed().round_to_seconds());
    best
}
