# Commit conventions

This repository uses **Conventional Commits**. The format is short, machine-parseable, and lets us derive changelogs and release notes without bespoke tooling. This file is the canonical reference for the conventions used here.

## Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

- **type** — one of the types below. Required.
- **scope** — usually a crate name or area. Optional but encouraged.
- **subject** — imperative mood, lowercase, no trailing period, ≤ 72 characters.
- **body** — explains *why*, not *what*. Wrap at ~72 characters. Optional.
- **footer** — refs, breaking-change markers, beads issue IDs. Optional.

## Types

| Type | Use for |
| ---- | ------- |
| `feat` | A new feature or capability visible to users (CLI command, dashboard view, generator output, policy rule). |
| `fix` | A bug fix. |
| `docs` | Documentation-only changes (PRD, README, AGENTS.md, ADRs, dev docs, generated-doc templates that produce identical output). |
| `refactor` | Code change that neither fixes a bug nor adds a feature. Generated output must be unchanged. |
| `perf` | Performance improvement. Include benchmark numbers in the body where possible. |
| `test` | Adding or fixing tests. |
| `build` | Build system, Cargo workspace, dependency, or `cargo deny` configuration changes. |
| `ci` | CI configuration only. |
| `chore` | Routine maintenance not covered above (e.g., updating issue templates). |
| `revert` | Reverting a previous commit. Reference the original commit SHA in the footer. |

If you're not sure which type fits, prefer the one that best describes the **intent** of the change.

## Scopes

Use a crate name (`gen-postgres`, `metadata`, `writer`, `lock`, `server`, `cli`, `templates`, `policy`, `audit`, `cache`, `diff`, `identity`, `validate`, `codegen`, `ir`, `contract`, `diagnostics`, `core`, `xtask`) or an area (`web`, `dashboard`, `docs`, `prd`, `adr`, `ci`, `release`, `fixtures`, `examples`).

Drop the `know_now_` prefix for readability; `gen-postgres` is clearer than `know_now_gen_postgres`.

## Examples

```
feat(gen-postgres): emit primary key constraints

Implements GEN-001 acceptance #1. The DDL IR exposes an explicit
PrimaryKey node so dialect emitters can position the constraint inline
or in a trailing block.

Closes know-now-42
Refs: docs/PRD.md §11.2
```

```
fix(writer): preserve previous output when promotion fails

A failure during artifact validation could leave the staging directory
half-promoted. The writer now treats promotion as all-or-nothing.

Refs: docs/PRD.md §11.7, GEN-008, GEN-011
```

```
docs(adr): add ADR-0004 for restricted template renderer profile
```

```
refactor(metadata): extract source-span index into a typed wrapper

No change to deterministic generated output (verified locally and via
fixture diff).
```

```
build(deps): pin pnpm to 9.12.3 in web/package.json

Refs: docs/dev/versioning.md, NFR-I9
```

## Breaking changes

Breaking changes affect a versioned compatibility surface (engine, metadata schema, generator contract, lockfile schema, local API, renderer profile). Mark them clearly:

```
feat(contract)!: rename `attributes` to `fields` in generator contract v2

BREAKING CHANGE: generator contract bumped to 2.0. Built-in generators
have been updated; downstream template packs must update their
references. Migration: see CHANGELOG and docs/dev/versioning.md.

Refs: docs/PRD.md §8.3, ADR-NNNN
```

The `!` after the type/scope, **and** a `BREAKING CHANGE:` footer, are both required so changelog tools and human readers cannot miss it.

## Footer fields

| Footer | Use |
| ------ | --- |
| `Closes know-now-NN` | The commit closes the named beads issue. |
| `Refs: …` | Loose references (PRD sections, ADRs, related issues). |
| `BREAKING CHANGE: …` | Required for breaking changes. |
| `Co-Authored-By: …` | Co-authorship attribution. |
| `Reviewed-by: …` | Optional. |

## Pre-commit / pre-merge expectations

- Do **not** skip git hooks (`--no-verify`, `--no-gpg-sign`) unless the maintainer has explicitly approved it for this commit.
- Run a fresh-eyes self-review before committing (see [`AGENTS.md`](../../AGENTS.md) §7.3 step 5).
- Generated-output changes that affect compatibility fixtures need the fixture diff classification in the **commit body** (PRD §20.2 — there is no PR description in this repo; see [`../../AGENTS.md`](../../AGENTS.md) §7.6). The commit message itself should still be a single, focused, conventional commit.

## Why these conventions

- A predictable header format lets us auto-generate changelogs and release notes (PRD §19.1, §20.3).
- A short, well-named type makes `git log` archaeology faster.
- Explicit `BREAKING CHANGE` markers reduce the risk of shipping a contract bump as a routine release.

## Anti-patterns

- ❌ `wip`, `update`, `fixes`, `more changes`.
- ❌ Title case subjects (`Add: ...`).
- ❌ Trailing periods in the subject.
- ❌ Combining a `feat` and a `refactor` in one commit.
- ❌ Logging the *what* in the body when the diff already shows it.
- ❌ Referencing the change in code comments instead of the commit body.
