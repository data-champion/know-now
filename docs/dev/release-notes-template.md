# Release Notes Template

Use this template when drafting release notes for a new version.

---

## know-now vX.Y.Z

### Highlights

- (bullet list of user-facing changes)

### Breaking Changes

- (list breaking changes; cite the relevant ADR or PRD update)
- (include fixture-diff classification per §20.2: expected formatting
  change, metadata schema change, generator behavior change, policy
  default change, bug fix, or breaking change)

### Bug Fixes

- (list bug fixes with issue/bead references)

### Installation

**Pre-built binaries (recommended):**

```bash
cargo binstall know_now_cli
```

**Source build:**

```bash
cargo install --locked know_now_cli --version X.Y.Z
```

**Direct download:** see the [release assets](https://github.com/data-champion/know-now/releases/tag/vX.Y.Z) for Linux (gnu + musl), macOS (x86_64 + aarch64), and Windows binaries with SHA-256 checksums.

### Compatibility

- Metadata schema version: (version)
- Generator contract version: (version)
- Lockfile schema version: (version)

### Security

- (list any CVEs addressed in this release)
- (list any dependency upgrades that address security advisories)
- If no security-relevant changes: "No security-relevant changes in this release."

### Full Changelog

(link to GitHub compare view between previous and current tag)
