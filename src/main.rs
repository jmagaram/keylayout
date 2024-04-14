use key::Key;
use letter::Letter;
use penalty::Penalty;

mod dictionary;
mod frequency;
mod genetic;
mod item_count;
mod key;
mod keyboard;
mod letter;
mod merge_keys;
mod partitions;
mod penalty;
mod permutable;
mod solution;
mod util;
mod word;

enum Run {
    Genetic(genetic::Args),
    MergeKeys(merge_keys::Args),
}

fn main() {
    let genetic = Run::Genetic(genetic::Args {
        threads: 8,
        die_threshold: Penalty::new(0.001),
        verbose_print: false,
    });

    let merge_keys = Run::MergeKeys(merge_keys::Args {
        max_penalty: Penalty::new(0.050),
        never_together: vec![
            Key::EMPTY
                .add(Letter::new('a'))
                .add(Letter::new('e'))
                .add(Letter::new('i'))
                .add(Letter::new('o'))
                .add(Letter::new('u')),
            Key::EMPTY
                .add(Letter::new('e'))
                .add(Letter::new('a'))
                .add(Letter::new('r'))
                .add(Letter::new('i'))
                .add(Letter::new('s')),
        ],
    });

    let run = genetic;

    match run {
        Run::Genetic(threads) => genetic::solve(threads),
        Run::MergeKeys(penalty) => merge_keys::solve(penalty),
    }
}
