use crate::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty};
use hashbrown::HashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use thousands::Separable;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TwoKeyCsv {
    key1: String,
    key2: String,
    penalty: f32,
}

pub struct TwoKeyPenalties(HashMap<(Key, Key), Penalty>);

impl TwoKeyPenalties {
    const FILE_NAME: &'static str = "two_key_penalties.csv";

    fn load_from_csv() -> Result<Vec<TwoKeyCsv>, csv::Error> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(Self::FILE_NAME)
            .unwrap();
        rdr.deserialize().collect::<Result<Vec<TwoKeyCsv>, _>>()
    }

    pub fn load_from() -> TwoKeyPenalties {
        let map_items = Self::load_from_csv().unwrap().into_iter().map(|c| {
            let key1 = Key::new(c.key1.as_str());
            let key2 = Key::new(c.key2.as_str());
            let p = Penalty::new(c.penalty);
            ((key1, key2), p)
        });
        let map = HashMap::from_iter(map_items);
        TwoKeyPenalties(map)
    }

    pub fn save_csv(&self) -> Result<(), csv::Error> {
        let mut wtr = csv::Writer::from_path(Self::FILE_NAME).unwrap();
        let _write_result = self
            .0
            .iter()
            .map(|((key1, key2), v)| TwoKeyCsv {
                key1: key1.to_string(),
                key2: key2.to_string(),
                penalty: v.to_f32(),
            })
            .map(|r| wtr.serialize(r))
            .collect::<Result<(), csv::Error>>()?;
        wtr.flush()?;
        Ok(())
    }

    pub fn new(dictionary: &Dictionary) -> TwoKeyPenalties {
        let count = Arc::new(AtomicU32::new(0));
        let alphabet = dictionary.alphabet();
        let pairs_of_pairs = alphabet
            .subsets_of_size(2)
            .flat_map(|key1| alphabet.subsets_of_size(2).map(move |key2| (key1, key2)))
            .filter(|(key1, key2)| key1.to_u32() < key2.to_u32())
            .filter(|(key1, key2)| (*key1).intersect(*key2) == Key::EMPTY)
            .collect::<Vec<(Key, Key)>>();
        let total = pairs_of_pairs.len();
        let penalties = pairs_of_pairs
            .into_par_iter()
            .map(|(key1, key2)| {
                let keyboard = Keyboard::with_keys(vec![key1, key2]).fill_missing(alphabet);
                let penalty = keyboard.penalty(dictionary, Penalty::MAX);
                ((key1, key2), penalty)
            })
            .inspect(|_| {
                let evaluated = count.fetch_add(1, Ordering::Relaxed);
                if evaluated.rem_euclid(1000) == 0 {
                    println!(
                        "Calculating penalties for {} of {} pairs of keys",
                        evaluated.separate_with_underscores(),
                        total.separate_with_underscores()
                    );
                }
            })
            .collect::<Vec<((Key, Key), Penalty)>>();
        let result: HashMap<(Key, Key), Penalty> = HashMap::from_iter(penalties);
        TwoKeyPenalties(result)
    }

    pub fn get(&self, key_1: Key, key_2: Key) -> Option<Penalty> {
        let ordered = {
            if key_1.to_u32() < key_2.to_u32() {
                (key_1, key_2)
            } else {
                (key_2, key_1)
            }
        };
        self.0.get(&ordered).map(|p| p.clone())
    }
}
