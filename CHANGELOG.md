# Changelog

All notable changes to know-now are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
for the engine version; other compatibility surfaces have their own version
cadences described in [docs/dev/versioning.md](docs/dev/versioning.md).

To generate draft release notes from commits:
```sh
cargo xtask release notes --range <previous-tag>..HEAD
```

Each release entry should link to the corresponding
[compatibility matrix](docs/dev/compatibility.md) snapshot when a versioned
surface changed.

## [Unreleased]

### Features

- `know-now generate` — full pipeline: parse, validate, graph, contract,
  plan, generate, write, manifest (Phase 2b)
- `know-now diff` — compare metadata against baseline with `--impact` and
  `--scan-custom` flags
- `know-now explain` — trace generated artifacts back to metadata origins
- `know-now issues` — track and manage deprecation issues
- `know-now review export` — Markdown review packs for stakeholder review
- `know-now support` — sanitized diagnostic bundles
- `know-now id backfill --apply` — deterministic stable ID insertion into
  metadata YAML with automatic backup
- Append-only audit log under `.knownow/audit.log`
- Approved-version catalog library (`know_now_catalog`) with drift
  classifier
- Custom declarative template packs (Phase 3, in progress)
- Local axum server (Phase 3, in progress)

### Tooling

- `cargo xtask release notes` — generate grouped release notes from
  conventional commits
- `cargo xtask release check-commits` — CI gate for breaking change
  footer compliance
- Compatibility matrix populated in `docs/dev/compatibility.md`

### Documentation

- Troubleshooting guide with all diagnostic codes
- Doctor, explain, and support bundle user guides
