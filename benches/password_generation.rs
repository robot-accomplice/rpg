use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::{SeedableRng, rngs::StdRng};
use rpg_util::{CharacterSet, PasswordConfig, generate_passwords};

fn bench_password_generation(c: &mut Criterion) {
    let config = PasswordConfig {
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
        count: 1,
    };

    let char_set = CharacterSet::from_config(&config).unwrap();
    let mut rng = StdRng::seed_from_u64(42);

    c.bench_function("generate_password_16", |b| {
        let cfg = PasswordConfig {
            length: 16,
            count: 1,
            ..config.clone()
        };
        b.iter(|| generate_passwords(black_box(&char_set), black_box(&cfg), &mut rng))
    });

    c.bench_function("generate_password_64", |b| {
        let cfg = PasswordConfig {
            length: 64,
            count: 1,
            ..config.clone()
        };
        b.iter(|| generate_passwords(black_box(&char_set), black_box(&cfg), &mut rng))
    });

    c.bench_function("generate_100_passwords", |b| {
        let cfg = PasswordConfig {
            length: 16,
            count: 100,
            ..config.clone()
        };
        b.iter(|| generate_passwords(black_box(&char_set), black_box(&cfg), &mut rng))
    });
}

criterion_group!(benches, bench_password_generation);
criterion_main!(benches);
