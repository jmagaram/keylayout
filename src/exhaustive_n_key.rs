use crate::keyboard::Pruneable;
use crate::partitions::Partitions;
use crate::vec_threads;
use crate::vec_threads::VecThreads;
use crate::{dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, solution::Solution};
use dialoguer::{Input, Select};
use humantime::{format_duration, FormattedDuration};
use std::sync::Arc;
use std::thread::{sleep, spawn, JoinHandle};
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
    threads: u8,
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
        let threads = Input::<u8>::new()
            .with_prompt("Threads")
            .default(8)
            .interact_text()
            .unwrap();
        Args {
            dictionary_size,
            key_count,
            min_key_size,
            max_key_size,
            threads,
        }
    }
}

pub fn best_n_key(args: Args) -> Option<Solution> {
    let dictionary = Arc::new(match args.dictionary_size {
        None => Dictionary::load(),
        Some(size) => Dictionary::load().filter_top_n_words(size),
    });
    let partitions = Partitions {
        sum: dictionary.alphabet().len(),
        parts: args.key_count,
        min: args.min_key_size,
        max: args.max_key_size,
    };
    let total_keyboards = partitions.total_unique_keyboards();
    let keyboards: VecThreads<Keyboard> = vec_threads::VecThreads::new();
    let best: VecThreads<Solution> = vec_threads::VecThreads::new();
    let evaluated = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let started_at = Instant::now();

    let spawn_enumerator = |partitions: &Partitions,
                            dictionary: &Arc<Dictionary>,
                            keyboards: &VecThreads<Keyboard>| {
        let partitions = partitions.clone();
        let mut keyboards = keyboards.clone();
        let dictionary = dictionary.clone();
        let enumerate = spawn(move || {
            let prune = |k: &Keyboard| k.clone();
            for k in Keyboard::with_dfs(dictionary.alphabet(), &partitions, &prune)
                .filter(|k| k.len() == partitions.parts as usize)
            {
                keyboards.push(k);
            }
        });
        enumerate
    };

    let spawn_evaluator = |dictionary: &Arc<Dictionary>,
                           keyboards: &VecThreads<Keyboard>,
                           best: &VecThreads<Solution>,
                           evaluated: &Arc<std::sync::atomic::AtomicU64>,
                           thread_id: u8| {
        let mut keyboards = keyboards.clone();
        let dictionary = dictionary.clone();
        let mut best = best.clone();
        let evaluated = evaluated.clone();
        let evaluate = spawn(move || {
            let mut best_penalty = Penalty::MAX;
            loop {
                match keyboards.pop() {
                    None => {
                        break;
                    }
                    Some(keyboard) => {
                        let current = evaluated.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let penalty = keyboard.penalty(&dictionary, best_penalty);
                        if penalty < best_penalty {
                            best_penalty = penalty;
                            let solution = keyboard.to_solution(penalty, "".to_string());
                            println!(
                                "Thread: {:<2} | Solution:{}| {}/{} | {}",
                                thread_id,
                                solution,
                                current.separate_with_underscores(),
                                total_keyboards.separate_with_underscores(),
                                started_at.elapsed().round_to_seconds()
                            );
                            best.push(solution);
                        } else if current.rem_euclid(100_000) == 0 {
                            println!(
                                "Thread: {:<2} | Evaluating: {}/{} | {}",
                                thread_id,
                                current.separate_with_underscores(),
                                total_keyboards.separate_with_underscores(),
                                started_at.elapsed().round_to_seconds()
                            )
                        }
                    }
                }
            }
        });
        evaluate
    };

    let enumerator = spawn_enumerator(&partitions, &dictionary, &keyboards);
    sleep(Duration::from_secs(3));
    let evaluators = (1..=args.threads)
        .map(|thread_id| spawn_evaluator(&dictionary, &keyboards, &best, &evaluated, thread_id))
        .collect::<Vec<JoinHandle<_>>>();

    enumerator.join().unwrap();
    for e in evaluators {
        e.join().unwrap();
    }

    println!("");
    let overall_best = best
        .items()
        .into_iter()
        .min_by(|a, b| a.penalty().cmp(&b.penalty()));
    match &overall_best {
        None => {
            println!("No solution found")
        }
        Some(best) => {
            println!("{}", best)
        }
    }
    overall_best
}
