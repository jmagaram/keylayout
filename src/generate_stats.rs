use thousands::Separable;

use crate::{
    dictionary::Dictionary, keyboard::Keyboard, partitions::Partitions, penalty::Penalty,
    prohibited::Prohibited,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub fn random_keyboards(
    count: usize,
    key_sizes: &Partitions,
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
