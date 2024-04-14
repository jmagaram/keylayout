use criterion::{criterion_group, criterion_main, Criterion};
use keylayout::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty};

fn calculate_penalty_score(c: &mut Criterion) {
    let d = Dictionary::load();
    let layout = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    c.bench_function("CALCULATE PENALTY", |b| {
        b.iter(|| {
            let keys = d.alphabet().random_subsets(&layout).collect::<Vec<Key>>();
            let _keyboard = Keyboard::new(keys).penalty(&d, Penalty::MAX);
            ()
        })
    });
}

fn spell_every_word(c: &mut Criterion) {
    let d = Dictionary::load();
    let layout = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    let keys = d.alphabet().random_subsets(&layout).collect::<Vec<Key>>();
    let keyboard = Keyboard::new(keys);
    c.bench_function("SPELL EVERY WORD", |b| {
        b.iter(|| {
            d.words()
                .iter()
                .map(|w| {
                    keyboard.spell(w);
                })
                .count();
            ()
        })
    });
}

fn load_dictionary(c: &mut Criterion) {
    c.bench_function("LOAD DICTIONARY", |b| {
        b.iter(|| {
            let _d = Dictionary::load().with_top_n_words(1000);
        })
    });
}

criterion_group!(
    benches,
    load_dictionary,
    calculate_penalty_score,
    spell_every_word
);
criterion_main!(benches);
