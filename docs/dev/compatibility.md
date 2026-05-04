# Compatibility matrix

Tracks supported platforms, targets, and tool versions for know-now.

PRD refs: §20.4, §17.7 (NFR-PO1..PO3).

## OS / architecture

| Platform | Architecture | CI status | Notes |
|----------|-------------|-----------|-------|
| Linux | x86_64 | Tested (`ubuntu-latest`) | Primary development platform |
| macOS | aarch64 (Apple Silicon) | Tested (`macos-latest`) | |
| Windows | x86_64 | Tested (`windows-latest`) | Native only; WSL is best-effort |

All three platforms run the full `cargo test --workspace` suite in CI (`.github/workflows/ci.yml` Rust matrix job). Portability-specific assertions (LF line endings, no BOM, non-ASCII roundtrip, deterministic output) are verified on every platform.

## Rust toolchain

Pinned via `rust-toolchain.toml`. See `rust-toolchain.toml` for the current channel and components.

## Frontend toolchain

| Tool | Version constraint | Notes |
|------|--------------------|-------|
| Node.js | LTS | Required for dashboard |
| pnpm | Pinned in `web/package.json` `packageManager` | ADR-0005 |
| TypeScript | Strict mode | `tsconfig.json` |

## Metadata schema

| Version | Status | Notes |
|---------|--------|-------|
| 1.0 | Current | Only supported version in Phase 3 |

## Generator contract

| Version | Status | Notes |
|---------|--------|-------|
| 1.0 | Current | Stable since Phase 2a |

## Renderer profiles

| Profile | Version | Status | Notes |
|---------|---------|--------|-------|
| know-now-minijinja-v1 | 1 | Current | Strict mode, fuel-limited |

## PostgreSQL targets

| Version | Status | Notes |
|---------|--------|-------|
| 16 | Floor / tested | Minimum supported |
| 17 | Tested | |
| 18 | Tested | Latest supported |

## dbt adapter modes

| Adapter | Version | Status | Notes |
|---------|---------|--------|-------|
| dbt-postgres | 1.7+ | Tested | Primary adapter |

## Dashboard browser support

| Browser | Version | Status | Notes |
|---------|---------|--------|-------|
| Chrome | Latest 2 major | Supported | Primary target |
| Firefox | Latest 2 major | Supported | |
| Safari | Latest 2 major | Best-effort | |
| Edge | Latest 2 major | Supported | Chromium-based |

## Release classification

When a release changes generated output, the commit body must include a
classification per PRD §20.2:

- **expected-formatting**: whitespace, indentation, comment placement
- **metadata-schema**: changes to metadata YAML structure
- **generator-behavior**: different SQL/dbt/docs output for same input
- **policy-default**: new or changed default policy rules
- **bug-fix**: corrects previously incorrect output
- **breaking**: incompatible change requiring user action
