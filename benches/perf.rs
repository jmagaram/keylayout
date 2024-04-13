use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keylayout::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty};

fn calculate_penalty_score(c: &mut Criterion) {
    let d = Dictionary::load_large_dictionary();
    let layout = vec![3, 3, 3, 3, 3, 3, 3, 2, 2, 2];
    c.bench_function("calculate penalty", |b| {
        b.iter(|| {
            let keys = d.alphabet().random_subsets(&layout).collect::<Vec<Key>>();
            let keyboard = Keyboard::new(keys);
            let _p = keyboard.penalty(&d, Penalty::MAX);
            ()
        })
    });
}

criterion_group!(benches, calculate_penalty_score);
criterion_main!(benches);
