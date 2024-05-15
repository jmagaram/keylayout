use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rusqlite::{Connection, Result};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
};
use thousands::Separable;

use crate::{dictionary::Dictionary, util::choose, word::Word};

const PATH: &str = "./pair_penalties.db3";

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

fn create_database() {
    // Can have duplicates; should be cleaned up
    let make_conflict_table = r#"
CREATE TABLE "conflict" (
  "word_id" INTEGER NOT NULL,
  "pair" TEXT NOT NULL,
  "letter_count" INTEGER NOT NULL,
  FOREIGN KEY("word_id") REFERENCES "word"("word_id")
); 
"#;
    let make_word_table = r#"
CREATE TABLE "word" (
  "word_id" INTEGER NOT NULL,
  "word" TEXT NOT NULL UNIQUE,
  "frequency" NUMERIC NOT NULL,
  PRIMARY KEY("word_id")
); 
"#;
    let conn = Connection::open(PATH).unwrap();
    conn.execute(make_word_table, []);
    conn.execute(make_conflict_table, []);
}

fn remove_conflict_duplicates() {
    let statement = r#"
DELETE FROM conflict
WHERE rowid NOT IN (
SELECT MIN(rowid) FROM conflict GROUP BY word_id, pair
);
"#;
    let conn = Connection::open(PATH).unwrap();
    conn.execute(statement, []);
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

pub fn run(dictionary_size: Option<usize>) {
    create_database();
    write_dictionary(dictionary_size).unwrap();
    let words = load_words().unwrap();
    let total_items = choose(words.len().try_into().unwrap(), 2);
    let processed: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    let max_keys = 3;
    let max_letters = 6;
    (0..words.len() - 1)
        .into_par_iter()
        .for_each_with(processed, |processed, word_a_index| {
            let mut conn = Connection::open(PATH).unwrap();
            let tx = conn.transaction().unwrap();
            for word_b_index in word_a_index + 1..words.len() {
                let processed_count = processed.fetch_add(1, Ordering::Relaxed);
                if processed_count > 0 && processed_count.rem_euclid(1_000_000_000) == 0 {
                    thread::spawn(|| {
                        println!("Cleaning...");
                        remove_conflict_duplicates();
                        vacuum();
                        println!("Done cleaning.");
                    });
                }
                if processed_count.rem_euclid(10_000_000) == 0 {
                    println!(
                        "SQL {} of {}",
                        processed_count.separate_with_underscores(),
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
            tx.commit().unwrap();
        });
    remove_conflict_duplicates();
    vacuum().unwrap();
}

pub fn write_dictionary(count: Option<usize>) -> Result<()> {
    println!("Writing dictionary...");
    let mut conn = Connection::open(PATH)?;
    let tx = conn.transaction()?;
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
