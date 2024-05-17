use crate::{dictionary::Dictionary, key_set::KeySet, penalty::Penalty, util::choose};
use humantime::{format_duration, FormattedDuration};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
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
pub struct OverlapPenalties(HashMap<KeySet, Penalty>);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CsvRow {
    pairs: String,
    penalty: f32,
}

impl OverlapPenalties {
    pub fn build(dictionary: &Dictionary) -> OverlapPenalties {
        let words_count = dictionary.words().len();
        let total_pairs = choose(words_count as u32, 2);
        let processed = Arc::new(AtomicU64::new(0));
        let result = Arc::new(Mutex::new(HashMap::<KeySet, Penalty>::new()));
        let started = Instant::now();
        for a_index in 0..words_count - 1 {
            (a_index + 1..words_count).into_par_iter().for_each_init(
                || (processed.clone(), result.clone()),
                |(processed, result), b_index| {
                    let processed_count = processed.fetch_add(1, Ordering::Relaxed);
                    if processed_count.rem_euclid(10_000_000) == 0 {
                        println!(
                            "Evaluated: {} of {} in {}",
                            processed_count.separate_with_underscores(),
                            total_pairs.separate_with_underscores(),
                            started.elapsed().round_to_seconds()
                        );
                    }
                    let word_a = dictionary.words().get(a_index).unwrap();
                    let word_b = dictionary.words().get(b_index).unwrap();
                    let diff = word_a.letter_pair_difference(&word_b);
                    if diff.len() == 1 && diff.letter_count() == 2 {
                        let penalty =
                            Penalty::new(word_a.frequency().min(&word_b.frequency()).to_f32());
                        let mut result = result.lock().unwrap();
                        let penalty_total = result.get_mut(&diff);
                        match penalty_total {
                            None => {
                                result.insert(diff, penalty);
                            }
                            Some(penalty_total) => {
                                *penalty_total = *penalty_total + penalty;
                            }
                        }
                    }
                },
            );
        }
        let result = HashMap::from_iter(
            result
                .lock()
                .unwrap()
                .iter()
                .map(|(k, penalty)| (k.clone(), penalty.clone())),
        );
        OverlapPenalties(result)
    }

    pub fn save_to_csv(&self, file_name: &str) -> Result<(), csv::Error> {
        let mut wtr = csv::Writer::from_path(file_name)?;
        let rows = self.0.iter().map(|(pairs, penalty)| {
            let pairs = pairs.to_string();
            CsvRow {
                pairs: pairs,
                penalty: penalty.to_f32(),
            }
        });
        let _write_result = rows
            .map(|r| wtr.serialize(r))
            .collect::<Result<(), csv::Error>>()?;
        wtr.flush().unwrap();
        Ok(())
    }

    pub fn load_from_csv(file_name: &str) -> OverlapPenalties {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_name)
            .unwrap();
        let mut result: HashMap<KeySet, Penalty> = HashMap::new();
        println!("Loading word overlap penalties...");
        for row in rdr.deserialize::<CsvRow>() {
            let row = row.unwrap();
            let key_set = KeySet::with_layout(&row.pairs);
            let penalty = Penalty::new(row.penalty);
            result.insert(key_set, penalty);
        }
        println!("Done loading word overlap penalties.");
        OverlapPenalties(result)
    }
}
