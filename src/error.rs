use std::fmt;

/// Error type for password generation and configuration.
///
/// Covers validation errors, character set construction failures,
/// and input parsing errors.
#[derive(Debug, Clone)]
pub enum PasswordError {
    /// Password length is zero.
    InvalidLength,
    /// Password length exceeds the maximum allowed.
    InvalidLengthTooLong,
    /// Password count is zero.
    InvalidCount,
    /// All characters have been excluded or disabled, leaving an empty set.
    EmptyCharacterSet,
    /// All character types are disabled and remaining characters are excluded.
    AllTypesDisabled,
    /// A character range specification is invalid (e.g., "z-a").
    InvalidCharRange {
        range: String,
        start: char,
        end: char,
    },
    /// A pattern string contains an unrecognized character.
    InvalidPatternChar(char),
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
            PasswordError::InvalidCharRange { range, start, end } => {
                write!(
                    f,
                    "Invalid range '{}': start character '{}' is greater than end character '{}'",
                    range, start, end
                )
            }
            PasswordError::InvalidPatternChar(c) => {
                write!(
                    f,
                    "Invalid pattern character: '{}'. Use L (lowercase), U (uppercase), N (numeric), S (symbol)",
                    c
                )
            }
        }
    }
}

impl std::error::Error for PasswordError {}
