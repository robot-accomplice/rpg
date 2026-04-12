//! # RPG - Rust Password Generator
//!
//! A fast, secure, and customizable password generator library.
//!
//! ## Features
//!
//! - Customizable character sets with pre-computed type subsets
//! - Character exclusion and inclusion with range support
//! - Minimum character type requirements
//! - Pattern-based generation
//! - Uniform character distribution
//!
//! ## Example
//!
//! ```rust
//! use rpg_util::{PasswordConfig, CharacterSet, generate_passwords};
//!
//! let config = PasswordConfig {
//!     capitals_off: false,
//!     numerals_off: false,
//!     symbols_off: false,
//!     exclude_chars: vec![],
//!     include_chars: None,
//!     min_capitals: None,
//!     min_numerals: None,
//!     min_symbols: None,
//!     pattern: None,
//!     length: 16,
//!     count: 1,
//! };
//!
//! let char_set = CharacterSet::from_config(&config).unwrap();
//! let mut rng = rand::rng();
//! let passwords = generate_passwords(&char_set, &config, &mut rng);
//! ```

pub mod charset;
pub mod config;
pub mod error;
pub mod generator;

pub use charset::{CharacterSet, calculate_entropy, parse_char_specs};
pub use config::{MAX_PASSWORD_LENGTH, PasswordConfig, PatternChar, parse_pattern};
pub use error::PasswordError;
pub use generator::generate_passwords;

// --- Backward compatibility aliases ---
// These allow existing consumers to migrate gradually.
// TODO: Remove in next major version.

/// Alias for [`PasswordConfig`]. Provided for backward compatibility.
#[deprecated(since = "2.0.0", note = "Use PasswordConfig instead")]
pub type PasswordArgs = PasswordConfig;

/// Alias for [`PasswordConfig`]. Provided for backward compatibility.
#[deprecated(since = "2.0.0", note = "Use PasswordConfig instead")]
pub type GenerationParams = PasswordConfig;

/// Alias for [`parse_char_specs`]. Provided for backward compatibility.
#[deprecated(since = "2.0.0", note = "Use parse_char_specs instead")]
pub fn parse_exclude_chars(specs: Vec<String>) -> Result<Vec<char>, String> {
    parse_char_specs(specs).map_err(|e| e.to_string())
}

/// Alias for [`PasswordConfig::validate`]. Provided for backward compatibility.
#[deprecated(since = "2.0.0", note = "Use PasswordConfig::validate() instead")]
pub fn validate_args(config: &PasswordConfig) -> Result<(), PasswordError> {
    config.validate()
}

/// Alias for [`CharacterSet::from_config`]. Provided for backward compatibility.
///
/// Returns the raw `Vec<u8>` for callers that expect the old API.
#[deprecated(since = "2.0.0", note = "Use CharacterSet::from_config() instead")]
pub fn build_char_set(config: &PasswordConfig) -> Result<Vec<u8>, PasswordError> {
    CharacterSet::from_config(config).map(|cs| cs.all().to_vec())
}

/// Calculates the number of columns for table output.
///
/// This is a presentation helper for CLI use. Library consumers
/// typically won't need this.
pub fn column_count(password_count: u32) -> usize {
    match password_count {
        1..=3 => 1,
        4..=8 => 2,
        9..=15 => 3,
        16..=24 => 4,
        _ => {
            if password_count.is_multiple_of(5) {
                5
            } else if password_count.is_multiple_of(4) {
                4
            } else if password_count.is_multiple_of(3) {
                3
            } else if password_count.is_multiple_of(2) {
                2
            } else {
                3
            }
        }
    }
}

/// Prints passwords in column format to stdout.
///
/// This is a presentation helper for CLI use. Library consumers
/// should format output themselves.
pub fn print_columns(passwords: Vec<String>, column_count: usize, show_header: bool) {
    if show_header {
        println!(
            "Printing {} passwords in {} columns",
            passwords.len(),
            column_count
        );
    }

    if column_count == 1 {
        for pass in passwords {
            println!("{}", pass);
        }
        return;
    }

    let max_width = passwords.iter().map(|p| p.len()).max().unwrap_or(0).max(1);

    let mut col = 0;
    for pass in passwords {
        print!("{:<width$}", pass, width = max_width);
        col += 1;
        if col == column_count {
            col = 0;
            println!();
        } else {
            print!(" ");
        }
    }
    if col != 0 {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_count() {
        assert_eq!(column_count(1), 1);
        assert_eq!(column_count(2), 1);
        assert_eq!(column_count(3), 1);
        assert_eq!(column_count(4), 2);
        assert_eq!(column_count(5), 2);
        assert_eq!(column_count(6), 2);
        assert_eq!(column_count(9), 3);
        assert_eq!(column_count(10), 3);
        assert_eq!(column_count(16), 4);
        assert_eq!(column_count(20), 4);
        assert_eq!(column_count(25), 5);
    }

    #[test]
    fn test_column_count_large() {
        let cols = column_count(100);
        assert!(cols >= 2 && cols <= 5);
    }

    #[test]
    fn test_column_count_multiples() {
        assert_eq!(column_count(25), 5);
        assert_eq!(column_count(30), 5);
        assert_eq!(column_count(35), 5);
        assert_eq!(column_count(28), 4);
        assert_eq!(column_count(32), 4);
        assert_eq!(column_count(27), 3);
        assert_eq!(column_count(33), 3);
        assert_eq!(column_count(26), 2);
        assert_eq!(column_count(34), 2);
        assert_eq!(column_count(29), 3);
        assert_eq!(column_count(31), 3);
    }

    #[test]
    fn test_print_columns_single_column() {
        let passwords = vec!["p1".into(), "p2".into(), "p3".into()];
        print_columns(passwords.clone(), 1, false);
        print_columns(passwords, 1, true);
    }

    #[test]
    fn test_print_columns_multiple_columns() {
        let passwords: Vec<String> = vec![
            "short".into(),
            "verylongpassword".into(),
            "medium".into(),
            "x".into(),
        ];
        print_columns(passwords.clone(), 2, false);
        print_columns(passwords.clone(), 2, true);
        print_columns(passwords.clone(), 3, false);
        print_columns(passwords, 4, false);
    }

    #[test]
    fn test_print_columns_incomplete_row() {
        let passwords: Vec<String> =
            vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
        print_columns(passwords, 3, false);
    }

    #[test]
    fn test_print_columns_empty() {
        let passwords: Vec<String> = vec![];
        print_columns(passwords.clone(), 1, false);
        print_columns(passwords, 3, true);
    }

    #[test]
    fn test_print_columns_single_password() {
        let passwords = vec!["password123".to_string()];
        print_columns(passwords.clone(), 1, false);
        print_columns(passwords, 3, false);
    }

    #[test]
    fn test_print_columns_very_long_passwords() {
        let passwords = vec!["a".repeat(100), "b".repeat(50), "c".repeat(150)];
        print_columns(passwords.clone(), 1, false);
        print_columns(passwords.clone(), 2, false);
        print_columns(passwords, 3, true);
    }

    #[test]
    fn test_print_columns_single_password_multi_column() {
        let passwords = vec!["single".to_string()];
        print_columns(passwords, 5, false);
    }

    // Backward compatibility tests
    #[allow(deprecated)]
    #[test]
    fn test_deprecated_parse_exclude_chars() {
        let result = parse_exclude_chars(vec!["a-z".to_string()]).unwrap();
        assert_eq!(result.len(), 26);
    }

    #[allow(deprecated)]
    #[test]
    fn test_deprecated_validate_args() {
        let cfg = PasswordConfig {
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
        assert!(validate_args(&cfg).is_ok());
    }

    #[allow(deprecated)]
    #[test]
    fn test_deprecated_build_char_set() {
        let cfg = PasswordConfig {
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
        let cs = build_char_set(&cfg).unwrap();
        assert!(cs.len() > 60);
    }
}
