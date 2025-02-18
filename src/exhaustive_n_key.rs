use crate::key::Key;
use crate::keyboard::Pruneable;
use crate::letter::Letter;
use crate::partitions::Partitions;
use crate::prohibited::Prohibited;
use crate::util::DurationFormatter;
use crate::vec_threads;
use crate::vec_threads::VecThreads;
use crate::{dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, solution::Solution};
use dialoguer::{Input, Select};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;
use std::time::Instant;
use thousands::Separable;

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

    // Similar to the DFS pruning approach but I am able to parallelize this.
    // The keyboards are generated quickly without penalty scoring, and then
    // they are evaluated by multiple worker threads.
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
                    let len = keyboards.push_get_len(k);
                    if len >= 10_000_000 {
                        // Approximately 4 gigabytes for 27 characters and 10 keys
                        println!(
                            "Thread: {:<2} | Sleeping, work queue reached {}",
                            0,
                            len.separate_with_underscores()
                        );
                        sleep(Duration::from_secs(60));
                    }
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

pub struct FillArgs {
    pub start: String,
    pub max_key_size: u8,
    pub update_every: u64,
}

impl FillArgs {
    pub fn solve(&self) {
        let args = self;
        use std::sync::atomic::*;
        let now = Instant::now();
        let count = Arc::new(AtomicU64::new(0));
        let dictionary = Dictionary::load();
        let existing_keys = args
            .start
            .as_str()
            .split(' ')
            .map(|rr| Key::new(rr))
            .collect::<Vec<Key>>();
        let existing_letters = Key::new(&args.start.replace(',', ""));
        let missing_letters = dictionary.alphabet().except(existing_letters);
        let missing_letters_count = missing_letters.len();
        let start = 0u128;
        let end = 10u128.pow(missing_letters_count as u32);
        let keyboards = (start..end)
            .into_par_iter()
            .map(|n| {
                let kbd_inx = count.fetch_add(1, Ordering::Relaxed);
                if kbd_inx.rem_euclid(args.update_every) == 0 {
                    println!(
                        "Created keyboard {} of {} in {}",
                        kbd_inx.separate_with_underscores(),
                        end.separate_with_underscores(),
                        now.elapsed().round_to_seconds()
                    );
                }
                let mut keys = existing_keys.clone();
                let mut missing = missing_letters.clone();
                (0usize..missing_letters_count as usize).for_each(|letter_index| {
                    let letter = missing.min_letter().unwrap();
                    missing = missing.remove(letter);
                    let insert_at =
                        n.div_euclid(10u128.pow(letter_index as u32)).rem_euclid(10) as usize;
                    keys[insert_at] = keys[insert_at].add(letter);
                });
                Keyboard::with_keys(keys)
            })
            .filter(|k| k.max_key_size().unwrap() <= args.max_key_size);
        let best: Arc<Mutex<Option<Solution>>> = Arc::new(Mutex::new(None));
        let best_final = best.clone();
        keyboards.for_each_with(best, |best, k| {
            let best_penalty = best
                .lock()
                .unwrap()
                .as_ref()
                .map_or(Penalty::MAX, |s| s.penalty());
            let penalty = k.penalty(&dictionary, best_penalty);
            if penalty < best_penalty {
                let solution = k.to_solution(penalty, "".to_string());
                println!("{}", solution);
                let mut best_solution = best.lock().unwrap();
                *best_solution = Some(solution);
            }
        });
        let best_final = best_final.lock().unwrap();
        match best_final.as_ref() {
            None => {
                println!("No solution found");
            }
            Some(e) => {
                println!("");
                println!("{}", e);
            }
        };
    }
}

pub struct PopularLetterPairingsArgs {
    pub pair_up: String,
    pub infrequent_replacement: char,
}

impl PopularLetterPairingsArgs {
    // let letters = "etaoinsrhldcumfgypwbvk'jxzq"; // first
    // let letters = "eaisrnotlcdumhgpbykfvw'zjxq"; // trying now
    pub fn solve(&self) -> Solution {
        assert_eq!(
            20,
            self.pair_up.len(),
            "Expected 20 popular characters to pair up."
        );
        assert!(
            self.pair_up.contains(self.infrequent_replacement) == false,
            "The popular letters can not include the designated infrequenly letter replacement."
        );
        let popular = Key::from_iter(self.pair_up.as_str().chars().map(|r| Letter::new(r)));
        let infrequent = Key::with_every_letter().except(popular);
        let infrequent_replacement = Letter::new(self.infrequent_replacement);
        let dictionary = Dictionary::load().replace_letters(infrequent, infrequent_replacement);
        let mut prohibited = Prohibited::new();
        for popular_letter in popular.letters() {
            prohibited.add(Key::EMPTY.add(popular_letter).add(infrequent_replacement));
        }
        let args = Args {
            dictionary_choice: DictionaryChoice::Custom(dictionary),
            key_count: 11,
            max_key_size: 2,
            min_key_size: 1,
            threads: 8,
            prohibited,
        };
        let best = args.solve().unwrap();
        println!("== FOUND BEST POPULAR LETTER PAIRINGS ==");
        println!("{}", best);
        println!("");
        best
    }
}
