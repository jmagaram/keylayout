use penalty::Penalty;

mod dictionary;
mod experiment;
mod frequency;
mod genetic;
mod item_count;
mod key;
mod keyboard;
mod letter;
mod merge_keys_dfs;
mod partitions;
mod penalty;
mod permutable;
mod solution;
mod util;
mod word;

enum Run {
    Genetic(genetic::Args),
    MergeKeys(merge_keys_dfs::Args),
}

fn main() {
    let genetic_args = Run::Genetic(genetic::Args { threads: 8 });

    let merge_keys_args = Run::MergeKeys(merge_keys_dfs::Args {
        max_penalty: Penalty::new(0.050),
    });

    let run = genetic_args;

    match run {
        Run::Genetic(threads) => genetic::solve(threads),
        Run::MergeKeys(penalty) => merge_keys_dfs::solve(penalty),
    }
}
