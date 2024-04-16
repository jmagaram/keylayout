use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keylayout::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty};

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
criterion_group!(
    benches,
    generate_big_subsets,
    generate_small_subsets,
    load_dictionary,
    calculate_penalty_score,
    spell_every_word
);
criterion_main!(benches);
