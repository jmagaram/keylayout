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

pub struct SingleKeyPenalties {
    map: HashMap<Key, Penalty>,
    max_key_size: u8,
}

impl SingleKeyPenalties {
    pub fn new(dictionary: &Dictionary, key_sizes: RangeInclusive<u8>) -> SingleKeyPenalties {
        let max_key_size = key_sizes.clone().max().unwrap();
        let count = Arc::new(AtomicU32::new(0));
        let alphabet = dictionary.alphabet();
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
