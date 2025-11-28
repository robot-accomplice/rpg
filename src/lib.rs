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
            (b < ASCII_LOWERCASE_START || b > ASCII_LOWERCASE_END)
                && (b < ASCII_UPPERCASE_START || b > ASCII_UPPERCASE_END)
                && (b < ASCII_NUMERAL_START || b > ASCII_NUMERAL_END)
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
            (b < ASCII_LOWERCASE_START || b > ASCII_LOWERCASE_END)
                && (b < ASCII_UPPERCASE_START || b > ASCII_UPPERCASE_END)
                && (b < ASCII_NUMERAL_START || b > ASCII_NUMERAL_END)
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
    length: u32,
    count: u32,
    min_capitals: Option<u32>,
    min_numerals: Option<u32>,
    min_symbols: Option<u32>,
    pattern: Option<&[PatternChar]>,
    rng: &mut R,
) -> Vec<String> {
    let mut passwords = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let pass = if let Some(pat) = pattern {
            generate_password_from_pattern(char_set, pat, rng)
        } else {
            generate_password_with_minimums(
                char_set,
                length,
                min_capitals,
                min_numerals,
                min_symbols,
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
}
