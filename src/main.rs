use dictionary::Dictionary;

use penalty::Penalty;

mod dictionary;
mod english;
mod evolution;
mod frequency;
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
mod word_tally;

enum Run {
    MergeKeys(merge_keys::Args),
    BestNKey(u32),
    Other,
}

fn main() {
    // For bad pairs, they show up around 90
    // For bad triples, they show up around 1880

    let dict = Dictionary::load();

    let merge_keys = Run::MergeKeys(merge_keys::Args {
        total_words: 200000,
        max_penalty: Penalty::new(0.0235),
        never_together: english::top_penalties(75, 500),
    });

    let other = Run::Other;

    let best_n_key = Run::BestNKey(2);

    let run = other;

    match run {
        Run::MergeKeys(penalty) => merge_keys::solve(penalty),
        Run::BestNKey(count) => scratch::best_n_key(count),
        Run::Other => loop {
            evolution::evolve_one_random_keyboard();
        },
    }
}
