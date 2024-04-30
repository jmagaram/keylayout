use dictionary::Dictionary;
use penalty::Penalty;
use prohibited::Prohibited;

mod dfs_pruning;
mod dictionary;
mod exhaustive_n_key;
mod frequency;
mod generate_stats;
mod genetic;
mod key;
mod key_sizes_tree;
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

fn generate_keyboard_stats() {
    let samples = 2_500;
    let pairs = 60;
    let dictionary = Dictionary::load();
    let file_name = format!("kbd_{}_pairs_full_dict.txt", pairs);
    let prohibited = Prohibited::with_top_n_letter_pairs(&dictionary, pairs);
    generate_stats::random_keyboards(samples, &dictionary, &prohibited, &file_name).unwrap();
}

fn dfs_pruning() {
    dfs_pruning::solve();
}

fn find_best_n_key() {
    let dict = Dictionary::load();
    let best = exhaustive_n_key::find_best_n_key(25, &dict);
    match best {
        None => {
            println!("None found");
        }
        Some(k) => {
            println!("{}", k);
        }
    }
}

fn genetic_solver() {
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
}

fn main() {
    dfs_pruning();
    // generate_keyboard_stats();
    // dfs_pruning();
}
