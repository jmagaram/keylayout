use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    prohibited::Prohibited, solution::Solution, vec_threads::VecThreads,
};
use std::{ops::RangeInclusive, sync::Arc, thread};

#[derive(Clone)]
pub struct Args<'a> {
    pub samples_per_key_count: usize,
    pub dictionary: &'a Dictionary,
    pub prohibited: &'a Prohibited,
    pub key_count: RangeInclusive<u8>,
    pub min_key_size: u8,
    pub max_key_size: u8,
    pub thread_count: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CsvOutput {
    index: usize,
    keyboard: String,
    keys: usize,
    max_key_size: u8,
    min_key_size: u8,
    penalty: f32,
    count_1: u8,
    count_2: u8,
    count_3: u8,
    count_4: u8,
    count_5: u8,
    count_6: u8,
}

impl CsvOutput {
    pub fn penalty(&self) -> Penalty {
        Penalty::new(self.penalty)
    }

    pub fn keys(&self) -> u32 {
        self.keys as u32
    }

    pub fn load_from_csv(file_name: &str) -> Result<Vec<CsvOutput>, csv::Error> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_name)
            .unwrap();
        rdr.deserialize().collect::<Result<Vec<CsvOutput>, _>>()
    }

    pub fn new(s: &Solution, index: usize) -> CsvOutput {
        let k = s.keyboard();
        let tally = k.key_sizes();
        CsvOutput {
            index,
            keyboard: k.to_string(),
            penalty: s.penalty().to_f32(),
            keys: k.len(),
            max_key_size: k.max_key_size().unwrap(),
            min_key_size: k.min_key_size().unwrap(),
            count_1: tally.count(&1) as u8,
            count_2: tally.count(&2) as u8,
            count_3: tally.count(&3) as u8,
            count_4: tally.count(&4) as u8,
            count_5: tally.count(&5) as u8,
            count_6: tally.count(&6) as u8,
        }
    }
}

impl<'a> Args<'a> {
    pub fn save_to_csv(&'a self) -> Result<(), csv::Error> {
        let file_name = format!(
            "kbd_penalties_{}_samples_{}_pairs.csv",
            self.samples_per_key_count,
            self.prohibited.len()
        );
        let mut wtr = csv::Writer::from_path(file_name)?;
        let _save_result = self
            .calculate()
            .iter()
            .enumerate()
            .map(|(index, solution)| CsvOutput::new(&solution, index))
            .map(|r| wtr.serialize(r))
            .collect::<Result<(), csv::Error>>()?;
        wtr.flush()?;
        Ok(())
    }

    fn partitions(&'a self) -> impl Iterator<Item = Partitions> + 'a {
        let alphabet_size = self.dictionary.alphabet().len();
        assert!(self
            .key_count
            .clone()
            .all(|key_count| key_count * self.min_key_size <= alphabet_size
                && key_count * self.max_key_size >= alphabet_size));
        self.key_count.clone().map(move |key_count| Partitions {
            sum: alphabet_size,
            parts: key_count,
            min: self.min_key_size,
            max: self.max_key_size,
        })
    }

    pub fn calculate(&'a self) -> Vec<Solution> {
        let work: VecThreads<Keyboard> = VecThreads::new();

        for p in self.partitions() {
            for k in Keyboard::random(self.dictionary.alphabet(), p, &self.prohibited)
                .take(self.samples_per_key_count)
            {
                work.clone().push(k);
            }
        }

        let mut children = vec![];
        let solutions: VecThreads<Solution> = VecThreads::new();
        let dictionary = Arc::new(self.dictionary.to_owned());
        for _consumer_index in 1..=self.thread_count {
            let mut work = work.clone();
            let mut solutions = solutions.clone();
            let dictionary = dictionary.clone();
            let handle = thread::spawn(move || loop {
                let k = work.pop();
                match k {
                    None => {
                        break;
                    }
                    Some(k) => {
                        let penalty = k.penalty(&dictionary, Penalty::MAX);
                        let solution = k.to_solution(penalty, "".to_string());
                        solutions.push(solution);
                    }
                }
            });
            children.push(handle);
        }
        for child in children {
            let _ = child.join();
        }

        solutions.items()
    }
}
