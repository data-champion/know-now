use regex::Regex;
use std::sync::LazyLock;

static PATTERNS: LazyLock<Vec<RedactionPattern>> = LazyLock::new(|| {
    vec![
        RedactionPattern::new("aws_access_key", r"(?:AKIA|ABIA|ACCA|ASIA)[A-Z0-9]{16}"),
        RedactionPattern::new(
            "aws_secret_key",
            r"(?i)(?:aws_secret_access_key|secret_key)\s*[=:]\s*\S{20,}",
        ),
        RedactionPattern::new("high_entropy", r"[A-Za-z0-9+/]{40,}={0,2}"),
        RedactionPattern::new(
            "pem_block",
            r"-----BEGIN [A-Z ]+-----[\s\S]*?-----END [A-Z ]+-----",
        ),
        RedactionPattern::new(
            "jwt_token",
            r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+",
        ),
        RedactionPattern::new(
            "env_secret",
            r"(?i)(?:password|secret|token|api_key|apikey)\s*[=:]\s*\S+",
        ),
    ]
});

const REDACTED: &str = "[REDACTED]";

struct RedactionPattern {
    _name: &'static str,
    regex: Regex,
}

impl RedactionPattern {
    fn new(name: &'static str, pattern: &str) -> Self {
        Self {
            _name: name,
            regex: Regex::new(pattern).unwrap_or_else(|e| {
                panic!("invalid redaction pattern `{name}`: {e}");
            }),
        }
    }
}

pub fn redact(input: &str) -> String {
    let mut result = input.to_owned();
    for pattern in PATTERNS.iter() {
        result = pattern.regex.replace_all(&result, REDACTED).into_owned();
    }
    result
}

pub fn contains_secret(input: &str) -> bool {
    PATTERNS.iter().any(|p| p.regex.is_match(input))
}

pub fn redact_home_path(input: &str, home: &str) -> String {
    input.replace(home, "$HOME")
}

pub fn redact_hostname(input: &str, hostname: &str) -> String {
    input.replace(hostname, "[HOSTNAME]")
}

pub fn redact_username(input: &str, username: &str) -> String {
    input.replace(username, "[USER]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_aws_access_key() {
        let input = "key=AKIAIOSFODNN7EXAMPLE";
        assert!(contains_secret(input));
        let redacted = redact(input);
        assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(redacted.contains(REDACTED));
    }

    #[test]
    fn redacts_high_entropy_string() {
        let input = "token: dGhpcyBpcyBhIHZlcnkgbG9uZyBiYXNlNjQgZW5jb2RlZCBzdHJpbmc=";
        assert!(contains_secret(input));
    }

    #[test]
    fn redacts_pem_block() {
        let input = "-----BEGIN RSA PRIVATE KEY-----\nMIIBog==\n-----END RSA PRIVATE KEY-----";
        assert!(contains_secret(input));
        let redacted = redact(input);
        assert!(!redacted.contains("MIIBog"));
    }

    #[test]
    fn redacts_jwt_token() {
        let input = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123_DEF";
        assert!(contains_secret(input));
        let redacted = redact(input);
        assert!(!redacted.contains("eyJhbGciOiJIUzI1NiJ9"));
    }

    #[test]
    fn redacts_env_secret() {
        let input = "PASSWORD=hunter2";
        assert!(contains_secret(input));
        let redacted = redact(input);
        assert!(!redacted.contains("hunter2"));
    }

    #[test]
    fn passes_clean_text() {
        let input = "this is a normal log message about entity customer_123";
        assert!(!contains_secret(input));
        assert_eq!(redact(input), input);
    }

    #[test]
    fn redacts_home_path() {
        let result = redact_home_path("/home/user/.config/know-now", "/home/user");
        assert_eq!(result, "$HOME/.config/know-now");
    }

    #[test]
    fn redacts_hostname() {
        let result = redact_hostname("connecting to myhost.local:8080", "myhost.local");
        assert_eq!(result, "connecting to [HOSTNAME]:8080");
    }
}
