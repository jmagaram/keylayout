use dictionary::Dictionary;
use penalty::Penalty;
use prohibited::Prohibited;

mod dfs_pruning;
mod dictionary;
mod exhaustive_n_key;
mod frequency;
mod genetic;
mod key;
mod keyboard;
mod lazy_tree;
mod letter;
mod partitions;
mod penalty;
mod penalty_goal;
mod prohibited;
mod solution;
mod tally;
mod util;
mod word;

fn main() {
    let dfs_pruning = || {
        dfs_pruning::solve();
    };

    let find_best_n_key = || {
        let dict = Dictionary::load();
        exhaustive_n_key::find_best_n_key(10, &dict);
    };

    let genetic_solver = move || {
        let dict = Dictionary::load();
        let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 50);
        let args = genetic::FindBestArgs {
            dictionary: &dict,
            die_threshold: Penalty::new(0.00001),
            key_count: 10,
            prohibited,
        };
        for result in genetic::find_best(args) {
            if let Some(solution) = result {
                println!("{}", solution);
            }
        }
    };

    find_best_n_key();
}
