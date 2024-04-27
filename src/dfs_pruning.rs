use crate::{
    dictionary::Dictionary, keyboard::Keyboard, penalty::Penalty, penalty_goal::PenaltyGoals,
    prohibited::Prohibited,
};
use core::fmt;
use hashbrown::HashMap;
use humantime::{format_duration, FormattedDuration};

use std::{
    sync::mpsc,
    thread,
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

#[derive(Clone, Copy)]
enum PruneKind {
    MaxKeySizeExceeded,
    ProhibitedLetters,
    Penalty(Penalty),
    KeyboardTooSmall,
}

#[derive(Clone, Copy)]
struct Prune {
    kind: PruneKind,
    key_count: usize,
}

struct KeyCountTotals {
    ok: u32,
    prohibited_letters: u32,
    max_key_size: u32,
    keyboard_too_small: u32,
    penalty: u32,
}

impl KeyCountTotals {
    pub fn new() -> KeyCountTotals {
        KeyCountTotals {
            keyboard_too_small: 0,
            max_key_size: 0,
            ok: 0,
            penalty: 0,
            prohibited_letters: 0,
        }
    }
}

struct Statistics {
    start_time: Instant,
    seen: u32,
    by_key_count: HashMap<usize, KeyCountTotals>,
    recent_ok: Option<Keyboard>,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            start_time: Instant::now(),
            by_key_count: HashMap::new(),
            recent_ok: None,
            seen: 0,
        }
    }

    pub fn max_key_size_exceeded_total(&self) -> u32 {
        self.by_key_count.values().map(|v| v.max_key_size).sum()
    }

    pub fn prohibited_total(&self) -> u32 {
        self.by_key_count
            .values()
            .map(|v| v.prohibited_letters)
            .sum()
    }

    pub fn penalty_total(&self) -> u32 {
        self.by_key_count.values().map(|v| v.penalty).sum()
    }

    pub fn add(&mut self, r: Result<Keyboard, Prune>) {
        self.seen = self.seen + 1;
        let key_count = match r {
            Ok(ref k) => k.len(),
            Err(e) => e.key_count,
        };
        if !self.by_key_count.contains_key(&key_count) {
            self.by_key_count.insert(key_count, KeyCountTotals::new());
        }
        let stat = self.by_key_count.get_mut(&key_count).unwrap();
        match r {
            Ok(ref k) => {
                self.recent_ok = Some(k.clone());
                stat.ok = stat.ok + 1;
            }
            Err(p) => match p.kind {
                PruneKind::MaxKeySizeExceeded => {
                    stat.max_key_size = stat.max_key_size + 1;
                }
                PruneKind::Penalty(_penalty) => {
                    stat.penalty = stat.penalty + 1;
                }
                PruneKind::KeyboardTooSmall => {
                    stat.keyboard_too_small = stat.keyboard_too_small + 1;
                }
                PruneKind::ProhibitedLetters => {
                    stat.prohibited_letters = stat.prohibited_letters + 1;
                }
            },
        }
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seen_per_second = ((self.seen as f32) / self.start_time.elapsed().as_secs_f32()) as i32;
        fn write_num(
            f: &mut fmt::Formatter<'_>,
            caption: &str,
            num: u32,
            total: u32,
        ) -> Result<(), std::fmt::Error> {
            writeln!(
                f,
                "{} {} ({:.0}%)",
                caption,
                num.separate_with_underscores(),
                100.0 * (num as f32) / (total as f32)
            )
        }
        writeln!(
            f,
            "Keyboards:    {} ({}/sec)",
            self.seen.separate_with_underscores(),
            seen_per_second.separate_with_underscores()
        )?;
        writeln!(
            f,
            "Recent:       {}",
            self.recent_ok
                .clone()
                .map_or("(none)".to_string(), |k| k.to_string())
        )?;
        writeln!(
            f,
            "Elapsed:      {}",
            self.start_time.elapsed().round_to_seconds()
        )?;
        write_num(
            f,
            "Key too big: ",
            self.max_key_size_exceeded_total(),
            self.seen,
        )?;
        write_num(f, "Prohibited:  ", self.prohibited_total(), self.seen)?;
        write_num(f, "Penalty:     ", self.penalty_total(), self.seen)?;
        (1usize..27)
            .filter_map(|key_count| {
                self.by_key_count
                    .get(&key_count)
                    .map(|i| (key_count, i.penalty))
            })
            .map(|(key_count, prune_count)| {
                writeln!(
                    f,
                    "                 {} : {} ({:.1}%)",
                    key_count,
                    prune_count.separate_with_underscores(),
                    100.0 * (prune_count as f32) / (self.seen as f32)
                )
            })
            .collect::<Result<(), _>>()?;
        Ok(())
    }
}

pub fn solve() {
    let start_time = Instant::now();
    let d = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&d, 45);
    let max_key_size = 4;
    let penalty_goals = PenaltyGoals::none(d.alphabet())
        .with(26, Penalty::new(0.00006))
        .with(25, Penalty::new(0.000174))
        .with(24, Penalty::new(0.000385))
        .with(23, Penalty::new(0.0007))
        .with(22, Penalty::new(0.0012))
        .with(21, Penalty::new(0.001974))
        .with(20, Penalty::new(0.002559))
        .with(19, Penalty::new(0.003633))
        .with(18, Penalty::new(0.004623))
        .with(17, Penalty::new(0.005569))
        .with(16, Penalty::new(0.007603))
        .with(15, Penalty::new(0.009746))
        .with(14, Penalty::new(0.013027))
        .with(13, Penalty::new(0.016709))
        .with(12, Penalty::new(0.02109))
        .with_adjustment(11..=23, 0.85)
        .with(10, Penalty::new(0.0246));
    let prune_result = |k: &Keyboard| -> Result<Keyboard, Prune> {
        Ok(k.clone())
            .and_then(|k| match k.len() < 10 {
                true => Err(Prune {
                    kind: PruneKind::KeyboardTooSmall,
                    key_count: k.len(),
                }),
                false => Ok(k),
            })
            .and_then(|k| match k.max_key_size() {
                Some(size) if size > max_key_size => Err(Prune {
                    kind: PruneKind::MaxKeySizeExceeded,
                    key_count: k.len(),
                }),
                _ => Ok(k),
            })
            .and_then(|k| match k.has_prohibited_keys(&prohibited) {
                false => Ok(k),
                true => Err(Prune {
                    kind: PruneKind::ProhibitedLetters,
                    key_count: k.len(),
                }),
            })
            .and_then(|k| {
                let penalty_to_beat = penalty_goals.get(k.len() as u8).unwrap_or(Penalty::MAX);
                let actual_penalty = k.penalty(&d, penalty_to_beat);
                match actual_penalty > penalty_to_beat {
                    true => Err(Prune {
                        kind: PruneKind::Penalty(actual_penalty),
                        key_count: k.len(),
                    }),
                    false => Ok(k),
                }
            })
    };
    let start = Keyboard::with_every_letter_on_own_key(d.alphabet());
    let (tx, rx) = mpsc::channel::<Result<Keyboard, Prune>>();
    let prune = |k: &Keyboard| -> bool {
        let result = prune_result(k);
        let should_prune = result.is_err();
        tx.send(result).unwrap();
        should_prune
    };
    let solutions = start
        .every_smaller_with(&prune)
        .filter(|k| k.len() == 10)
        .map(|k| {
            let penalty = k.penalty(&d, Penalty::MAX);
            k.to_solution(penalty, "".to_string())
        });
    let _join_handle = thread::spawn(move || {
        let mut progress_stats = Statistics::new();
        loop {
            let prune_result = rx.recv();
            match prune_result {
                Ok(prune_result) => {
                    progress_stats.add(prune_result);
                    if progress_stats.seen.rem_euclid(1_000) == 0 {
                        println!("{}", progress_stats);
                    }
                }
                Err(_err) => {}
            }
        }
    });
    for s in solutions {
        println!("{}", s);
    }
    println!("=== DONE ===");
}
