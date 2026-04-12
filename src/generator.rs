use rand::Rng;

use crate::charset::CharacterSet;
use crate::config::{PasswordConfig, PatternChar};

/// Generates passwords using the provided character set and RNG.
pub fn generate_passwords<R: Rng>(
    char_set: &CharacterSet,
    config: &PasswordConfig,
    rng: &mut R,
) -> Vec<String> {
    let length = config.effective_length();
    let mut passwords = Vec::with_capacity(config.count as usize);

    for _ in 0..config.count {
        let pass = if let Some(ref pat) = config.pattern {
            generate_from_pattern(char_set, pat, rng)
        } else {
            generate_with_minimums(
                char_set,
                length,
                config.min_capitals,
                config.min_numerals,
                config.min_symbols,
                rng,
            )
        };
        passwords.push(pass);
    }

    passwords
}

/// Generates a password from a pattern specification.
fn generate_from_pattern<R: Rng>(
    char_set: &CharacterSet,
    pattern: &[PatternChar],
    rng: &mut R,
) -> String {
    let mut pass = String::with_capacity(pattern.len());

    for &pat_char in pattern {
        let subset = match pat_char {
            PatternChar::Lowercase => char_set.lowercase(),
            PatternChar::Uppercase => char_set.uppercase(),
            PatternChar::Numeric => char_set.numeric(),
            PatternChar::Symbol => char_set.symbols(),
        };
        let byte = if subset.is_empty() {
            pick_random(char_set.all(), rng)
        } else {
            pick_random(subset, rng)
        };
        pass.push(byte as char);
    }

    pass
}

/// Generates a password ensuring minimum character type requirements.
fn generate_with_minimums<R: Rng>(
    char_set: &CharacterSet,
    length: u32,
    min_capitals: Option<u32>,
    min_numerals: Option<u32>,
    min_symbols: Option<u32>,
    rng: &mut R,
) -> String {
    let mut pass_vec: Vec<char> = Vec::with_capacity(length as usize);

    // Fill required minimums first
    push_minimum_chars(&mut pass_vec, char_set.uppercase(), min_capitals, rng);
    push_minimum_chars(&mut pass_vec, char_set.numeric(), min_numerals, rng);
    push_minimum_chars(&mut pass_vec, char_set.symbols(), min_symbols, rng);

    // Fill remaining positions randomly from the full set
    while pass_vec.len() < length as usize {
        pass_vec.push(pick_random(char_set.all(), rng) as char);
    }

    // Shuffle to randomize positions of the minimum-required characters
    use rand::seq::SliceRandom;
    pass_vec.shuffle(rng);

    pass_vec.into_iter().collect()
}

/// Pushes `count` random characters from `subset` into `dest`.
/// If `subset` is empty, skips silently.
fn push_minimum_chars<R: Rng>(
    dest: &mut Vec<char>,
    subset: &[u8],
    count: Option<u32>,
    rng: &mut R,
) {
    if let Some(n) = count
        && !subset.is_empty()
    {
        for _ in 0..n {
            dest.push(pick_random(subset, rng) as char);
        }
    }
}

/// Picks a random element from a non-empty slice.
fn pick_random<R: Rng>(slice: &[u8], rng: &mut R) -> u8 {
    slice[rng.random_range(0..slice.len())]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charset::CharacterSet;
    use crate::config::{PasswordConfig, PatternChar};
    use rand::{SeedableRng, rngs::StdRng};

    fn test_char_set() -> CharacterSet {
        CharacterSet::from_raw(vec![
            b'a', b'b', b'c', b'A', b'B', b'C', b'0', b'1', b'2', b'!', b'@', b'#',
        ])
    }

    fn default_config() -> PasswordConfig {
        PasswordConfig {
            capitals_off: false,
            numerals_off: false,
            symbols_off: false,
            exclude_chars: vec![],
            include_chars: None,
            min_capitals: None,
            min_numerals: None,
            min_symbols: None,
            pattern: None,
            length: 8,
            count: 1,
        }
    }

    #[test]
    fn test_generate_passwords_basic() {
        let cs = CharacterSet::from_raw(vec![b'a', b'b', b'c', b'1', b'2', b'3']);
        let mut cfg = default_config();
        cfg.length = 5;
        cfg.count = 3;
        let mut rng = StdRng::seed_from_u64(202);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords.len(), 3);
        for p in &passwords {
            assert_eq!(p.len(), 5);
        }
    }

    #[test]
    fn test_generate_passwords_with_pattern() {
        let cs = test_char_set();
        let mut cfg = default_config();
        cfg.pattern = Some(vec![
            PatternChar::Lowercase,
            PatternChar::Uppercase,
            PatternChar::Numeric,
            PatternChar::Symbol,
        ]);
        cfg.count = 2;
        let mut rng = StdRng::seed_from_u64(303);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords.len(), 2);
        for p in &passwords {
            assert_eq!(p.len(), 4);
        }
    }

    #[test]
    fn test_generate_passwords_with_minimums() {
        let cs = test_char_set();
        let mut cfg = default_config();
        cfg.length = 8;
        cfg.count = 2;
        cfg.min_capitals = Some(1);
        cfg.min_numerals = Some(1);
        cfg.min_symbols = Some(1);
        let mut rng = StdRng::seed_from_u64(404);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords.len(), 2);
        for pass in &passwords {
            assert_eq!(pass.len(), 8);
            assert!(pass.chars().any(|c| c.is_ascii_uppercase()));
            assert!(pass.chars().any(|c| c.is_ascii_digit()));
            assert!(pass.chars().any(|c| !c.is_alphanumeric()));
        }
    }

    #[test]
    fn test_pattern_with_empty_subsets() {
        let cs = CharacterSet::from_raw(vec![b'a', b'b', b'c']);
        let mut cfg = default_config();
        cfg.pattern = Some(vec![
            PatternChar::Lowercase,
            PatternChar::Uppercase, // fallback
            PatternChar::Numeric,   // fallback
            PatternChar::Symbol,    // fallback
        ]);
        let mut rng = StdRng::seed_from_u64(123);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords[0].len(), 4);
        for c in passwords[0].chars() {
            assert!(c.is_ascii_lowercase());
        }
    }

    #[test]
    fn test_pattern_empty_lowercase() {
        let cs = CharacterSet::from_raw(vec![b'A', b'B', b'0', b'1', b'!', b'@']);
        let mut cfg = default_config();
        cfg.pattern = Some(vec![PatternChar::Lowercase]);
        let mut rng = StdRng::seed_from_u64(456);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords[0].len(), 1);
        assert!(
            cs.all()
                .contains(&(passwords[0].chars().next().unwrap() as u8))
        );
    }

    #[test]
    fn test_minimums_with_empty_subsets() {
        let cs = CharacterSet::from_raw(vec![b'a', b'b', b'c']);
        let mut cfg = default_config();
        cfg.length = 5;
        cfg.min_capitals = Some(2);
        cfg.min_numerals = Some(2);
        cfg.min_symbols = Some(2);
        let mut rng = StdRng::seed_from_u64(789);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords[0].len(), 5);
        for c in passwords[0].chars() {
            assert!(c.is_ascii_lowercase());
        }
    }

    #[test]
    fn test_minimums_no_minimums() {
        let cs = test_char_set();
        let mut cfg = default_config();
        cfg.length = 8;
        let mut rng = StdRng::seed_from_u64(101);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords[0].len(), 8);
    }

    #[test]
    fn test_minimums_exceeding_length() {
        let cs = CharacterSet::from_raw(vec![b'a', b'b', b'A', b'B', b'0', b'1', b'!', b'@']);
        let mut cfg = default_config();
        cfg.length = 4;
        cfg.min_capitals = Some(5);
        let mut rng = StdRng::seed_from_u64(1001);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert!(passwords[0].len() >= 5);
        let capitals = passwords[0]
            .chars()
            .filter(|c| c.is_ascii_uppercase())
            .count();
        assert!(capitals >= 5);
    }

    #[test]
    fn test_minimums_sum_exceeds_length() {
        let cs = test_char_set();
        let mut cfg = default_config();
        cfg.length = 6;
        cfg.min_capitals = Some(3);
        cfg.min_numerals = Some(3);
        cfg.min_symbols = Some(3);
        let mut rng = StdRng::seed_from_u64(1002);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert!(passwords[0].len() >= 9);
    }

    #[test]
    fn test_minimums_exact_length() {
        let cs = CharacterSet::from_raw(vec![b'a', b'b', b'A', b'B', b'0', b'1', b'!', b'@']);
        let mut cfg = default_config();
        cfg.length = 4;
        cfg.min_capitals = Some(2);
        cfg.min_numerals = Some(2);
        let mut rng = StdRng::seed_from_u64(1003);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords[0].len(), 4);
    }

    #[test]
    fn test_pattern_all_same_type() {
        let cs = CharacterSet::from_raw(vec![b'a', b'b', b'c', b'A', b'B', b'C', b'0', b'1', b'2']);
        let mut cfg = default_config();
        cfg.pattern = Some(vec![
            PatternChar::Lowercase,
            PatternChar::Lowercase,
            PatternChar::Lowercase,
        ]);
        let mut rng = StdRng::seed_from_u64(2001);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords[0].len(), 3);
        for c in passwords[0].chars() {
            assert!(c.is_ascii_lowercase());
        }
    }

    #[test]
    fn test_pattern_very_long() {
        let cs = CharacterSet::from_raw(vec![b'a', b'b', b'A', b'B', b'0', b'1', b'!', b'@']);
        let mut cfg = default_config();
        cfg.pattern = Some(
            (0..100)
                .map(|i| match i % 4 {
                    0 => PatternChar::Lowercase,
                    1 => PatternChar::Uppercase,
                    2 => PatternChar::Numeric,
                    _ => PatternChar::Symbol,
                })
                .collect(),
        );
        let mut rng = StdRng::seed_from_u64(2002);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        assert_eq!(passwords[0].len(), 100);
    }

    #[test]
    fn test_from_pattern_correct_types() {
        let cs = test_char_set();
        let mut cfg = default_config();
        cfg.pattern = Some(vec![
            PatternChar::Lowercase,
            PatternChar::Uppercase,
            PatternChar::Numeric,
            PatternChar::Symbol,
        ]);
        let mut rng = StdRng::seed_from_u64(42);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        let chars: Vec<char> = passwords[0].chars().collect();
        assert!(chars[0].is_ascii_lowercase());
        assert!(chars[1].is_ascii_uppercase());
        assert!(chars[2].is_ascii_digit());
        assert!(!chars[3].is_alphanumeric());
    }

    #[test]
    fn test_minimums_counted() {
        let cs = test_char_set();
        let mut cfg = default_config();
        cfg.length = 10;
        cfg.min_capitals = Some(2);
        cfg.min_numerals = Some(2);
        cfg.min_symbols = Some(2);
        let mut rng = StdRng::seed_from_u64(456);
        let passwords = generate_passwords(&cs, &cfg, &mut rng);
        let pass = &passwords[0];
        assert_eq!(pass.len(), 10);
        assert!(pass.chars().filter(|c| c.is_ascii_uppercase()).count() >= 2);
        assert!(pass.chars().filter(|c| c.is_ascii_digit()).count() >= 2);
        assert!(pass.chars().filter(|c| !c.is_alphanumeric()).count() >= 2);
    }
}
