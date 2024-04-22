use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keylayout::{
    dictionary::Dictionary, english, exhaustive, key::Key, keyboard::Keyboard,
    partitions::Partitions, penalty::Penalty, penalty_goal::PenaltyGoals, tally::Tally,
};

fn calculate_penalty_score(c: &mut Criterion) {
    let d = Dictionary::load();
    let layout = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    c.bench_function("CALCULATE PENALTY", |b| {
        b.iter(|| {
            let keys = d.alphabet().random_subsets(&layout).collect::<Vec<Key>>();
            let _keyboard = Keyboard::new_from_keys(keys).penalty(&d, black_box(Penalty::MAX));
            ()
        })
    });
}

fn spell_every_word(c: &mut Criterion) {
    let d = Dictionary::load();
    let layout = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    let keys = d.alphabet().random_subsets(&layout).collect::<Vec<Key>>();
    let keyboard = Keyboard::new_from_keys(keys);
    c.bench_function("SPELL EVERY WORD", |b| {
        b.iter(|| {
            d.words()
                .iter()
                .map(|w| {
                    keyboard.spell(black_box(w));
                })
                .count();
            ()
        })
    });
}

fn load_dictionary(c: &mut Criterion) {
    c.bench_function("LOAD DICTIONARY", |b| {
        b.iter(|| {
            let _d = Dictionary::load().with_top_n_words(black_box(1000));
        })
    });
}

fn generate_big_subsets(c: &mut Criterion) {
    c.bench_function("GENERATE BIG SUBSETS", |b| {
        b.iter(|| {
            let key = Key::with_every_letter();
            let sizes = [7];
            for s in sizes {
                key.subsets_of_size(black_box(s)).count();
            }
        })
    });
}

fn generate_small_subsets(c: &mut Criterion) {
    c.bench_function("GENERATE SMALL SUBSETS", |b| {
        b.iter(|| {
            let key = Key::with_every_letter();
            let sizes = [1, 2, 3, 4];
            for s in sizes {
                key.subsets_of_size(black_box(s)).count();
            }
        })
    });
}

fn distribute_keys(c: &mut Criterion) {
    fn make_key_sizes() -> Tally<u32> {
        Tally::from_iter([4, 4, 3, 3, 3, 2, 2, 2, 2, 1, 1])
    }
    c.bench_function("PERMUTE KEY SIZES", |b| {
        b.iter(|| {
            let key_sizes = black_box(make_key_sizes());
            key_sizes.combinations();
        })
    });
}

fn partition_sum(c: &mut Criterion) {
    let source = Partitions {
        sum: 27,
        parts: 10,
        min: 1,
        max: 27,
    };
    c.bench_function("PARTITION SUM", |b| {
        b.iter(|| {
            black_box(source.calculate());
        })
    });
}

fn distribute_letters(c: &mut Criterion) {
    c.bench_function("DISTRIBUTE LETTERS", |b| {
        b.iter(|| {
            let key = Key::with_every_letter();
            let key_sizes = Tally::from([3, 3, 3, 3, 3, 3, 3, 2, 2, 2]);
            let keyboard_count = 1000;
            let results = key.distribute(key_sizes).take(black_box(keyboard_count));
        })
    });
}

fn random_subsets(c: &mut Criterion) {
    let key = Key::with_every_letter();
    let key_sizes = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    c.bench_function("RANDOM SUBSETS", |b| {
        b.iter(|| {
            for _ in 1..1000 {
                let keys = key.random_subsets(black_box(&key_sizes));
                let _keys_materialized = keys.collect::<Vec<Key>>();
                // let kbd = Keyboard::new_from_keys(keys_materialized);
                // println!("{}", kbd);
            }
        })
    });
}

fn every_combine_two_keys(c: &mut Criterion) {
    let d = Dictionary::load();
    let prohibited = english::top_penalties(40, 0);
    c.bench_function("EVERY COMBINE TWO KEYS", |b| {
        b.iter(|| {
            let start = Keyboard::new_every_letter_on_own_key(d.alphabet());
            let _result = start
                .every_combine_two_keys(black_box(Some(&prohibited)))
                .all(|k| k.key_count() >= 1);
        })
    });
}

fn dfs_perf(c: &mut Criterion) {
    let d = Dictionary::load();
    c.bench_function("DFS", |b| {
        b.iter(|| {
            let start = Keyboard::new_every_letter_on_own_key(d.alphabet());
            let penalty_goals =
                PenaltyGoals::none(d.alphabet()).with_specific(10, Penalty::new(0.5));
            let max_letters_per_key = 5;
            let desired_keys = 10;
            let solution =
                exhaustive::dfs(&d, start, max_letters_per_key, desired_keys, &penalty_goals);
            match solution {
                None => {
                    println!("No solution found")
                }
                Some(solution) => {
                    println!("{}", solution);
                }
            }
        })
    });
}
criterion_group!(
    benches,
    // generate_big_subsets,
    // generate_small_subsets,
    // load_dictionary,
    // calculate_penalty_score,
    // spell_every_word,
    every_combine_two_keys,
    // dfs_perf,
    // distribute_keys,
    // partition_sum,
    // distribute_letters,
    // random_subsets
);
criterion_main!(benches);
