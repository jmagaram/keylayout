use dictionary::Dictionary;
use keyboard::Keyboard;
use overlap_penalties::OverlapPenalties;
use partitions::Partitions;
use penalty::Penalty;
use prohibited::Prohibited;
use single_key_penalties::SingleKeyPenalties;
use thousands::Separable;
use word_overlap::WordOverlap;

mod dfs;
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
mod overlap_penalties;
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

fn penalty_estimate_comparison() {
    let dict = Dictionary::load();
    let dict_small = Dictionary::load().filter_top_n_words(100_000);
    let overlaps = WordOverlap::load_from_csv(&dict, "./word_overlaps_200k.csv");
    let overlap_penalties = OverlapPenalties::load_from_csv("./overlap_penalties.csv");
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
        let two_pair_penalties = k.penalty_estimate3(&overlap_penalties);
        println!("");
        println!("precise: {}", precise);
        println!("kludge:  {}", kludge);
        println!("simp pr: {}", two_pair_penalties);
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

fn calculate_overlaps() {
    let d = Dictionary::load().filter_top_n_words(100_000);
    let overlap_penalties = OverlapPenalties::build(&d);
    let file_name = "./overlap_penalties.csv";
    overlap_penalties.save_to_csv(file_name).unwrap();
    let _ = OverlapPenalties::load_from_csv(file_name);
}

fn custom() {
    println!("No custom action defined yet.");
}

fn dfs_simpler() {
    let args = dfs::Args {
        max_key_size: 5,
        ten_key_prune_threshold: Penalty::new(0.0260),
        prohibited_pairs: 65,
        prune_factor: 0.90, // 0.87 is good
        prune_from_key_count: 12,
        prune_to_key_count: 20,
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

fn genetic() {
    let dict = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 50);
    let single_key_penalties = SingleKeyPenalties::load();
    let dict_small = Dictionary::load().filter_top_n_words(100_000);
    let word_overlap = WordOverlap::load_from_csv(&dict_small, "./word_overlaps_200k.csv");
    let args = genetic::FindBestArgs {
        dictionary: &dict,
        die_threshold: Penalty::new(0.000001),
        key_count: 10,
        prohibited,
        single_key_penalties: &single_key_penalties,
        word_overlap: &word_overlap,
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

fn pairings() {
    let args = pairing::Args {
        threads: 4,
        max_key_size: 5,
        pairings_to_ignore: 75,
        prune_threshold: Penalty::new(0.246),
    };
    args.solve();
}

fn main() {
    use dialoguer::Select;
    let choices: Vec<(&str, fn() -> ())> = vec![
        ("DFS search", dfs_pruning),
        ("DFS preconfigured", dfs_pruning_preconfigured),
        ("DFS using pairing technique", pairings),
        ("DFS optimized", dfs_simpler),
        ("Genetic algorithm", genetic),
        ("Find best N key", find_best_n_key),
        ("Print keyboard score", print_keyboard_score),
        ("Word overlaps with sql", calculate_overlaps_with_sql),
        ("Word overlaps in mem", calculate_overlaps),
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
        .default(3)
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
