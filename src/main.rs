use dictionary::Dictionary;
use humantime::{format_duration, FormattedDuration};
use key_set::KeySet;
use keyboard::Keyboard;
use pair_penalties::PairPenalties;
use partitions::Partitions;
use penalty::Penalty;
use prohibited::Prohibited;
use single_key_penalties::SingleKeyPenalties;
use std::time::Duration;
use thousands::Separable;

mod dfs_pruning;
mod dictionary;
mod exhaustive_n_key;
mod frequency;
mod genetic;
mod key;
mod key_set;
mod keyboard;
mod lazy_tree;
mod letter;
mod pair_penalties;
mod pair_penalties_with_sqlite;
mod pairing;
mod partitions;
mod penalty;
mod penalty_goal;
mod prohibited;
mod single_key_penalties;
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

fn show_unique_keyboard_totals() {
    for max_key_size in 3..=7 {
        println!();
        println!("Maximum key size: {}", max_key_size);
        for total_keys in 10..=27 {
            let p = Partitions {
                sum: 27,
                parts: total_keys,
                min: 1,
                max: max_key_size,
            };
            let total_keyboards = p.total_unique_keyboards();
            println!(
                "  keys: {:<2} {}",
                total_keys,
                total_keyboards.separate_with_underscores()
            );
        }
    }
}

fn calculate_pair_penalties() {
    let _ = pair_penalties_with_sqlite::run(None).unwrap();
}

fn custom() {
    println!("Loading...");
    let p = PairPenalties::load();
    println!("Done loading.");

    let ks = KeySet::with_layout("ai");
    println!("{}, {}", ks, p.penalty(&ks));

    let ks = KeySet::with_layout("xy");
    println!("{}, {}", ks, p.penalty(&ks));
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

fn genetic() {
    let dict = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 50);
    let single_key_penalties = SingleKeyPenalties::load();
    let args = genetic::FindBestArgs {
        dictionary: &dict,
        die_threshold: Penalty::new(0.000001),
        key_count: 10,
        prohibited,
        single_key_penalties: &single_key_penalties,
    };
    for result in genetic::find_best(args) {
        if let Some(solution) = result {
            let keyboard = solution.keyboard().clone();
            let penalty = keyboard.penalty(&dict, Penalty::MAX);
            let solution = keyboard.to_solution(penalty, solution.notes());
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
    let choices: Vec<(&str, fn() -> ())> = vec![
        ("DFS search", dfs_pruning),
        ("DFS preconfigured", dfs_pruning_preconfigured),
        ("Genetic algorithm", genetic),
        ("Find best N key", find_best_n_key),
        ("Print keyboard score", print_keyboard_score),
        ("Calculate word pair penalties", calculate_pair_penalties),
        ("Show unique keyboard totals", show_unique_keyboard_totals),
    ];
    let selection = choices
        .iter()
        .map(|(c, _)| c)
        .fold(
            Select::new().with_prompt("What do you want to do?"),
            |menu, item| menu.item(item),
        )
        .default(2)
        .interact()
        .unwrap();
    let command = choices.iter().nth(selection).map(|(_, f)| f);
    match command {
        Some(f) => {
            f();
            println!();
        }
        None => {
            panic!("Do not know how to handle that selection.");
        }
    }
}
