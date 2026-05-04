# Portability Tests

Cross-platform tests verifying NFR-PO1..PO3 (PRD §17.7).

## Coverage

| NFR | What | Status |
|-----|------|--------|
| PO1 | Linux + macOS + Windows native | CI matrix in `.github/workflows/ci.yml` |
| PO2 | Path-handling (spaces, Unicode, emoji, separators) | `know_now_writer::manifest` tests + future writer tests |
| PO3 | Line endings (LF), UTF-8 (no BOM) | `know_now_writer::manifest` tests |

## WSL

WSL is best-effort, not a separate platform target. Windows native
coverage is what matters. If a test fails under WSL but passes on
native Windows, the native result is authoritative.

## Running portability tests locally

```bash
cargo test --workspace
```

All portability assertions are part of the standard test suite. The CI
matrix runs them on Linux, macOS, and Windows.

## Adding portability tests

When the writer crate gains file-writing functionality, add tests here
that verify:
- Path separators are normalized to forward slashes in generated content
- Generated files use LF line endings on all platforms
- No BOM in UTF-8 output
- Non-ASCII identifiers produce byte-identical output across platforms
- Spaces, emoji, and Unicode (NFC/NFD) in paths work correctly
