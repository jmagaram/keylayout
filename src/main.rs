use dictionary::Dictionary;

use penalty::Penalty;

mod dictionary;
mod english;
mod evolution;
mod exhaustive;
mod frequency;
mod key;
mod keyboard;
mod lazy_tree;
mod letter;
mod partitions;
mod penalty;
mod penalty_goal;
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

    let run_genetic_solver = move || {
        for result in evolution::find_best(&dict, 10, Penalty::new(0.0001)) {
            if let Some(solution) = result {
                println!("{}", solution);
            }
        }
    };

    find_best_dfs();
}
