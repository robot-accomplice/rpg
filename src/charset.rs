use std::collections::HashSet;

use crate::config::PasswordConfig;
use crate::error::PasswordError;

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

/// A validated, non-empty set of characters for password generation.
///
/// Pre-computes subsets by character type (lowercase, uppercase, numeric, symbol)
/// so generation functions don't need to re-derive them on every call.
#[derive(Debug, Clone)]
pub struct CharacterSet {
    all: Vec<u8>,
    lowercase: Vec<u8>,
    uppercase: Vec<u8>,
    numeric: Vec<u8>,
    symbols: Vec<u8>,
}

impl CharacterSet {
    /// Builds a `CharacterSet` from the given configuration.
    ///
    /// Returns `Err` if the resulting set would be empty.
    pub fn from_config(config: &PasswordConfig) -> Result<Self, PasswordError> {
        let raw = build_raw_chars(config)?;
        Ok(Self::from_raw(raw))
    }

    /// Creates a `CharacterSet` directly from a raw byte vector.
    ///
    /// Pre-computes the character-type subsets.
    ///
    /// # Panics
    /// Panics if `raw` is empty. Use `from_config` for validated construction.
    pub fn from_raw(raw: Vec<u8>) -> Self {
        assert!(!raw.is_empty(), "CharacterSet cannot be empty");

        let lowercase: Vec<u8> = raw
            .iter()
            .filter(|&&b| (ASCII_LOWERCASE_START..=ASCII_LOWERCASE_END).contains(&b))
            .copied()
            .collect();
        let uppercase: Vec<u8> = raw
            .iter()
            .filter(|&&b| (ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END).contains(&b))
            .copied()
            .collect();
        let numeric: Vec<u8> = raw
            .iter()
            .filter(|&&b| (ASCII_NUMERAL_START..=ASCII_NUMERAL_END).contains(&b))
            .copied()
            .collect();
        let symbols: Vec<u8> = raw
            .iter()
            .filter(|&&b| {
                !(ASCII_LOWERCASE_START..=ASCII_LOWERCASE_END).contains(&b)
                    && !(ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END).contains(&b)
                    && !(ASCII_NUMERAL_START..=ASCII_NUMERAL_END).contains(&b)
            })
            .copied()
            .collect();

        Self {
            all: raw,
            lowercase,
            uppercase,
            numeric,
            symbols,
        }
    }

    /// All characters in the set.
    pub fn all(&self) -> &[u8] {
        &self.all
    }

    /// Lowercase characters in the set.
    pub fn lowercase(&self) -> &[u8] {
        &self.lowercase
    }

    /// Uppercase characters in the set.
    pub fn uppercase(&self) -> &[u8] {
        &self.uppercase
    }

    /// Numeric characters in the set.
    pub fn numeric(&self) -> &[u8] {
        &self.numeric
    }

    /// Symbol characters in the set.
    pub fn symbols(&self) -> &[u8] {
        &self.symbols
    }

    /// Total number of characters in the set.
    pub fn len(&self) -> usize {
        self.all.len()
    }

    /// Returns `true` if the character set is empty.
    ///
    /// In practice this is always `false` because `from_config` rejects
    /// empty sets, but the method is provided for API completeness.
    pub fn is_empty(&self) -> bool {
        self.all.is_empty()
    }
}

/// Builds the raw character vector based on configuration.
fn build_raw_chars(config: &PasswordConfig) -> Result<Vec<u8>, PasswordError> {
    let mut chars = Vec::new();

    if let Some(ref include_chars) = config.include_chars {
        for &c in include_chars {
            chars.push(c as u8);
        }
    } else {
        let estimated_capacity = if config.symbols_off { 62 } else { 94 };
        chars.reserve(estimated_capacity);

        // Lowercase always included
        chars.extend(ASCII_LOWERCASE_START..=ASCII_LOWERCASE_END);

        if !config.capitals_off {
            chars.extend(ASCII_UPPERCASE_START..=ASCII_UPPERCASE_END);
        }
        if !config.numerals_off {
            chars.extend(ASCII_NUMERAL_START..=ASCII_NUMERAL_END);
        }
        if !config.symbols_off {
            chars.extend(ASCII_SYMBOL_RANGE_1_START..=ASCII_SYMBOL_RANGE_1_END);
            chars.extend(ASCII_SYMBOL_RANGE_2_START..=ASCII_SYMBOL_RANGE_2_END);
            chars.extend(ASCII_SYMBOL_RANGE_3_START..=ASCII_SYMBOL_RANGE_3_END);
            chars.extend(ASCII_SYMBOL_RANGE_4_START..=ASCII_SYMBOL_RANGE_4_END);
        }
    }

    let exclude_set: HashSet<char> = config.exclude_chars.iter().cloned().collect();
    chars.retain(|&b| !exclude_set.contains(&(b as char)));

    if chars.is_empty() {
        return Err(PasswordError::EmptyCharacterSet);
    }

    Ok(chars)
}

/// Parses character specification strings, expanding ranges like "a-z" or "0-9".
///
/// Each string can be:
/// - A range like "a-z" (three characters, middle is '-')
/// - Individual characters like "abc"
///
/// Duplicates are removed.
pub fn parse_char_specs(specs: Vec<String>) -> Result<Vec<char>, PasswordError> {
    let mut chars = Vec::new();

    for s in specs {
        if s.len() == 3 {
            let cs: Vec<char> = s.chars().collect();
            if cs[1] == '-' {
                let start = cs[0] as u8;
                let end = cs[2] as u8;

                if start <= end && start >= 32 && end < 127 {
                    for byte in start..=end {
                        chars.push(byte as char);
                    }
                    continue;
                } else if start > end {
                    return Err(PasswordError::InvalidCharRange {
                        range: s,
                        start: cs[0],
                        end: cs[2],
                    });
                }
            }
        }

        for c in s.chars() {
            if !chars.contains(&c) {
                chars.push(c);
            }
        }
    }

    Ok(chars)
}

/// Calculates password entropy in bits.
pub fn calculate_entropy(char_set_size: usize, length: u32) -> f64 {
    (char_set_size as f64).log2() * length as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PasswordConfig;

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
            length: 16,
            count: 1,
        }
    }

    #[test]
    fn test_from_config_default() {
        let cs = CharacterSet::from_config(&default_config()).unwrap();
        assert!(cs.len() > 60);
        assert!(!cs.lowercase().is_empty());
        assert!(!cs.uppercase().is_empty());
        assert!(!cs.numeric().is_empty());
        assert!(!cs.symbols().is_empty());
    }

    #[test]
    fn test_from_config_no_capitals() {
        let mut cfg = default_config();
        cfg.capitals_off = true;
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert!(cs.uppercase().is_empty());
        assert!(!cs.lowercase().is_empty());
    }

    #[test]
    fn test_from_config_no_numerals() {
        let mut cfg = default_config();
        cfg.numerals_off = true;
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert!(cs.numeric().is_empty());
    }

    #[test]
    fn test_from_config_no_symbols() {
        let mut cfg = default_config();
        cfg.symbols_off = true;
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert!(cs.symbols().is_empty());
    }

    #[test]
    fn test_from_config_with_exclusions() {
        let mut cfg = default_config();
        cfg.exclude_chars = vec!['a', 'b', 'c'];
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert!(!cs.all().contains(&b'a'));
        assert!(!cs.all().contains(&b'b'));
        assert!(!cs.all().contains(&b'c'));
        assert!(cs.all().contains(&b'd'));
    }

    #[test]
    fn test_from_config_all_excluded() {
        let mut cfg = default_config();
        cfg.capitals_off = true;
        cfg.numerals_off = true;
        cfg.symbols_off = true;
        cfg.exclude_chars = (b'a'..=b'z').map(|b| b as char).collect();
        let result = CharacterSet::from_config(&cfg);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PasswordError::EmptyCharacterSet
        ));
    }

    #[test]
    fn test_from_config_include_chars() {
        let mut cfg = default_config();
        cfg.include_chars = Some(vec!['a', 'b', 'c', '1', '2', '!']);
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert_eq!(cs.len(), 6);
        assert!(cs.all().contains(&b'a'));
        assert!(!cs.all().contains(&b'd'));
    }

    #[test]
    fn test_from_config_include_chars_with_exclusions() {
        let mut cfg = default_config();
        cfg.include_chars = Some(vec!['a', 'b', 'c']);
        cfg.exclude_chars = vec!['a'];
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert!(!cs.all().contains(&b'a'));
        assert!(cs.all().contains(&b'b'));
        assert!(cs.all().contains(&b'c'));
    }

    #[test]
    fn test_from_config_include_empty_after_exclude() {
        let mut cfg = default_config();
        cfg.include_chars = Some(vec!['a', 'b', 'c']);
        cfg.exclude_chars = vec!['a', 'b', 'c'];
        let result = CharacterSet::from_config(&cfg);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_config_all_types_disabled_lowercase_available() {
        let mut cfg = default_config();
        cfg.capitals_off = true;
        cfg.numerals_off = true;
        cfg.symbols_off = true;
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert!(!cs.all().is_empty());
        assert!(cs.all().contains(&b'a'));
        assert!(cs.all().contains(&b'z'));
        assert!(!cs.all().contains(&b'A'));
    }

    #[test]
    fn test_from_config_include_chars_partial_exclude() {
        let mut cfg = default_config();
        cfg.include_chars = Some(vec!['a', 'b', 'c', 'd', 'e']);
        cfg.exclude_chars = vec!['a'];
        let cs = CharacterSet::from_config(&cfg).unwrap();
        assert_eq!(cs.len(), 4);
    }

    #[test]
    fn test_parse_char_specs_range() {
        let result = parse_char_specs(vec!["a-z".to_string()]).unwrap();
        assert_eq!(result.len(), 26);
        assert!(result.contains(&'a'));
        assert!(result.contains(&'z'));
    }

    #[test]
    fn test_parse_char_specs_numeric_range() {
        let result = parse_char_specs(vec!["0-9".to_string()]).unwrap();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_parse_char_specs_small_range() {
        let result = parse_char_specs(vec!["a-c".to_string()]).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&'a'));
        assert!(result.contains(&'b'));
        assert!(result.contains(&'c'));
    }

    #[test]
    fn test_parse_char_specs_individual() {
        let result = parse_char_specs(vec!["abc".to_string()]).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_char_specs_mixed() {
        let result =
            parse_char_specs(vec!["a-c".to_string(), "x".to_string(), "0-2".to_string()]).unwrap();
        assert!(result.contains(&'a'));
        assert!(result.contains(&'b'));
        assert!(result.contains(&'c'));
        assert!(result.contains(&'x'));
        assert!(result.contains(&'0'));
        assert!(result.contains(&'1'));
        assert!(result.contains(&'2'));
    }

    #[test]
    fn test_parse_char_specs_invalid_range() {
        let result = parse_char_specs(vec!["z-a".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_char_specs_duplicate_handling() {
        let result =
            parse_char_specs(vec!["a".to_string(), "a".to_string(), "b".to_string()]).unwrap();
        assert_eq!(result.iter().filter(|&&c| c == 'a').count(), 1);
    }

    #[test]
    fn test_parse_char_specs_duplicate_in_range_and_individual() {
        let result = parse_char_specs(vec!["a-c".to_string(), "b".to_string()]).unwrap();
        assert_eq!(result.iter().filter(|&&c| c == 'b').count(), 1);
    }

    #[test]
    fn test_parse_char_specs_boundary_conditions() {
        let result = parse_char_specs(vec![" -~".to_string()]).unwrap();
        assert!(result.contains(&' '));
        assert!(result.contains(&'~'));
    }

    #[test]
    fn test_parse_char_specs_non_printable_start() {
        let result = parse_char_specs(vec!["\x1f-9".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_char_specs_non_printable_end() {
        let result = parse_char_specs(vec!["a-\x7f".to_string()]);
        assert!(result.is_ok());
        assert!(result.unwrap().contains(&'a'));
    }

    #[test]
    fn test_calculate_entropy() {
        let e1 = calculate_entropy(26, 8);
        assert!(e1 > 0.0);
        let e2 = calculate_entropy(62, 16);
        assert!(e2 > e1);
        let e3 = calculate_entropy(94, 20);
        assert!(e3 > e2);
        let e4 = calculate_entropy(62, 32);
        assert!(e4 > e2);
    }

    #[test]
    fn test_from_raw_precomputes_subsets() {
        let raw = vec![b'a', b'B', b'3', b'!'];
        let cs = CharacterSet::from_raw(raw);
        assert_eq!(cs.lowercase(), &[b'a']);
        assert_eq!(cs.uppercase(), &[b'B']);
        assert_eq!(cs.numeric(), &[b'3']);
        assert_eq!(cs.symbols(), &[b'!']);
    }
}
