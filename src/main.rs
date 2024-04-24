use dictionary::Dictionary;
use penalty::Penalty;
use prohibited::Prohibited;

mod dictionary;
mod exhaustive;
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
    let dict = Dictionary::load();

    fn best_n_key(key_count: u32) {
        match exhaustive::best_n_key(key_count) {
            None => {
                println!("No solution found")
            }
            Some(best) => {
                println!("Done searching for best {} key keyboard.", key_count);
                println!("{}", best);
            }
        }
    }

    fn find_best_dfs() {
        exhaustive::run_dfs();
    }

    let genetic_solver = move || {
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
    };

    genetic_solver();
}
