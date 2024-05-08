use crate::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty, util::choose};
use hashbrown::HashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    ops::RangeInclusive,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use thousands::Separable;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CsvOutput {
    key: String,
    penalty: f32,
}

pub struct SingleKeyPenalties {
    map: HashMap<Key, Penalty>,
    max_key_size: u8,
}

impl SingleKeyPenalties {
    fn file_name(max_key_size: u8) -> String {
        format!("single_key_penalties_{}.csv", max_key_size)
    }

    fn load_from_csv(file_name: &str) -> Result<Vec<CsvOutput>, csv::Error> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_name)
            .unwrap();
        rdr.deserialize().collect::<Result<Vec<CsvOutput>, _>>()
    }

    pub fn load() -> SingleKeyPenalties {
        let key_size = 6;
        let file_name = Self::file_name(key_size);
        Self::load_from(file_name.as_str())
    }

    pub fn load_from(file_name: &str) -> SingleKeyPenalties {
        let map_items = Self::load_from_csv(file_name)
            .unwrap()
            .into_iter()
            .map(|c| {
                let k = Key::new(c.key.as_str());
                let p = Penalty::new(c.penalty);
                (k, p)
            });
        let map = HashMap::from_iter(map_items);
        let max_key_size = map.keys().map(|k| k.len()).max().unwrap();
        SingleKeyPenalties { map, max_key_size }
    }

    pub fn save_csv(&self) -> Result<(), csv::Error> {
        let file_name = Self::file_name(self.max_key_size);
        let mut wtr = csv::Writer::from_path(file_name.as_str()).unwrap();
        let _write_result = self
            .map
            .iter()
            .map(|(k, v)| CsvOutput {
                key: k.to_string(),
                penalty: v.to_f32(),
            })
            .map(|r| wtr.serialize(r))
            .collect::<Result<(), csv::Error>>()?;
        wtr.flush()?;
        Ok(())
    }

    pub fn new(dictionary: &Dictionary, max_key_size: u8) -> SingleKeyPenalties {
        let count = Arc::new(AtomicU32::new(0));
        let alphabet = dictionary.alphabet();
        let key_sizes = 2..=max_key_size;
        let total = key_sizes.clone().fold(0, |total, i| {
            total + choose(alphabet.len() as u32, i as u32)
        });
        let penalties_per_key = key_sizes
            .flat_map(|key_size| alphabet.subsets_of_size(key_size))
            .collect::<Vec<Key>>()
            .into_par_iter()
            .map(|k| {
                let keyboard = Keyboard::with_keys(vec![k]).fill_missing(alphabet);
                let penalty = keyboard.penalty(dictionary, Penalty::MAX);
                (k, penalty)
            })
            .inspect(|_| {
                let evaluated = count.fetch_add(1, Ordering::Relaxed);
                if evaluated.rem_euclid(1000) == 0 {
                    println!(
                        "Calculating penalties for {} of {} keys",
                        evaluated.separate_with_underscores(),
                        total.separate_with_underscores()
                    );
                }
            })
            .collect::<Vec<(Key, Penalty)>>();
        let result: HashMap<Key, Penalty> = HashMap::from_iter(penalties_per_key);
        SingleKeyPenalties {
            map: result,
            max_key_size,
        }
    }

    pub fn max_key_size(&self) -> u8 {
        self.max_key_size
    }

    pub fn get(&self, key: Key) -> Option<Penalty> {
        self.map.get(&key).map(|p| p.clone())
    }

    pub fn of_key_size<'a>(&'a self, key_size: u8) -> impl Iterator<Item = (Key, Penalty)> + 'a {
        self.map
            .iter()
            .filter(move |(k, _p)| k.len() == key_size)
            .map(|(k, p)| (*k, *p))
    }
}
