//! # RPG - Rust Password Generator
//!
//! A fast, secure, and customizable password generator library.
//!
//! ## Features
//!
//! - Customizable character sets
//! - Character exclusion with range support
//! - Minimum character type requirements
//! - Pattern-based generation
//! - Uniform character distribution
//!
//! ## Example
//!
//! ```rust
//! use rpg_util::{GenerationParams, PasswordArgs, build_char_set, generate_passwords};
//! use rand::Rng;
//!
//! let args = PasswordArgs {
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
//!     password_count: 1,
//! };
//!
//! let char_set = build_char_set(&args).unwrap();
//! let mut rng = rand::rng();
//! let gen_params = rpg_util::GenerationParams {
//!     length: 16,
//!     count: 1,
//!     min_capitals: None,
//!     min_numerals: None,
//!     min_symbols: None,
//!     pattern: None,
//! };
//! let passwords = rpg_util::generate_passwords(&char_set, &gen_params, &mut rng);
//! ```

use rand::Rng;
use std::collections::HashSet;
use std::fmt;

/// Calculates password entropy in bits
pub fn calculate_entropy(char_set_size: usize, length: u32) -> f64 {
    (char_set_size as f64).log2() * length as f64
}

/// Custom error type for password generation
#[derive(Debug, Clone)]
pub enum PasswordError {
    InvalidLength,
    InvalidLengthTooLong,
    InvalidCount,
    EmptyCharacterSet,
    AllTypesDisabled,
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordError::InvalidLength => {
                write!(f, "Error: Password length must be greater than 0.")
            }
            PasswordError::InvalidLengthTooLong => {
                write!(
                    f,
                    "Error: Password length exceeds maximum of 10,000 characters."
                )
            }
            PasswordError::InvalidCount => {
                write!(f, "Error: Password count must be greater than 0.")
            }
            PasswordError::EmptyCharacterSet => {
                write!(
                    f,
                    "Error: All characters have been excluded or disabled. Cannot generate passwords.\n\
                    Hint: Try removing some character exclusions or enabling character types."
                )
            }
            PasswordError::AllTypesDisabled => {
                write!(
                    f,
                    "Error: All character types are disabled and/or all remaining characters are excluded.\n\
                    Hint: At least one character type must be enabled. Try removing --capitals-off, --numerals-off, or --symbols-off."
                )
            }
        }
    }
}

impl std::error::Error for PasswordError {}

// ASCII character range constants
const ASCII_LOWERCASE_START: u8 = b'a';
const ASCII_LOWERCASE_END: u8 = b'z';
const ASCII_UPPERCASE_START: u8 = b'A';
const ASCII_UPPERCASE_END: u8 = b'Z';
const ASCII_NUMERAL_START: u8 = b'0';
const ASCII_NUMERAL_END: u8 = b'9';
const ASCII_SYMBOL_RANGE_1_START: u8 = 33; // !
const ASCII_SYMBOL_RANGE_1_END: u8 = 47; // /
const ASCII_SYMBOL_RANGE_2_START: u8 = 58; // :
const ASCII_SYMBOL_RANGE_2_END: u8 = 64; // @
const ASCII_SYMBOL_RANGE_3_START: u8 = 91; // [
const ASCII_SYMBOL_RANGE_3_END: u8 = 96; // `
const ASCII_SYMBOL_RANGE_4_START: u8 = 123; // {
const ASCII_SYMBOL_RANGE_4_END: u8 = 126; // ~

/// Pattern character types
#[derive(Debug, Clone, Copy)]
pub enum PatternChar {
    Lowercase,
    Uppercase,
    Numeric,
    Symbol,
}

/// Parameters for password generation
#[derive(Debug, Clone)]
pub struct GenerationParams {
    pub length: u32,
    pub count: u32,
    pub min_capitals: Option<u32>,
    pub min_numerals: Option<u32>,
    pub min_symbols: Option<u32>,
    pub pattern: Option<Vec<PatternChar>>,
}

/// Arguments structure for password generation
pub struct PasswordArgs {
    pub capitals_off: bool,
    pub numerals_off: bool,
    pub symbols_off: bool,
    pub exclude_chars: Vec<char>,
    pub include_chars: Option<Vec<char>>,
    pub min_capitals: Option<u32>,
    pub min_numerals: Option<u32>,
    pub min_symbols: Option<u32>,
    pub pattern: Option<Vec<PatternChar>>,
    pub length: u32,
    pub password_count: u32,
}

/// Parses character exclusion strings, expanding ranges like "a-z" or "0-9"
/// Returns a vector of individual characters to exclude
///
/// # Examples
/// - "a-z" expands to all lowercase letters
/// - "0-9" expands to all digits
/// - "a-c" expands to 'a', 'b', 'c'
/// - "abc" is treated as individual characters 'a', 'b', 'c'
/// - "a-z,0-9,b" combines ranges and individual characters
pub fn parse_exclude_chars(exclude_strings: Vec<String>) -> Result<Vec<char>, String> {
    let mut exclude_chars = Vec::new();

    for s in exclude_strings {
        // Check if it's a range (contains a dash with characters on both sides)
        // Range format: "X-Y" where X and Y are single characters
        if s.len() == 3 {
            let chars: Vec<char> = s.chars().collect();
            if chars[1] == '-' {
                let start = chars[0] as u8;
                let end = chars[2] as u8;

                // Validate range (start must be <= end, and both must be ASCII printable)
                if start <= end && start >= 32 && end < 127 {
                    for byte in start..=end {
                        exclude_chars.push(byte as char);
                    }
                    continue;
                } else if start > end {
                    return Err(format!(
                        "Invalid range '{}': start character '{}' is greater than end character '{}'",
                        s, chars[0], chars[2]
                    ));
                }
            }
        }

        // If not a range, treat as individual character(s)
        for c in s.chars() {
            if !exclude_chars.contains(&c) {
                exclude_chars.push(c);
            }
        }
    }

    Ok(exclude_chars)
}

/// Builds the character set based on command-line arguments
/// Returns a vector of valid characters that can be used for password generation
pub fn build_char_set(args: &PasswordArgs) -> Result<Vec<u8>, PasswordError> {
    let mut chars = Vec::new();

    // If include_chars is specified, use only those characters
    if let Some(ref include_chars) = args.include_chars {
        for &c in include_chars {
            chars.push(c as u8);
        }
    } else {
        // Pre-allocate with estimated capacity (max ~94 printable ASCII chars)
        let estimated_capacity = if args.symbols_off {
            62 // 26 lowercase + 26 uppercase + 10 numerals
        } else {
            94 // All printable ASCII
        };
        chars.reserve(estimated_capacity);

        // Add lowercase letters (always included)
        chars.extend(ASCII_LOWERCASE_START..=ASCII_LOWERCASE_END);

        // Add uppercase letters if not disabled
        if !args.capitals_off {
            chars.extend(ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END);
        }

        // Add numerals if not disabled
        if !args.numerals_off {
            chars.extend(ASCII_NUMERAL_START..=ASCII_NUMERAL_END);
        }

        // Add symbols if not disabled (complete ASCII printable symbol ranges)
        if !args.symbols_off {
            chars.extend(ASCII_SYMBOL_RANGE_1_START..=ASCII_SYMBOL_RANGE_1_END);
            chars.extend(ASCII_SYMBOL_RANGE_2_START..=ASCII_SYMBOL_RANGE_2_END);
            chars.extend(ASCII_SYMBOL_RANGE_3_START..=ASCII_SYMBOL_RANGE_3_END);
            chars.extend(ASCII_SYMBOL_RANGE_4_START..=ASCII_SYMBOL_RANGE_4_END);
        }
    }

    // Convert exclude_chars Vec to HashSet for O(1) lookup
    let exclude_set: HashSet<char> = args.exclude_chars.iter().cloned().collect();

    // Filter out excluded characters
    chars.retain(|&b| !exclude_set.contains(&(b as char)));

    // Validate that we have at least one character available
    if chars.is_empty() {
        return Err(PasswordError::EmptyCharacterSet);
    }

    Ok(chars)
}

/// Maximum allowed password length to prevent memory issues
const MAX_PASSWORD_LENGTH: u32 = 10_000;

/// Validates command-line arguments
pub fn validate_args(args: &PasswordArgs) -> Result<(), PasswordError> {
    if args.length == 0 {
        return Err(PasswordError::InvalidLength);
    }

    if args.length > MAX_PASSWORD_LENGTH {
        return Err(PasswordError::InvalidLengthTooLong);
    }

    if args.password_count == 0 {
        return Err(PasswordError::InvalidCount);
    }

    // Check if all character types are disabled
    if args.capitals_off && args.numerals_off && args.symbols_off {
        // Only lowercase letters remain, which is valid
        // But we should check if they're all excluded
        let test_set = build_char_set(args)?;
        if test_set.is_empty() {
            return Err(PasswordError::AllTypesDisabled);
        }
    }

    Ok(())
}

/// Calculates the number of columns for table output
pub fn column_count(password_count: u32) -> usize {
    // Use a more reasonable default: prefer 3-4 columns for readability
    // but adapt based on count
    match password_count {
        1..=3 => 1,
        4..=8 => 2,
        9..=15 => 3,
        16..=24 => 4,
        _ => {
            // For larger counts, use a divisor that makes sense
            if password_count.is_multiple_of(5) {
                5
            } else if password_count.is_multiple_of(4) {
                4
            } else if password_count.is_multiple_of(3) {
                3
            } else if password_count.is_multiple_of(2) {
                2
            } else {
                3 // Default to 3 columns for readability
            }
        }
    }
}

/// Parses a pattern string like "LLLNNNSSS" into PatternChar vector
pub fn parse_pattern(pattern: &str) -> Result<Vec<PatternChar>, String> {
    let mut result = Vec::new();
    for c in pattern.chars() {
        match c {
            'L' | 'l' => result.push(PatternChar::Lowercase),
            'U' | 'u' => result.push(PatternChar::Uppercase),
            'N' | 'n' => result.push(PatternChar::Numeric),
            'S' | 's' => result.push(PatternChar::Symbol),
            _ => {
                return Err(format!(
                    "Invalid pattern character: '{}'. Use L (lowercase), U (uppercase), N (numeric), S (symbol)",
                    c
                ));
            }
        }
    }
    Ok(result)
}

/// Generates a password from a pattern
fn generate_password_from_pattern<R: Rng>(
    char_set: &[u8],
    pattern: &[PatternChar],
    rng: &mut R,
) -> String {
    let mut pass = String::with_capacity(pattern.len());

    let lowercase: Vec<u8> = (ASCII_LOWERCASE_START..=ASCII_LOWERCASE_END)
        .filter(|&b| char_set.contains(&b))
        .collect();
    let uppercase: Vec<u8> = (ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END)
        .filter(|&b| char_set.contains(&b))
        .collect();
    let numeric: Vec<u8> = (ASCII_NUMERAL_START..=ASCII_NUMERAL_END)
        .filter(|&b| char_set.contains(&b))
        .collect();
    let symbols: Vec<u8> = char_set
        .iter()
        .filter(|&&b| {
            !(ASCII_LOWERCASE_START..=ASCII_LOWERCASE_END).contains(&b)
                && !(ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END).contains(&b)
                && !(ASCII_NUMERAL_START..=ASCII_NUMERAL_END).contains(&b)
        })
        .copied()
        .collect();

    for &pat_char in pattern {
        let char_byte = match pat_char {
            PatternChar::Lowercase => {
                if lowercase.is_empty() {
                    char_set[rng.random_range(0..char_set.len())]
                } else {
                    lowercase[rng.random_range(0..lowercase.len())]
                }
            }
            PatternChar::Uppercase => {
                if uppercase.is_empty() {
                    char_set[rng.random_range(0..char_set.len())]
                } else {
                    uppercase[rng.random_range(0..uppercase.len())]
                }
            }
            PatternChar::Numeric => {
                if numeric.is_empty() {
                    char_set[rng.random_range(0..char_set.len())]
                } else {
                    numeric[rng.random_range(0..numeric.len())]
                }
            }
            PatternChar::Symbol => {
                if symbols.is_empty() {
                    char_set[rng.random_range(0..char_set.len())]
                } else {
                    symbols[rng.random_range(0..symbols.len())]
                }
            }
        };
        pass.push(char_byte as char);
    }

    pass
}

/// Generates a single password ensuring minimum character type requirements
fn generate_password_with_minimums<R: Rng>(
    char_set: &[u8],
    length: u32,
    min_capitals: Option<u32>,
    min_numerals: Option<u32>,
    min_symbols: Option<u32>,
    rng: &mut R,
) -> String {
    let mut pass_vec: Vec<char> = Vec::with_capacity(length as usize);

    // First, ensure minimum requirements are met

    // Collect character sets for each type
    let capitals: Vec<u8> = (ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END)
        .filter(|&b| char_set.contains(&b))
        .collect();
    let numerals: Vec<u8> = (ASCII_NUMERAL_START..=ASCII_NUMERAL_END)
        .filter(|&b| char_set.contains(&b))
        .collect();
    let symbols: Vec<u8> = char_set
        .iter()
        .filter(|&&b| {
            !(ASCII_LOWERCASE_START..=ASCII_LOWERCASE_END).contains(&b)
                && !(ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END).contains(&b)
                && !(ASCII_NUMERAL_START..=ASCII_NUMERAL_END).contains(&b)
        })
        .copied()
        .collect();

    // Add required capitals
    if let Some(min) = min_capitals {
        for _ in 0..min {
            if !capitals.is_empty() {
                let idx = rng.random_range(0..capitals.len());
                pass_vec.push(capitals[idx] as char);
            }
        }
    }

    // Add required numerals
    if let Some(min) = min_numerals {
        for _ in 0..min {
            if !numerals.is_empty() {
                let idx = rng.random_range(0..numerals.len());
                pass_vec.push(numerals[idx] as char);
            }
        }
    }

    // Add required symbols
    if let Some(min) = min_symbols {
        for _ in 0..min {
            if !symbols.is_empty() {
                let idx = rng.random_range(0..symbols.len());
                pass_vec.push(symbols[idx] as char);
            }
        }
    }

    // Fill the rest randomly
    while pass_vec.len() < length as usize {
        let c_byte = char_set[rng.random_range(0..char_set.len())];
        pass_vec.push(c_byte as char);
    }

    // Shuffle to randomize positions
    use rand::seq::SliceRandom;
    pass_vec.shuffle(rng);

    pass_vec.into_iter().collect()
}

/// Generates passwords using the provided character set and RNG
pub fn generate_passwords<R: Rng>(
    char_set: &[u8],
    params: &GenerationParams,
    rng: &mut R,
) -> Vec<String> {
    let mut passwords = Vec::with_capacity(params.count as usize);

    for _ in 0..params.count {
        let pass = if let Some(ref pat) = params.pattern {
            generate_password_from_pattern(char_set, pat, rng)
        } else {
            generate_password_with_minimums(
                char_set,
                params.length,
                params.min_capitals,
                params.min_numerals,
                params.min_symbols,
                rng,
            )
        };
        passwords.push(pass);
    }

    passwords
}

/// Prints passwords in column format
pub fn print_columns(passwords: Vec<String>, column_count: usize, show_header: bool) {
    if show_header {
        println!(
            "Printing {} passwords in {} columns",
            passwords.len(),
            column_count
        );
    }

    if column_count == 1 {
        // Simple one-per-line output
        for pass in passwords {
            println!("{}", pass);
        }
        return;
    }

    // Calculate column width for alignment
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
    // Add trailing newline if last row is incomplete
    if col != 0 {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args(
        capitals_off: bool,
        numerals_off: bool,
        symbols_off: bool,
        exclude_chars: Vec<char>,
    ) -> PasswordArgs {
        PasswordArgs {
            capitals_off,
            numerals_off,
            symbols_off,
            exclude_chars,
            include_chars: None,
            min_capitals: None,
            min_numerals: None,
            min_symbols: None,
            pattern: None,
            length: 16,
            password_count: 1,
        }
    }

    #[test]
    fn test_build_char_set_default() {
        let args = create_test_args(false, false, false, vec![]);
        let char_set = build_char_set(&args).unwrap();
        // Should include lowercase, uppercase, numerals, and symbols
        assert!(!char_set.is_empty());
        assert!(char_set.len() > 60); // At least 26 + 26 + 10 + some symbols
    }

    #[test]
    fn test_build_char_set_no_capitals() {
        let args = create_test_args(true, false, false, vec![]);
        let char_set = build_char_set(&args).unwrap();
        // Should not include uppercase letters
        assert!(!char_set.contains(&b'A'));
        assert!(!char_set.contains(&b'Z'));
        // Should still include lowercase
        assert!(char_set.contains(&b'a'));
    }

    #[test]
    fn test_build_char_set_no_numerals() {
        let args = create_test_args(false, true, false, vec![]);
        let char_set = build_char_set(&args).unwrap();
        // Should not include numerals
        assert!(!char_set.contains(&b'0'));
        assert!(!char_set.contains(&b'9'));
    }

    #[test]
    fn test_build_char_set_no_symbols() {
        let args = create_test_args(false, false, true, vec![]);
        let char_set = build_char_set(&args).unwrap();
        // Should not include symbols
        assert!(!char_set.contains(&b'!'));
        assert!(!char_set.contains(&b'@'));
    }

    #[test]
    fn test_build_char_set_with_exclusions() {
        let args = create_test_args(false, false, false, vec!['a', 'b', 'c']);
        let char_set = build_char_set(&args).unwrap();
        // Should not include excluded characters
        assert!(!char_set.contains(&b'a'));
        assert!(!char_set.contains(&b'b'));
        assert!(!char_set.contains(&b'c'));
        // Should still include other lowercase
        assert!(char_set.contains(&b'd'));
    }

    #[test]
    fn test_build_char_set_all_excluded() {
        // Exclude all lowercase letters when only lowercase is available
        let mut exclude_all = Vec::new();
        for c in b'a'..=b'z' {
            exclude_all.push(c as char);
        }
        let args = create_test_args(true, true, true, exclude_all);
        let result = build_char_set(&args);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PasswordError::EmptyCharacterSet
        ));
    }

    #[test]
    fn test_validate_args_valid() {
        let args = create_test_args(false, false, false, vec![]);
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_length() {
        let mut args = create_test_args(false, false, false, vec![]);
        args.length = 0;
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PasswordError::InvalidLength));
    }

    #[test]
    fn test_validate_args_invalid_count() {
        let mut args = create_test_args(false, false, false, vec![]);
        args.password_count = 0;
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PasswordError::InvalidCount));
    }

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
        // Test that large numbers default to reasonable values
        let cols = column_count(100);
        assert!(cols >= 2 && cols <= 5);
    }

    #[test]
    fn test_parse_exclude_chars_range() {
        let result = parse_exclude_chars(vec!["a-z".to_string()]).unwrap();
        assert_eq!(result.len(), 26);
        assert!(result.contains(&'a'));
        assert!(result.contains(&'z'));
        assert!(result.contains(&'m'));
    }

    #[test]
    fn test_parse_exclude_chars_numeric_range() {
        let result = parse_exclude_chars(vec!["0-9".to_string()]).unwrap();
        assert_eq!(result.len(), 10);
        assert!(result.contains(&'0'));
        assert!(result.contains(&'9'));
        assert!(result.contains(&'5'));
    }

    #[test]
    fn test_parse_exclude_chars_small_range() {
        let result = parse_exclude_chars(vec!["a-c".to_string()]).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&'a'));
        assert!(result.contains(&'b'));
        assert!(result.contains(&'c'));
    }

    #[test]
    fn test_parse_exclude_chars_individual() {
        let result = parse_exclude_chars(vec!["abc".to_string()]).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&'a'));
        assert!(result.contains(&'b'));
        assert!(result.contains(&'c'));
    }

    #[test]
    fn test_parse_exclude_chars_mixed() {
        let result =
            parse_exclude_chars(vec!["a-c".to_string(), "x".to_string(), "0-2".to_string()])
                .unwrap();
        assert!(result.contains(&'a'));
        assert!(result.contains(&'b'));
        assert!(result.contains(&'c'));
        assert!(result.contains(&'x'));
        assert!(result.contains(&'0'));
        assert!(result.contains(&'1'));
        assert!(result.contains(&'2'));
    }

    #[test]
    fn test_parse_exclude_chars_invalid_range() {
        let result = parse_exclude_chars(vec!["z-a".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid range"));
    }

    #[test]
    fn test_calculate_entropy() {
        // Test with different character set sizes and lengths
        let entropy1 = calculate_entropy(26, 8); // lowercase only, 8 chars
        assert!(entropy1 > 0.0);

        let entropy2 = calculate_entropy(62, 16); // alphanumeric, 16 chars
        assert!(entropy2 > entropy1);

        let entropy3 = calculate_entropy(94, 20); // all printable ASCII, 20 chars
        assert!(entropy3 > entropy2);

        // Verify entropy increases with length
        let entropy4 = calculate_entropy(62, 32);
        assert!(entropy4 > entropy2);
    }

    #[test]
    fn test_password_error_display() {
        let err1 = PasswordError::InvalidLength;
        assert!(
            err1.to_string()
                .contains("Password length must be greater than 0")
        );

        let err2 = PasswordError::InvalidLengthTooLong;
        assert!(err2.to_string().contains("exceeds maximum of 10,000"));

        let err3 = PasswordError::InvalidCount;
        assert!(
            err3.to_string()
                .contains("Password count must be greater than 0")
        );

        let err4 = PasswordError::EmptyCharacterSet;
        assert!(
            err4.to_string()
                .contains("All characters have been excluded")
        );
        assert!(err4.to_string().contains("Hint"));

        let err5 = PasswordError::AllTypesDisabled;
        assert!(
            err5.to_string()
                .contains("All character types are disabled")
        );
        assert!(err5.to_string().contains("Hint"));
    }

    #[test]
    fn test_build_char_set_with_include_chars() {
        let mut args = create_test_args(false, false, false, vec![]);
        args.include_chars = Some(vec!['a', 'b', 'c', '1', '2', '!']);
        let char_set = build_char_set(&args).unwrap();

        assert_eq!(char_set.len(), 6);
        assert!(char_set.contains(&b'a'));
        assert!(char_set.contains(&b'b'));
        assert!(char_set.contains(&b'c'));
        assert!(char_set.contains(&b'1'));
        assert!(char_set.contains(&b'2'));
        assert!(char_set.contains(&b'!'));
        // Should not include other characters
        assert!(!char_set.contains(&b'd'));
        assert!(!char_set.contains(&b'A'));
    }

    #[test]
    fn test_build_char_set_with_include_chars_and_exclusions() {
        let mut args = create_test_args(false, false, false, vec!['a']);
        args.include_chars = Some(vec!['a', 'b', 'c']);
        let char_set = build_char_set(&args).unwrap();

        // 'a' should be excluded even though it's in include_chars
        assert!(!char_set.contains(&b'a'));
        assert!(char_set.contains(&b'b'));
        assert!(char_set.contains(&b'c'));
    }

    #[test]
    fn test_validate_args_too_long() {
        let mut args = create_test_args(false, false, false, vec![]);
        args.length = 10_001;
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PasswordError::InvalidLengthTooLong
        ));
    }

    #[test]
    fn test_validate_args_all_types_disabled_with_exclusions() {
        // All types disabled and all lowercase excluded
        // This should result in EmptyCharacterSet from build_char_set, which gets propagated
        let mut exclude_all = Vec::new();
        for c in b'a'..=b'z' {
            exclude_all.push(c as char);
        }
        let args = create_test_args(true, true, true, exclude_all);
        let result = validate_args(&args);
        assert!(result.is_err());
        // When all types are disabled and all chars excluded, build_char_set returns EmptyCharacterSet
        // which gets propagated through the ? operator
        let err = result.unwrap_err();
        assert!(
            matches!(err, PasswordError::EmptyCharacterSet)
                || matches!(err, PasswordError::AllTypesDisabled)
        );
    }

    #[test]
    fn test_column_count_multiples() {
        // Test multiples of 5
        assert_eq!(column_count(25), 5);
        assert_eq!(column_count(30), 5);
        assert_eq!(column_count(35), 5);

        // Test multiples of 4 (but not 5)
        assert_eq!(column_count(28), 4);
        assert_eq!(column_count(32), 4);

        // Test multiples of 3 (but not 4 or 5)
        assert_eq!(column_count(27), 3);
        assert_eq!(column_count(33), 3);

        // Test multiples of 2 (but not 3, 4, or 5)
        assert_eq!(column_count(26), 2);
        assert_eq!(column_count(34), 2);

        // Test prime numbers (should default to 3)
        assert_eq!(column_count(29), 3);
        assert_eq!(column_count(31), 3);
    }

    #[test]
    fn test_parse_pattern() {
        // Test valid patterns
        let pattern1 = parse_pattern("LLL").unwrap();
        assert_eq!(pattern1.len(), 3);
        assert!(matches!(pattern1[0], PatternChar::Lowercase));
        assert!(matches!(pattern1[1], PatternChar::Lowercase));
        assert!(matches!(pattern1[2], PatternChar::Lowercase));

        let pattern2 = parse_pattern("UUNNSS").unwrap();
        assert_eq!(pattern2.len(), 6);
        assert!(matches!(pattern2[0], PatternChar::Uppercase));
        assert!(matches!(pattern2[1], PatternChar::Uppercase));
        assert!(matches!(pattern2[2], PatternChar::Numeric));
        assert!(matches!(pattern2[3], PatternChar::Numeric));
        assert!(matches!(pattern2[4], PatternChar::Symbol));
        assert!(matches!(pattern2[5], PatternChar::Symbol));

        // Test case insensitivity
        let pattern3 = parse_pattern("lluunnss").unwrap();
        assert_eq!(pattern3.len(), 8);
        assert!(matches!(pattern3[0], PatternChar::Lowercase));
        assert!(matches!(pattern3[2], PatternChar::Uppercase));
        assert!(matches!(pattern3[4], PatternChar::Numeric));
        assert!(matches!(pattern3[6], PatternChar::Symbol));

        // Test invalid pattern
        let result = parse_pattern("LLX");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid pattern character"));

        // Test empty pattern
        let pattern4 = parse_pattern("").unwrap();
        assert_eq!(pattern4.len(), 0);
    }

    #[test]
    fn test_generate_password_from_pattern() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![
            b'a', b'b', b'c', b'A', b'B', b'C', b'0', b'1', b'2', b'!', b'@', b'#',
        ];
        let pattern = vec![
            PatternChar::Lowercase,
            PatternChar::Uppercase,
            PatternChar::Numeric,
            PatternChar::Symbol,
        ];

        let mut rng = StdRng::seed_from_u64(42);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 4);
        // Verify each character type (we can't predict exact chars due to randomness,
        // but we can verify the pattern was followed by checking character classes)
        let chars: Vec<char> = password.chars().collect();
        assert!(chars[0].is_ascii_lowercase());
        assert!(chars[1].is_ascii_uppercase());
        assert!(chars[2].is_ascii_digit());
        assert!(!chars[3].is_alphanumeric());
    }

    #[test]
    fn test_generate_password_from_pattern_empty_sets() {
        use rand::{SeedableRng, rngs::StdRng};

        // Character set with no uppercase, no numeric, no symbols
        let char_set = vec![b'a', b'b', b'c'];
        let pattern = vec![
            PatternChar::Lowercase,
            PatternChar::Uppercase, // Will fallback to char_set
            PatternChar::Numeric,   // Will fallback to char_set
            PatternChar::Symbol,    // Will fallback to char_set
        ];

        let mut rng = StdRng::seed_from_u64(123);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 4);
        // All should be lowercase since that's all that's available
        for c in password.chars() {
            assert!(c.is_ascii_lowercase());
        }
    }

    #[test]
    fn test_generate_password_from_pattern_empty_lowercase() {
        use rand::{SeedableRng, rngs::StdRng};

        // Character set with no lowercase (only uppercase, numeric, symbols)
        let char_set = vec![b'A', b'B', b'0', b'1', b'!', b'@'];
        let pattern = vec![PatternChar::Lowercase]; // Will fallback to char_set

        let mut rng = StdRng::seed_from_u64(456);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 1);
        // Should fallback to any character from char_set
        assert!(char_set.contains(&(password.chars().next().unwrap() as u8)));
    }

    #[test]
    fn test_generate_password_from_pattern_empty_uppercase() {
        use rand::{SeedableRng, rngs::StdRng};

        // Character set with no uppercase
        let char_set = vec![b'a', b'b', b'0', b'1', b'!', b'@'];
        let pattern = vec![PatternChar::Uppercase]; // Will fallback to char_set

        let mut rng = StdRng::seed_from_u64(789);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 1);
        assert!(char_set.contains(&(password.chars().next().unwrap() as u8)));
    }

    #[test]
    fn test_generate_password_from_pattern_empty_numeric() {
        use rand::{SeedableRng, rngs::StdRng};

        // Character set with no numeric
        let char_set = vec![b'a', b'b', b'A', b'B', b'!', b'@'];
        let pattern = vec![PatternChar::Numeric]; // Will fallback to char_set

        let mut rng = StdRng::seed_from_u64(1011);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 1);
        assert!(char_set.contains(&(password.chars().next().unwrap() as u8)));
    }

    #[test]
    fn test_generate_password_from_pattern_empty_symbols() {
        use rand::{SeedableRng, rngs::StdRng};

        // Character set with no symbols
        let char_set = vec![b'a', b'b', b'A', b'B', b'0', b'1'];
        let pattern = vec![PatternChar::Symbol]; // Will fallback to char_set

        let mut rng = StdRng::seed_from_u64(1213);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 1);
        assert!(char_set.contains(&(password.chars().next().unwrap() as u8)));
    }

    #[test]
    fn test_generate_password_with_minimums() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![
            b'a', b'b', b'c', // lowercase
            b'A', b'B', b'C', // uppercase
            b'0', b'1', b'2', // numeric
            b'!', b'@', b'#', // symbols
        ];

        let mut rng = StdRng::seed_from_u64(456);
        let password =
            generate_password_with_minimums(&char_set, 10, Some(2), Some(2), Some(2), &mut rng);

        assert_eq!(password.len(), 10);

        // Count character types
        let mut capitals = 0;
        let mut numerals = 0;
        let mut symbols = 0;

        for c in password.chars() {
            if c.is_ascii_uppercase() {
                capitals += 1;
            } else if c.is_ascii_digit() {
                numerals += 1;
            } else if !c.is_alphanumeric() {
                symbols += 1;
            }
        }

        assert!(capitals >= 2);
        assert!(numerals >= 2);
        assert!(symbols >= 2);
    }

    #[test]
    fn test_generate_password_with_minimums_empty_sets() {
        use rand::{SeedableRng, rngs::StdRng};

        // Only lowercase available
        let char_set = vec![b'a', b'b', b'c'];

        let mut rng = StdRng::seed_from_u64(789);
        let password =
            generate_password_with_minimums(&char_set, 5, Some(2), Some(2), Some(2), &mut rng);

        assert_eq!(password.len(), 5);
        // All should be lowercase since that's all available
        for c in password.chars() {
            assert!(c.is_ascii_lowercase());
        }
    }

    #[test]
    fn test_generate_password_with_minimums_no_minimums() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![b'a', b'b', b'c', b'A', b'B', b'0', b'1', b'!', b'@'];

        let mut rng = StdRng::seed_from_u64(101);
        let password = generate_password_with_minimums(&char_set, 8, None, None, None, &mut rng);

        assert_eq!(password.len(), 8);
    }

    #[test]
    fn test_generate_passwords() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![b'a', b'b', b'c', b'1', b'2', b'3'];
        let params = GenerationParams {
            length: 5,
            count: 3,
            min_capitals: None,
            min_numerals: None,
            min_symbols: None,
            pattern: None,
        };

        let mut rng = StdRng::seed_from_u64(202);
        let passwords = generate_passwords(&char_set, &params, &mut rng);

        assert_eq!(passwords.len(), 3);
        for pass in &passwords {
            assert_eq!(pass.len(), 5);
        }
    }

    #[test]
    fn test_generate_passwords_with_pattern() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![b'a', b'b', b'A', b'B', b'0', b'1', b'!', b'@'];
        let pattern = vec![
            PatternChar::Lowercase,
            PatternChar::Uppercase,
            PatternChar::Numeric,
            PatternChar::Symbol,
        ];
        let params = GenerationParams {
            length: 4,
            count: 2,
            min_capitals: None,
            min_numerals: None,
            min_symbols: None,
            pattern: Some(pattern),
        };

        let mut rng = StdRng::seed_from_u64(303);
        let passwords = generate_passwords(&char_set, &params, &mut rng);

        assert_eq!(passwords.len(), 2);
        for pass in &passwords {
            assert_eq!(pass.len(), 4);
        }
    }

    #[test]
    fn test_generate_passwords_with_minimums() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![
            b'a', b'b', b'c', b'A', b'B', b'C', b'0', b'1', b'2', b'!', b'@', b'#',
        ];
        let params = GenerationParams {
            length: 8,
            count: 2,
            min_capitals: Some(1),
            min_numerals: Some(1),
            min_symbols: Some(1),
            pattern: None,
        };

        let mut rng = StdRng::seed_from_u64(404);
        let passwords = generate_passwords(&char_set, &params, &mut rng);

        assert_eq!(passwords.len(), 2);
        for pass in &passwords {
            assert_eq!(pass.len(), 8);
            // Verify minimums are met
            let mut has_capital = false;
            let mut has_numeral = false;
            let mut has_symbol = false;
            for c in pass.chars() {
                if c.is_ascii_uppercase() {
                    has_capital = true;
                } else if c.is_ascii_digit() {
                    has_numeral = true;
                } else if !c.is_alphanumeric() {
                    has_symbol = true;
                }
            }
            assert!(has_capital);
            assert!(has_numeral);
            assert!(has_symbol);
        }
    }

    #[test]
    fn test_print_columns_single_column() {
        let passwords = vec![
            "pass1".to_string(),
            "pass2".to_string(),
            "pass3".to_string(),
        ];
        // We can't easily test print! without capturing stdout, so we'll test the logic
        // by verifying the function doesn't panic
        print_columns(passwords.clone(), 1, false);
        print_columns(passwords, 1, true);
    }

    #[test]
    fn test_print_columns_multiple_columns() {
        let passwords: Vec<String> = vec![
            "short".to_string(),
            "verylongpassword".to_string(),
            "medium".to_string(),
            "x".to_string(),
        ];
        // Test that it doesn't panic
        print_columns(passwords.clone(), 2, false);
        print_columns(passwords.clone(), 2, true);
        print_columns(passwords.clone(), 3, false);
        print_columns(passwords, 4, false);
    }

    #[test]
    fn test_print_columns_incomplete_row() {
        let passwords = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(), // 5 passwords, 3 columns = incomplete last row
        ];
        // Should not panic with incomplete row
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
    fn test_parse_exclude_chars_non_printable_start() {
        // Test range with start character < 32 (non-printable)
        // This should skip the range logic and treat as individual chars
        let result = parse_exclude_chars(vec!["\x1f-9".to_string()]);
        // Should succeed but treat as individual characters, not a range
        assert!(result.is_ok());
        let chars = result.unwrap();
        // Should contain the characters from the string, not a range expansion
        assert!(chars.len() >= 2); // At least \x1f, -, and 9
    }

    #[test]
    fn test_parse_exclude_chars_non_printable_end() {
        // Test range with end character >= 127 (non-printable)
        // This should skip the range logic
        let result = parse_exclude_chars(vec!["a-\x7f".to_string()]);
        // Should succeed but treat as individual characters
        assert!(result.is_ok());
        // Should not expand as a range
        let chars = result.unwrap();
        // The range logic should be skipped due to end >= 127
        // So it should treat as individual characters
        assert!(chars.contains(&'a'));
    }

    #[test]
    fn test_parse_exclude_chars_duplicate_handling() {
        // Test that duplicates are properly handled
        let result = parse_exclude_chars(vec!["a".to_string(), "a".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        let chars = result.unwrap();
        // Should only contain one 'a' and one 'b'
        assert_eq!(chars.iter().filter(|&&c| c == 'a').count(), 1);
        assert_eq!(chars.iter().filter(|&&c| c == 'b').count(), 1);
    }

    #[test]
    fn test_parse_exclude_chars_duplicate_in_range_and_individual() {
        // Test that characters in ranges are not duplicated when also specified individually
        let result = parse_exclude_chars(vec!["a-c".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        let chars = result.unwrap();
        // Should contain a, b, c each once
        assert_eq!(chars.iter().filter(|&&c| c == 'a').count(), 1);
        assert_eq!(chars.iter().filter(|&&c| c == 'b').count(), 1);
        assert_eq!(chars.iter().filter(|&&c| c == 'c').count(), 1);
    }

    #[test]
    fn test_parse_exclude_chars_boundary_conditions() {
        // Test range exactly at printable ASCII boundaries
        // Space (32) to ~ (126) should work
        let result = parse_exclude_chars(vec![" -~".to_string()]);
        assert!(result.is_ok());
        let chars = result.unwrap();
        // Should expand to all printable ASCII
        assert!(chars.contains(&' '));
        assert!(chars.contains(&'~'));
    }

    #[test]
    fn test_build_char_set_all_types_disabled_lowercase_available() {
        // Test with all character types disabled, but lowercase still available
        let args = create_test_args(true, true, true, vec![]);
        let char_set = build_char_set(&args).unwrap();
        // Should only contain lowercase letters
        assert!(!char_set.is_empty());
        assert!(char_set.contains(&b'a'));
        assert!(char_set.contains(&b'z'));
        assert!(!char_set.contains(&b'A'));
        assert!(!char_set.contains(&b'0'));
        assert!(!char_set.contains(&b'!'));
    }

    #[test]
    fn test_build_char_set_include_chars_empty_after_exclude() {
        // Test include_chars where exclude_chars removes all characters
        let mut args = create_test_args(false, false, false, vec!['a', 'b', 'c']);
        args.include_chars = Some(vec!['a', 'b', 'c']);
        let result = build_char_set(&args);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PasswordError::EmptyCharacterSet
        ));
    }

    #[test]
    fn test_build_char_set_include_chars_with_exclusions_partial() {
        // Test include_chars with exclude_chars that removes some but not all
        let mut args = create_test_args(false, false, false, vec!['a']);
        args.include_chars = Some(vec!['a', 'b', 'c', 'd', 'e']);
        let char_set = build_char_set(&args).unwrap();
        assert_eq!(char_set.len(), 4); // b, c, d, e
        assert!(!char_set.contains(&b'a'));
        assert!(char_set.contains(&b'b'));
        assert!(char_set.contains(&b'c'));
        assert!(char_set.contains(&b'd'));
        assert!(char_set.contains(&b'e'));
    }

    #[test]
    fn test_password_error_display_all_variants() {
        // Ensure all error variants are tested for Display implementation
        let err = PasswordError::InvalidLength;
        let msg = err.to_string();
        assert!(msg.contains("Password length must be greater than 0"));

        let err = PasswordError::InvalidLengthTooLong;
        let msg = err.to_string();
        assert!(msg.contains("exceeds maximum of 10,000"));

        let err = PasswordError::InvalidCount;
        let msg = err.to_string();
        assert!(msg.contains("Password count must be greater than 0"));

        let err = PasswordError::EmptyCharacterSet;
        let msg = err.to_string();
        assert!(msg.contains("All characters have been excluded"));
        assert!(msg.contains("Hint"));

        let err = PasswordError::AllTypesDisabled;
        let msg = err.to_string();
        assert!(msg.contains("All character types are disabled"));
        assert!(msg.contains("Hint"));
    }

    #[test]
    fn test_password_error_source() {
        // Test that PasswordError implements std::error::Error
        let err = PasswordError::InvalidLength;
        // Should be able to use as Error trait object
        let _err_ref: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_generate_password_with_minimums_exceeding_length() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![b'a', b'b', b'A', b'B', b'0', b'1', b'!', b'@'];
        // Request 5 minimums but length is only 4
        // Minimums take precedence, so password will be length 5
        let mut rng = StdRng::seed_from_u64(1001);
        let password = generate_password_with_minimums(&char_set, 4, Some(5), None, None, &mut rng);

        // Should generate a password with at least 5 capitals (minimum takes precedence)
        assert!(password.len() >= 5);
        let capitals = password.chars().filter(|c| c.is_ascii_uppercase()).count();
        assert!(capitals >= 5); // Minimum requirement is met
    }

    #[test]
    fn test_generate_password_with_minimums_sum_exceeds_length() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![
            b'a', b'b', b'c', b'A', b'B', b'C', b'0', b'1', b'2', b'!', b'@', b'#',
        ];
        // Request min_capitals=3, min_numerals=3, min_symbols=3, but length=6
        // Minimums take precedence, so password will be at least length 9
        let mut rng = StdRng::seed_from_u64(1002);
        let password =
            generate_password_with_minimums(&char_set, 6, Some(3), Some(3), Some(3), &mut rng);

        // Password length should be at least 9 (sum of minimums)
        // May be more if minimums are applied then filled up to length
        assert!(password.len() >= 9);
        let capitals = password.chars().filter(|c| c.is_ascii_uppercase()).count();
        let numerals = password.chars().filter(|c| c.is_ascii_digit()).count();
        let symbols = password.chars().filter(|c| !c.is_alphanumeric()).count();

        // Should meet all minimum requirements
        assert!(capitals >= 3);
        assert!(numerals >= 3);
        assert!(symbols >= 3);
    }

    #[test]
    fn test_generate_password_with_minimums_exact_length() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![b'a', b'b', b'A', b'B', b'0', b'1', b'!', b'@'];
        // Request min_capitals=2, min_numerals=2, length=4
        let mut rng = StdRng::seed_from_u64(1003);
        let password =
            generate_password_with_minimums(&char_set, 4, Some(2), Some(2), None, &mut rng);

        assert_eq!(password.len(), 4);
        let capitals = password.chars().filter(|c| c.is_ascii_uppercase()).count();
        let numerals = password.chars().filter(|c| c.is_ascii_digit()).count();

        assert!(capitals >= 2);
        assert!(numerals >= 2);
    }

    #[test]
    fn test_generate_password_from_pattern_all_same_type() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![b'a', b'b', b'c', b'A', b'B', b'C', b'0', b'1', b'2'];
        let pattern = vec![
            PatternChar::Lowercase,
            PatternChar::Lowercase,
            PatternChar::Lowercase,
        ];

        let mut rng = StdRng::seed_from_u64(2001);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 3);
        for c in password.chars() {
            assert!(c.is_ascii_lowercase());
        }
    }

    #[test]
    fn test_generate_password_from_pattern_very_long() {
        use rand::{SeedableRng, rngs::StdRng};

        let char_set = vec![b'a', b'b', b'A', b'B', b'0', b'1', b'!', b'@'];
        // Create a pattern of length 100
        let pattern: Vec<PatternChar> = (0..100)
            .map(|i| match i % 4 {
                0 => PatternChar::Lowercase,
                1 => PatternChar::Uppercase,
                2 => PatternChar::Numeric,
                _ => PatternChar::Symbol,
            })
            .collect();

        let mut rng = StdRng::seed_from_u64(2002);
        let password = generate_password_from_pattern(&char_set, &pattern, &mut rng);

        assert_eq!(password.len(), 100);
    }

    #[test]
    fn test_print_columns_very_long_passwords() {
        let passwords = vec!["a".repeat(100), "b".repeat(50), "c".repeat(150)];
        // Test width calculation with very long passwords
        print_columns(passwords.clone(), 1, false);
        print_columns(passwords.clone(), 2, false);
        print_columns(passwords, 3, true);
    }

    #[test]
    fn test_print_columns_single_password_multi_column() {
        // Test single password with multiple columns (should still print it)
        let passwords = vec!["single".to_string()];
        print_columns(passwords, 5, false);
    }

    #[test]
    fn test_validate_args_all_types_disabled_lowercase_available() {
        // Test validate_args when all types are disabled but lowercase available
        let args = create_test_args(true, true, true, vec![]);
        let result = validate_args(&args);
        assert!(result.is_ok()); // Should be valid since lowercase is still available
    }
}
