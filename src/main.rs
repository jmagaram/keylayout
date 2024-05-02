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
mod random_solutions;
mod solution;
mod tally;
mod util;
mod word;

fn generate_keyboard_stats() {
    let samples = 2_500;
    let pairs = 60;
    let dictionary = Dictionary::load().filter_top_n_words(120_000);
    let file_name = format!("kbd_{}_pairs_120k_dict.txt", pairs);
    let prohibited = Prohibited::with_top_n_letter_pairs(&dictionary, pairs);
    generate_stats::random_keyboards(samples, &dictionary, &prohibited, &file_name).unwrap();
}

fn dfs_pruning() {
    let args = dfs_pruning::SolveArgs::new_from_prompts();
    dfs_pruning::solve(&args);
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
use dialoguer::{Input, Select};

fn main() {
    let selection = Select::new()
        .with_prompt("What do you want to do?")
        .item("DFS search")
        .item("Genetic algorithm")
        .default(0)
        .interact()
        .unwrap();
    match selection {
        0 => dfs_pruning(),
        1 => genetic_solver(),
        _ => {
            println!("Not sure what do with that selection");
        }
    }
}
