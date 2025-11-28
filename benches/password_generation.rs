use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::{SeedableRng, rngs::StdRng};
use rpg::{PasswordArgs, build_char_set, generate_passwords};

fn bench_password_generation(c: &mut Criterion) {
    let args = PasswordArgs {
        capitals_off: false,
        numerals_off: false,
        symbols_off: false,
        exclude_chars: vec![],
        include_chars: None,
        min_capitals: None,
        min_numerals: None,
        min_symbols: None,
        pattern: None,
        length: 16,
        password_count: 1,
    };

    let char_set = build_char_set(&args).unwrap();
    let mut rng = StdRng::seed_from_u64(42);

    c.bench_function("generate_password_16", |b| {
        b.iter(|| {
            generate_passwords(
                black_box(&char_set),
                black_box(16),
                black_box(1),
                None,
                None,
                None,
                None,
                &mut rng,
            )
        })
    });

    c.bench_function("generate_password_64", |b| {
        b.iter(|| {
            generate_passwords(
                black_box(&char_set),
                black_box(64),
                black_box(1),
                None,
                None,
                None,
                None,
                &mut rng,
            )
        })
    });

    c.bench_function("generate_100_passwords", |b| {
        b.iter(|| {
            generate_passwords(
                black_box(&char_set),
                black_box(16),
                black_box(100),
                None,
                None,
                None,
                None,
                &mut rng,
            )
        })
    });
}

criterion_group!(benches, bench_password_generation);
criterion_main!(benches);
