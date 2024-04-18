use key::Key;
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
    SmarterGenetic,
    BestNKey(u32),
}

fn main() {
    let genetic = Run::Genetic(genetic::Args {
        threads: 8,
        die_threshold: Penalty::new(0.0001),
        verbose_print: false,
    });

    let merge_keys = Run::MergeKeys(merge_keys::Args {
        total_words: 90000,
        max_penalty: Penalty::new(0.020),
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

    let smarter_genetic = Run::SmarterGenetic;

    let best_n_key = Run::BestNKey(2);

    let run = best_n_key;

    match run {
        Run::Genetic(threads) => genetic::solve(threads),
        Run::MergeKeys(penalty) => merge_keys::solve(penalty),
        Run::SmarterGenetic => genetic::smarter_genetic(),
        Run::BestNKey(count) => scratch::best_n_key(count),
    }
}
