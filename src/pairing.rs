use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, letter::Letter, penalty::Penalty,
};
use crossbeam_channel::*;
use humantime::{format_duration, FormattedDuration};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    thread::{self},
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
}

#[derive(Clone)]
struct Pair {
    i: Letter,
    j: Letter,
    penalty: Penalty,
}

impl Pair {
    pub fn as_key(&self) -> Key {
        Key::EMPTY.add(self.i).add(self.j)
    }

    pub fn all_by_penalty() -> Vec<Pair> {
        let d = Dictionary::load();
        let d_ref = &d;
        let mut pairs = (0..Letter::ALPHABET_SIZE - 1)
            .flat_map(|i_index| {
                (i_index + 1..Letter::ALPHABET_SIZE).map(move |j_index| {
                    let i = Letter::new(Letter::ALPHABET[i_index]);
                    let j = Letter::new(Letter::ALPHABET[j_index]);
                    let key = Key::EMPTY.add(i).add(j);
                    let alphabet = d_ref.alphabet();
                    let keyboard = Keyboard::with_keys(vec![key]).fill_missing(alphabet);
                    let penalty = keyboard.penalty(d_ref, Penalty::MAX);
                    Pair { i, j, penalty }
                })
            })
            .collect::<Vec<Pair>>();
        pairs.sort_by(|i, j| i.penalty.cmp(&j.penalty));
        pairs
    }
}

struct BuildKeyboardsArgs<'a> {
    pairs: &'a Vec<Pair>,
    pairs_index: usize,
    k: Keyboard,
    prohibited: Vec<Key>,
    channel: &'a Sender<Keyboard>,
    max_key_size: u8,
    created: &'a mut u128,
}

impl Args {
    fn build_keyboards<'a>(args: BuildKeyboardsArgs<'a>) {
        if args.k.len() == 10 {
            *args.created = *args.created + 1;
            if args.created.rem_euclid(1_000_000) == 0 {
                println!("Generated: {}", args.created.separate_with_underscores());
            }
            args.channel.send(args.k).unwrap();
        } else {
            if let Some(pair) = args.pairs.get(args.pairs_index) {
                let (k_smaller, combined) = args.k.combine_keys_with_letters(pair.i, pair.j);
                if combined.len() > args.max_key_size {
                    let mut prohibited_new = args.prohibited;
                    prohibited_new.push(pair.as_key());
                    let args = BuildKeyboardsArgs {
                        pairs_index: args.pairs_index + 1,
                        prohibited: prohibited_new,
                        ..args
                    };
                    Self::build_keyboards(args);
                } else {
                    let combined_before = k_smaller.len() == args.k.len();
                    if combined_before {
                        let args = BuildKeyboardsArgs {
                            pairs_index: args.pairs_index + 1,
                            ..args
                        };
                        Self::build_keyboards(args);
                    } else {
                        let created = args.created;
                        let is_prohibited =
                            args.prohibited.iter().any(|pro| combined.contains_all(pro));
                        if !is_prohibited {
                            let prohibited_clone = args.prohibited.clone();
                            let args = BuildKeyboardsArgs {
                                pairs_index: args.pairs_index + 1,
                                k: k_smaller,
                                prohibited: prohibited_clone,
                                created,
                                ..args
                            };
                            Self::build_keyboards(args);
                        }
                        let mut prohibited_new = args.prohibited;
                        prohibited_new.push(pair.as_key());
                        let args = BuildKeyboardsArgs {
                            pairs_index: args.pairs_index + 1,
                            prohibited: prohibited_new,
                            created,
                            ..args
                        };
                        Self::build_keyboards(args);
                    }
                }
            }
        }
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
        use thread::*;
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
            let mut pairs = Pair::all_by_penalty();
            let pairs_to_consider = pairs.len() - self.pairings_to_ignore as usize;
            pairs = pairs
                .into_iter()
                .take(pairs_to_consider)
                .collect::<Vec<Pair>>();
            let max_key_size = self.max_key_size;
            spawn(move || {
                let mut created: u128 = 0;
                let args = BuildKeyboardsArgs {
                    pairs: &pairs,
                    pairs_index: 0,
                    k: Keyboard::with_every_letter_on_own_key(d.alphabet()),
                    prohibited: vec![],
                    channel: &sdr,
                    max_key_size,
                    created: &mut created,
                };
                Self::build_keyboards(args);
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
