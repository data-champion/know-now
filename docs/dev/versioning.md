# Versioning policy

know-now exposes **multiple versioned compatibility surfaces**. Treating "the version" as a single number (e.g., the engine SemVer) is wrong here: each surface evolves on its own cadence and has its own compatibility implications. This document is the maintainer's reference for which version to bump when.

The PRD authoritatively defines what each surface is for:

| Surface | PRD reference | What it controls |
| ------- | ------------- | ---------------- |
| Engine version | §8.11 manifest, §17.8, §20.3 | The CLI/binary as a whole. SemVer. |
| Metadata schema version | §10 (METADATA), META-007 | The user-authored YAML format. |
| Generator contract version | §8.3, §8.4, §8.11, §17.6 | The validated graph shape passed to generators (built-in and template). |
| Local API contract version | §13.4, NFR-I8 | `/api/v1/...` shape consumed by the dashboard / generated TS client. |
| Lockfile schema version | §9.5 | `know-now.lock` structure. |
| Renderer profile version | §15.1, §15.1.1, ADR-0004 | `know-now-minijinja-v1` and successors. |
| Policy pack version | §14, ADMIN-001 | Per-pack version recorded in lockfile + manifest. |
| Template pack version | §15.1, EXT-004 | Per-pack version recorded in lockfile + manifest. |

Each lockfile entry and manifest record makes these versions visible per project.

## Principles

1. **Compatibility surfaces are public.** Anything in the table above is a public contract. Internal types and crate-private interfaces are not.
2. **Bump the smallest surface that changed.** A bug fix in the PostgreSQL emitter does not bump the metadata schema. A new metadata field doesn't bump the renderer profile.
3. **Breaking ≠ minor.** Breaking changes get a major bump on their own surface. There is no "rolling beta" lane.
4. **Migration is a feature.** Every breaking change to a versioned surface ships with a migration note in the changelog and (where applicable) compatibility-fixture diffs (PRD §20.2).
5. **Lockfile records resolved versions.** `know-now.lock` records the exact resolved engine, generator-contract, policy-pack, template-pack, and renderer-profile versions plus content hashes (PRD §9.5). The manifest records the lockfile hash used.

## Engine version (SemVer)

- `MAJOR.MINOR.PATCH`.
- `PATCH`: bug fixes, no behavior changes that affect deterministic output.
- `MINOR`: new commands, new features, additive changes that don't break existing projects.
- `MAJOR`: anything that changes default behavior in a way existing projects must adapt to.

Pre-1.0: `0.x.y` — `MINOR` acts as `MAJOR` for breaking changes; `PATCH` acts as `MINOR`. We will keep the engine on `0.x` until Phase 2A is publicly released and consulting validation completes (PRD §5.2).

## Metadata schema version

- Strings of the form `"1.0"` (declared in YAML as `version: "1.0"`).
- The major component is breaking; the minor component is additive (new optional fields, new enum members documented as optional).
- Unsupported major versions are rejected (META-007).
- Adding a new attribute or field that is **optional** and not load-bearing for existing generators: `MINOR` (new optional field).
- Renaming, removing, or changing the type of an existing field: `MAJOR`.
- Migrations between metadata schema majors are deterministic and documented in release notes.

## Generator contract version

- Strings of the form `"1.0"`.
- Major bump = breaking change to the structure passed to generators (built-in or template). Generators must declare which contract versions they accept (PRD §8.4).
- Adding new optional fields to the contract is a `MINOR` bump.
- A `MAJOR` contract bump requires:
  - An ADR if the change is design-shaping.
  - Compatibility-fixture diffs.
  - `know-now lock update --accept-contract-upgrade` for projects that need to opt in.

## Local API contract version

- The local API is namespaced under `/api/v1`. The path version is the major.
- `MINOR` evolution is **additive only** within the major path: new fields, new endpoints. Existing fields and endpoints keep their shape.
- Removing or repurposing a field is a `MAJOR` bump and requires a new path version (`/api/v2`). The old version may be kept for one release for migration, then removed.
- `/api/v1/version` reports engine version, API contract version, dashboard asset version, and compatibility status (PRD §13.4).
- The bundled dashboard and the server API contract are tested together in CI (NFR-M6).

## Lockfile schema version

- `know-now.lock` declares its own schema version.
- Migrations between lockfile schemas are deterministic and reported in release notes (PRD §9.5).
- A breaking change to lockfile shape requires a major engine bump; reading an older lockfile is supported through migration for at least one minor cycle.

## Renderer profile version

- Profile names are explicit and versioned: `know-now-minijinja-v1`, future `-v2`, etc.
- The profile's compatibility surface is **the documented set of features**, not the underlying engine's full capabilities. Adding features to the profile is `MINOR`. Removing or changing them is `MAJOR` and requires a new profile name (`-v2`).
- Profile compatibility is tested with fixtures and release-note diff summaries (NFR-M9).

## Policy pack and template pack versions

- Per-pack SemVer recorded in the lockfile and manifest along with content hashes.
- Approved-version catalogs may pin allowed major or minor ranges per pack (PRD §16.7 ADMIN-009).
- Drift classifications: none / patch drift / minor drift / major drift / unknown / unapproved (PRD §14.6, ADMIN-005).

## When you change something — quick decision table

| Change | Surface to bump |
| ------ | --------------- |
| Bug fix in a generator | engine `PATCH` |
| New CLI command | engine `MINOR` |
| New optional metadata field | metadata schema `MINOR` |
| Renamed metadata field | metadata schema `MAJOR` + migration note |
| New optional field on the generator contract | generator contract `MINOR` |
| Restructured contract (renamed/removed field) | generator contract `MAJOR` + ADR + fixture diff |
| New `/api/v1/...` endpoint | API contract `MINOR` |
| Breaking change to `/api/v1` shape | introduce `/api/v2`, deprecate `/api/v1` |
| New required limit on the renderer profile | renderer profile `MAJOR` (`-v2`) |
| New template-pack feature in the profile | renderer profile `MINOR` |
| Lockfile shape change | lockfile schema bump + migration |
| New policy rule in the default pack | default pack `MINOR` if non-blocking, `MAJOR` if blocking |

## Release artifacts

Release outputs (engine binaries, checksums, attestations, SBOMs, fixture diffs, migration notes) are listed in PRD §20.3. Every release that changes a versioned surface must include the corresponding migration note in the changelog.

## Cross-references

- PRD §8.3 — Metadata contract layers.
- PRD §8.4 — Generator capability registry.
- PRD §9.5 — Reproducibility lockfile.
- PRD §13.4 — API contract rules.
- PRD §15.1.1 — Restricted template renderer architecture.
- PRD §17.6 NFR-M5/M6/M7/M8/M9 — fitness tests.
- PRD §20.3 — Release artifacts.
- ADR-0004 — Restricted MiniJinja-based template renderer profile.
