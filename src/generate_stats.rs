use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    prohibited::Prohibited, solution::Solution, util,
};
use humantime::{format_duration, FormattedDuration};
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::RangeInclusive,
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
pub fn random_keyboards_of_key_count(
    count: usize,
    key_sizes: Partitions,
    dictionary: &Dictionary,
    prohibited: &Prohibited,
    file_name: &str,
) -> Result<(), std::io::Error> {
    let file = File::create(file_name)?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "index,keyboard,keys,penalty,max_key,min_key,count_1,count_2,count_3,count_4,count_5,count_6"
    )?;
    Keyboard::random(dictionary.alphabet(), key_sizes, prohibited)
        .take(count)
        .enumerate()
        .map(|(index, k)| {
            if index.rem_euclid(100) == 0 {
                println!(
                    "Generated {} of {}",
                    index.separate_with_underscores(),
                    count.separate_with_underscores()
                );
            }
            let tally = k.key_sizes();
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                index,
                k.to_string(),
                k.len(),
                k.penalty(dictionary, Penalty::MAX).to_f32(),
                k.max_key_size().unwrap(),
                k.min_key_size().unwrap(),
                tally.count(&1),
                tally.count(&2),
                tally.count(&3),
                tally.count(&4),
                tally.count(&5),
                tally.count(&6),
            )
        })
        .collect::<Result<(), _>>()?;
    writer.flush()?;
    Ok(())
}

#[derive(Clone)]
pub struct Args<'a> {
    pub samples_per_key_count: usize,
    pub dictionary: &'a Dictionary,
    pub prohibited: &'a Prohibited,
    pub key_count: RangeInclusive<u32>,
    pub min_key_size: u32,
    pub max_key_size: u32,
}

impl<'a> Args<'a> {
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

    pub fn keyboards(&'a self) -> impl Iterator<Item = Solution> + 'a {
        self.partitions().flat_map(|p| {
            Keyboard::random(self.dictionary.alphabet(), p, self.prohibited)
                .take(self.samples_per_key_count)
                .map(|k| {
                    let penalty = k.penalty(&self.dictionary, Penalty::MAX);
                    k.to_solution(penalty, "".to_string())
                })
        })
    }

    pub fn print(&self) {
        let now = Instant::now();
        for k in self.keyboards() {
            println!("{}", k);
        }
        println!("Elapsed: {}", now.elapsed().round_to_seconds())
    }
}

pub fn random_keyboards(
    samples_per_key_count: usize,
    dictionary: &Dictionary,
    prohibited: &Prohibited,
    file_name: &str,
) -> Result<(), std::io::Error> {
    let file = File::create(file_name)?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "index,keyboard,keys,penalty,max_key,min_key,count_1,count_2,count_3,count_4,count_5,count_6"
    )?;
    let letter_count = dictionary.alphabet().len();
    (10..=letter_count - 1)
        .map(|key_count| {
            let key_sizes = Partitions {
                sum: letter_count,
                parts: key_count,
                min: 1,
                max: ((letter_count / key_count) + 3).min(letter_count),
            };
            Keyboard::random(dictionary.alphabet(), key_sizes, &prohibited)
                .take(samples_per_key_count)
                .enumerate()
                .map(|(index, k)| {
                    if index.rem_euclid(100) == 0 {
                        println!(
                            "key_count: {} of {}, generated {} of {}",
                            key_count,
                            letter_count,
                            index.separate_with_underscores(),
                            samples_per_key_count.separate_with_underscores()
                        );
                    }
                    let tally = k.key_sizes();
                    writeln!(
                        writer,
                        "{},{},{},{},{},{},{},{},{},{},{},{}",
                        index,
                        k.to_string(),
                        k.len(),
                        k.penalty(dictionary, Penalty::MAX).to_f32(),
                        k.max_key_size().unwrap(),
                        k.min_key_size().unwrap(),
                        tally.count(&1),
                        tally.count(&2),
                        tally.count(&3),
                        tally.count(&4),
                        tally.count(&5),
                        tally.count(&6),
                    )
                })
                .collect::<Result<(), _>>()
        })
        .collect::<Result<(), _>>()?;
    writer.flush()?;
    Ok(())
}
