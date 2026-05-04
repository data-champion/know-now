use std::fmt;

use serde::{Deserialize, Serialize};

const MAX_PG_IDENTIFIER_LEN: usize = 63;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentifierError {
    Empty,
    TooLong { len: usize },
    InvalidStart { ch: char },
    InvalidChar { ch: char, pos: usize },
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "identifier must not be empty"),
            Self::TooLong { len } => write!(
                f,
                "identifier length {len} exceeds PostgreSQL limit of {MAX_PG_IDENTIFIER_LEN}"
            ),
            Self::InvalidStart { ch } => {
                write!(
                    f,
                    "identifier must start with a letter or underscore, got '{ch}'"
                )
            }
            Self::InvalidChar { ch, pos } => {
                write!(f, "invalid character '{ch}' at position {pos}")
            }
        }
    }
}

impl Identifier {
    /// # Errors
    /// Returns `IdentifierError` if the value is empty, too long, starts with
    /// a digit, or contains characters other than `[a-zA-Z0-9_]`.
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        let first = value.chars().next().ok_or(IdentifierError::Empty)?;
        if value.len() > MAX_PG_IDENTIFIER_LEN {
            return Err(IdentifierError::TooLong { len: value.len() });
        }
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(IdentifierError::InvalidStart { ch: first });
        }
        for (pos, ch) in value.chars().enumerate() {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(IdentifierError::InvalidChar { ch, pos });
            }
        }
        Ok(Self {
            value: value.to_owned(),
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub fn quoted(&self) -> String {
        format!("\"{}\"", self.value)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identifiers() {
        for name in ["customer", "order_item", "_private", "Col1", "a"] {
            assert!(Identifier::new(name).is_ok(), "should be valid: {name}");
        }
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(Identifier::new(""), Err(IdentifierError::Empty));
    }

    #[test]
    fn rejects_digit_start() {
        let err = Identifier::new("1abc").unwrap_err();
        assert!(matches!(err, IdentifierError::InvalidStart { ch: '1' }));
    }

    #[test]
    fn rejects_special_chars() {
        let err = Identifier::new("my-table").unwrap_err();
        assert!(matches!(err, IdentifierError::InvalidChar { ch: '-', .. }));
    }

    #[test]
    fn rejects_spaces() {
        let err = Identifier::new("my table").unwrap_err();
        assert!(matches!(err, IdentifierError::InvalidChar { ch: ' ', .. }));
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(64);
        let err = Identifier::new(&long).unwrap_err();
        assert!(matches!(err, IdentifierError::TooLong { len: 64 }));
    }

    #[test]
    fn max_length_is_accepted() {
        let name = "a".repeat(63);
        assert!(Identifier::new(&name).is_ok());
    }

    #[test]
    fn quoted_output() {
        let id = Identifier::new("customer").unwrap();
        assert_eq!(id.quoted(), "\"customer\"");
    }

    #[test]
    fn display_is_unquoted() {
        let id = Identifier::new("order_item").unwrap();
        assert_eq!(id.to_string(), "order_item");
    }

    #[test]
    fn identifiers_are_ordered() {
        let a = Identifier::new("alpha").unwrap();
        let b = Identifier::new("beta").unwrap();
        assert!(a < b);
    }
}
