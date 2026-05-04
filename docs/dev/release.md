# Release Process

## Triggering a release

Releases are triggered by pushing a semver tag from `main`:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The tag must point to a commit on `main`. Tags from forks or non-main
branches are rejected by the release workflow.

## What the release produces

| Artifact | Platform |
|----------|----------|
| `know-now-x86_64-unknown-linux-gnu.tar.gz` | Linux x86_64 (glibc) |
| `know-now-x86_64-unknown-linux-musl.tar.gz` | Linux x86_64 (static) |
| `know-now-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon |
| `know-now-x86_64-apple-darwin.tar.gz` | macOS Intel |
| `know-now-x86_64-pc-windows-msvc.zip` | Windows x86_64 |

Each archive has a paired `.sha256` checksum file.

## Installation methods

### Direct binary download

Download the archive for your platform from the GitHub release page and
extract it.

### cargo-binstall

```bash
cargo binstall know_now_cli
```

The `[package.metadata.binstall]` section in `crates/know_now_cli/Cargo.toml`
provides the download URL template.

### Source build fallback

```bash
cargo install --locked know_now_cli --version <version>
```

This builds from source and works on any platform with a Rust toolchain.

## Verification

The release workflow runs `cargo install --locked` on Linux, macOS, and
Windows to verify the source-build path works. The binary is executed with
`know-now --help` as a smoke test.

## Checksums

Every binary archive has a SHA-256 checksum published alongside it.
Verify after download:

```bash
sha256sum -c know-now-x86_64-unknown-linux-gnu.tar.gz.sha256
```

## Attestations and SBOMs

Not yet implemented. These are "where practical" per PRD §20.3 and will
be added when the project reaches Phase 2B+ maturity. The decision is
documented here rather than in an ADR since it's a deferral, not an
architectural choice.
