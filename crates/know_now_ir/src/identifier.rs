use std::fmt;

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

const MAX_PG_IDENTIFIER_LEN: usize = 63;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentifierError {
    Empty,
    TooLong { len: usize },
    ContainsNul,
    PathTraversal,
    NonNfc { normalized: String },
    ContainsNewline { pos: usize },
    ContainsControlChar { ch: char, pos: usize },
    NonAscii { ch: char, pos: usize },
    InvalidStart { ch: char },
    InvalidChar { ch: char, pos: usize },
}

impl IdentifierError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "META-IDENT-EMPTY",
            Self::TooLong { .. } => "META-IDENT-TOO-LONG",
            Self::ContainsNul => "META-IDENT-NUL",
            Self::PathTraversal => "META-IDENT-PATH-TRAVERSAL",
            Self::NonNfc { .. } => "META-IDENT-NON-NFC",
            Self::ContainsNewline { .. } => "META-IDENT-NEWLINE",
            Self::ContainsControlChar { .. } => "META-IDENT-CONTROL-CHAR",
            Self::NonAscii { .. } => "META-IDENT-NON-ASCII",
            Self::InvalidStart { .. } => "META-IDENT-INVALID-START",
            Self::InvalidChar { .. } => "META-IDENT-INVALID-CHAR",
        }
    }
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "identifier must not be empty"),
            Self::TooLong { len } => write!(
                f,
                "identifier length {len} exceeds PostgreSQL limit of {MAX_PG_IDENTIFIER_LEN}"
            ),
            Self::ContainsNul => write!(f, "identifier contains NUL byte"),
            Self::PathTraversal => {
                write!(f, "identifier contains path-traversal segment '..'")
            }
            Self::NonNfc { normalized } => write!(
                f,
                "identifier must be NFC-normalized (normalized form: '{normalized}')"
            ),
            Self::ContainsNewline { pos } => {
                write!(f, "identifier contains newline at position {pos}")
            }
            Self::ContainsControlChar { ch, pos } => {
                write!(
                    f,
                    "identifier contains control character U+{:04X} at position {pos}",
                    *ch as u32
                )
            }
            Self::NonAscii { ch, pos } => {
                write!(
                    f,
                    "identifier contains non-ASCII character '{ch}' (U+{:04X}) at position {pos}",
                    *ch as u32
                )
            }
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
    /// Returns `IdentifierError` with a stable `META-IDENT-*` diagnostic code
    /// if the value is empty, too long, contains dangerous characters (NUL,
    /// control chars, newlines, path-traversal segments, non-ASCII), starts
    /// with a digit, or contains characters other than `[a-zA-Z0-9_]`.
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        if value.is_empty() {
            return Err(IdentifierError::Empty);
        }
        if value.len() > MAX_PG_IDENTIFIER_LEN {
            return Err(IdentifierError::TooLong { len: value.len() });
        }
        if value.contains('\0') {
            return Err(IdentifierError::ContainsNul);
        }
        if value.contains("..") {
            return Err(IdentifierError::PathTraversal);
        }
        let normalized = value.nfc().collect::<String>();
        if normalized != value {
            return Err(IdentifierError::NonNfc { normalized });
        }
        for (pos, ch) in value.chars().enumerate() {
            if ch == '\n' || ch == '\r' {
                return Err(IdentifierError::ContainsNewline { pos });
            }
            if ch.is_control() {
                return Err(IdentifierError::ContainsControlChar { ch, pos });
            }
            if !ch.is_ascii() {
                return Err(IdentifierError::NonAscii { ch, pos });
            }
            if pos == 0 && !ch.is_ascii_alphabetic() && ch != '_' {
                return Err(IdentifierError::InvalidStart { ch });
            }
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

    #[must_use]
    pub fn sql(&self) -> String {
        if needs_quoting_for_postgres(&self.value) {
            self.quoted()
        } else {
            self.value.clone()
        }
    }

    #[must_use]
    pub fn is_reserved_keyword(&self) -> bool {
        is_reserved_keyword(&self.value)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

fn needs_quoting_for_postgres(value: &str) -> bool {
    let is_simple_unquoted = value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_');

    !is_simple_unquoted || is_reserved_keyword(value)
}

fn is_reserved_keyword(value: &str) -> bool {
    matches!(
        value,
        "all"
            | "analyse"
            | "analyze"
            | "and"
            | "any"
            | "array"
            | "as"
            | "asc"
            | "asymmetric"
            | "authorization"
            | "between"
            | "binary"
            | "both"
            | "case"
            | "cast"
            | "check"
            | "collate"
            | "column"
            | "constraint"
            | "create"
            | "current_catalog"
            | "current_date"
            | "current_role"
            | "current_time"
            | "current_timestamp"
            | "current_user"
            | "default"
            | "deferrable"
            | "desc"
            | "distinct"
            | "do"
            | "else"
            | "end"
            | "except"
            | "false"
            | "fetch"
            | "for"
            | "foreign"
            | "from"
            | "grant"
            | "group"
            | "having"
            | "in"
            | "initially"
            | "intersect"
            | "into"
            | "leading"
            | "limit"
            | "localtime"
            | "localtimestamp"
            | "not"
            | "null"
            | "offset"
            | "on"
            | "only"
            | "or"
            | "order"
            | "placing"
            | "primary"
            | "references"
            | "returning"
            | "select"
            | "session_user"
            | "some"
            | "symmetric"
            | "table"
            | "then"
            | "to"
            | "trailing"
            | "true"
            | "union"
            | "unique"
            | "user"
            | "using"
            | "variadic"
            | "when"
            | "where"
            | "window"
            | "with"
    )
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
        let err = Identifier::new("").unwrap_err();
        assert_eq!(err, IdentifierError::Empty);
        assert_eq!(err.code(), "META-IDENT-EMPTY");
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(64);
        let err = Identifier::new(&long).unwrap_err();
        assert!(matches!(err, IdentifierError::TooLong { len: 64 }));
        assert_eq!(err.code(), "META-IDENT-TOO-LONG");
    }

    #[test]
    fn max_length_is_accepted() {
        let name = "a".repeat(63);
        assert!(Identifier::new(&name).is_ok());
    }

    #[test]
    fn rejects_nul_byte() {
        let err = Identifier::new("abc\0def").unwrap_err();
        assert_eq!(err, IdentifierError::ContainsNul);
        assert_eq!(err.code(), "META-IDENT-NUL");
    }

    #[test]
    fn rejects_path_traversal() {
        let err = Identifier::new("a..b").unwrap_err();
        assert_eq!(err, IdentifierError::PathTraversal);
        assert_eq!(err.code(), "META-IDENT-PATH-TRAVERSAL");
    }

    #[test]
    fn rejects_embedded_newline() {
        let err = Identifier::new("abc\ndef").unwrap_err();
        assert!(matches!(err, IdentifierError::ContainsNewline { pos: 3 }));
        assert_eq!(err.code(), "META-IDENT-NEWLINE");
    }

    #[test]
    fn rejects_embedded_carriage_return() {
        let err = Identifier::new("abc\rdef").unwrap_err();
        assert!(matches!(err, IdentifierError::ContainsNewline { pos: 3 }));
        assert_eq!(err.code(), "META-IDENT-NEWLINE");
    }

    #[test]
    fn rejects_control_chars() {
        let err = Identifier::new("abc\x07def").unwrap_err();
        assert!(matches!(
            err,
            IdentifierError::ContainsControlChar { ch: '\x07', pos: 3 }
        ));
        assert_eq!(err.code(), "META-IDENT-CONTROL-CHAR");
    }

    #[test]
    fn rejects_delete_control_char() {
        let err = Identifier::new("abc\x7Fdef").unwrap_err();
        assert!(matches!(
            err,
            IdentifierError::ContainsControlChar { ch: '\x7F', pos: 3 }
        ));
    }

    #[test]
    fn rejects_non_ascii() {
        let err = Identifier::new("naïve").unwrap_err();
        assert!(matches!(err, IdentifierError::NonAscii { ch: 'ï', pos: 2 }));
        assert_eq!(err.code(), "META-IDENT-NON-ASCII");
    }

    #[test]
    fn rejects_non_nfc_unicode() {
        // NFD form: 'e' + combining acute accent (U+0301)
        let nfd = "caf\u{0065}\u{0301}";
        let err = Identifier::new(nfd).unwrap_err();
        assert!(matches!(err, IdentifierError::NonNfc { .. }));
        assert_eq!(err.code(), "META-IDENT-NON-NFC");
    }

    #[test]
    fn rejects_digit_start() {
        let err = Identifier::new("1abc").unwrap_err();
        assert!(matches!(err, IdentifierError::InvalidStart { ch: '1' }));
        assert_eq!(err.code(), "META-IDENT-INVALID-START");
    }

    #[test]
    fn rejects_special_chars() {
        let err = Identifier::new("my-table").unwrap_err();
        assert!(matches!(err, IdentifierError::InvalidChar { ch: '-', .. }));
        assert_eq!(err.code(), "META-IDENT-INVALID-CHAR");
    }

    #[test]
    fn rejects_spaces() {
        let err = Identifier::new("my table").unwrap_err();
        assert!(matches!(err, IdentifierError::InvalidChar { ch: ' ', .. }));
    }

    #[test]
    fn reserved_word_allowed_but_detected() {
        let id = Identifier::new("order").unwrap();
        assert!(id.is_reserved_keyword());
        assert_eq!(id.sql(), "\"order\"");
    }

    #[test]
    fn non_reserved_word_not_detected() {
        let id = Identifier::new("customer").unwrap();
        assert!(!id.is_reserved_keyword());
        assert_eq!(id.sql(), "customer");
    }

    #[test]
    fn quoted_output() {
        let id = Identifier::new("customer").unwrap();
        assert_eq!(id.quoted(), "\"customer\"");
    }

    #[test]
    fn sql_unquoted_for_simple_identifier() {
        let id = Identifier::new("customer_01").unwrap();
        assert_eq!(id.sql(), "customer_01");
    }

    #[test]
    fn sql_quotes_reserved_keyword() {
        let id = Identifier::new("order").unwrap();
        assert_eq!(id.sql(), "\"order\"");
    }

    #[test]
    fn sql_quotes_mixed_case() {
        let id = Identifier::new("Customer").unwrap();
        assert_eq!(id.sql(), "\"Customer\"");
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

    #[test]
    fn error_codes_are_stable() {
        let cases: Vec<(IdentifierError, &str)> = vec![
            (IdentifierError::Empty, "META-IDENT-EMPTY"),
            (IdentifierError::TooLong { len: 64 }, "META-IDENT-TOO-LONG"),
            (IdentifierError::ContainsNul, "META-IDENT-NUL"),
            (IdentifierError::PathTraversal, "META-IDENT-PATH-TRAVERSAL"),
            (
                IdentifierError::ContainsNewline { pos: 0 },
                "META-IDENT-NEWLINE",
            ),
            (
                IdentifierError::NonNfc {
                    normalized: "café".into(),
                },
                "META-IDENT-NON-NFC",
            ),
            (
                IdentifierError::ContainsControlChar { ch: '\x07', pos: 0 },
                "META-IDENT-CONTROL-CHAR",
            ),
            (
                IdentifierError::NonAscii { ch: 'ü', pos: 0 },
                "META-IDENT-NON-ASCII",
            ),
            (
                IdentifierError::InvalidStart { ch: '1' },
                "META-IDENT-INVALID-START",
            ),
            (
                IdentifierError::InvalidChar { ch: '-', pos: 0 },
                "META-IDENT-INVALID-CHAR",
            ),
        ];
        for (err, expected_code) in cases {
            assert_eq!(err.code(), expected_code, "wrong code for {err:?}");
        }
    }

    #[test]
    fn error_display_messages() {
        assert!(IdentifierError::Empty.to_string().contains("empty"));
        assert!(IdentifierError::ContainsNul.to_string().contains("NUL"));
        assert!(IdentifierError::PathTraversal
            .to_string()
            .contains("path-traversal"));
        assert!(IdentifierError::NonNfc {
            normalized: "café".into()
        }
        .to_string()
        .contains("NFC-normalized"));
        assert!(IdentifierError::ContainsNewline { pos: 5 }
            .to_string()
            .contains("newline"));
        assert!(IdentifierError::ContainsControlChar { ch: '\x07', pos: 3 }
            .to_string()
            .contains("control character"));
        assert!(IdentifierError::NonAscii { ch: 'ü', pos: 2 }
            .to_string()
            .contains("non-ASCII"));
    }
}
