use dictionary::Dictionary;
use humantime::{format_duration, FormattedDuration};
use keyboard::Keyboard;
use penalty::Penalty;
use prohibited::Prohibited;
use std::time::Duration;

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
mod solution_samples;
mod tally;
mod util;
mod vec_threads;
mod word;

trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

fn save_random_keyboard_penalties() {
    let d = Dictionary::load();
    for prohibited_pairs in [0, 20, 40, 60, 80] {
        println!("Working on prohibited pairs: {}", prohibited_pairs);
        let prohibited = Prohibited::with_top_n_letter_pairs(&d, prohibited_pairs);
        let args = solution_samples::Args {
            dictionary: &d,
            key_count: 10..=26,
            min_key_size: 1,
            max_key_size: 5,
            prohibited: &prohibited,
            samples_per_key_count: 5000,
            thread_count: 4,
        };
        args.save_to_csv().unwrap();
    }
}

fn pairs_then_infrequent() {
    use exhaustive_n_key::*;
    let args = PopularLetterPairingsArgs {
        pair_up: "eaisrnotlcdumhgpbykf".to_string(),
        infrequent_replacement: 'z',
    };
    let best_pairings = args.solve();
    let pairs = best_pairings.keyboard().filter_keys(|k| k.len() > 1);
    let args = FillArgs {
        start: pairs.to_string(),
        max_key_size: 6,
        update_every: 100_000,
    };
    args.solve();
}

fn dfs_pruning() {
    let args = dfs_pruning::SolveArgs::new_from_prompts();
    dfs_pruning::solve(&args);
}

fn dfs_pruning_preconfigured() {
    let args = dfs_pruning::SolveArgs::preconfigured();
    dfs_pruning::solve(&args);
}

fn find_best_n_key() {
    let args = exhaustive_n_key::Args::new_from_prompts();
    let best = args.solve();
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
        die_threshold: Penalty::new(0.0001),
        key_count: 10,
        prohibited,
    };
    for result in genetic::find_best(args) {
        if let Some(solution) = result {
            println!("{}", solution);
        }
    }
}

fn print_keyboard_score() {
    let layout = "afj bn cl dhx' evwz gr im kpy oqt su";
    let keyboard = Keyboard::with_layout(layout);
    let dict_full = Dictionary::load();
    let penalty = keyboard.penalty(&dict_full, Penalty::MAX);
    let solution = keyboard.to_solution(penalty, "".to_string());
    println!("{}", solution);
}

fn main() {
    use dialoguer::Select;
    let selection = Select::new()
        .with_prompt("What do you want to do?")
        .item("DFS search")
        .item("DFS preconfigured")
        .item("Genetic algorithm")
        .item("Find best N key")
        .item("Save random keyboard penalties to CSV")
        .item("Print keyboard score")
        .item("Custom")
        .default(0)
        .interact()
        .unwrap();
    println!();
    match selection {
        0 => dfs_pruning(),
        1 => dfs_pruning_preconfigured(),
        2 => genetic_solver(),
        3 => find_best_n_key(),
        4 => save_random_keyboard_penalties(),
        5 => print_keyboard_score(),
        6 => pairs_then_infrequent(),
        _ => panic!("Do not know how to handle that selection."),
    }
}
