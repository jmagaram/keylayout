use dictionary::Dictionary;
use partitions::Partitions;
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
    let p = Partitions {
        sum: 27,
        min: 1,
        max: 5,
        parts: 10,
    };
    let file_name = "keyboard_stats.txt";
    let samples = 100_000;
    let dictionary = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&dictionary, 30);
    generate_stats::random_keyboards(samples, &p, &dictionary, &prohibited, file_name).unwrap();
}

fn main() {
    let dfs_pruning = || {
        dfs_pruning::solve();
    };

    let find_best_n_key = || {
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

    dfs_pruning();
}
