use std::{
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

use crate::{
    dictionary::Dictionary, key::Key, keyboard::Keyboard, letter::Letter, penalty::Penalty,
};
use crossbeam_channel::*;
use thousands::Separable;

pub struct Args {
    pub threads: u8,
    pub max_key_size: u8,
}

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
                    let alphabet = (d_ref.alphabet());
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

impl Args {
    fn build_keyboards(
        pairs: &Vec<Pair>,
        pairs_index: usize,
        k: Keyboard,
        prohibited: Vec<Key>,
        channel: &Sender<Keyboard>,
        max_key_size: u8,
        created: &mut u128,
    ) {
        if k.len() == 10 {
            *created = *created + 1;
            if created.rem_euclid(100_000) == 0 {
                println!("Generated: {}", created.separate_with_underscores());
            }
            channel.send(k).unwrap();
        } else {
            if let Some(pair) = pairs.get(pairs_index) {
                let (k_smaller, combined) = k.combine_keys_with_letters(pair.i, pair.j);
                let combined_before = k_smaller.len() == k.len();
                if combined_before {
                    Self::build_keyboards(
                        pairs,
                        pairs_index + 1,
                        k,
                        prohibited,
                        channel,
                        max_key_size,
                        created,
                    );
                } else {
                    let is_prohibited = prohibited.iter().any(|pro| combined.contains_all(pro));
                    if !is_prohibited {
                        Self::build_keyboards(
                            pairs,
                            pairs_index + 1,
                            k_smaller,
                            prohibited.clone(),
                            channel,
                            max_key_size,
                            created,
                        );
                    }
                    let mut prohibited = prohibited;
                    prohibited.push(pair.as_key());
                    Self::build_keyboards(
                        pairs,
                        pairs_index + 1,
                        k,
                        prohibited,
                        channel,
                        max_key_size,
                        created,
                    );
                }
            }
        }
    }

    pub fn solve(&self) {
        use crossbeam_channel::*;
        use std::sync::atomic::*;
        use thread::*;
        let d = Arc::new(Dictionary::load());
        let best = Arc::new(Mutex::new(
            Keyboard::empty().to_solution(Penalty::MAX, "".to_string()),
        ));
        let done_generating = Arc::new(AtomicBool::new(false));
        let (sdr, rcr) = bounded::<Keyboard>(1_000_000);
        let spawn_keyboard_generator = || {
            let sdr = sdr.clone();
            let d = d.clone();
            let done_generating_keyboards = done_generating.clone();
            let pairs = Pair::all_by_penalty();
            let max_key_size = self.max_key_size;
            spawn(move || {
                let mut created: u128 = 0;
                Self::build_keyboards(
                    &pairs,
                    0,
                    Keyboard::with_every_letter_on_own_key(d.alphabet()),
                    vec![],
                    &sdr,
                    max_key_size,
                    &mut created,
                );
                done_generating_keyboards.fetch_or(true, Ordering::Relaxed)
            })
        };
        let spawn_keyboard_evaluator = || {
            let rcr = rcr.clone();
            let d = d.clone();
            let best = best.clone();
            let done_generating_keyboards = done_generating.clone();
            spawn(move || loop {
                match rcr.recv_timeout(Duration::from_secs(60)) {
                    Ok(kbd) => {
                        let best_penalty = best.lock().map(|s| s.penalty()).unwrap_or(Penalty::MAX);
                        let penalty = kbd.penalty(&d, best_penalty);
                        if penalty < best_penalty {
                            if let Ok(solution) = best.lock().as_deref_mut() {
                                let new_solution = kbd.to_solution(penalty, "".to_string());
                                println!("{}", new_solution);
                                *solution = new_solution;
                            }
                        };
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
    }
}
