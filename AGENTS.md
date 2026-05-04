# AGENTS.md

Guidance for humans and AI assistants working in this repository.

This file is intentionally short on aspirations and long on **invariants**: the things that must hold for know-now to remain correct, reproducible, and safe. The product specification is in [`docs/PRD.md`](docs/PRD.md) — that is the source of truth. This document tells you how to *work* in the repo without violating it.

If anything here conflicts with the PRD, the PRD wins. Land an update to this file directly on `main` per the workflow in §7.

---

## 1. What this project is, in one paragraph

know-now is a local-first metadata-driven data platform generation engine. Users author YAML metadata under `metadata/`. The Rust engine validates it, builds a canonical project graph, projects it onto a versioned generator contract, runs deterministic generators, and atomically writes artifacts under `generated/`. A local Rust `axum` server and a TypeScript dashboard provide read-only stakeholder visibility. Custom logic is *declarative only* — through policy packs and restricted MiniJinja-based template packs. There is no plugin runtime, no telemetry, no required cloud service.

For positioning, personas, scope by phase, and detailed architecture, read [`docs/PRD.md`](docs/PRD.md).

---

## 2. Repository status

The repository is **Phase 1 complete; Phase 2A active** (PRD §23.1 exit criteria met at `190fe48`). The Rust workspace builds, the metadata parser enforces the YAML subset with source-aware diagnostics, the canonical ProjectGraph is constructed, generators produce PostgreSQL DDL and Markdown docs, output is provably deterministic, and the writer provides atomic promotion with path safety. Volatile run state is separated from generated output. `cargo xtask e2e phase1` passes all 14 integration tests.

Treat any code you find as authoritative over any doc that disagrees with it.

---

## 3. Where to find canonical information

When you have a question, look here before asking:

| Question | PRD section |
| -------- | ----------- |
| What is the product trying to be? | §1 Executive summary, §2 Positioning |
| Who is it for? | §3 Personas |
| What are the non-negotiable principles? | §4 Product principles |
| What is in scope for the current phase? | §6 Product phases, §22 Build sequence |
| What does the architecture look like? | §8 Core architecture |
| How is the project laid out? | §9 Project structure and ownership |
| What does the metadata model look like? | §10 Metadata model |
| What artifacts are generated? | §11 Artifact generation |
| What CLI commands exist? | §12 CLI product design |
| What dashboard/local API exists? | §13 Dashboard and local API |
| How do policy packs work? | §14 Policy packs |
| How do template packs / extensions work? | §15 Extension model |
| What are the functional requirements? | §16 |
| What are the non-functional requirements? | §17 |
| How is it released? | §20 CI/CD and release quality, §18 Distribution |
| What decisions have been made? | §24 Decisions table — and increasingly [`docs/adr/`](docs/adr/) |

Cross-reference PRD section IDs in commits, PRs, and issue descriptions where possible.

---

## 4. Architecture invariants — do not break these

These are derived from PRD §4 (principles), §8 (architecture), §9 (ownership), §17 (NFRs), and the architecture fitness tests listed in §17.6. Architecture fitness tests in CI must enforce them; treat the list below as the source of truth that those tests verify.

### 4.1 Layering and isolation

- **Generators never read YAML.** They consume only the validated `GeneratorContract` produced from the canonical `ProjectGraph`. (PRD §8.5, §8.7, §17.6)
- **Parser dependencies are isolated to `know_now_metadata`.** No other crate may depend on `serde-saphyr`, `marked-yaml`, or `saphyr-parser`. (PRD §10.2, NFR-S13, NFR-M8)
- **Generator crates have no direct cross-dependencies.** Adding a new generator must not require modifying an existing one. (NFR-M1, NFR-SC2)
- **Built-in generators do not write files directly.** All writes go through `know_now_writer`. (PRD §9.3, §17.6)
- **Template packs do not write files directly.** They return artifact descriptors; the writer enforces path safety, ownership markers, manual-edit detection, stale handling, and atomic promotion. (PRD §15.1.1, NFR-S17)
- **Policy validation cannot mutate metadata or the canonical graph.** Policy-provided defaults are applied through an explicit, traceable resolution step. (PRD §14.4, §17.6)
- **The local server's write endpoints are disabled unless explicitly enabled** and require both server-level opt-in and request-level confirmation. (PRD §13.2, §17.6)

### 4.2 Determinism and reproducibility

- Identical input must produce **byte-identical** deterministic generated output across supported OSes. (NFR-R1, GEN-007)
- The deterministic manifest (`generated/manifest.json`) must not contain timestamps, machine-local paths, usernames, or environment-specific data. (PRD §8.11, §9.5)
- Volatile run state (timestamps, durations, machine-local IDs) goes under `.knownow/` only. (PRD §8.11, NFR-R13)
- Independent generators may run concurrently, but final artifact ordering must remain deterministic. (NFR-P13)
- Incremental generation must produce the same final output as a full rebuild. (PRD §8.6, GEN-014, NFR-R10)

### 4.3 Ownership boundaries — never violate

| Path | Owner | Behavior |
| ---- | ----- | -------- |
| `metadata/` | User | Read-only in early phases. Never rewritten by default. |
| `custom/` | User | **Never written by know-now.** |
| `generated/` | Engine | Recreated atomically with manual-edit detection. |
| `.knownow/` | Engine state | Cache, manifests, issues, review state, audit log, locks, run logs. |
| `know-now.yml` | User | Created by `init`; modified only by explicit commands. |
| `know-now.lock` | Engine, user-reviewed | Records resolved versions and hashes. |

If your change risks writing into `custom/` or modifying `metadata/`, stop and re-read PRD §9.

### 4.4 Safety

- **No raw SQL string interpolation for identifiers or literals.** Use the typed DDL IR and dialect emitters. (PRD §8.9, NFR-S1)
- **Validate metadata identifiers before DDL IR construction.** (NFR-S2)
- **Never promote invalid generated output.** Failure preserves the previous artifact set. (PRD §11.7, NFR-R7, GEN-008, GEN-011)
- **No telemetry by default.** No outbound network calls in core commands. (PRD §4.4, CLI-007)
- **Local server binds to `127.0.0.1` by default.** Network exposure requires explicit flag and warning. (NFR-S14)
- **Support bundles redact secrets and sensitive environment details.** (NFR-S18)
- **Custom template packs cannot register native functions, filters, tests, or loaders, read environment variables, run processes, open network connections, access databases, or write files.** (PRD §15.1, NFR-S22..S25)

### 4.5 Banned dependencies

Until a dependency-policy review explicitly approves them, the following are **not allowed**:

- `serde_yaml`, `serde_yml`, or any unmaintained YAML stack.
- C/FFI YAML parsers.
- Any Rust crate that pulls in unrestricted `minijinja` features (custom functions, custom loaders, etc.) for *custom* template packs.
- Any TypeScript dependency added to `web/` without updating `pnpm-lock.yaml` via a frozen-lockfile-compatible workflow.
- Any dependency that would impose a non-permissive license on generated output.

See PRD §10.2 (parser rules), §15.1 (renderer rules), NFR-S5/S19 (audit posture).

---

## 5. Languages, toolchains, and tooling

| Layer | Language / tool | Notes |
| ----- | --------------- | ----- |
| CLI / engine | Rust (workspace under `crates/`) | Pinned via `rust-toolchain.toml` once added. |
| Local server | Rust `axum` | Same workspace. |
| Dashboard | TypeScript + React + Vite | Lives under `web/`. |
| Frontend package manager | **pnpm** | `web/pnpm-lock.yaml` committed; pnpm version pinned in `web/package.json` `packageManager` field. **Do not commit `package-lock.json`, `yarn.lock`, `bun.lock`, or `bun.lockb`.** |
| Repo task runner | `cargo xtask` | For release, fixture, benchmark, and maintenance tasks. |
| Issue tracker | beads (`br` CLI) | Prefix `know-now`. See §7.4. |
| Python | not used in product | If a maintenance script ever needs Python, use [`uv`](https://docs.astral.sh/uv/). Do not introduce `pip`, `poetry`, or `pipx` workflows. |

### 5.1 Common commands

These commands will exist once the workspace is scaffolded. Use them; do not invent your own.

```bash
# Rust
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check
cargo xtask check          # repo-wide check (fmt + lint + tests + fitness + fixtures)

# TypeScript dashboard
cd web
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```

If a command listed in the PRD does not yet exist in the codebase, do not silently substitute — open an issue (`br`) or scaffold the missing tooling under its own bead.

---

## 6. Coding conventions

### 6.1 Rust

- `cargo fmt` is the formatter; do not hand-format.
- `cargo clippy --all-targets --all-features -- -D warnings` must pass.
- Public APIs of library crates are typed and rustdoc-documented (NFR-M2). Do not expose `String` where a domain newtype would do.
- Errors use a typed error model with stable codes (PRD §8 diagnostics layer, §12.4).
- Avoid `unsafe` outside justified, reviewed, and tested locations.
- Keep crate boundaries clean: `know_now_codegen` defines traits; per-target generator crates implement them. Generator crates do not depend on each other.

### 6.2 TypeScript

- `tsconfig.json` runs in **strict mode** (NFR-M3). Recommended additional flags: `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride`.
- The dashboard consumes the local API only via the documented contract (or generated client). It does not reach into Rust internals.
- No `any` without a comment that explains why.

### 6.3 YAML metadata fixtures

- Write only the documented YAML subset (PRD §10.2): top-level mapping, scalar string keys, no anchors, aliases, merge keys, custom tags, include directives, multi-document files, or duplicate keys.
- Parser fixtures must include text-and-JSON snapshot tests for both happy and unsupported-feature paths (PRD §10.2, §20.1).

### 6.4 Comments and docstrings

- Default to writing **no** comments. Add one only when the *why* is non-obvious.
- Don't explain *what* the code does — well-named identifiers do that.
- Don't reference current tasks, fixes, or callers in comments — that belongs in the commit body or the bead thread and rots over time.

---

## 7. Workflow

### 7.1 Source of truth and decision flow

1. Product/scope/architecture questions: consult PRD.
2. If a previously-undecided architectural choice is needed, write an ADR. The ADR template lives at [`docs/adr/0000-template.md`](docs/adr/0000-template.md). The process is in [`docs/adr/README.md`](docs/adr/README.md).
3. If a *decided* item in PRD §24 is being revisited, update the PRD and add an ADR that supersedes the prior reasoning.
4. Implementation details should generally not require an ADR — only architectural choices do.

### 7.2 The Core Flywheel

Day-to-day work in this repo follows the **Core Flywheel** ([agent-flywheel.com/core-flywheel](https://agent-flywheel.com/core-flywheel)). It is a coordination loop built from three tools that together form one operating system:

| Tool | Role | Skip it and… |
| ---- | ---- | ------------ |
| `br` (beads) | Make work **explicit** as self-contained beads with dependencies and acceptance criteria. | …work stays vague and hidden in chat. |
| `bv` (beads viewer) | Make task choice **graph-aware** — use `bv --robot-plan` for graph context around the `br ready --json` ready set. | …agents pick by convenience, not impact. |
| `mcp-agent-mail` MCP | **Externalize coordination** — agent identities, threads anchored to bead IDs, file reservations, status signals. | …agents overlap, duplicate work, and lose history. |

The repeating cycle is: **route → claim → reserve → implement → fresh-eyes review → close → ask `br` what's ready**. Every step has a concrete action, and most of them are agent automatic once configured. Read the agent-flywheel page once at session start; this section is the project-specific instantiation.

Three properties drive the value:

1. Work is **explicit** (in beads, not in chat).
2. Coordination is **externalized** (in mcp-agent-mail, not in your head).
3. Task choice is **graph-aware** (ready set from `br ready --json`, graph context from `bv --robot-plan`, not vibes).

### 7.3 Working a bead — the full lifecycle

When you start work in this repo:

**1. Verify the graph and pick a bead**

```bash
bv --robot-triage    # sanity-check the graph; surfaces orphaned/blocked beads
br ready --json      # authoritative ready set; safe workaround for bv --robot-next
bv --robot-plan      # optional graph context and parallel tracks
```

Use `br ready --json` as the source of truth for claimable beads. As of `bv` v0.16.0, `bv --robot-next` treats `parent-child` rollup edges as blockers and can return "No actionable items available" even when `br ready --json` lists ready beads. Until that is fixed in the pinned viewer, do not use `bv --robot-next` for routing. If `br ready --json` is empty or its ready set doesn't match the user's request, stop and clarify before improvising.

**2. Open a thread and claim the bead**

Threads are anchored to the bead ID (`[know-now-NN]`). Use `mcp__mcp-agent-mail__macro_prepare_thread` to set the thread up, then post a Start message:

```
[know-now-42] Start: Postgres DDL emitter — primary key path
Claiming know-now-42. Reserving crates/know_now_gen_postgres/** and
fixtures/snapshots/postgres/**.
```

Then transition status with `br`:

```bash
br update know-now-42 --status in_progress
```

**Project rule:** when you set a Story to `in_progress`, also set its parent Epic to `in_progress` if it isn't already. Epic rollups stay honest.

**3. Reserve the files you'll touch**

Reserve before editing — a reservation is a coordination lock that prevents collision with other agents. Use `mcp__mcp-agent-mail__macro_file_reservation_cycle` (preferred) or `mcp__mcp-agent-mail__file_reservation_paths` directly. Reserve the **full** set of paths you expect to touch (source, tests, fixtures, snapshots, docs).

If a reservation conflicts with another agent's, do **not** force-release. Send a thread message asking how they want to coordinate, and pick another bead in the meantime via `br ready --json`.

`renew_file_reservations` extends the lock for long work. `release_file_reservations` releases on completion. `force_release_file_reservation` exists only for recovery (see §7.4).

**4. Implement**

Standard rules from the rest of this file apply: respect architecture invariants (§4), use the documented commands (§5.1), follow the coding conventions (§6), and never violate the hard "don'ts" (§8).

Post brief Progress messages in the thread at meaningful checkpoints (e.g., "list view wired", "switched approach to typed IR per ADR-0004"). Other agents (and the maintainer) consume these instead of chat scrollback.

**5. Fresh-eyes review on your own code**

Before any other review, re-read **all the code you wrote** with fresh eyes, looking for obvious bugs, off-by-ones, missed edge cases, leaks of internal types across crate boundaries, or violations of the architecture invariants. Fix anything you find. Repeat until you find nothing new.

**6. Stage your changes and hand off to the reviewer agent — gate before commit**

There are no feature branches in this repo (§7.6). All work lands directly on `main`, so the **review payload is the implementer's staged changes**, not a branch. After your fresh-eyes self-review:

```bash
# Stage exactly the paths you reserved — and nothing else.
git add <reserved-paths-only>
git diff --cached    # sanity-check what the reviewer will see
```

Then transition the bead and request review:

```bash
br update know-now-42 --status review
```

Post a Review-request in the bead's thread addressed to the reviewer agent (default identity `know-now-reviewer`), referencing the staged diff. **Stop.** Do not `git commit`. The implementer never runs `bmad-bmm-code-review` on their own code. Resume only after the reviewer has posted `Approved` (or after you've addressed their feedback, restaged, and they re-approve). See §7.8 for the full protocol, including the reviewer's own checklist.

**7. Commit directly to `main` and push**

After `Approved`:

```bash
git commit            # Conventional Commits — see §7.7 and docs/dev/commit-conventions.md
git pull --rebase     # keep main linear; replay your commit if others landed in the meantime
git push origin main
```

No branch, no PR. The reviewer-agent verdict in the thread *is* the review. Cite `Closes know-now-NN` and the relevant PRD section(s) in the commit body.

**8. Close the bead and release reservations**

```bash
br close know-now-42       # or `br update --status closed` per project policy
```

Release reservations:

```
mcp__mcp-agent-mail__release_file_reservations
```

Post the completion message:

```
[know-now-42] Completed
Commit <sha> pushed to main. Reservations released. Next: br ready --json.
```

**9. Ask `br` what's ready**

```bash
br ready --json
```

Then loop back to step 2.

### 7.4 mcp-agent-mail in this repo

`mcp-agent-mail` provides agent identities, contacts, threads, file reservations, and a thread-bound message bus. The high-leverage entry points are the **macro_** tools — prefer them over composing primitive calls by hand.

**Session start (every fresh agent):**

```
mcp__mcp-agent-mail__macro_start_session   # registers identity + fetches inbox
mcp__mcp-agent-mail__ensure_project        # confirm project context
mcp__mcp-agent-mail__health_check          # surface server / config issues early
mcp__mcp-agent-mail__fetch_inbox           # see anything addressed to you
```

If an agent identity for you doesn't exist yet, use `register_agent` / `create_agent_identity`.

**Discovery and contacts:**

- `whois` — look up an agent.
- `list_contacts` — see who you can reach.
- `request_contact` / `respond_contact` / `set_contact_policy` — handshake / control inbound.
- `macro_contact_handshake` — convenience for the full request → respond round-trip.

**Threads (one thread per bead):**

- Always anchor a thread to the bead ID. Subject prefix `[know-now-NN]`.
- `macro_prepare_thread` — bootstrap the thread, link participants.
- `send_message` / `reply_message` — Start / Progress / Decision / Completion messages.
- `fetch_inbox`, `mark_message_read`, `acknowledge_message` — keep your inbox clean.
- `search_messages` — recover prior thread context.
- `summarize_thread` — produce a compact summary for long threads (use before handoff or at completion).

**File reservations:**

- `macro_file_reservation_cycle` — reserve → renew → release flow for an editing session.
- `file_reservation_paths` — reserve specific paths.
- `renew_file_reservations` — extend during long work.
- `release_file_reservations` — required at bead close (and when you abandon a bead).
- `force_release_file_reservation` — **only** for recovery from an agent that disappeared. Send a thread message first; never use this to win a conflict.

**Pre-commit guard:**

The repo can install `mcp-agent-mail`'s pre-commit guard (`install_precommit_guard`) to catch commits with unreleased reservations or stale thread state. If installed locally, do not bypass it with `--no-verify`. Removal is `uninstall_precommit_guard`.

**Recovery: when an agent disappears mid-bead**

1. Read the thread for the last Progress message — `summarize_thread` if long.
2. If the bead is still claimed by the missing agent, ask the maintainer (or coordinate via the thread) before taking it.
3. After agreement, `force_release_file_reservation` for the abandoned paths, claim the bead, reserve the paths, and continue from the code state recorded in the thread.

### 7.5 Standard marching orders for new agent sessions

There are two roles: **implementer** and **reviewer**. Each has its own marching orders. A given session is exactly one role — never both — because the BMAD code-review gate (§7.8) requires a *different agent* to review the implementer's code.

**Implementer marching orders** — use this prompt verbatim when launching an implementer agent.

```
You are an IMPLEMENTER agent on the know-now repository. First read
ALL of AGENTS.md and README.md carefully. Use code investigation mode
to understand the architecture and the architecture invariants in
AGENTS.md §4. Register with mcp-agent-mail (macro_start_session) under
an implementer identity and introduce yourself. Check the agent-mail
inbox promptly. Proceed meticulously with assigned beads systematically
— claim via br, reserve files via mcp-agent-mail, implement, do a
fresh-eyes review on your own code, then STOP and post a Review-request
message in the bead's thread addressed to the reviewer agent
(default identity: `know-now-reviewer`). DO NOT run
`bmad-bmm-code-review` on your own code. DO NOT git commit until the
reviewer has posted `Approved` in the thread. Track progress via beads
(br update) and per-bead agent-mail threads ([know-now-NN]). Don't get
stuck in communication purgatory. When unsure what to work on next,
run `bv --robot-triage`, then use `br ready --json` as the authoritative
ready set. Use `bv --robot-plan` for graph context. Use ultrathink.
```

**Reviewer marching orders** — use this prompt verbatim when launching a reviewer agent.

```
You are the REVIEWER agent on the know-now repository. Your sole job
is to perform `bmad-bmm-code-review` on changes produced by IMPLEMENTER
agents and to act as the BMAD commit gate (AGENTS.md §7.8). First read
ALL of AGENTS.md, README.md, and docs/PRD.md §4 / §8 / §9 / §17.6 so
the architecture invariants are fresh. Register with mcp-agent-mail
(macro_start_session) under the reviewer identity (default:
`know-now-reviewer`). Subscribe to / poll the inbox for Review-request
messages on `[know-now-NN]` threads. For each request:
  1. Acknowledge in the thread.
  2. Run `bmad-bmm-code-review` against the implementer's STAGED changes
     (`git diff --cached`). There are no feature branches in this repo;
     the staged set is the review payload (AGENTS.md §7.6, §7.8).
  3. Cross-check the change against AGENTS.md §4 invariants and §8
     hard "don'ts", and against the PRD section(s) the implementer
     cited.
  4. Post findings as a Review-feedback message — concrete file:line
     references, severity-tagged, with a clear final verdict:
     `Approved`, `Changes requested`, or `Blocked`.
  5. On `Changes requested`, wait for the implementer to address and
     re-request review. Loop until `Approved` or `Blocked`.
You do NOT implement, commit, or claim implementation beads. You do NOT
review your own work. If asked to implement, decline and refer to the
implementer-agent marching orders. Use ultrathink.
```

If you launch multiple agents, **stagger starts by 30+ seconds** to avoid the thundering-herd problem on the agent-mail registration / `bv` triage.

### 7.6 Branching policy — trunk-based on `main`

**All work happens directly on `main`. There are no feature branches and no pull requests.**

- Conflict prevention is the job of mcp-agent-mail file reservations (§7.4), not of branch isolation. Reserve before you edit; release at bead close.
- The "review payload" is the implementer's **staged changes** in the working tree (`git diff --cached`), reviewed by a different agent via mcp-agent-mail (§7.8). The reviewer-agent verdict replaces traditional PR review.
- After `Approved`, the implementer commits directly on `main`, runs `git pull --rebase` to absorb anything that landed in the meantime, and pushes.
- Keep history linear: rebase, never merge. Conventional Commits (§7.7) carry the metadata that PR descriptions usually carry.
- Force-pushing `main` is **forbidden** without explicit maintainer approval — there is no upstream to recover from. Ordinary `git push origin main` only.
- Each implementer agent stages **only the paths it reserved**. This keeps the review payload to one bead's worth of change even when other implementers have unstaged work-in-progress in the same working tree.
- A short-lived branch is permissible **only** for genuine maintainer operations that cannot be performed on `main` directly (e.g., bisecting an incident). Do not use branches as a workflow shortcut.

### 7.7 Commits

- Commit messages follow **Conventional Commits**. See [`docs/dev/commit-conventions.md`](docs/dev/commit-conventions.md).
- **Do not commit before the BMAD code-review gate (§7.8) has produced an `Approved` verdict from the reviewer agent.**
- Each commit cites the relevant PRD section(s) and the beads issue ID (`Closes know-now-NN`) in the body or footer.
- Generated-output changes that affect compatibility fixtures must include the fixture diff classification in the **commit body** (since there is no PR description): `expected formatting change`, `metadata schema change`, `generator behavior change`, `policy default change`, `bug fix`, or `breaking change` (PRD §20.2).
- Commits that touch parser, writer, generator contract, lockfile schema, renderer profile, or local API contract require an ADR or PRD update if behavior changes. Land the ADR/PRD update **before** or **in the same commit** as the surface change — not after.
- Stage only the paths you reserved (§7.6). One bead per commit. Do not bundle unrelated changes from other agents' in-progress work that happen to be present in the working tree.
- `git pull --rebase` before pushing. `git push origin main` only — never `--force` (§7.6).

### 7.8 Code-review gate (BMAD) — performed by a different agent via mcp-agent-mail

This repo follows the maintainer's **BMAD Dev Story Workflow** as the gate before any commit. The non-negotiable rule:

> **The implementer never reviews their own code.** `bmad-bmm-code-review` is run by a *separate* agent identity (the reviewer agent), and the handoff happens through mcp-agent-mail.

#### Roles

| Role | Identity (default) | Permitted actions |
| ---- | ------------------ | ----------------- |
| Implementer | one identity per implementer session (e.g. `know-now-impl-<n>`) | Claim beads, reserve files, implement, fresh-eyes self-review, request review, address feedback, commit **only after `Approved`**. |
| Reviewer | `know-now-reviewer` (singleton, but the maintainer may run multiple) | Run `bmad-bmm-code-review`, cross-check against AGENTS.md §4 / §8 and the cited PRD sections, post findings, issue verdicts. **Does not implement.** |

The reviewer must not be the same agent process as the implementer. The reviewer should be a freshly-launched session whose context is uncontaminated by the implementation. If you are unsure whether you qualify as "different enough," you don't — request a separate reviewer agent.

#### Protocol (one round)

1. **Implementer** finishes implementation, runs the fresh-eyes self-review (§7.3 step 5), then sets the bead status to `review`:
   ```bash
   br update know-now-NN --status review
   ```
2. **Implementer** stages exactly the paths they reserved (`git add <reserved-paths>`), verifies with `git diff --cached`, and posts a Review-request in the bead's thread:
   ```
   [know-now-NN] Review request
   To: know-now-reviewer
   Base: main @ <HEAD-sha>
   Staged paths:
     - crates/know_now_gen_postgres/src/...
     - fixtures/snapshots/postgres/...
   PRD refs: §<X.Y>, <NFR / GEN / META id>
   Notes: <anything the reviewer should pay particular attention to>
   ```
   Use `mcp__mcp-agent-mail__send_message` (or `reply_message` if continuing the thread). File reservations stay in place — do **not** release them until after `Approved` and commit. There is no branch to point at; the staged diff is the review payload.
3. **Implementer** stops. No `git commit`. No further edits to staged paths until the reviewer responds. Other agents' unstaged work-in-progress in the same working tree is irrelevant to this review — the reviewer reads `git diff --cached` only.
4. **Reviewer** acknowledges (`acknowledge_message`) and runs `bmad-bmm-code-review` against the implementer's staged changes (`git diff --cached`). The reviewer may also inspect the working tree for context, but the review is scoped to the staged set.
5. **Reviewer** cross-checks: architecture invariants (§4), hard "don'ts" (§8), the PRD section(s) the implementer cited, fixture diffs, lockfile / contract / API / renderer-profile compatibility, dependency policy.
6. **Reviewer** posts a Review-feedback message in the same thread with concrete `file:line` references, severity-tagged findings, and **exactly one** of these final verdicts:
   - `Approved` — implementer may commit.
   - `Changes requested` — implementer addresses feedback, re-requests review, loop.
   - `Blocked` — work cannot proceed without intervention from the maintainer (e.g., requires an ADR, a PRD update, or scope change). Implementer escalates to the maintainer in the thread.

#### Loop

`Changes requested` cycles back to step 1, scoped to the requested changes. Each round is a new Review-request message in the same thread. Reviewer acknowledges, re-checks the diff, and re-issues a verdict. There is no implicit cap, but if a bead requires more than three review rounds, that is a signal the bead is underspecified — escalate to the maintainer (Core Flywheel "step back into bead space").

#### After `Approved`

1. **Implementer** commits the staged changes per §7.7 (Conventional Commits, no `--no-verify`, no `--force`).
2. **Implementer** runs `git pull --rebase` to absorb anything that landed on `main` in the meantime, then `git push origin main`. There is no PR — the reviewer-agent verdict in the thread is the review of record.
3. **Implementer** closes the bead (`br close know-now-NN`) and releases file reservations (`release_file_reservations`).
4. **Implementer** posts a Completion message in the thread (`[know-now-NN] Completed`) including the resulting commit SHA.
5. **Implementer** asks `br ready --json` for the next ready bead.

#### Recovery

- **Reviewer unavailable:** if no reviewer agent is registered or the inbox is unattended, the implementer asks the maintainer in the thread before doing anything else. **Do not** self-approve.
- **Reviewer disappears mid-review:** the maintainer launches a fresh reviewer agent, who reads the thread (`summarize_thread` if long), runs the review, and issues a verdict.
- **Implementer disappears mid-revision:** another implementer agent picks up the bead per §7.4 recovery, and the existing thread continues.

#### Hard rules (also see §8)

- The implementer MUST NOT run `bmad-bmm-code-review` on their own code.
- The implementer MUST NOT commit before an `Approved` verdict from a different agent.
- The reviewer MUST NOT review code they wrote (across any role).
- Bypassing the gate (including via `--no-verify` or by misrepresenting agent identity) is a hard "don't."

If `bmad-bmm-code-review` or mcp-agent-mail isn't available in your environment, **ask the maintainer** rather than skipping the gate.

### 7.9 Reviews

- Architectural changes (parser, writer, generator contract, lockfile schema, renderer profile, local API) require explicit review by a **maintainer agent or human** in addition to the reviewer-agent gate (§7.8). Address them in the bead's thread before staging the change.
- Generator output changes require the fixture-diff classification in the **commit body** (PRD §20.2 — see §7.7).
- Dependency additions/upgrades require `cargo deny check` to remain green; run it locally before requesting review.

---

## 8. Hard "don'ts" — failure modes that have been pre-decided

If your change requires any of these, stop and find a different approach.

- ❌ Reading raw YAML in a generator crate.
- ❌ Adding a YAML parser dependency to any crate other than `know_now_metadata`.
- ❌ Writing files outside `know_now_writer`.
- ❌ Writing into `custom/` from anywhere in the engine.
- ❌ Modifying files under `metadata/` from non-explicit commands (`validate`, `check`, `generate`, `diff`, `issues`, `serve`, `doctor`, `schema`, `version`, `config inspect` — PRD §10.11).
- ❌ Putting timestamps, hostnames, or random IDs in `generated/manifest.json`.
- ❌ Embedding a self-hash of a generated file's content into that file (manifest hashes only — PRD §11.8).
- ❌ Adding telemetry, analytics, or background network calls to any core command.
- ❌ Binding the local server to `0.0.0.0` by default.
- ❌ Allowing query-string tokens for normal API requests after the launch-token exchange.
- ❌ Allowing custom template packs to register native MiniJinja functions, filters, tests, or loaders.
- ❌ Allowing dynamic include paths in templates.
- ❌ Allowing template includes to escape the pack root.
- ❌ Skipping `cargo deny check`, license checks, or architecture fitness tests in CI.
- ❌ Committing generated dashboard build artifacts under `web/dist/` to the repo (they are release assets, not source).
- ❌ Skipping git hooks (`--no-verify`, `--no-gpg-sign`) without an explicit, recorded reason.
- ❌ Committing before the reviewer agent has posted `Approved` in the bead thread (§7.8).
- ❌ Reviewing your own code with `bmad-bmm-code-review`. The reviewer must be a *different agent identity* (§7.8).
- ❌ Self-approving (issuing your own `Approved` verdict on your own implementation), or impersonating the reviewer identity.
- ❌ Editing files without a current reservation through mcp-agent-mail (§7.3 step 3, §7.4).
- ❌ Using `force_release_file_reservation` to win a conflict instead of coordinating in the bead's thread.
- ❌ Picking the next bead by convenience instead of asking `br ready --json`.
- ❌ Using ad-hoc chat / scrollback for coordination that belongs in a `[know-now-NN]` agent-mail thread.
- ❌ Creating a feature branch. All work lands on `main`; conflicts are prevented by mcp-agent-mail reservations (§7.6).
- ❌ Opening a pull request. There are no PRs in this repo; the reviewer-agent verdict in the bead thread is the review of record (§7.6, §7.8).
- ❌ Force-pushing `main` (`git push --force` / `--force-with-lease`) without explicit maintainer approval. There is no upstream branch to recover from.
- ❌ `git add -A` / `git add .` / `git commit -a`. Stage only the paths you reserved (§7.6, §7.7).
- ❌ Bundling another agent's unstaged work-in-progress into your commit by accident.

---

## 9. Adding new things — quick recipes

### 9.1 Add a new built-in generator

1. Decide if the generator contract needs to grow. If yes: ADR + version bump for the contract + compatibility fixture diff.
2. Create `know_now_gen_<target>` crate. It depends on `know_now_codegen`, `know_now_contract`, and `know_now_ir` — never on parser crates or other generator crates.
3. Implement the generator trait. Return artifact descriptors only; do not write files.
4. Register the generator in the capability registry with declared contract versions, target dialects, supported types, validation gates, and known unsupported constructs (PRD §8.4).
5. Add snapshot tests, property tests where applicable, and at least one compatibility fixture (PRD §20.1, §20.2).
6. Update the compatibility matrix and release notes.

### 9.2 Add a new policy rule

1. Add the rule to the default policy pack with a stable code, severity, rationale, and remediation example (PRD §14.7).
2. Make sure evaluation is side-effect-free; defaults flow through the explicit resolution step.
3. Add tests covering valid, warning, error, and blocking outcomes.
4. Document the code under `docs/user/policy/` (once that directory is created).

### 9.3 Add a new ADR

1. Copy [`docs/adr/0000-template.md`](docs/adr/0000-template.md) to `docs/adr/<NNNN>-<slug>.md` with the next available number.
2. Status starts `Proposed`. After agreement: `Accepted`. If superseded: `Superseded by ADR-NNNN`.
3. Link from [`docs/adr/README.md`](docs/adr/README.md) so the index stays current.
4. Reference the ADR from the relevant PRD section and from the commit body that introduces the ADR.

### 9.4 Change the metadata schema, generator contract, lockfile schema, renderer profile, or local API contract

These are versioned compatibility surfaces. Any change requires:

- An ADR (proposed → accepted) or PRD update.
- A version bump on the affected surface (see [`docs/dev/versioning.md`](docs/dev/versioning.md)).
- A migration note in the changelog.
- Compatibility fixture updates, with the fixture diff classification recorded in the commit body (§7.7, PRD §20.2).

---

## 10. AI assistant notes

If you are an AI agent working in this repo:

- **Run the marching orders in §7.5 at session start.** That sequence (read AGENTS.md + README.md, register with mcp-agent-mail, check inbox, ask `bv` for the next bead) is non-optional.
- **Use `br` + `bv` + mcp-agent-mail as one system** (the Core Flywheel — §7.2). Skipping any of them breaks coordination for everyone else.
- **One bead at a time, one thread per bead.** Subject line `[know-now-NN]`. Post Start / Progress / Completion messages so other agents and the maintainer don't have to read your chat scrollback.
- **Reserve files before editing.** If you hit a reservation conflict, send a thread message and pick another bead — never `force_release_file_reservation` to win a conflict.
- **Read the PRD before proposing architectural changes.** It is long but comprehensive. Use the section index in §3 to jump.
- **Don't fabricate behavior.** If the codebase doesn't yet have a CLI, don't invent commands; check what exists.
- **Don't bypass the workflow.** `br` + `bv` + mcp-agent-mail + BMAD review-before-commit are how this repo operates. If a tool isn't available locally, ask the user — don't skip the step.
- **Don't commit before the reviewer agent posts `Approved`** in the bead's thread (§7.8). The implementer never runs `bmad-bmm-code-review` on their own code — request it from a different agent (default identity `know-now-reviewer`) via mcp-agent-mail. Setting the bead to `review` + posting a Review-request + stopping is the correct end-of-implementation state.
- **One role per session.** If you were launched as an implementer, do not also act as the reviewer (and vice versa). If asked to switch roles mid-session, decline and ask for a fresh session under the other identity.
- **Work directly on `main`.** No feature branches, no PRs (§7.6). Stage only the paths you reserved, request review, then commit straight to `main` after `Approved`. Conflict prevention is mcp-agent-mail's job, not git's.
- **When in doubt, prefer reading more code over guessing.** Check the architecture invariants in §4 before suggesting refactors.
- **Confirm before destructive actions** (force-push, branch deletion, dropping fixtures, removing crates, regenerating large fixture sets, `force_release_file_reservation`) even when permissions allow.
- **Use the PRD section IDs in your output** so reviewers can verify your reasoning quickly.
- **Re-read AGENTS.md after any context compaction.** If you're still confused after rereading, request a fresh session rather than improvising.

---

## 11. Pointers

- Product spec: [`docs/PRD.md`](docs/PRD.md)
- Documentation index: [`docs/README.md`](docs/README.md)
- ADR process and index: [`docs/adr/README.md`](docs/adr/README.md)
- Repo layout: [`docs/dev/repo-layout.md`](docs/dev/repo-layout.md)
- Versioning policy: [`docs/dev/versioning.md`](docs/dev/versioning.md)
- Commit conventions: [`docs/dev/commit-conventions.md`](docs/dev/commit-conventions.md)
- Human contributor guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Core Flywheel reference: [agent-flywheel.com/core-flywheel](https://agent-flywheel.com/core-flywheel)

<!-- bv-agent-instructions-v2 -->

---

## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`) for issue tracking and [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) (`bv`) for graph-aware triage. Issues are stored in `.beads/` and tracked in git.

### Using bv as an AI sidecar

bv is a graph-aware triage engine for Beads projects (.beads/beads.jsonl). Instead of parsing JSONL or hallucinating graph traversal, use robot flags for deterministic, dependency-aware outputs with precomputed metrics (PageRank, betweenness, critical path, cycles, HITS, eigenvector, k-core).

**Scope boundary:** bv handles *what to work on* (triage, priority, planning). `br` handles creating, modifying, and closing beads.

**CRITICAL: Use ONLY --robot-* flags. Bare bv launches an interactive TUI that blocks your session.**

#### The Workflow: Start With Triage

**`bv --robot-triage` is your single entry point.** It returns everything you need in one call:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores, reasons, unblock info
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `project_health`: status/type/priority distributions, graph metrics
- `commands`: copy-paste shell commands for next steps

```bash
bv --robot-triage        # THE MEGA-COMMAND: start here
bv --robot-next          # Minimal: just the single top pick + claim command

# Token-optimized output (TOON) for lower LLM context usage:
bv --robot-triage --format toon
```

#### Other bv Commands

| Command | Returns |
|---------|---------|
| `--robot-plan` | Parallel execution tracks with unblocks lists |
| `--robot-priority` | Priority misalignment detection with confidence |
| `--robot-insights` | Full metrics: PageRank, betweenness, HITS, eigenvector, critical path, cycles, k-core |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, label suggestions, cycle breaks |
| `--robot-diff --diff-since <ref>` | Changes since ref: new/closed/modified issues |
| `--robot-graph [--graph-format=json\|dot\|mermaid]` | Dependency graph export |

#### Scoping & Filtering

```bash
bv --robot-plan --label backend              # Scope to label's subgraph
bv --robot-insights --as-of HEAD~30          # Historical point-in-time
bv --recipe actionable --robot-plan          # Pre-filter: ready to work (no blockers)
bv --recipe high-impact --robot-triage       # Pre-filter: top PageRank scores
```

### br Commands for Issue Management

```bash
br ready              # Show issues ready to work (no blockers)
br list --status=open # All open issues
br show <id>          # Full issue details with dependencies
br create --title="..." --type=task --priority=2
br update <id> --status=in_progress
br close <id> --reason="Completed"
br close <id1> <id2>  # Close multiple issues at once
br sync --flush-only  # Export DB to JSONL
```

### Workflow Pattern

1. **Triage**: Run `bv --robot-triage` to find the highest-impact actionable work
2. **Claim**: Use `br update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id>`
5. **Sync**: Always run `br sync --flush-only` at session end

### Key Concepts

- **Dependencies**: Issues can block other issues. `br ready` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers 0-4, not words)
- **Types**: task, bug, feature, epic, chore, docs, question
- **Blocking**: `br dep add <issue> <depends-on>` to add dependencies

### Session Protocol

```bash
git status              # Check what changed
git add <files>         # Stage code changes
br sync --flush-only    # Export beads changes to JSONL
git commit -m "..."     # Commit everything
git push                # Push to remote
```

<!-- end-bv-agent-instructions -->
