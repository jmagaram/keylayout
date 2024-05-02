use dictionary::Dictionary;
use penalty::Penalty;
use prohibited::Prohibited;

mod dfs_pruning;
mod dictionary;
mod exhaustive_n_key;
mod frequency;
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
mod vec_threads;
mod word;

fn save_random_keyboard_penalties() {
    let dictionary = Dictionary::load();
    for prohibited_pairs in [0, 20, 40, 60, 80] {
        println!("Working on prohibited pairs: {}", prohibited_pairs);
        let prohibited = Prohibited::with_top_n_letter_pairs(&dictionary, prohibited_pairs);
        let args = random_solutions::Args {
            dictionary: &dictionary,
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

fn main() {
    use dialoguer::Select;
    let selection = Select::new()
        .with_prompt("What do you want to do?")
        .item("DFS search")
        .item("Genetic algorithm")
        .item("Save random keyboard penalties to CSV")
        .default(0)
        .interact()
        .unwrap();
    match selection {
        0 => dfs_pruning(),
        1 => genetic_solver(),
        2 => save_random_keyboard_penalties(),
        _ => panic!("Did not know how to handle that selection."),
    }
}
