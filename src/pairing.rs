use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty,
    penalty_goal::PenaltyGoals, single_key_penalties::SingleKeyPenalties, tally::Tally,
};
use crossbeam_channel::*;
use humantime::{format_duration, FormattedDuration};
use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    sync::{Arc, Mutex},
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
            }
            if let Some(pair) = self.pairs.get(self.pairs_index) {
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
    // Starts with a keyboard with every letter having its own key. Then try to combine
    // letters recursively until there are 10 keys. Evaulate the score of every 10 key
    // keyboard and compare it to the best so far. Combining occurs by making a list of
    // every possible 2 letter pair, sorted by best (like z') to worst. In the recursive
    // process, every pair is accepted AND rejected, which generates a huge number of
    // combinations.
    pub fn solve(&self) {
        use crossbeam_channel::*;
        use std::sync::atomic::*;
        let d = Arc::new(Dictionary::load());
        let best = Arc::new(Mutex::new(
            Keyboard::empty().to_solution(Penalty::MAX, "".to_string()),
        ));
        let start_time = Instant::now();
        let done_generating = Arc::new(AtomicBool::new(false));
        let kbd_seen = Arc::new(AtomicU64::new(0));
        let (sdr, rcr) = bounded::<Keyboard>(1_000_000);
        let spawn_keyboard_generator = || {
            let sdr = sdr.clone();
            let d = d.clone();
            let done_generating_keyboards = done_generating.clone();
            let single_key_penalties = SingleKeyPenalties::load();
            let pairs_to_consider = {
                let mut pairs = single_key_penalties
                    .of_key_size(2)
                    .collect::<Vec<(Key, Penalty)>>();
                pairs.sort_by(|a, b| a.1.cmp(&b.1));
                pairs
                    .iter()
                    .take(pairs.len() - self.pairings_to_ignore as usize)
                    .map(|(k, _p)| *k)
                    .collect::<Vec<Key>>()
            };
            let max_key_size = self.max_key_size;
            let pruned: Cell<u64> = Cell::new(0);
            let pruned_at: RefCell<Tally<usize>> = RefCell::new(Tally::new());
            let prune_threshold = self.prune_threshold;
            let prune = move |k: &Keyboard| {
                let key_count = k.len();
                let prune_from = 10;
                let prune_to = 18;
                if key_count >= prune_from && key_count <= prune_to {
                    let estimate = k.penalty_estimate(&single_key_penalties);
                    let should_prune = estimate >= prune_threshold;
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
            spawn(move || {
                let created = Cell::new(0);
                let created_tally = RefCell::new(Tally::new());
                let args = BuildKeyboardsArgs {
                    pairs: &pairs_to_consider,
                    pairs_index: 0,
                    k: Keyboard::with_every_letter_on_own_key(d.alphabet()),
                    prohibited: vec![],
                    channel: &sdr,
                    max_key_size,
                    created: &created,
                    created_tally: &created_tally,
                    prune: &prune,
                    report_every_created: 10_000_000,
                };
                args.build_keyboards();
                done_generating_keyboards.fetch_or(true, Ordering::Relaxed)
            })
        };
        let spawn_keyboard_evaluator = || {
            let rcr = rcr.clone();
            let d = d.clone();
            let best = best.clone();
            let done_generating_keyboards = done_generating.clone();
            let kbd_seen = kbd_seen.clone();
            spawn(move || loop {
                match rcr.recv_timeout(Duration::from_secs(60)) {
                    Ok(kbd) => {
                        let best_penalty = best.lock().map(|s| s.penalty()).unwrap_or(Penalty::MAX);
                        let penalty = kbd.penalty(&d, best_penalty);
                        let seen = kbd_seen.fetch_add(1, Ordering::Relaxed);
                        if penalty < best_penalty {
                            if let Ok(solution) = best.lock().as_deref_mut() {
                                let new_solution = kbd.to_solution(
                                    penalty,
                                    format!("kbd {}", seen.separate_with_underscores()),
                                );
                                println!("{}", new_solution);
                                *solution = new_solution;
                            }
                        } else if seen.rem_euclid(250_000) == 0 {
                            println!(
                                "Evaluated: {} in {}",
                                seen.separate_with_underscores(),
                                start_time.elapsed().round_to_seconds()
                            );
                            if let Ok(solution) = best.lock().as_deref() {
                                println!("{}", solution);
                            }
                        }
                    }
                    Err(_) => {
                        if done_generating_keyboards.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
            })
        };
        let generate_keyboards = spawn_keyboard_generator();
        let evaluators = (1..=self.threads)
            .map(|_| spawn_keyboard_evaluator())
            .collect::<Vec<JoinHandle<_>>>();
        generate_keyboards.join().unwrap();
        for i in evaluators.into_iter() {
            i.join().unwrap();
        }
        let best_solution = best.lock().unwrap().deref().clone();
        println!("====== BEST ======");
        println!("{}", best_solution);
    }
}
