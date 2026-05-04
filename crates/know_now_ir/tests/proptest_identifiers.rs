//! Property-based tests for the `Identifier` validation logic.
//!
//! Verifies that structurally valid identifiers are always accepted and that
//! identifiers containing special characters are always rejected.

use know_now_ir::identifier::Identifier;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate strings that must always be valid identifiers: start with an ASCII
/// letter, followed by up to 62 ASCII alphanumeric or underscore characters
/// (total length 1-63).
fn valid_identifier_strategy() -> impl Strategy<Value = String> {
    // First char: a-zA-Z or underscore
    // Remaining: a-zA-Z0-9_ (0 to 62 chars, to stay within 63 limit)
    (
        prop::sample::select(
            ('a'..='z')
                .chain('A'..='Z')
                .chain(std::iter::once('_'))
                .collect::<Vec<char>>(),
        ),
        prop::collection::vec(
            prop::sample::select(
                ('a'..='z')
                    .chain('A'..='Z')
                    .chain('0'..='9')
                    .chain(std::iter::once('_'))
                    .collect::<Vec<char>>(),
            ),
            0..=62,
        ),
    )
        .prop_map(|(first, rest)| {
            let mut s = String::with_capacity(1 + rest.len());
            s.push(first);
            s.extend(rest);
            s
        })
        .prop_filter("must be <= 63 chars", |s| s.len() <= 63)
}

/// Generate strings that must always be rejected: contain at least one
/// special character that is not alphanumeric and not underscore.
fn invalid_special_char_strategy() -> impl Strategy<Value = String> {
    // Build a string: valid prefix + at least one bad char + valid suffix
    let bad_chars = prop::sample::select(vec![
        '-', '.', ' ', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '+', '=', '{', '}', '[',
        ']', '|', '\\', '/', '?', '<', '>', ',', ';', ':', '\'', '"', '`', '~',
    ]);
    (
        prop::string::string_regex("[a-z]{1,5}").expect("valid regex"),
        bad_chars,
        prop::string::string_regex("[a-z]{0,5}").expect("valid regex"),
    )
        .prop_map(|(prefix, bad, suffix)| format!("{prefix}{bad}{suffix}"))
}

/// Generate strings that start with a digit -- always invalid.
fn digit_start_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[0-9][a-z_]{0,10}")
        .expect("valid regex")
        .prop_filter("must not be empty", |s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Any ASCII alphanumeric+underscore string starting with a letter (or _)
    /// and at most 63 chars should always be a valid identifier.
    #[test]
    fn valid_identifiers_always_accepted(name in valid_identifier_strategy()) {
        let id = Identifier::new(&name);
        prop_assert!(
            id.is_ok(),
            "Expected valid identifier, got error for {:?}: {:?}",
            name,
            id.unwrap_err()
        );
        let id = id.unwrap();
        prop_assert_eq!(id.as_str(), name.as_str());
    }

    /// Any string containing a non-alphanumeric non-underscore ASCII
    /// character should always be rejected.
    #[test]
    fn special_chars_always_rejected(name in invalid_special_char_strategy()) {
        let result = Identifier::new(&name);
        prop_assert!(
            result.is_err(),
            "Expected rejection for {:?}, but got Ok",
            name
        );
    }

    /// Strings that start with a digit should always be rejected with
    /// InvalidStart.
    #[test]
    fn digit_start_always_rejected(name in digit_start_strategy()) {
        let result = Identifier::new(&name);
        prop_assert!(result.is_err(), "Expected rejection for digit-start {:?}", name);
    }

    /// The empty string is always rejected.
    #[test]
    fn empty_string_rejected(_seed in 0u32..1) {
        let result = Identifier::new("");
        prop_assert!(result.is_err());
    }

    /// Strings longer than 63 bytes should always be rejected.
    #[test]
    fn too_long_always_rejected(extra_len in 1usize..100) {
        let name = "a".repeat(63 + extra_len);
        let result = Identifier::new(&name);
        prop_assert!(result.is_err(), "Expected rejection for length {}", name.len());
    }

    /// Valid identifiers round-trip through Display and back.
    #[test]
    fn display_roundtrip(name in valid_identifier_strategy()) {
        let id = Identifier::new(&name).expect("should be valid");
        let displayed = id.to_string();
        let id2 = Identifier::new(&displayed).expect("displayed form should also be valid");
        prop_assert_eq!(id.as_str(), id2.as_str());
    }
}
