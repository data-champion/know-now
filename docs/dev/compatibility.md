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

## Remaining sections

The following sections will be populated by bsf.9 (compatibility matrix maintenance bead):

- Metadata schema version
- Generator contract version
- Renderer profile versions
- PostgreSQL target versions
- dbt adapter modes
- Dashboard browser support
