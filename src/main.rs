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

enum Run {
    BestNKey(u32),
    Other,
}

fn main() {
    // For bad pairs, they show up around 90
    // For bad triples, they show up around 1880

    let dict = Dictionary::load();

    let other = Run::Other;

    let best_n_key = Run::BestNKey(18);

    let run = other;

    match run {
        Run::BestNKey(count) => {
            exhaustive::best_n_key(count);
        }
        Run::Other => {
            exhaustive::run_dfs();
            // for r in evolution::find_best(&dict, 23, Penalty::new(0.001)) {
            //     match r {
            //         None => println!("Nothing found"),
            //         Some(r) => {
            //             println!("{}", r);
            //         }
            //     }
            // }
            // exhaustive::best_n_key(23);
            // exhaustive::dumb_run_dfs();
        }
    }
}
