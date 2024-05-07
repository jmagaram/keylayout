use crate::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty};
use hashbrown::HashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::ops::RangeInclusive;

pub struct SingleKeyPenalties {
    map: HashMap<Key, Penalty>,
    min_key_size: u8,
    max_key_size: u8,
}

impl SingleKeyPenalties {
    pub fn new(dictionary: &Dictionary, key_sizes: RangeInclusive<u8>) -> SingleKeyPenalties {
        let min_key_size = key_sizes.clone().min().unwrap();
        let max_key_size = key_sizes.clone().max().unwrap();
        let alphabet = dictionary.alphabet();
        let penalties_per_key = key_sizes
            .flat_map(|key_size| alphabet.subsets_of_size(key_size))
            .collect::<Vec<Key>>()
            .into_par_iter()
            .map(|k| {
                let keyboard = Keyboard::with_keys(vec![k]).fill_missing(alphabet);
                let penalty = keyboard.penalty(dictionary, Penalty::MAX);
                (k, penalty)
            })
            .collect::<Vec<(Key, Penalty)>>();
        let result: HashMap<Key, Penalty> = HashMap::from_iter(penalties_per_key);
        SingleKeyPenalties {
            map: result,
            min_key_size,
            max_key_size,
        }
    }

    pub fn min_key_size(&self) -> u8 {
        self.min_key_size
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
