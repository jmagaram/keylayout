use penalty::Penalty;

mod dictionary;
mod english;
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
    // For bad pairs, they show up around 90
    // For bad triples, they show up around 1880

    let genetic = Run::Genetic(genetic::Args {
        threads: 8,
        die_threshold: Penalty::new(0.0001),
        verbose_print: false,
        exclude_on_any_key: english::top_penalties(75, 200),
        words_in_dictionary: 150000,
    });

    let merge_keys = Run::MergeKeys(merge_keys::Args {
        total_words: 90000,
        max_penalty: Penalty::new(0.021),
        never_together: english::top_penalties(75, 1000),
    });

    let best_n_key = Run::BestNKey(2);

    let run = genetic;

    match run {
        Run::Genetic(threads) => genetic::solve(threads),
        Run::MergeKeys(penalty) => merge_keys::solve(penalty),
        Run::BestNKey(count) => scratch::best_n_key(count),
    }
}
