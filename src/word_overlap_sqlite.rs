use crate::{dictionary::Dictionary, util::choose, word::Word};
use rusqlite::{Connection, Result};
use thousands::Separable;

const PATH: &str = "./pair_penalties.db3";

fn create_database() {
    let make_conflict_table = r#"
CREATE TABLE "conflict" (
  "pair" TEXT NOT NULL,
  "word_id" INTEGER NOT NULL,
  "letter_count" INTEGER NOT NULL,
  FOREIGN KEY("word_id") REFERENCES "word"("word_id")
); 
"#;
    let pair_index = r#"CREATE INDEX "pair_index" ON "conflict" ("pair");"#;
    let letter_count_index = r#"CREATE INDEX "letter_count_index" ON "conflict" ("letter_count");"#;
    let make_word_table = r#"
CREATE TABLE "word" (
  "word_id" INTEGER NOT NULL,
  "word" TEXT NOT NULL UNIQUE,
  "frequency" NUMERIC NOT NULL,
  PRIMARY KEY("word_id")
); 
"#;
    let conn = Connection::open(PATH).unwrap();
    conn.execute(make_word_table, []).unwrap();
    conn.execute(make_conflict_table, []).unwrap();
    conn.execute(pair_index, []).unwrap();
    conn.execute(letter_count_index, []).unwrap();
}

pub fn vacuum() -> Result<()> {
    let conn = Connection::open(PATH)?;
    println!("Vacuum...");
    conn.execute("VACUUM", ())?;
    println!("Vacuum - done!");
    Ok(())
}

fn remove_conflict_duplicates() {
    let statement = r#"
DELETE FROM conflict
WHERE rowid NOT IN (
SELECT MIN(rowid) FROM conflict GROUP BY word_id, pair
);
"#;
    let conn = Connection::open(PATH).unwrap();
    println!("Removing pair penalty duplicates...");
    let _ = conn.execute(statement, []);
    println!("Removing pair penalty duplicates - done!");
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
    let mut processed: u64 = 0;
    let max_keys = 3;
    let max_letters = 6;
    for word_a_index in 0..words.len() - 1 {
        let mut conn = Connection::open(PATH).unwrap();
        let tx = conn.transaction().unwrap();
        for word_b_index in word_a_index + 1..words.len() {
            processed = processed + 1;
            if processed.rem_euclid(10_000_000) == 0 {
                println!(
                    "SQL {} of {}",
                    processed.separate_with_underscores(),
                    total_items.separate_with_underscores()
                );
            }
            if processed.rem_euclid(5_000_000_000) == 0 {
                remove_conflict_duplicates();
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
    }
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
    println!("Writing dictionary - done!");
    Ok(())
}
