# Contributing to know-now

Thanks for considering a contribution. This guide focuses on **how** to contribute. The **what** lives in [`docs/PRD.md`](docs/PRD.md), and the invariants and conventions every contributor (human or AI) must respect live in [`AGENTS.md`](AGENTS.md). Read both before opening a non-trivial PR.

---

## 1. Before you start

1. Read [`AGENTS.md`](AGENTS.md) §4 (architecture invariants) and §8 (hard "don'ts"). These are non-negotiable.
2. Skim the PRD section relevant to your change. The PRD section index is in [`AGENTS.md`](AGENTS.md#3-where-to-find-canonical-information).
3. Check open issues (`br q` / `br ready --json`) and the [decisions table](docs/PRD.md) (PRD §24) so you don't duplicate work or revisit a decided question.
4. For anything beyond a typo or a docs fix, open a beads issue first describing scope, intent, and PRD section. Linking the issue from your PR makes review faster.

---

## 2. Local setup

The repository is **pre-implementation** as of this writing. Once the workspace is scaffolded, the expected setup is:

### Rust

```bash
# Pinned via rust-toolchain.toml (will be added in Phase 1).
rustup show
cargo build
cargo test
```

Recommended once-per-machine tooling:

```bash
cargo install cargo-deny           # supply-chain checks
cargo install cargo-nextest        # faster test runner (optional)
cargo install cargo-binstall       # optional, for installing release binaries
```

### TypeScript dashboard

```bash
cd web
# Use the pinned pnpm version from web/package.json's `packageManager` field.
# CI activates this version explicitly; locally you can use Corepack or install pnpm directly.
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
```

Do **not** commit `package-lock.json`, `yarn.lock`, `bun.lock`, or `bun.lockb` under `web/`. Only `pnpm-lock.yaml` is allowed.

### Toolchain versions

- **Rust:** pinned by `rust-toolchain.toml` once added.
- **Node.js:** must satisfy the `engines.node` constraint in `web/package.json` (kept in sync with the active Vite version).
- **pnpm:** pinned in `web/package.json`'s `packageManager` field. Do **not** rely on Corepack being bundled with the active Node.js version — install or activate the pinned pnpm version explicitly.
- **Python:** the product does not use Python. If you write a maintenance script that needs Python, use [`uv`](https://docs.astral.sh/uv/).

---

## 3. Make your change

### 3.1 Pick a workflow

This repo replaces standard branch + PR practice with three coordinated conventions:

- **Trunk-based on `main`** — no feature branches, no PRs. All work lands directly on `main`; mcp-agent-mail file reservations prevent collisions. See [`AGENTS.md`](AGENTS.md) §7.6.
- **Core Flywheel** (`br` + `bv` + `mcp-agent-mail`) for routing, claiming, file reservations, and per-bead threads. See [`AGENTS.md`](AGENTS.md) §7.2–§7.4.
- **Beads (`br`)** for issue tracking. When you claim a Story (`in_progress`), set its parent Epic to `in_progress` if it isn't already.
- **BMAD Dev Story Workflow with cross-agent review.** After implementing a story, **stage only the paths you reserved** (`git add <reserved-paths>`), set the bead status to `review`, post a Review-request in the bead's `[know-now-NN]` thread to a *different* agent (default identity `know-now-reviewer`), and **stop**. The reviewer agent runs `bmad-bmm-code-review` against `git diff --cached` and posts `Approved` / `Changes requested` / `Blocked`. **Do not** `git commit` until the reviewer posts `Approved`. Implementers never review their own code. See [`AGENTS.md`](AGENTS.md) §7.8 for the full protocol.

If those tools aren't available in your environment, ask the maintainer rather than skipping the workflow.

### 3.2 Code style

- Rust: `cargo fmt` is the only formatter. `cargo clippy --all-targets --all-features -- -D warnings` must pass.
- TypeScript: strict mode. Lint and format as configured under `web/`.
- YAML metadata fixtures: follow the documented authoring subset (PRD §10.2).
- Prefer typed domain newtypes over stringly-typed APIs.
- Default to no comments. Add one only when the *why* is non-obvious.

### 3.3 Tests

| Category | Required when |
| -------- | ------------- |
| Unit / integration | always for behavior changes |
| Snapshot tests | adding or modifying generator output |
| Property-based tests | metadata validation, diff classification, deterministic ordering |
| Compatibility fixtures | adding/changing a generator, schema, contract, or renderer profile |
| Architecture fitness tests | adding a crate or shifting layering |
| `cargo deny check` | adding/upgrading any dependency |

See PRD §20.1 for the full CI matrix.

### 3.4 Performance

If your change affects a performance-sensitive path (parsing, validation, generation, dashboard API), include benchmark numbers and check against PRD §17.1 targets. Regressions >20% fail CI unless explicitly approved (NFR-P10).

### 3.5 Determinism

If your change touches generation output, run it twice and confirm byte-identical output. Manifest output must not embed timestamps, hostnames, or machine paths (PRD §8.11).

---

## 4. Commits

Use **Conventional Commits**. The full guide is at [`docs/dev/commit-conventions.md`](docs/dev/commit-conventions.md).

Quick form:

```
<type>(<scope>): <subject>

<body — what changed and why, not how>

<footer — refs, breaking changes, beads ids>
```

Examples:

```
feat(gen-postgres): emit primary key constraints

Closes know-now-42. Implements GEN-001 acceptance #1.
```

```
fix(writer): preserve previous output when promotion fails

The previous behavior partially-promoted the staging directory if a single
artifact failed validation, violating GEN-008/GEN-011.

Refs: docs/PRD.md §11.7
```

Notes:

- Do **not** skip git hooks (`--no-verify`, `--no-gpg-sign`) unless explicitly asked.
- Do **not** amend or rebase commits that have already been pushed to `main`.
- Do **not** force-push `main`. There is no upstream branch to recover from.
- Stage only the paths you reserved. Avoid `git add -A` / `git add .` / `git commit -a` — they pull in other agents' unstaged work-in-progress.
- The maintainer's BMAD workflow blocks committing before reviewer-agent approval — respect it.

---

## 5. Landing your change

There are no pull requests in this repo. The reviewer-agent verdict in the bead's `[know-now-NN]` thread is the review of record (see [`AGENTS.md`](AGENTS.md) §7.6, §7.8). After `Approved`, you push directly to `main`.

The end-to-end sequence is:

1. **Stage** exactly the paths you reserved: `git add <reserved-paths>`. Verify with `git diff --cached`.
2. **Request review.** Set the bead to `review` and post a Review-request in the bead thread (template in [`AGENTS.md`](AGENTS.md) §7.8). Stop. Do not commit.
3. **Address feedback.** On `Changes requested`, fix, restage, re-request review in the same thread.
4. **Commit and push** after `Approved`:
   ```bash
   git commit            # Conventional Commits — see docs/dev/commit-conventions.md
   git pull --rebase     # absorb anything that landed on main in the meantime
   git push origin main  # never --force
   ```
5. **Close the bead** (`br close know-now-NN`), release file reservations, and post a Completion message including the resulting commit SHA.
6. **Pick the next bead** with `br ready --json`. Use `bv --robot-plan` for graph context while `bv --robot-next` treats `parent-child` rollup edges as blockers.

Each commit must include in its body or footer:

1. **Summary** — what changed, in 1–3 bullets (the commit subject covers the headline).
2. **PRD reference** — the section(s) your change implements or affects (e.g., "Refs: docs/PRD.md §11.2, GEN-001 acceptance #1, NFR-S1").
3. **Beads issue** — `Closes know-now-NN`.
4. **Compatibility impact** — for changes affecting the metadata schema, generator contract, lockfile schema, renderer profile, or local API contract: describe the version bump and migration story. See [`docs/dev/versioning.md`](docs/dev/versioning.md).
5. **Generated-output diff classification** — for any change that alters compatibility-fixture output, classify as `expected formatting change`, `metadata schema change`, `generator behavior change`, `policy default change`, `bug fix`, or `breaking change` (PRD §20.2).
6. **Test plan** — what you ran and what passed.

CI runs on every push to `main` and must remain green:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` (or `cargo nextest run`)
- `cargo deny check`
- Architecture fitness tests
- Snapshot tests, property tests, compatibility fixtures
- `pnpm typecheck && pnpm test && pnpm build` for `web/` changes
- Cross-platform matrix where applicable (PRD §20.1)

If CI breaks `main`, the implementer who pushed is responsible for the fix-forward (or revert) commit, coordinated in the relevant bead thread.

The reviewer agent (and the maintainer) pay particular attention to:

- Architecture invariants ([`AGENTS.md`](AGENTS.md) §4).
- Determinism and ownership-boundary rules.
- Generator-contract / lockfile / API compatibility.
- Dependency licenses and supply-chain posture.

---

## 6. Architecture decisions

Significant architectural choices are recorded as ADRs under [`docs/adr/`](docs/adr/). The process is in [`docs/adr/README.md`](docs/adr/README.md). If your change introduces or revisits an architectural decision, include the ADR in the same PR (or a preceding one).

---

## 7. Reporting bugs and security issues

- **Bugs:** open a beads issue with reproduction steps, expected vs actual behavior, and `know-now version` output.
- **Security:** do **not** open public issues for vulnerabilities. Contact the maintainer directly. (Public security policy will be added when the project enters public release.)

---

## 8. License

License selection is an open decision (PRD §24). Do not introduce dependencies whose license would compromise the future choice or contaminate generated output. When in doubt, ask before adding the dependency.

---

## 9. Pointers

- Product spec: [`docs/PRD.md`](docs/PRD.md)
- Agent / contributor invariants: [`AGENTS.md`](AGENTS.md)
- ADR process and index: [`docs/adr/README.md`](docs/adr/README.md)
- Repo layout: [`docs/dev/repo-layout.md`](docs/dev/repo-layout.md)
- Versioning policy: [`docs/dev/versioning.md`](docs/dev/versioning.md)
- Commit conventions: [`docs/dev/commit-conventions.md`](docs/dev/commit-conventions.md)
