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

fn display_unique_keyboard_totals() {
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

fn save_single_key_penalties() {
    let d = Dictionary::load();
    let p = SingleKeyPenalties::new(&d, 6);
    p.save_csv().unwrap();
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

fn custom() {
    println!("Loading...");
    let p = PairPenalties::load();
    println!("Done loading.");

    let ks = KeySet::with_layout("ai");
    println!("{}, {}", ks, p.penalty(&ks));

    let ks = KeySet::with_layout("xy");
    println!("{}, {}", ks, p.penalty(&ks));
}

fn combine_infrequent_pairs() {
    let args = pairing::Args {
        threads: 6,
        max_key_size: 6,
        pairings_to_ignore: 70,
        prune_threshold: Penalty::new(0.0250),
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
    let selection = Select::new()
        .with_prompt("What do you want to do?")
        .item("DFS search")
        .item("DFS preconfigured")
        .item("Genetic algorithm")
        .item("Find best N key")
        .item("Save random keyboard penalties to CSV")
        .item("Save single key penalties to CSV")
        .item("Print keyboard score")
        .item("Recursively pair letters")
        .item("Display unique keyboard totals")
        .item("Custom")
        .default(6)
        .interact()
        .unwrap();
    println!();
    match selection {
        0 => dfs_pruning(),
        1 => dfs_pruning_preconfigured(),
        2 => genetic_solver(),
        3 => find_best_n_key(),
        4 => save_random_keyboard_penalties(),
        5 => save_single_key_penalties(),
        6 => print_keyboard_score(),
        7 => combine_infrequent_pairs(),
        8 => display_unique_keyboard_totals(),
        9 => custom(),
        _ => panic!("Do not know how to handle that selection."),
    }
}
