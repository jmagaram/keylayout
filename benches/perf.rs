#![allow(dead_code)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hashbrown::HashSet;
use keylayout::{
    dictionary::Dictionary,
    key::Key,
    keyboard::{Keyboard, Pruneable},
    partitions::Partitions,
    penalty::Penalty,
    prohibited::Prohibited,
    tally::Tally,
    util,
};

fn calculate_penalty(c: &mut Criterion) {
    let d = Dictionary::load();
    let layout = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    c.bench_function("CALCULATE PENALTY", |b| {
        b.iter(|| {
            let keys = d.alphabet().random_subsets(&layout).collect::<Vec<Key>>();
            let _keyboard = Keyboard::with_keys(keys).penalty(&d, black_box(Penalty::MAX));
            ()
        })
    });
}

fn load_dictionary(c: &mut Criterion) {
    c.bench_function("LOAD DICTIONARY", |b| {
        b.iter(|| {
            let _d = Dictionary::load().filter_top_n_words(black_box(1000));
        })
    });
}

fn generate_big_subsets(c: &mut Criterion) {
    c.bench_function("GENERATE BIG SUBSETS", |b| {
        b.iter(|| {
            let key = Key::with_every_letter();
            key.subsets_of_size(black_box(7)).count();
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

fn distribute_letters(c: &mut Criterion) {
    c.bench_function("DISTRIBUTE LETTERS", |b| {
        b.iter(|| {
            let key = Key::with_every_letter();
            let key_sizes = Tally::from([3, 3, 3, 3, 3, 3, 3, 2, 2, 2]);
            let keyboard_count = 100;
            let _results = key.distribute(key_sizes).take(black_box(keyboard_count));
        })
    });
}

fn random_subsets(c: &mut Criterion) {
    let key = Key::with_every_letter();
    let key_sizes = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    c.bench_function("RANDOM SUBSETS", |b| {
        b.iter(|| {
            let keys = key.random_subsets(black_box(&key_sizes));
            let _keys_materialized = keys.collect::<Vec<Key>>();
        })
    });
}

fn random_keyboards(c: &mut Criterion) {
    let dict = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 50);
    c.bench_function("RANDOM KEYBOARDS", |b| {
        b.iter(|| {
            let partition = Partitions {
                sum: 27,
                parts: 10,
                min: 2,
                max: 5,
            };
            Keyboard::random(dict.alphabet(), partition, black_box(&prohibited))
                .take(2000)
                .count();
        })
    });
}

fn random_keyboards_limited_alphabet(c: &mut Criterion) {
    let total_letters = 10;
    let alphabet = Key::with_first_n_letters(total_letters);
    let prohibited = Prohibited::new();
    c.bench_function("RANDOM KEYBOARDS LIMITED ALPHABET", |b| {
        b.iter(|| {
            let partition = Partitions {
                sum: total_letters,
                parts: total_letters / 2,
                min: 1,
                max: total_letters,
            };
            Keyboard::random(alphabet, partition, black_box(&prohibited))
                .take(20_000)
                .count();
        })
    });
}

fn set_bits(c: &mut Criterion) {
    c.bench_function("SET BITS ITERATOR", |b| {
        b.iter(|| {
            for i in [u32::MAX, 23, 76, 543_423, 0, 432, 12_345_731, 123_456_789] {
                util::set_bits(black_box(i)).count();
            }
        })
    });
}

fn count_letters_in_key(c: &mut Criterion) {
    c.bench_function("COUNT LETTERS ON KEY", |b| {
        b.iter(|| {
            let data = [
                "a",
                "ab",
                "abc",
                "abcd",
                "abcdefg",
                "abcdefghijklmnopqrstuvwxyz'",
                "abcdefghijklmno",
            ];
            for d in data {
                let key = Key::new(black_box(d));
                key.count_letters();
            }
        })
    });
}

fn iterate_letters_in_key(c: &mut Criterion) {
    c.bench_function("ITERATE LETTERS ON KEY", |b| {
        b.iter(|| {
            let data = [
                "a",
                "ab",
                "abc",
                "abcd",
                "abcdefg",
                "abcdefghijklmnopqrstuvwxyz'",
                "abcdefghijklmno",
            ];
            for d in data {
                let _count = Key::new(black_box(d)).letters().count();
            }
        })
    });
}

fn check_keyboard_for_invalid_pairs(c: &mut Criterion) {
    let dict = Dictionary::load();
    let prohibited = Prohibited::with_top_n_letter_pairs(&dict, 35);
    let p = Partitions {
        sum: 27,
        min: 2,
        max: 4,
        parts: 10,
    };
    let keyboards = Keyboard::random(dict.alphabet(), p, &prohibited)
        .take(100)
        .collect::<Vec<Keyboard>>();
    c.bench_function("CHECK KEYBOARD INVALID PAIRS", |b| {
        b.iter(|| {
            for k in &keyboards {
                k.has_prohibited_keys(black_box(&prohibited));
            }
        })
    });
}

#[derive(Clone)]
struct KeyboardStatus(Keyboard);

impl Pruneable for KeyboardStatus {
    fn should_prune(&self) -> bool {
        false
    }
}

fn generate_unique_keyboards_with_dfs(c: &mut Criterion) {
    let dict = Dictionary::load().filter_top_n_words(100_000);
    let p = Partitions {
        sum: 27,
        min: 1,
        max: 4,
        parts: 10,
    };
    let prune = |k: &Keyboard| {
        let k_filled = k.fill_missing(dict.alphabet());
        let _penalty = k_filled.penalty(&dict, Penalty::MAX);
        let result = KeyboardStatus(k.clone());
        result
    };
    c.bench_function("GENERATE UNIQUE KEYBOARDS", |b| {
        b.iter(|| {
            let mut unique = HashSet::new();
            // let keyboards =
            //     Keyboard::with_dfs_util_2(dict.alphabet(), &p, &prune).filter(|k| k.0.len() == 10);
            let keyboards = Keyboard::with_dfs(dict.alphabet(), &p, &prune)
                .filter(|k| k.0.len() == p.parts as usize);
            for k in keyboards {
                unique.insert(k.0.to_string());
                if unique.len() == 50 {
                    break;
                }
            }
        })
    });
}

criterion_group!(
    benches,
    generate_big_subsets,
    generate_small_subsets,
    check_keyboard_for_invalid_pairs,
    generate_unique_keyboards_with_dfs,
    load_dictionary,
    calculate_penalty,
    set_bits,
    count_letters_in_key,
    iterate_letters_in_key,
    random_keyboards,
    random_keyboards_limited_alphabet,
    distribute_keys,
    distribute_letters,
    random_subsets
);
criterion_main!(benches);
