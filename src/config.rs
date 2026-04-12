use crate::error::PasswordError;

/// Pattern character types for pattern-based generation.
#[derive(Debug, Clone, Copy)]
pub enum PatternChar {
    Lowercase,
    Uppercase,
    Numeric,
    Symbol,
}

/// Maximum allowed password length to prevent memory issues.
pub const MAX_PASSWORD_LENGTH: u32 = 10_000;

/// Unified configuration for password generation.
///
/// Combines character set options and generation parameters
/// into a single struct.
#[derive(Debug, Clone)]
pub struct PasswordConfig {
    /// Disable capital letters in the character set.
    pub capitals_off: bool,
    /// Disable numerals in the character set.
    pub numerals_off: bool,
    /// Disable symbols in the character set.
    pub symbols_off: bool,
    /// Characters to exclude from the set.
    pub exclude_chars: Vec<char>,
    /// If set, use only these characters (overrides type flags).
    pub include_chars: Option<Vec<char>>,
    /// Minimum number of capital letters required per password.
    pub min_capitals: Option<u32>,
    /// Minimum number of numerals required per password.
    pub min_numerals: Option<u32>,
    /// Minimum number of symbols required per password.
    pub min_symbols: Option<u32>,
    /// Pattern for structured generation (overrides `length`).
    pub pattern: Option<Vec<PatternChar>>,
    /// Length of each password (ignored when `pattern` is set).
    pub length: u32,
    /// Number of passwords to generate.
    pub count: u32,
}

impl PasswordConfig {
    /// Validates this configuration.
    ///
    /// Checks length bounds, count, and that the resulting character set
    /// won't be empty when all types are disabled.
    pub fn validate(&self) -> Result<(), PasswordError> {
        if self.length == 0 {
            return Err(PasswordError::InvalidLength);
        }
        if self.length > MAX_PASSWORD_LENGTH {
            return Err(PasswordError::InvalidLengthTooLong);
        }
        if self.count == 0 {
            return Err(PasswordError::InvalidCount);
        }

        if self.capitals_off && self.numerals_off && self.symbols_off {
            use crate::charset::CharacterSet;
            if CharacterSet::from_config(self).is_err() {
                return Err(PasswordError::AllTypesDisabled);
            }
        }

        Ok(())
    }

    /// Returns the effective password length — pattern length if set, otherwise `self.length`.
    pub fn effective_length(&self) -> u32 {
        self.pattern
            .as_ref()
            .map(|p| p.len() as u32)
            .unwrap_or(self.length)
    }
}

/// Parses a pattern string like "LLLNNNSSS" into a `PatternChar` vector.
///
/// Characters are case-insensitive: L/l = lowercase, U/u = uppercase,
/// N/n = numeric, S/s = symbol.
pub fn parse_pattern(pattern: &str) -> Result<Vec<PatternChar>, PasswordError> {
    pattern
        .chars()
        .map(|c| match c {
            'L' | 'l' => Ok(PatternChar::Lowercase),
            'U' | 'u' => Ok(PatternChar::Uppercase),
            'N' | 'n' => Ok(PatternChar::Numeric),
            'S' | 's' => Ok(PatternChar::Symbol),
            _ => Err(PasswordError::InvalidPatternChar(c)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_validate_valid() {
        assert!(default_config().validate().is_ok());
    }

    #[test]
    fn test_validate_zero_length() {
        let mut cfg = default_config();
        cfg.length = 0;
        assert!(matches!(
            cfg.validate().unwrap_err(),
            PasswordError::InvalidLength
        ));
    }

    #[test]
    fn test_validate_too_long() {
        let mut cfg = default_config();
        cfg.length = 10_001;
        assert!(matches!(
            cfg.validate().unwrap_err(),
            PasswordError::InvalidLengthTooLong
        ));
    }

    #[test]
    fn test_validate_zero_count() {
        let mut cfg = default_config();
        cfg.count = 0;
        assert!(matches!(
            cfg.validate().unwrap_err(),
            PasswordError::InvalidCount
        ));
    }

    #[test]
    fn test_validate_all_types_disabled_but_lowercase_available() {
        let mut cfg = default_config();
        cfg.capitals_off = true;
        cfg.numerals_off = true;
        cfg.symbols_off = true;
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_all_types_disabled_all_excluded() {
        let mut cfg = default_config();
        cfg.capitals_off = true;
        cfg.numerals_off = true;
        cfg.symbols_off = true;
        cfg.exclude_chars = (b'a'..=b'z').map(|b| b as char).collect();
        let err = cfg.validate().unwrap_err();
        assert!(
            matches!(err, PasswordError::AllTypesDisabled)
                || matches!(err, PasswordError::EmptyCharacterSet)
        );
    }

    #[test]
    fn test_effective_length_no_pattern() {
        let cfg = default_config();
        assert_eq!(cfg.effective_length(), 16);
    }

    #[test]
    fn test_effective_length_with_pattern() {
        let mut cfg = default_config();
        cfg.pattern = Some(vec![
            PatternChar::Lowercase,
            PatternChar::Uppercase,
            PatternChar::Numeric,
        ]);
        assert_eq!(cfg.effective_length(), 3);
    }

    #[test]
    fn test_parse_pattern_valid() {
        let p = parse_pattern("LLL").unwrap();
        assert_eq!(p.len(), 3);
        assert!(matches!(p[0], PatternChar::Lowercase));

        let p2 = parse_pattern("UUNNSS").unwrap();
        assert_eq!(p2.len(), 6);
        assert!(matches!(p2[0], PatternChar::Uppercase));
        assert!(matches!(p2[2], PatternChar::Numeric));
        assert!(matches!(p2[4], PatternChar::Symbol));
    }

    #[test]
    fn test_parse_pattern_case_insensitive() {
        let p = parse_pattern("lluunnss").unwrap();
        assert_eq!(p.len(), 8);
        assert!(matches!(p[0], PatternChar::Lowercase));
        assert!(matches!(p[2], PatternChar::Uppercase));
        assert!(matches!(p[4], PatternChar::Numeric));
        assert!(matches!(p[6], PatternChar::Symbol));
    }

    #[test]
    fn test_parse_pattern_invalid() {
        let result = parse_pattern("LLX");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pattern_empty() {
        let p = parse_pattern("").unwrap();
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn test_error_display() {
        let err = PasswordError::InvalidLength;
        assert!(
            err.to_string()
                .contains("Password length must be greater than 0")
        );

        let err = PasswordError::InvalidLengthTooLong;
        assert!(err.to_string().contains("exceeds maximum of 10,000"));

        let err = PasswordError::InvalidCount;
        assert!(
            err.to_string()
                .contains("Password count must be greater than 0")
        );

        let err = PasswordError::EmptyCharacterSet;
        assert!(
            err.to_string()
                .contains("All characters have been excluded")
        );
        assert!(err.to_string().contains("Hint"));

        let err = PasswordError::AllTypesDisabled;
        assert!(err.to_string().contains("All character types are disabled"));
        assert!(err.to_string().contains("Hint"));

        let err = PasswordError::InvalidPatternChar('X');
        assert!(err.to_string().contains("Invalid pattern character"));

        let err = PasswordError::InvalidCharRange {
            range: "z-a".into(),
            start: 'z',
            end: 'a',
        };
        assert!(err.to_string().contains("Invalid range"));
    }

    #[test]
    fn test_error_is_std_error() {
        let err = PasswordError::InvalidLength;
        let _err_ref: &dyn std::error::Error = &err;
    }
}
