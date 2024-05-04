use crate::keyboard::Pruneable;
use crate::partitions::Partitions;
use crate::prohibited::Prohibited;
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

#[derive(Clone)]
struct PruneableKeyboard {
    keyboard: Keyboard,
    should_prune: bool,
}

impl Pruneable for PruneableKeyboard {
    fn should_prune(&self) -> bool {
        self.should_prune
    }
}

pub enum DictionaryChoice {
    Full,
    TopNWords(u32),
    Custom(Dictionary),
}

impl DictionaryChoice {
    pub fn new_from_prompts() -> DictionaryChoice {
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
        match dictionary_size_index {
            0 => DictionaryChoice::Full,
            1 => DictionaryChoice::TopNWords(120_000),
            2 => DictionaryChoice::TopNWords(90_000),
            3 => DictionaryChoice::TopNWords(25_000),
            4 => DictionaryChoice::TopNWords(5_000),
            _ => panic!("Do not know how to handle that input for dictionary size."),
        }
    }

    pub fn get(self) -> Dictionary {
        use DictionaryChoice::*;
        match self {
            Full => Dictionary::load(),
            TopNWords(top_n_words) => Dictionary::load().filter_top_n_words(top_n_words as usize),
            Custom(dictionary) => dictionary,
        }
    }
}

pub struct Args {
    pub dictionary_choice: DictionaryChoice,
    pub prohibited: Prohibited,
    pub key_count: u8,
    pub min_key_size: u8,
    pub max_key_size: u8,
    pub threads: u8,
}

impl Args {
    pub fn new_from_prompts() -> Args {
        let dictionary_size = DictionaryChoice::new_from_prompts();
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
            dictionary_choice: dictionary_size,
            key_count,
            min_key_size,
            max_key_size,
            threads,
            prohibited: Prohibited::new(),
        }
    }

    pub fn solve(self) -> Option<Solution> {
        let args = self;
        use std::sync::atomic::*;
        let dictionary = Arc::new(args.dictionary_choice.get());
        let partitions = Partitions {
            sum: dictionary.alphabet().len(),
            parts: args.key_count,
            min: args.min_key_size,
            max: args.max_key_size,
        };
        let keyboards: VecThreads<Keyboard> = vec_threads::VecThreads::new();
        let best: VecThreads<Solution> = vec_threads::VecThreads::new();
        let evaluated = Arc::new(AtomicU64::new(0));
        let generated = Arc::new(AtomicU64::new(0));
        let done_generating = Arc::new(AtomicBool::new(false));
        let started_at = Instant::now();

        let spawn_enumerator = |partitions: &Partitions,
                                dictionary: &Arc<Dictionary>,
                                generated: &Arc<AtomicU64>,
                                keyboards: &VecThreads<Keyboard>,
                                done_generating: &Arc<AtomicBool>| {
            let partitions = partitions.clone();
            let generated = generated.clone();
            let done_generating = done_generating.clone();
            let mut keyboards = keyboards.clone();
            let dictionary = dictionary.clone();
            let enumerate = spawn(move || {
                let prune = |k: &Keyboard| {
                    let should_prune = k.has_prohibited_keys(&args.prohibited);
                    PruneableKeyboard {
                        keyboard: k.clone(),
                        should_prune,
                    }
                };
                for k in Keyboard::with_dfs(dictionary.alphabet(), &partitions, &prune)
                    .filter_map(|p| match p.should_prune {
                        true => None,
                        false => Some(p.keyboard),
                    })
                    .filter(|k| k.len() == partitions.parts as usize)
                {
                    keyboards.push(k);
                    let total_generated = generated.fetch_add(1, Ordering::Relaxed);
                    if total_generated.rem_euclid(100_000) == 0 {
                        println!(
                            "Thread: {:<2} | Generated: {}",
                            0,
                            total_generated.separate_with_underscores()
                        );
                    }
                }
                done_generating.fetch_or(true, Ordering::Relaxed);
            });
            enumerate
        };

        let spawn_evaluator = |dictionary: &Arc<Dictionary>,
                               keyboards: &VecThreads<Keyboard>,
                               best: &VecThreads<Solution>,
                               evaluated: &Arc<AtomicU64>,
                               generated: &Arc<AtomicU64>,
                               done_generating: &Arc<AtomicBool>,
                               thread_id: u8| {
            let mut keyboards = keyboards.clone();
            let dictionary = dictionary.clone();
            let mut best = best.clone();
            let evaluated = evaluated.clone();
            let generated = generated.clone();
            let done_generating = done_generating.clone();
            let evaluate = spawn(move || {
                let mut best_penalty = Penalty::MAX;
                loop {
                    match keyboards.pop() {
                        None => {
                            if done_generating.load(Ordering::Relaxed) {
                                println!("Thread: {:<2} | Ended", thread_id);
                                break;
                            } else {
                                sleep(Duration::from_secs(30));
                            }
                        }
                        Some(keyboard) => {
                            let current = evaluated.fetch_add(1, Ordering::SeqCst);
                            let penalty = keyboard.penalty(&dictionary, best_penalty);
                            if penalty < best_penalty {
                                best_penalty = penalty;
                                let solution = keyboard.to_solution(penalty, "".to_string());
                                println!(
                                    "Thread: {:<2} | Solution:{}| {} of {} | {}",
                                    thread_id,
                                    solution,
                                    current.separate_with_underscores(),
                                    generated
                                        .load(Ordering::Relaxed)
                                        .separate_with_underscores(),
                                    started_at.elapsed().round_to_seconds()
                                );
                                best.push(solution);
                            } else if current.rem_euclid(100_000) == 0 {
                                println!(
                                    "Thread: {:<2} | Evaluating: {} of {} | {}",
                                    thread_id,
                                    current.separate_with_underscores(),
                                    generated
                                        .load(Ordering::Relaxed)
                                        .separate_with_underscores(),
                                    started_at.elapsed().round_to_seconds()
                                )
                            }
                        }
                    }
                }
            });
            evaluate
        };

        let enumerator = spawn_enumerator(
            &partitions,
            &dictionary,
            &generated,
            &keyboards,
            &done_generating,
        );
        let evaluators = (1..=args.threads)
            .map(|thread_id| {
                spawn_evaluator(
                    &dictionary,
                    &keyboards,
                    &best,
                    &evaluated,
                    &generated,
                    &done_generating,
                    thread_id,
                )
            })
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
}
