use key::Key;
use keylayout::keyboard::Keyboard;
use letter::Letter;
use penalty::Penalty;

mod dictionary;
mod frequency;
mod genetic;
mod key;
mod keyboard;
mod lazy_tree;
mod letter;
mod merge_keys;
mod partitions;
mod penalty;
mod scratch;
mod solution;
mod tally;
mod util;
mod word;

enum Run {
    Genetic(genetic::Args),
    MergeKeys(merge_keys::Args),
    BestNKey(u32),
}

fn main() {
    let bad_pairs_to_avoid = 75;
    let bad_pairs = Keyboard::PAIR_PENALTIES
        .into_iter()
        .take(bad_pairs_to_avoid)
        .map(|i| {
            let (letters, _) = i;
            Key::from_iter(letters.chars().map(|c| Letter::new(c)))
        })
        .collect::<Vec<Key>>();

    let genetic = Run::Genetic(genetic::Args {
        threads: 8,
        die_threshold: Penalty::new(0.0001),
        verbose_print: false,
        exclude_on_any_key: bad_pairs.to_vec(),
        words_in_dictionary: 150000,
    });

    let merge_keys = Run::MergeKeys(merge_keys::Args {
        total_words: 90000,
        max_penalty: Penalty::new(0.020),
        never_together: bad_pairs.to_vec(),
    });

    let best_n_key = Run::BestNKey(2);

    let run = genetic;

    match run {
        Run::Genetic(threads) => genetic::solve(threads),
        Run::MergeKeys(penalty) => merge_keys::solve(penalty),
        Run::BestNKey(count) => scratch::best_n_key(count),
    }
}
