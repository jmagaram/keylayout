use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty,
    single_key_penalties::SingleKeyPenalties, solution::Solution, tally::Tally,
};
use crossbeam_channel::*;
use humantime::{format_duration, FormattedDuration};
use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{spawn, JoinHandle},
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

pub struct Args {
    pub threads: u8,
    pub max_key_size: u8,
    pub pairings_to_ignore: u8,
    pub prune_threshold: Penalty,
}

struct BuildKeyboardsArgs<'a, F>
where
    F: Fn(&Keyboard) -> bool,
{
    pairs: &'a Vec<Key>,
    pairs_index: usize,
    k: Keyboard,
    prohibited: Vec<Key>,
    channel: &'a Sender<Keyboard>,
    max_key_size: u8,
    created: &'a Cell<u128>,
    created_tally: &'a RefCell<Tally<usize>>,
    prune: &'a F,
    report_every_created: u128,
}

impl<'a, F> BuildKeyboardsArgs<'a, F>
where
    F: Fn(&Keyboard) -> bool,
{
    pub fn with_prohibited_pair(self, pair: Key) -> BuildKeyboardsArgs<'a, F> {
        let mut prohibited = self.prohibited;
        prohibited.push(pair);
        BuildKeyboardsArgs {
            pairs_index: self.pairs_index + 1,
            prohibited,
            ..self
        }
    }

    pub fn with_next_pair_on_deck(self) -> BuildKeyboardsArgs<'a, F> {
        BuildKeyboardsArgs {
            pairs_index: self.pairs_index + 1,
            ..self
        }
    }

    pub fn with_smaller_keyboard(&self, keyboard: Keyboard) -> BuildKeyboardsArgs<'a, F> {
        BuildKeyboardsArgs {
            pairs_index: self.pairs_index + 1,
            k: keyboard,
            prohibited: self.prohibited.clone(),
            ..*self
        }
    }

    pub fn build_keyboards(self: BuildKeyboardsArgs<'a, F>) {
        self.created.set(self.created.get() + 1);
        self.created_tally.borrow_mut().increment(self.k.len());
        if self.created.get().rem_euclid(self.report_every_created) == 0 {
            println!("Created {}", self.created.get().separate_with_underscores());
            (10..=26).for_each(|key_count| {
                let total = self.created_tally.borrow().count(&key_count);
                if total > 0 {
                    println!(
                        "Created size {:<2} : {}",
                        key_count,
                        total.separate_with_underscores()
                    );
                }
            });
        }
        let k = &(&self).k;
        if false == (*self.prune)(k) {
            if k.len() == 10 {
                self.channel.send(k.clone()).unwrap();
            } else if let Some(pair) = self.pairs.get(self.pairs_index) {
                let (k_smaller, combined) = k.combine_keys_with_letters(
                    pair.letters().nth(0).unwrap(),
                    pair.letters().nth(1).unwrap(),
                );
                if combined.len() > self.max_key_size {
                    self.with_prohibited_pair(*pair).build_keyboards();
                } else {
                    let combined_before = k_smaller.len() == k.len();
                    if combined_before {
                        self.with_next_pair_on_deck().build_keyboards();
                    } else {
                        let is_prohibited =
                            self.prohibited.iter().any(|pro| combined.contains_all(pro));
                        if !is_prohibited {
                            self.with_smaller_keyboard(k_smaller.clone())
                                .build_keyboards();
                        }
                        self.with_prohibited_pair(*pair).build_keyboards();
                    }
                }
            }
        }
    }
}

impl Args {
    fn spawn_keyboard_generator(
        sender: Sender<Keyboard>,
        alphabet: Key,
        generated_all_keyboards: Arc<AtomicBool>,
        max_key_size: u8,
        pairs_to_ignore: usize,
        ten_key_threshold: Penalty,
    ) -> JoinHandle<bool> {
        let penalties = SingleKeyPenalties::load();
        let pairs = {
            let mut pairs = penalties.of_key_size(2).collect::<Vec<(Key, Penalty)>>();
            pairs.sort_by(|a, b| a.1.cmp(&b.1));
            pairs
                .iter()
                .take(pairs.len() - pairs_to_ignore)
                .map(|(k, _p)| *k)
                .collect::<Vec<Key>>()
        };
        let pruned: Cell<u64> = Cell::new(0);
        let pruned_at: RefCell<Tally<usize>> = RefCell::new(Tally::new());
        let prune = move |k: &Keyboard| {
            let key_count = k.len();
            let prune_from = 10;
            let prune_to = 18;
            if key_count >= prune_from && key_count <= prune_to {
                let factor = 0.85f32.powi(key_count as i32 - 10);
                let threshold_for_key_count = ten_key_threshold.to_f32() * factor;
                let estimate = k.penalty_estimate(&penalties);
                let should_prune = estimate.to_f32() > threshold_for_key_count;
                if should_prune {
                    pruned_at.borrow_mut().increment(key_count);
                    pruned.set(pruned.get() + 1);
                    if pruned.get().rem_euclid(10_000_000) == 0 {
                        println!("Pruned {}", pruned.get().separate_with_underscores());
                        (prune_from..=prune_to).for_each(|key_count| {
                            let total = pruned_at.borrow().count(&key_count);
                            if total > 0 {
                                println!(
                                    "Pruned size {:<2} : {}",
                                    key_count,
                                    total.separate_with_underscores()
                                );
                            }
                        });
                    }
                }
                should_prune
            } else {
                false
            }
        };
        let created = Cell::new(0);
        let created_tally = RefCell::new(Tally::new());
        spawn(move || {
            let args = BuildKeyboardsArgs {
                pairs: &pairs,
                pairs_index: 0,
                k: Keyboard::with_every_letter_on_own_key(alphabet),
                prohibited: vec![],
                channel: &sender,
                max_key_size,
                created: &created,
                created_tally: &created_tally,
                prune: &prune,
                report_every_created: 10_000_000,
            };
            args.build_keyboards();
            generated_all_keyboards.fetch_or(true, Ordering::Relaxed)
        })
    }

    fn spawn_keyboard_evaluator(
        receiver: Receiver<Keyboard>,
        dictionary: Arc<Dictionary>,
        best: Arc<Mutex<Solution>>,
        generated_all_keyboards: Arc<AtomicBool>,
        keyboards_seen: Arc<AtomicU64>,
        start_time: Instant,
    ) -> JoinHandle<()> {
        spawn(move || loop {
            match receiver.recv_timeout(Duration::from_secs(60)) {
                Ok(kbd) => {
                    let best_penalty = best.lock().map(|s| s.penalty()).unwrap_or(Penalty::MAX);
                    let penalty = kbd.penalty(&dictionary, best_penalty);
                    let count = keyboards_seen.fetch_add(1, Ordering::Relaxed);
                    if penalty < best_penalty {
                        if let Ok(solution) = best.lock().as_deref_mut() {
                            let new_solution = kbd.to_solution(
                                penalty,
                                format!("kbd {}", count.separate_with_underscores()),
                            );
                            println!("{}", new_solution);
                            *solution = new_solution;
                        }
                    } else if count.rem_euclid(250_000) == 0 {
                        println!(
                            "Evaluated: {} in {}",
                            count.separate_with_underscores(),
                            start_time.elapsed().round_to_seconds()
                        );
                        if let Ok(solution) = best.lock().as_deref() {
                            println!("{}", solution);
                        }
                    }
                }
                Err(_) => {
                    if generated_all_keyboards.load(Ordering::Relaxed) {
                        break;
                    }
                }
            }
        })
    }

    // Starts with a keyboard with every letter having its own key. Then try to combine
    // letters recursively until there are 10 keys. Evaulate the score of every 10 key
    // keyboard and compare it to the best so far. Combining occurs by making a list of
    // every possible 2 letter pair, sorted by best (like z') to worst. In the recursive
    // process, every pair is accepted AND rejected, which generates a huge number of
    // combinations.
    pub fn solve(&self) {
        use crossbeam_channel::*;
        use std::sync::atomic::*;
        let dictionary = Arc::new(Dictionary::load());
        let best = Arc::new(Mutex::new(
            Keyboard::empty().to_solution(Penalty::MAX, "".to_string()),
        ));
        let start_time = Instant::now();
        let generated_all_keyboards = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = bounded::<Keyboard>(100_000);
        let keyboards_seen = Arc::new(AtomicU64::new(0));
        let generator = Self::spawn_keyboard_generator(
            sender,
            dictionary.alphabet(),
            generated_all_keyboards.clone(),
            self.max_key_size,
            70,
            self.prune_threshold,
        );
        let evaluators = (1..=self.threads)
            .map(|_| {
                Self::spawn_keyboard_evaluator(
                    receiver.clone(),
                    dictionary.clone(),
                    best.clone(),
                    generated_all_keyboards.clone(),
                    keyboards_seen.clone(),
                    start_time,
                )
            })
            .collect::<Vec<JoinHandle<_>>>();
        generator.join().unwrap();
        for i in evaluators.into_iter() {
            i.join().unwrap();
        }
        let best_solution = best.lock().unwrap().deref().clone();
        println!("====== BEST ======");
        println!("{}", best_solution);
    }
}
