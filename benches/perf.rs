use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keylayout::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

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

criterion_group!(benches, criterion_benchmark, calculate_penalty_score);
criterion_main!(benches);
