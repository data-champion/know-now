# Installation

## Prerequisites

- **Operating system:** Linux (x86_64), macOS (Apple Silicon), or Windows (x86_64).
- **For binary install:** [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) (recommended) or `curl`/`tar` for direct download.
- **For source build:** Rust toolchain (stable). See [`rust-toolchain.toml`](../../rust-toolchain.toml) for the pinned version.

## Recommended: cargo-binstall

```bash
cargo binstall know-now
```

This downloads a prebuilt binary for your platform. Fast (seconds, not minutes) and avoids compiling from source.

If you don't have `cargo-binstall`:

```bash
cargo install cargo-binstall
cargo binstall know-now
```

## Direct binary download

Download the release archive for your platform from the [GitHub releases page](https://github.com/data-champion/know-now/releases) and extract it to a directory on your `PATH`:

```bash
# Linux (x86_64)
curl -fsSL https://github.com/data-champion/know-now/releases/latest/download/know-now-x86_64-unknown-linux-gnu.tar.gz \
  | tar xz -C /usr/local/bin

# macOS (Apple Silicon)
curl -fsSL https://github.com/data-champion/know-now/releases/latest/download/know-now-aarch64-apple-darwin.tar.gz \
  | tar xz -C /usr/local/bin
```

On Windows, download the `.zip` archive and add the extracted directory to your `PATH`.

## Source build (fallback)

```bash
cargo install --locked know-now
```

This builds from source and takes several minutes. Use this only if prebuilt binaries are not available for your platform.

Do not omit `--locked` — without it, dependency versions are not pinned.

## Verification

After installation, verify that the CLI is available and shows the expected version:

```bash
know-now version
```

Expected output:

```
know-now 0.1.0
```

For JSON output with schema versions:

```bash
know-now version --format json
```

## Upgrading

Re-run the same install command with the new version. `cargo binstall` will replace the existing binary. For direct downloads, replace the binary in your `PATH`.

## CI installation

For CI pipelines, see the templates under [`examples/ci/`](../../examples/ci/). These pin a specific version and use the direct binary download path for speed.

## Uninstalling

If installed via `cargo binstall` or `cargo install`:

```bash
cargo uninstall know-now
```

If installed via direct download, remove the binary from your `PATH`.
