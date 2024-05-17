use crate::{
    dictionary::Dictionary, key_set::KeySet, penalty::Penalty, util::choose,
    util::DurationFormatter, word::Word,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};
use thousands::Separable;

pub struct OverlapPenalties(HashMap<KeySet, Penalty>);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CsvRow {
    pairs: String,
    penalty: f32,
}

type Common = String;

impl OverlapPenalties {
    pub fn penalty_for(&self, key_set: &KeySet) -> Penalty {
        self.0
            .get(key_set)
            .map(|p| p.clone())
            .unwrap_or(Penalty::ZERO)
    }

    pub fn build(dictionary: &Dictionary) -> OverlapPenalties {
        let words_count = dictionary.words().len();
        let total_pairs = choose(words_count as u32, 2);
        let processed = Arc::new(AtomicU64::new(0));
        let result: Arc<Mutex<HashMap<KeySet, HashMap<Common, HashSet<Word>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
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
                    if diff.len() > 0 && diff.len() <= 2 && diff.letter_count() <= 4 {
                        let common = word_a.overlap(&word_b, '_').unwrap();
                        let mut result = result.lock().unwrap();
                        let word_set = result.entry(diff).or_default().entry(common).or_default();
                        word_set.insert(word_a.clone());
                        word_set.insert(word_b.clone());
                    }
                },
            );
        }
        let result = result
            .lock()
            .unwrap()
            .iter()
            .map(|(pair, m)| {
                let key_set = pair.clone();
                let penalty = Penalty::new(
                    m.iter()
                        .map(|(_common, words)| {
                            words
                                .iter()
                                .map(|w| w.frequency().to_f32())
                                .enumerate()
                                .map(|(index, f)| index.min(4) as f32 * f)
                                .fold(0.0, |total, i| total + i)
                        })
                        .sum(),
                );
                (key_set, penalty)
            })
            .collect::<HashMap<KeySet, Penalty>>();
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
