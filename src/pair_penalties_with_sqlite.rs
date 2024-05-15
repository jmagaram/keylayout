use std::collections::HashMap;

use rusqlite::{Connection, Result};
use thousands::Separable;

use crate::{dictionary::Dictionary, util::choose, word::Word};

const PATH: &str = "./storage.db3";

// pub fn create_database()->Result<()> {
//     let conn = Connection::open(PATH)?;
//     conn.execute("CREATE TABLE ", )
//     Ok(())
// }

pub fn delete_dictionary() -> Result<()> {
    println!("Deleting dictionary...");
    let conn = Connection::open(PATH)?;
    conn.execute("DELETE FROM word", ())?;
    Ok(())
}

pub fn vacuum() -> Result<()> {
    println!("Vacuum...");
    let conn = Connection::open(PATH)?;
    conn.execute("VACUUM", ())?;
    Ok(())
}

pub fn delete_conflict() -> Result<()> {
    println!("Deleting conflicts...");
    let conn = Connection::open(PATH)?;
    conn.execute("DELETE FROM conflict", ())?;
    Ok(())
}

pub fn load_words() -> Result<Vec<(usize, Word)>> {
    let conn = Connection::open(PATH)?;
    let mut qry = conn.prepare("SELECT word, frequency, word_id FROM word ORDER BY word_id")?;
    let words_iter = qry.query_map([], |row| {
        let w: String = row.get(0)?;
        let f: f32 = row.get(1)?;
        let id: usize = row.get(2)?;
        let word = Word::new(w.as_str(), f);
        Ok((id, word))
    })?;
    let result = words_iter
        .map(|i| i.unwrap())
        .collect::<Vec<(usize, Word)>>();
    Ok(result)
}

pub fn run(dictionary_size: Option<usize>) -> Result<()> {
    delete_conflict()?;
    delete_dictionary()?;
    vacuum()?;
    write_dictionary(dictionary_size)?;
    let words = load_words()?;
    let total_items = choose(words.len().try_into().unwrap(), 2);
    let mut processed: u128 = 0;
    let mut conn = Connection::open(PATH)?;
    let max_keys = 3;
    let max_letters = 6;
    for word_a_index in 0..words.len() - 1 {
        let mut tx = conn.transaction()?;
        for word_b_index in word_a_index + 1..words.len() {
            processed = processed + 1;
            if processed.rem_euclid(100_000) == 0 {
                println!(
                    "{} of {}",
                    processed.separate_with_underscores(),
                    total_items.separate_with_underscores()
                );
            }
            let (word_a_id, word_a) = words[word_a_index].clone();
            let (word_b_id, word_b) = words[word_b_index].clone();
            let diff = word_a.letter_pair_difference(&word_b);
            if diff.len() >= 1 && diff.len() <= max_keys && diff.letter_count() <= max_letters {
                let letter_count = diff.letter_count();
                let diff_as_string = diff.to_string();
                let _ = tx.execute(
                    "INSERT INTO conflict (pair, word_id, letter_count) VALUES (?1, ?2, ?3)",
                    (&diff_as_string, &word_a_id, &letter_count),
                );
                let _ = tx.execute(
                    "INSERT INTO conflict (pair, word_id, letter_count) VALUES (?1, ?2, ?3)",
                    (&diff_as_string, &word_b_id, &letter_count),
                );
            }
        }
        tx.commit()?;
    }
    Ok(())
}

// pub fn write_conflicts() -> Result<()> {
//     let dictionary = load_words()?;
//     let words = dictionary.words();
//     let mut conn = Connection::open(PATH)?;
//     let max_keys = 4;
//     let max_letters = 8;
//     for word_a_index in 0..words.len() - 1 {
//         println!("{}", word_a_index.separate_with_underscores());
//         let mut tx = conn.transaction()?;
//         for word_b_index in word_a_index + 1..words.len() {
//             let word_a = &words[word_a_index];
//             let word_a_string = word_a.to_string();
//             let word_b = &words[word_b_index];
//             let word_b_string = word_b.to_string();
//             let word_b_real_index = word_a_index + word_b_index + 1;
//             let diff = word_a.letter_pair_difference(&word_b);
//             let letter_count = diff.letter_count();
//             if diff.len() >= 1 && diff.len() <= max_keys && diff.letter_count() <= max_letters {
//                 let diff_as_string = diff.to_string();
//                 tx.execute(
//                     "INSERT INTO conflict (pair, word_id, letter_count) VALUES (?1, ?2, ?3)",
//                     (&diff_as_string, &word_a_index, &letter_count),
//                 );
//                 tx.execute(
//                     "INSERT INTO conflict (pair, word_id, letter_count) VALUES (?1, ?2, ?3)",
//                     (&diff_as_string, &word_b_real_index, &letter_count),
//                 );
//             }
//         }
//         tx.commit()?;
//     }
//     Ok(())
// }

pub fn write_dictionary(count: Option<usize>) -> Result<()> {
    println!("Writing dictionary...");
    let mut conn = Connection::open(PATH)?;
    let mut tx = conn.transaction()?;
    let d = match count {
        Some(count) => Dictionary::load().filter_top_n_words(count),
        None => Dictionary::load(),
    };
    for (index, w) in d.words().iter().enumerate() {
        tx.execute(
            "INSERT INTO word (word_id, word, frequency) VALUES (?1, ?2, ?3)",
            (&index, &w.to_string(), &w.frequency().to_f32()),
        )?;
    }
    tx.commit()?;
    Ok(())
}
