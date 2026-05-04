# Secret Redaction

know-now redacts secrets from log events, support bundles, and audit output to prevent sensitive data leakage (NFR-S18, NFR-S12, NFR-S9).

## Crate location

Redaction primitives live in `know_now_audit::redaction` (PRD §8.2). The diagnostics layer consumes these; it does not own them.

## Public API

| Function | Purpose |
|---|---|
| `redact(input)` | Replace all matched patterns with `[REDACTED]` |
| `contains_secret(input)` | Check if input matches any secret pattern |
| `redact_home_path(input, home)` | Replace home directory path with `$HOME` |
| `redact_hostname(input, hostname)` | Replace hostname with `[HOSTNAME]` |
| `redact_username(input, username)` | Replace username with `[USER]` |

## Pattern catalog

| Pattern name | Matches | Example |
|---|---|---|
| `aws_access_key` | `AKIA\|ABIA\|ACCA\|ASIA` followed by 16 uppercase alphanumeric chars | `AKIAIOSFODNN7EXAMPLE` |
| `aws_secret_key` | `aws_secret_access_key` or `secret_key` followed by 20+ non-whitespace chars | `aws_secret_access_key=wJalrXUtnFEMI/…` |
| `high_entropy` | Base64-like strings of 40+ characters | `dGhpcyBpcyBhIHZlcnkgbG9uZy…` |
| `pem_block` | `-----BEGIN … -----` through `-----END … -----` | PEM-encoded keys and certificates |
| `jwt_token` | Three dot-separated base64url segments starting with `eyJ` | `eyJhbGci….eyJzdWIi….abc123` |
| `env_secret` | `password\|secret\|token\|api_key\|apikey` followed by `=` or `:` and a value | `PASSWORD=hunter2` |

## Adding a new pattern

1. Add a `RedactionPattern::new(name, regex)` entry to the `PATTERNS` vector in `crates/know_now_audit/src/redaction.rs`.
2. Add a unit test in the same file covering detection and redaction.
3. Update this table.
4. Consider whether the pattern should be included in the redaction fuzz test suite (know-now-bsf.2).

## Invariants

- `know_now_audit` must not depend on `know_now_diagnostics` (avoids a dependency cycle; enforced by architecture fitness test).
- Redaction runs before log event serialization.
- Support bundles (`.knownow/runs/<run_id>.json`) include the structured event stream only after redaction.
- The deterministic manifest (`generated/manifest.json`) never contains log content, timestamps, hostnames, or absolute paths (AGENTS.md §4.2).

## Test affordances

- Unit tests in `know_now_audit::redaction::tests` cover each pattern type.
- The `assert_no_secrets!` macro in `know_now_diagnostics::test_support` runs `contains_secret()` over captured events.
- Redaction fuzz testing (know-now-bsf.2) injects 10,000+ randomly generated secret-shaped strings and asserts zero leakage.
