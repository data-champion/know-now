# Portability Tests

Cross-platform tests verifying NFR-PO1..PO3 (PRD §17.7).

## NFR-Portability trace

| NFR | Requirement | Implementing bead | Test coverage |
|-----|-------------|-------------------|---------------|
| PO1 | CLI works on Linux, macOS, Windows native | 42e.9 AC #1, #7 | CI matrix (`ubuntu-latest`, `macos-latest`, `windows-latest`) in `.github/workflows/ci.yml` |
| PO2 | Path handling: spaces, Unicode, platform separators | 42e.9 AC #2 | `know_now_writer::manifest` tests (space + Unicode roundtrip), `know_now_core::determinism` tests |
| PO3 | Generated output: stable LF line endings, UTF-8, no BOM | 42e.9 ACs #3, #5, #6 | `manifest_json_uses_lf_line_endings`, `manifest_json_no_bom`, `non_ascii_identifiers_roundtrip` in `know_now_writer::manifest`; `byte_identical_across_two_runs` LF assertion in `know_now_core::determinism` |

## WSL

WSL is best-effort, not a separate platform target (42e.9 AC #7). Windows native
coverage is what matters. If a test fails under WSL but passes on
native Windows, the native result is authoritative.

## Running portability tests locally

```bash
cargo test --workspace
```

All portability assertions are part of the standard test suite. The CI
matrix runs them on Linux, macOS, and Windows.

## Adding portability tests

When adding new generated output paths, verify:
- Path separators are normalized to forward slashes in generated content
- Generated files use LF line endings on all platforms
- No BOM in UTF-8 output
- Non-ASCII identifiers produce byte-identical output across platforms
- Spaces, emoji, and Unicode (NFC/NFD) in paths work correctly
