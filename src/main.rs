use dictionary::Dictionary;
use humantime::{format_duration, FormattedDuration};
use keyboard::Keyboard;
use partitions::Partitions;
use penalty::Penalty;
use prohibited::Prohibited;
use single_key_penalties::SingleKeyPenalties;
use std::time::Duration;
use thousands::Separable;
use word_overlap::WordOverlap;

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
mod word_overlap;
mod word_overlap_sqlite;

trait DurationFormatter {
    fn round_to_seconds(&self) -> FormattedDuration;
}

impl DurationFormatter for Duration {
    fn round_to_seconds(&self) -> FormattedDuration {
        format_duration(Duration::from_secs(self.as_secs()))
    }
}

fn penalty_estimate_comparison() {
    let dict = Dictionary::load();
    let dict_small = Dictionary::load().filter_top_n_words(100_000);
    let overlaps = WordOverlap::load_from_csv(&dict_small, "./word_overlaps_200k.csv");
    let layout = Partitions {
        sum: 27,
        parts: 10,
        min: 2,
        max: 4,
    };
    let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 50);
    let single_keys = SingleKeyPenalties::load();
    for k in Keyboard::random(dict.alphabet(), layout, &prohibited).take(50) {
        let precise = k.penalty(&dict, Penalty::MAX);
        let small_dict = k.penalty(&dict_small, Penalty::MAX);
        let kludge = k.penalty_estimate(&single_keys);
        let two_pr = k.penalty_estimate2(&overlaps, true);
        let one_pr = k.penalty_estimate2(&overlaps, false);
        println!("");
        println!("precise:  {}", precise);
        println!("kludge:     {}", kludge);
        println!("new 2pr: {}", two_pr);
        println!("new 1pr: {}", one_pr);
        println!("small:   {}", small_dict);
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

fn calculate_overlaps_with_sql() {
    word_overlap_sqlite::run(None);
}

fn calculate_overlaps_with_memory() {
    let dictionary = Dictionary::load().filter_top_n_words(200_000);
    let file_name = "./word_overlaps_200k.csv";
    let overlap = WordOverlap::calculate(&dictionary, 2);
    overlap.save_to_csv(file_name).unwrap();
    let overlap_read = WordOverlap::load_from_csv(&dictionary, file_name);
    overlap_read.print();
}

fn custom() {
    println!("Undefined custom action");
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
        ("Word overlap with sql", calculate_overlaps_with_sql),
        ("Word overlap in mem", calculate_overlaps_with_memory),
        ("Show unique keyboard totals", show_unique_keyboard_totals),
        ("Penalty estimate comparisons", penalty_estimate_comparison),
        ("Custom", custom),
    ];
    let selection = choices
        .iter()
        .map(|(c, _)| c)
        .fold(
            Select::new().with_prompt("What do you want to do?"),
            |menu, item| menu.item(item),
        )
        .default(9)
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
