# Product Requirements Document — know-now

**Product:** know-now  
**Document status:** Fully rewritten consolidated PRD
**Primary implementation stack:** Rust + TypeScript  
**Primary distribution model:** Native Rust CLI with prebuilt binaries and Cargo-compatible installation paths  
**Primary product mode:** Local-first open-source metadata engine with optional local dashboard and future collaboration/control-plane expansion  
**Runtime model:** Native Rust CLI, Rust generation engine, Rust local server, and TypeScript dashboard. External ecosystem tools such as dbt may be invoked only through explicit toolchain adapters.  
**Metadata parser decision:** `serde-saphyr` is the primary YAML parser/deserializer; `marked-yaml` is the fallback candidate if the Phase 1 parser spike cannot enforce the know-now YAML subset with high-quality source-aware diagnostics.
**Template renderer decision:** `minijinja` is used through the restricted `know-now-minijinja-v1` renderer profile for Phase 3 declarative template packs. Custom packs are data-only, strict, fuel-limited, path-isolated, and unable to register arbitrary functions, execute code, access the network, or write files directly.

---

## 1. Executive summary

**know-now** is a metadata-driven data platform generation engine. Users define the *what* of an analytical data platform — domains, modules, entities, attributes, relationships, source mappings, semantic types, logical types, business keys, quality rules, business rules, ownership boundaries, policies, governance metadata, documentation metadata, open questions, and assumptions — and know-now deterministically generates the *how*: PostgreSQL DDL, dbt project artifacts, dbt tests, provider-neutral data quality contracts, Markdown documentation, Mermaid ER diagrams, generation manifests, traceability metadata, review summaries, local dashboard data, and custom artifacts from restricted declarative template packs.

The product is designed for consultants and data teams who repeatedly translate workshop outcomes, whiteboard models, source mappings, and agreed business concepts into production-ready platform artifacts. Instead of hand-authoring the same boilerplate across DDL, dbt, quality tests, documentation, diagrams, and governance reports, users maintain one canonical metadata source of truth and regenerate safely.

The initial product is a local-first Rust CLI and deterministic generation engine, supported in later phases by a TypeScript dashboard served locally by a Rust server. Core commands work offline and do not require cloud services, telemetry, authentication, external APIs, or hosted infrastructure. External tools such as dbt are optional validation targets, not know-now runtime dependencies.

The core value proposition is:

> **One metadata source of truth. Deterministic generated artifacts. Clear ownership boundaries. Safe regeneration. Fast feedback. Traceable decisions. Transparent architecture for technical teams, stakeholders, administrators, and maintainers.**

Phase 1 deliberately starts with an architecture-contract spike before building the full CLI surface. The product must prove its metadata schema, canonical graph, generator contract, deterministic artifact writer, deterministic manifest, source-aware diagnostics, and realistic demo output before expanding into dbt generation, quality contracts, dashboard features, governance workflows, restricted template rendering, and extensibility.

---

## 2. Product positioning

### 2.1 Differentiator hypothesis

know-now aims to unify artifact-generation concerns that are often handled separately:

* PostgreSQL DDL.
* dbt source definitions.
* dbt staging models.
* dbt mart models.
* dbt schema tests and custom generic tests.
* Provider-neutral data quality contracts.
* Project documentation.
* Mermaid ER diagrams.
* Generation manifests.
* Artifact traceability.
* Project health diagnostics.
* Local stakeholder dashboard data.
* Review summaries and open-question registers.
* Governance metadata and administrator health checks.
* Future API, lineage, metrics, Data Vault, MDM, and governance artifacts.

The strategic bet is that a canonical metadata model can become an infrastructure-as-code language for analytical data platforms.

The differentiator must be validated through a maintained competitive comparison matrix before it is used as a public marketing claim. know-now should avoid unsupported claims unless backed by current research and explicit comparison criteria.

### 2.2 Open-source plus consulting flywheel

know-now is an open-source product backed by Data Champions consulting. The open-source CLI demonstrates expertise, drives community visibility, and provides a practical tool for client engagements. Consulting engagements validate real-world needs and feed improvements back into the tool.

The intended flywheel is:

1. Open-source CLI adoption.
2. Content and examples show practical data-platform automation expertise.
3. Data Champions uses know-now on real engagements.
4. Real projects reveal missing features, edge cases, governance needs, and usability friction.
5. Improvements make the tool more useful.
6. Better tooling creates more adoption, leads, and consulting credibility.

### 2.3 Commercial paths to validate

The PRD does not assume hosted revenue from day one. Commercial paths should be validated in this order:

1. **Consulting acceleration:** faster delivery of data platforms during Data Champions engagements.
2. **Private policy/template packs:** paid client-specific standards, generators, and implementation support.
3. **Paid support:** support contracts for teams adopting the open-source CLI.
4. **Team collaboration workspace:** hosted or self-hosted workspace in later phases.
5. **Enterprise governance:** RBAC, audit exports, approved template registries, compatibility dashboards, policy drift detection, release preview, and multi-project governance reporting.

---

## 3. Target users and personas

### 3.1 Marco — solo data consultant

**Profile:** Independent data consultant with deep dbt and SQL experience. Maintains data platforms for multiple clients. Currently spends too much time translating models into repetitive platform artifacts.

**Core need:** Turn an agreed data model into a runnable, documented data platform quickly while retaining the ability to customize and explain it.

**Main product surface:** CLI, generated dbt/DDL/docs, dashboard for client demos, explainability features, review exports, support bundles.

### 3.2 Priya — data engineer / open-source adopter

**Profile:** Data engineer at a growing SaaS company. Maintains a hand-rolled dbt/Postgres stack and wants to reduce boilerplate while keeping control.

**Core need:** Adopt a tool that generates understandable artifacts from a clear metadata format and fits into CI/CD.

**Main product surface:** CLI, demo project, JSON Schema autocomplete, generated dbt project, `check` command, CI integration, extension templates, lockfile, impact scanning.

### 3.3 David — non-technical stakeholder

**Profile:** VP of Data or client stakeholder who understands business concepts but does not read SQL, YAML, or dbt code.

**Core need:** See, review, and approve the data architecture without relying on opaque explanations.

**Main product surface:** local dashboard, generated documentation, ER diagrams, review summary, open-question register, stakeholder-safe explanations, change summary.

### 3.4 Sanne — project administrator / data platform lead

**Profile:** Responsible for governance, standards, version pinning, template approval, policy compliance, release safety, and project health across multiple know-now projects.

**Core need:** Ensure teams use approved policies, compatible tool versions, safe generation workflows, auditable project state, reproducible generation, and healthy metadata practices across many repositories.

**Main product surface:** `doctor`, `check`, `policy status`, `policy explain`, `admin scan`, approved-version catalogs, project health dashboard, lockfile, audit log, compatibility checks, support bundle.

### 3.5 Maintainer — Data Champions / open-source maintainer

**Profile:** Maintains the open-source project, release process, example projects, compatibility matrix, documentation, template packs, policy packs, and consulting reuse.

**Core need:** Keep the project reliable, secure, extensible, reproducible, and easy for contributors to understand.

**Main product surface:** Rust workspace, TypeScript dashboard, test suite, benchmark suite, release pipeline, architecture fitness tests, compatibility fixtures, policy/template pack registry, supply-chain release artifacts.

---

## 4. Product principles

### 4.1 Metadata is the source of truth

The metadata model is the primary product interface. Generated artifacts are derived from it. Users should trust that changing metadata and regenerating produces predictable downstream changes.

### 4.2 Generated output must be understandable

Generated code should be readable, conventional, and easy to extend. The goal is not to generate the fewest lines of code; the goal is to generate code that real data engineers can run, inspect, review, and safely customize.

### 4.3 Ownership boundaries are explicit

The engine owns generated artifacts. Users own metadata, custom code, templates, policy configuration, and project configuration. know-now must never overwrite user-owned files.

### 4.4 Local-first by default

Core commands work offline. The default user experience does not require cloud services, telemetry, authentication, external APIs, or hosted infrastructure.

Future hosted/control-plane features must not weaken local-first behavior. Local-only projects remain fully supported.

### 4.5 Rust core, TypeScript interface

Rust powers the deterministic engine, CLI, validation, generation, local server, artifact writing, lockfile handling, diagnostics, and supply-chain-friendly distribution. TypeScript powers the dashboard, future visual UI, and generated API clients.

### 4.6 Safety beats magic

The product should avoid unsafe automation such as arbitrary template execution, silent destructive migrations, implicit network exposure, invisible metadata rewriting, non-deterministic generation, or overwriting manually edited generated files without explicit user action.

### 4.7 Extensibility must not compromise reliability

Custom template packs and future plugins must be permissioned, isolated, testable, versioned, resource-limited, and clearly marked as built-in, approved, experimental, or untrusted. Custom template rendering is exposed as a versioned know-now renderer profile, not as unrestricted access to an underlying template engine.

### 4.8 Reproducibility is a product feature

Generation must be reproducible across machines, operating systems, and CI runs. know-now should provide lockfiles, content hashes, deterministic ordering, stable generator contracts, capability declarations, deterministic manifests, and explicit compatibility profiles.

### 4.9 Explainability is required

Users must be able to answer:

* Why did this file change?
* Which metadata object produced this artifact?
* Which policy caused this warning?
* Which generated artifacts are affected by this entity?
* Which default was inferred, and which value was explicitly declared?
* Which tool versions and generator contract versions were involved?

### 4.10 Administrator visibility matters

know-now must serve not only the person generating artifacts, but also administrators responsible for standards, compatibility, policy drift, approved versions, auditability, and support.

### 4.11 Early product non-goals

Early phases are not:

* a dbt replacement
* a database migration execution tool
* a general-purpose data catalog
* a hosted governance platform
* an AI modeling assistant
* a BI metrics platform
* a full visual data modeling suite
* a data warehouse runtime
* a general-purpose workflow orchestrator

Early phases may generate artifacts for adjacent tools, but the core product remains a deterministic metadata validation and generation engine.

---

## 5. Success criteria

### 5.1 User success

| Outcome | Target |
| ------- | ------ |
| 5-minute first value | A new user installs know-now, runs `know-now init --demo`, validates metadata, runs `know-now check`, generates artifacts, and opens generated docs or dashboard within 5 minutes. |
| Aha moment | User changes a semantic type or logical type, regenerates, and sees DDL/dbt/tests/docs update consistently. |
| Regeneration confidence | No user-owned files are overwritten. Manually edited generated files are detected. Failed generation leaves prior generated output intact. |
| Clear diagnostics | Invalid metadata produces file, line, column, YAML path, error code, related object ID, and suggested fix where possible. |
| Stakeholder visibility | Non-technical stakeholders can inspect entities, relationships, generated docs, open questions, warnings, and changes without reading code. |
| CI readiness | `check`, `validate`, `generate --dry-run`, `diff --format json`, and `generate --locked` work as reliable PR checks. |
| Explainability | Users can trace generated artifacts back to metadata objects, policy rules, inferred defaults, generator versions, and lockfile state. |
| Reproducibility | Team members generate byte-identical deterministic output using the same metadata, config, lockfile, and engine version. |
| Admin confidence | Administrators can scan multiple projects and detect policy drift, unsupported versions, unresolved issues, and unsafe configuration. |

### 5.2 Business success

| Outcome | Target |
| ------- | ------ |
| Consulting validation | Used successfully on at least two real Data Champions engagements before broad public launch. |
| Manual edit ratio | Generated artifacts require less than 20% manual modification for supported patterns. |
| Repeat usage | At least one consultant uses know-now across multiple projects. |
| Adoption signal | GitHub stars, issues, discussions, and external mentions trend upward after launch. |
| Lead signal | Inbound consulting conversations can be traced to know-now content, GitHub, examples, or demos. |
| Willingness to pay | At least three teams express willingness to pay for support, private templates, governance features, or hosted collaboration. |

### 5.3 Technical success

| Metric | Target |
| ------ | ------ |
| CLI startup | <500ms cold start for common commands. |
| `know-now --help` | <200ms. |
| Metadata validation, 100 entities | <2s. |
| Full generation, 10 entities | <5s. |
| Full generation, 100 entities | <60s. |
| Incremental single-entity regeneration, 100-entity project | <10s. |
| Custom reference scan, 100 files | <5s. |
| Local dashboard FCP | <1s. |
| Local API entity list, 100 entities | <200ms p95. |
| Generated output determinism | Byte-identical deterministic generated artifacts for identical input across supported OSes; volatile run state is isolated under `.knownow/`. |
| Test coverage | 90% overall, 95% for core metadata and generation crates. |
| Peak memory, 100-entity project | <512MB excluding external toolchains. |

---

## 6. Product phases

## 6.1 Phase 1 — architecture contract spike

Phase 1 proves the core architecture before building a full product surface.

### Included

* Minimal Rust workspace with metadata, core, diagnostics, contract, writer, and one generator crate.
* One realistic demo metadata model with at least 8 entities, multiple source mappings, semantic types, logical types, domains/modules, relationships, open questions, and governance metadata.
* External YAML authoring schema draft.
* Explicit YAML authoring subset.
* `serde-saphyr` parser/deserializer spike with source spans, duplicate-key rejection, safe limits, and unsupported-feature diagnostics.
* `marked-yaml` fallback assessment if the primary parser spike cannot enforce know-now's subset with high-quality diagnostics.
* Source-aware diagnostics draft.
* Canonical `ProjectGraph` draft.
* Versioned generator contract draft.
* Generator capability declaration draft.
* Deterministic artifact manifest draft.
* Volatile run log draft under `.knownow/`.
* Atomic write proof of concept.
* Path safety proof of concept.
* Deterministic snapshot test proving byte-identical generated output.
* One generated PostgreSQL DDL artifact.
* One generated Markdown documentation artifact.
* Basic source-aware diagnostics.

### Excluded

* Full CLI command set.
* Dashboard.
* dbt generation.
* Policy packs.
* Template packs.
* Diffing.
* Incremental generation.
* External generator protocol.
* Metadata rewriting.

### Exit criteria

* Metadata parses with source spans.
* Parser spike proves `serde-saphyr` can enforce know-now's YAML subset directly or through an explicit pre-scan/event validation layer.
* Unsupported YAML features produce source-aware diagnostics.
* Duplicate keys fail validation with source-aware diagnostics.
* Parser file-size, nesting, and expansion limits are tested.
* Graph validation catches at least 10 meaningful errors.
* Generator never reads YAML directly.
* Generated artifacts are deterministic across consecutive runs.
* Deterministic generated artifacts do not contain volatile timestamps.
* Atomic write proof of concept preserves prior output on failure.
* Path safety rejects writes outside generated roots.
* The metadata model is expressive enough for at least one real consulting-style fixture.

---

## 6.2 Phase 2A — Rust CLI MVP

Phase 2A delivers the smallest useful Rust CLI and proves safe local generation without requiring the dashboard.

### Included

* Rust workspace and CLI.
* Project initialization.
* Project profiles.
* Demo project.
* Metadata YAML discovery across files.
* Safe YAML parsing with `serde-saphyr`, source spans, duplicate-key rejection, parser budgets, and explicit YAML subset enforcement.
* Typed Rust metadata model.
* Semantic validation.
* Built-in default policy profile and policy validation engine for built-in rules.
* Canonical `ProjectGraph`.
* Stable object ID validation.
* `id suggest` command.
* Versioned generator contract.
* Generator capability registry.
* JSON Schema export for IDE autocomplete.
* PostgreSQL DDL generation.
* Markdown documentation generation.
* Generation manifest.
* Volatile run log.
* `know-now.lock` reproducibility lockfile.
* Artifact ownership markers.
* Manual edit detection.
* Atomic write/promote system.
* Path safety validation.
* Stale artifact detection.
* CLI commands: `init`, `validate`, `check`, `schema`, `generate`, `lock update`, `lock check`, `id check`, `id suggest`, `examples list`, `config inspect`, `version`.
* CI integration examples.
* Snapshot, integration, compatibility fixture, and property-based tests.

### Excluded

* Visual editing.
* Dashboard.
* SaaS or hosted collaboration.
* Team auth.
* Source database introspection.
* CSV/Excel import.
* Natural language metadata input.
* Data Vault generation.
* MDM.
* Metrics layer generation.
* dbt generation.
* Mermaid generation.
* Third-party/custom policy packs and organization policy drift catalogs.
* Declarative template packs.

### Exit criteria

* Demo project initializes, validates, checks, and generates PostgreSQL DDL and Markdown docs.
* Atomic write/promote is production-ready.
* Manifest and lockfile behavior are stable enough for CI.
* Diagnostics include file, line, column, YAML path, error code, object ID, and suggested fix where possible.
* Generated output is deterministic, excluding explicitly volatile local state under `.knownow/`.
* JSON Schema is exported and usable in VS Code.
* Project files under `metadata/` and `custom/` are not modified by validation or generation.

---

## 6.3 Phase 2B — dbt, quality, diagrams, and artifact validation

Phase 2B expands generation coverage once the metadata graph, lockfile, artifact writer, and diagnostics are stable.

### Included

* dbt project generation.
* dbt source definitions.
* dbt staging models.
* dbt mart models.
* dbt tests and custom generic tests from metadata constraints and semantic types.
* Provider-neutral quality contract YAML.
* Mermaid ER diagram generation.
* Optional dbt toolchain adapters for compile validation.
* Generated artifact validation beyond Postgres DDL and Markdown.
* Deterministic demo fixture data for bundled examples.
* Expanded fixture suite for dbt and quality outputs.

### Exit criteria

* Generated dbt project compiles where validation is configured.
* Generated quality contracts cover all generated quality expectations.
* Mermaid diagrams validate.
* Demo fixture data is deterministic when enabled.
* Compatibility fixtures classify generated output changes.
* External validation adapters are capability-detected and optional unless explicitly required by project config.

---

## 6.4 Phase 3 — change safety, visibility, and administrator support

Phase 3 adds project change management, dashboard visibility, safe local serving, traceability, and administrator support.

### Included

* `diff` command.
* `diff --impact` and `--scan-custom`.
* `issues` command.
* `doctor` command.
* `serve` command.
* `explain` command.
* `support bundle` command.
* `policy status` command.
* `policy explain` command.
* `admin scan` command.
* `admin catalog check` command.
* `review export` command.
* Stable object ID diffing and rename detection.
* Incremental generation and content-addressed cache.
* Deprecation issue store.
* Migration SQL stubs for additive changes and confirmed renames.
* Local audit log.
* Approved-version catalog.
* Policy drift detection.
* Local Rust `axum` server.
* TypeScript/React dashboard.
* Versioned local API under `/api/v1`.
* Generated TypeScript API client.
* Entity browser.
* Entity detail pages.
* Relationship graph with accessible table fallback.
* Generated docs viewer.
* Project health/admin view.
* Generation traceability view.
* Artifact explanation view.
* Stakeholder review summary.
* Open-question register.
* Review export.
* Custom declarative template packs using the restricted `know-now-minijinja-v1` renderer profile.
* Custom/project policy packs beyond the built-in default policy profile.
* Policy pack version pinning, explanation, and drift workflows.

### Exit criteria

* Diff and issues workflows handle additive, rename, removal, and ambiguous changes.
* Incremental generation matches full generation output.
* `doctor` catches common setup and project health problems.
* `explain` traces artifacts to metadata and policy rules.
* `support bundle` creates sanitized diagnostic bundles.
* `admin scan` produces usable multi-project governance JSON.
* Dashboard renders entity list, entity detail, graph, graph table fallback, docs, manifest, health, review summary, and traceability views.
* Local server defaults are safe.
* Custom declarative template packs work safely through strict, fuel-limited, path-isolated rendering.
* Policy pack validation is usable.
* Policy drift detection is usable.
* Stakeholder review summary is usable in a real review meeting.
* At least two real consulting engagements or realistic pilots use the tool successfully.

---

## 6.5 Phase 4 — collaboration and bootstrapping

Phase 4 expands know-now from a local generation engine into a collaborative modeling and governance tool.

### Candidate scope

* Visual entity-relationship modeling UI.
* Bidirectional visual/config sync with safe metadata patching.
* CSV/Excel import for initial metadata seeding.
* Source database introspection.
* Rule-based bulk metadata transformations.
* Metadata schema migration tooling.
* Snowflake DDL generation.
* BigQuery and DuckDB DDL generation if validated by demand.
* Lineage tracking via generated and parsed SQL.
* Hosted or self-hosted multi-tenant control plane.
* Team auth and RBAC.
* Approved template/policy pack registry.
* Organization audit exports.
* External generator protocol over JSON stdin/stdout.

### Control-plane boundary

* The local engine is the data plane for validation and generation.
* Hosted services may coordinate collaboration, approved registries, policy catalogs, and review workflows.
* Hosted services must not require uploading generated artifacts or full metadata by default.
* Any cloud sync is explicit, documented, and reversible.
* Local-only projects remain fully supported.

---

## 6.6 Phase 5 — advanced intelligence and expanded targets

Phase 5 adds advanced modeling, semantic automation, and additional data platform targets.

### Candidate scope

* Natural language metadata input.
* AI-assisted metadata editing through explicit, reviewable workflows.
* Data Vault 2.0 generation.
* OSI-compatible metrics layer generation.
* MDM with probabilistic matching and steward queues.
* Reference data management.
* Business Vault SQL editor.
* Schema-aware autocomplete.
* Reconciliation SQL generation.
* Rich synthetic data generation from semantic types and constraints.
* WASI plugin runtime for stronger sandboxing.
* Optional hosted collaboration features.

---

## 7. User journeys

### 7.1 Journey 1 — Marco creates a client data platform

Marco just finished a kickoff workshop for an e-commerce client. The team agreed on customers, orders, products, inventory, suppliers, returns, customer segments, source systems, and open questions. Normally, Marco would spend days or weeks creating DDL, dbt models, schema tests, quality checks, documentation, and diagrams.

He runs:

```bash
know-now init ecommerce-platform --profile consultant-postgres-dbt
cd ecommerce-platform
```

He edits metadata files under `metadata/`, defining domains, modules, entities, relationships, semantic types, logical types, source mappings, business keys, governance metadata, quality rules, assumptions, and open questions.

He runs:

```bash
know-now validate
```

The CLI reports two validation errors with file, line, column, YAML path, object ID, and suggested fixes: a relationship references a nonexistent entity and an inventory entity has no business key. Marco fixes both.

He runs:

```bash
know-now check
know-now generate --locked
```

know-now builds a canonical project graph, creates a generation plan, generates PostgreSQL DDL, a dbt project, dbt tests, quality contracts, Markdown documentation, and Mermaid ER diagrams. It validates generated artifacts, writes them to a staging directory, and atomically promotes them.

Marco reviews the generated dbt models. They are readable and conventional. His custom dbt work belongs under `custom/dbt/`, which know-now never overwrites.

When the client later asks for returns, Marco adds metadata and regenerates. The changed artifacts are deterministic, the manifest records the affected artifacts, and the documentation updates automatically.

If Marco wonders why a SQL file changed, he runs:

```bash
know-now explain generated/ddl/postgres/schema.sql
```

The output shows which entities, attributes, policy rules, generator version, lockfile hash, and metadata spans affected the file.

### 7.2 Journey 2 — Priya adopts know-now for an existing dbt stack

Priya sees a demo and installs the CLI. She runs:

```bash
know-now init --demo
know-now check
know-now generate
```

The demo produces a runnable generated dbt project and PostgreSQL DDL. Priya inspects the metadata and sees how semantic types drive generated tests. Changing `customer.email` to semantic type `email` adds a generated email quality check and documentation notes.

She adds this to CI:

```bash
know-now check --format json --locked
know-now generate --dry-run --format json --locked
```

Before merging a rename, she runs:

```bash
know-now diff --impact --scan-custom
```

The report shows affected generated artifacts and possible references in `custom/`.

When she needs internal API documentation, she creates a declarative template pack under `custom/templates/`. The pack uses the restricted `know-now-minijinja-v1` renderer profile, writes only to its declared generated subdirectory through the artifact writer, and cannot overwrite built-in outputs.

### 7.3 Journey 3 — David reviews architecture without reading code

David receives a local dashboard URL from Marco. He opens it and sees entities, descriptions, attribute counts, relationships, warnings, governance labels, open questions, and generated documentation. He clicks Customer and sees attributes, constraints, semantic types, business key, source mappings, related entities, and stakeholder-friendly explanations.

In the relationship graph, he notices a segmentation reference table that was not discussed in the kickoff. He raises it in the next meeting. Marco updates the metadata and regenerates.

David opens the review summary, which lists changed entities, open questions, warnings, documentation gaps, assumptions, and items needing confirmation. He trusts the architecture because he can see it and because generated documentation matches the artifacts delivered to the technical team.

### 7.4 Journey 4 — Sanne governs project standards

Sanne supports multiple teams using know-now. She wants consistent naming, required audit columns, approved semantic type mappings, approved target versions, approved template packs, approved template renderer profiles, compatible generator versions, and known policy versions.

She configures a policy pack in `know-now.yml`:

```yaml
policy:
  pack: dc_standard
  version: "1.0"
```

Running `know-now validate` now reports policy violations with severity. A missing `updated_at` attribute is a warning in one project and an error in another, depending on the policy profile.

Sanne runs:

```bash
know-now doctor
know-now policy status
know-now admin scan ../projects --format json
```

The commands report project health, metadata schema version, target database version, dbt validation mode, unresolved deprecation issues, template pack versions, template renderer profiles, policy drift, lockfile status, last generation manifest status, and security warnings.

When a team opens a support issue, Sanne asks them to run:

```bash
know-now support bundle
```

The bundle excludes secrets and includes sanitized diagnostics, compatibility status, lockfile hash, manifest summaries, policy/template versions, and doctor output.

### 7.5 Journey 5 — maintainer releases a new generator version safely

The maintainer updates the dbt generator. CI runs Rust tests, property-based tests, architecture fitness tests, snapshot tests, compatibility fixture diffs, generated DDL execution tests, dbt compile tests through configured toolchain adapters, dependency policy checks, frontend type checks, and benchmark tests.

The release pipeline builds release binaries, publishes checksums, publishes attestations and SBOMs where practical, updates documentation, and runs compatibility fixtures. If generated output changes, the release notes include a generated artifact diff summary with change classifications.

---

## 8. Core architecture

## 8.1 Technology stack

| Layer | Technology |
| ----- | ---------- |
| CLI | Rust, `clap`-style command model |
| Core engine | Rust workspace crates |
| Diagnostics | Rust diagnostic model with source spans, stable codes, text/JSON/SARIF renderers |
| Metadata parsing | `serde-saphyr` primary parser/deserializer; strict know-now YAML subset; `marked-yaml` fallback candidate if the Phase 1 parser spike fails subset or diagnostic requirements |
| Metadata model | Rust structs/enums with Serde-compatible deserialization and custom validators |
| Metadata contracts | Versioned authoring schema, normalized graph schema, and generator contract schema |
| JSON Schema | Generated from Rust metadata types and curated manually where cross-reference autocomplete requires enhancement |
| Artifact writing | Rust writer crate with path safety, staging, atomic promotion, manual edit detection, and stale artifact handling |
| Local API server | Rust `axum` |
| Dashboard | TypeScript, React, Vite |
| Frontend package manager | pnpm with committed `web/pnpm-lock.yaml`, exact pinned `packageManager` field, and frozen-lockfile CI installs |
| Template renderer | `minijinja` through restricted `know-now-minijinja-v1` profile for declarative template packs |
| State/data fetching | TypeScript client generated from OpenAPI or shared JSON contracts |
| SQL/DDL generation | Typed internal DDL IR with dialect emitters |
| dbt validation | Optional external toolchain adapters: none, dbt executable, Fusion-compatible executable, or Docker |
| Artifact validation | Parser validation, optional dbt validation, Mermaid validation, live PostgreSQL execution in CI |
| Testing | Rust unit/integration/snapshot/property tests, TypeScript tests, generated artifact integration tests, compatibility fixtures |
| Distribution | Prebuilt binaries, Cargo-compatible source install fallback, Docker images for full local demo stack |

## 8.2 Rust workspace layout

```text
know-now/
  Cargo.toml
  crates/
    know_now_cli/           # CLI entrypoint
    know_now_core/          # orchestration, project loading, generation plans
    know_now_diagnostics/   # diagnostics, source spans, renderers, JSON/SARIF output
    know_now_metadata/      # metadata types, parsing, source spans, schema generation
    know_now_contract/      # stable generator/API contract schemas
    know_now_identity/      # stable object IDs, ID suggestions, rename matching primitives
    know_now_validate/      # semantic validation and policy validation
    know_now_codegen/       # generator traits and artifact model
    know_now_ir/            # typed SQL/DDL/documentation intermediate representations
    know_now_writer/        # staging, path safety, ownership markers, atomic promotion
    know_now_lock/          # lockfile schema, resolution, locked/unlocked checks
    know_now_gen_postgres/  # PostgreSQL DDL generator
    know_now_gen_dbt/       # dbt generator
    know_now_gen_quality/   # quality contracts and dbt tests
    know_now_gen_docs/      # Markdown and Mermaid generation
    know_now_diff/          # graph diffing, stable ID matching, change classification
    know_now_server/        # local axum API server
    know_now_policy/        # policy pack loading and evaluation
    know_now_templates/     # restricted MiniJinja-based declarative template pack rendering
    know_now_cache/         # content-addressed cache and dependency tracking
    know_now_toolchain/     # external toolchain adapters such as dbt validation
    know_now_audit/         # audit events, redaction, support-bundle summaries
    xtask/                  # release, fixture, benchmark, and maintenance tasks
  web/
    package.json
    pnpm-lock.yaml
    src/
    vite.config.ts
  examples/
  docs/
  fixtures/
```

## 8.3 Metadata contract layers

know-now separates metadata handling into three explicit contracts:

1. **Authoring schema** — the human-authored YAML format under `metadata/`.
2. **Normalized graph schema** — the validated canonical `ProjectGraph` used internally.
3. **Generator contract schema** — the stable, versioned structure passed to built-in generators, declarative templates, and future external generators.

Rules:

* The authoring schema may evolve for user ergonomics.
* The normalized graph may evolve for engine correctness and performance.
* The generator contract must be versioned and compatibility-tested.
* Built-in generators must not depend on raw YAML nodes.
* External generators must not depend on internal Rust-only graph structures.
* Breaking generator contract changes require migration notes and fixture diffs.

## 8.4 Generator capability registry

Each built-in generator declares capabilities:

* generator name and version
* generator contract versions accepted
* artifact kinds produced
* supported target dialects and versions
* supported logical and semantic type mappings
* validation gates supported
* known unsupported constructs
* experimental features

The registry powers:

* `know-now version --capabilities`
* `know-now doctor`
* generation planning
* compatibility matrix generation
* dashboard health/admin views

Template renderer profiles also declare supported profile versions, safety limits, compatibility status, and known unsupported template features.

## 8.5 Generation pipeline

know-now uses a staged deterministic pipeline:

1. **Discovery** — find project config and metadata files.
2. **Parsing** — parse YAML with `serde-saphyr` into source-preserved raw metadata nodes while enforcing parser budgets.
3. **YAML subset validation** — reject anchors, aliases, merge keys, custom tags, include directives, multi-document files, duplicate keys, and unsupported syntax before semantic validation.
4. **Deserialization** — convert raw nodes into typed Rust metadata structs.
5. **Semantic validation** — build a canonical `ProjectGraph` with source spans.
6. **Policy validation** — apply configured policy packs without mutating metadata.
7. **Default resolution** — apply explicit, traceable defaults from policy/profile/config.
8. **Contract projection** — project the internal graph into versioned generator contracts.
9. **Capability check** — ensure configured targets are supported by available generators.
10. **Planning** — produce a deterministic `GenerationPlan` listing target artifacts, dependencies, ownership, and validation gates.
11. **Generation** — run independent built-in generators and configured declarative template packs against the validated generator contract. Independent generators may run concurrently, but final artifact ordering must remain deterministic. Declarative template packs return artifact descriptors and content; they never write files directly.
12. **Artifact validation** — parse, compile, lint, or execute generated artifacts where possible.
13. **Manual edit detection** — compare existing generated files against the previous deterministic manifest.
14. **Path safety validation** — reject path traversal, symlink escape, absolute output paths, and writes outside declared generated roots.
15. **Stale artifact planning** — identify previously generated artifacts that are no longer produced.
16. **Atomic writing** — write to staging and promote only on success.
17. **Rollback/cleanup** — preserve previous output on failure and clean incomplete staging directories.
18. **Manifesting** — record deterministic generation metadata, artifact paths, hashes, warnings, and traceability data.
19. **Run logging** — record volatile command execution details under `.knownow/`.

## 8.6 Incremental generation and cache

know-now maintains a content-addressed cache under `.knownow/cache/`.

The cache stores:

* parsed metadata file hashes
* normalized graph hash
* generator input hashes
* artifact output hashes
* dependency edges from metadata objects to artifacts
* prior generation plan summaries

Generation planning determines which artifacts are affected by changed metadata objects.

Commands:

```bash
know-now generate --changed
know-now diff --affected-artifacts
```

Rules:

* Incremental generation must produce the same final output as full generation.
* `--no-cache` forces a full rebuild.
* Cache corruption is detected by hash mismatch and automatically discarded.
* CI may disable cache for maximum reproducibility.

## 8.7 Canonical project graph

The `ProjectGraph` is the internal source of truth after validation. It contains:

* project metadata
* domains
* modules
* entities
* attributes
* relationships
* source systems
* source mappings
* business keys
* semantic types
* logical types
* quality rules
* policy annotations
* documentation metadata
* governance metadata
* open questions
* assumptions
* stable object IDs
* source file spans
* dependency graph
* source-to-graph trace map
* graph-to-contract trace map

Generators never parse YAML directly. They receive only the validated generator contract and generation context.

## 8.8 Stable object IDs

Names are user-facing labels. Stable IDs are used for diffing, rename detection, issue tracking, artifact traceability, review state, and migration generation.

Example:

```yaml
entities:
  - id: ent_customer
    name: customer
    description: Customer master entity.
    attributes:
      - id: attr_customer_email
        name: email
        logical_type: string
        semantic_type: email
        required: true
```

Rules:

* IDs are recommended in Phase 2A and required for migration-safe workflows.
* If IDs are missing, know-now uses deterministic matching heuristics but labels changes as lower-confidence.
* Migration SQL for renames requires matching IDs or explicit user confirmation.
* IDs are globally unique within a project unless a specific namespace rule says otherwise.
* IDs use lowercase ASCII, digits, and underscores.
* IDs should use stable prefixes such as `dom_`, `mod_`, `ent_`, `attr_`, `rel_`, `src_`, `rule_`, and `pol_`.
* Attribute IDs remain stable even if an attribute is renamed or moved within an entity.

Commands:

```bash
know-now id check
know-now id suggest
know-now id backfill --dry-run
know-now id backfill --apply
```

Migration-safe mode:

```bash
know-now diff --migration-safe
know-now generate --migration-safe
```

Migration-safe commands fail when required stable IDs are missing.

## 8.9 SQL/DDL generation architecture

know-now uses a Rust-native typed DDL model rather than raw SQL string interpolation.

Pipeline:

```text
ProjectGraph
  -> GeneratorContract
  -> LogicalSchema
  -> DialectDdlIr
  -> DeterministicSqlEmitter
  -> Parser validation
  -> Optional live database execution validation
  -> Artifact set
```

Safety rules:

* Metadata identifiers are validated before DDL IR construction.
* Identifiers and literals are emitted only through typed emitters.
* Raw string interpolation for SQL identifiers/literals is forbidden.
* Generated SQL is parse-validated before writing.
* CI executes generated DDL against supported PostgreSQL versions.

## 8.10 dbt toolchain adapter

know-now does not bundle dbt. dbt validation is performed only through a configurable external toolchain adapter.

Supported validation modes:

* `none` — generate dbt artifacts but do not compile them locally.
* `dbt` — run a configured dbt executable and detect available capabilities.
* `dbt-core` — require detected dbt Core-compatible behavior.
* `dbt-fusion` — require detected Fusion-compatible behavior and supported adapter capabilities.
* `docker` — run validation in a configured container image.

Example:

```yaml
dbt:
  validation:
    mode: dbt
    executable: dbt
    required_in_ci: true
```

Adapter behavior:

* The toolchain adapter must detect executable identity using `--version` or equivalent and classify it as `core`, `fusion`, `compatible`, or `unknown`.
* Validation gates are capability-based, not name-based.
* The adapter records detected dbt version, adapter/plugin availability, supported commands, and validation limitations.
* If Fusion-compatible validation is configured but the generated target, adapter, or command set is unsupported, `doctor` reports a clear warning or error before generation validation is attempted.
* The compatibility matrix records dbt Core versions, Fusion-compatible versions, supported adapters, generated target compatibility, and validation commands.
* dbt remains an optional external validation target, not a know-now runtime dependency.

## 8.11 Deterministic manifest and volatile run logs

Every successful generation writes a deterministic artifact manifest. Volatile execution metadata is written separately under `.knownow/`.

Deterministic manifest example:

```json
{
  "engine_version": "1.0.0",
  "metadata_schema_version": "1.0",
  "generator_contract_version": "1.0",
  "project_id": "ecommerce_demo",
  "input_hash": "sha256:...",
  "lockfile_hash": "sha256:...",
  "target_database": {
    "kind": "postgres",
    "version": "18",
    "compatibility_floor": "16"
  },
  "policy": {
    "pack": "dc_standard",
    "version": "1.0",
    "hash": "sha256:..."
  },
  "template_renderers": [
    {
      "profile": "know-now-minijinja-v1",
      "engine": "minijinja",
      "profile_version": "1",
      "limits": {
        "max_fuel": 50000,
        "max_output_bytes": 10485760
      }
    }
  ],
  "artifacts": [
    {
      "path": "generated/ddl/postgres/schema.sql",
      "kind": "postgres_ddl",
      "artifact_id": "art_postgres_schema",
      "generator": "know_now_gen_postgres",
      "generator_version": "1.0.0",
      "hash": "sha256:...",
      "metadata_object_ids": ["ent_customer", "attr_customer_email"],
      "trace": [
        {
          "artifact_span": {"line_start": 12, "line_end": 18},
          "metadata_object_ids": ["ent_customer", "attr_customer_email"],
          "policy_rule_ids": ["pol_email_max_length"]
        }
      ]
    }
  ],
  "warnings": []
}
```

Volatile run metadata example:

```json
{
  "run_id": "run_20260502_120000_abc123",
  "started_at": "2026-05-02T12:00:00Z",
  "finished_at": "2026-05-02T12:00:04Z",
  "command": "generate --locked",
  "result": "success",
  "manifest_hash": "sha256:...",
  "duration_ms": 4210
}
```

Rules:

* `generated/manifest.json` must be byte-identical for identical inputs.
* `.knownow/last_generation.json`, `.knownow/audit.log`, and `.knownow/runs/` may contain timestamps and machine-local details.
* Tests must explicitly distinguish deterministic generated output from volatile local state.
* `SOURCE_DATE_EPOCH` may be supported for deterministic fixtures and release tests, but normal reproducibility must not depend on timestamps.

---

## 9. Project structure and ownership model

## 9.1 Default project layout

```text
my-knownow-project/
  know-now.yml
  know-now.lock
  metadata/
    project.yml
    domains/
    modules/
    entities/
    sources/
    relationships/
    rules/
    governance/
    questions/
  generated/
    ddl/
    dbt/
    quality_contracts/
    docs/
    diagrams/
    review/
    manifest.json
  custom/
    dbt/
      models/
      macros/
      seeds/
      exposures/
    quality/
    templates/
    docs/
  .knownow/
    issues.json
    review_state.json
    audit.log
    cache/
    locks/
    runs/
    last_generation.json
  docs/
    exported/
```

## 9.2 Ownership rules

| Path | Owner | Write behavior |
| ---- | ----- | -------------- |
| `metadata/` | User | Read-only in early phases. Never rewritten by default. |
| `custom/` | User | Never written by know-now. |
| `generated/` | Engine | Recreated through atomic generation, with manual-edit detection. |
| `.knownow/` | Engine/project state | Stores cache, manifests, issue state, review state, audit logs, locks, and run logs. |
| `docs/exported/` | User or engine by explicit export | Written only through explicit export commands. |
| `know-now.yml` | User | Created by init; modified only by explicit commands in future phases. |
| `know-now.lock` | Engine with user review | Records resolved engine, generator, policy, template, contract, and compatibility versions for reproducible generation. |

## 9.3 Artifact writer rules

* All generated writes go through `know_now_writer`.
* Output paths must be relative, normalized, and inside an approved generated root.
* Absolute paths are rejected.
* Path traversal is rejected.
* Symlinks inside generated output are not followed during writes by default.
* Commands that write `generated/` or mutable `.knownow/` state acquire a project-scoped lock under `.knownow/locks/`.
* Stale generated artifacts are reported.
* Deletion of stale artifacts follows configured pruning behavior.
* `--prune stale` deletes only files recorded in the previous deterministic manifest and never deletes untracked files.
* Previously generated files that were deleted by the user are reported as missing prior artifacts and may be recreated during generation.
* Failed generation preserves the previous generated artifact set.
* Writer behavior is tested on Linux, macOS, and Windows.

## 9.4 Stale artifact behavior

Generated artifacts that existed in the previous manifest but are absent from the new generation plan are stale.

Generation supports:

```bash
know-now generate --prune none
know-now generate --prune stale
```

Defaults:

* Local solo profiles default to `--prune none`.
* Governed profiles may default to reporting stale artifacts as warnings.
* CI examples should make pruning behavior explicit.

## 9.5 Reproducibility lockfile

`know-now.lock` is a committed project file for team workflows.

It records:

* lockfile schema version
* know-now engine version
* metadata schema version
* generator contract version
* built-in generator versions
* policy pack names, versions, and content hashes
* template pack names, versions, and content hashes
* template renderer profile names and versions
* target compatibility profiles
* resolved semantic type mappings
* approved registry source hashes where applicable
* optional external toolchain constraints, not local executable paths

Commands:

```bash
know-now lock update
know-now lock check
know-now generate --locked
```

Rules:

* CI should use `know-now generate --locked` or `know-now check --locked`.
* If resolved pack versions differ from the lockfile, generation fails unless explicitly unlocked.
* `--locked` mode does not allow floating generator, policy, template, renderer profile, or contract versions.
* Project config may express allowed version ranges, but the lockfile records exact resolved versions and content hashes.
* `init --demo` creates a lockfile.
* The deterministic manifest records the lockfile hash used for generation.
* Local machine paths, timestamps, usernames, and environment-specific executable locations are not written to the lockfile.
* Lockfile updates are explicit and reviewable.
* Breaking generator contract upgrades require `know-now lock update --accept-contract-upgrade`.
* Lockfile schema migrations are deterministic and reported in release notes.
* `lock check --format json` is suitable for CI policy enforcement.

## 9.6 dbt ownership model

Generated dbt artifacts are written under:

```text
generated/dbt/
  dbt_project.yml
  models/
    staging/
    marts/
    schema.yml
  sources.yml
  tests/
    generic/
```

User-owned dbt artifacts live under:

```text
custom/dbt/
  models/
  macros/
  seeds/
  exposures/
```

The generated `dbt_project.yml` includes both generated and custom paths. know-now never overwrites `custom/dbt/`.

## 9.7 Git policy

Generated output may be ignored or committed depending on team workflow.

`know-now init` supports:

```bash
know-now init --generated-git-policy ignore
know-now init --generated-git-policy commit
know-now init --generated-git-policy ask
```

Defaults:

| Profile | Default generated output policy |
| ------- | ------------------------------- |
| `minimal` | `ignore` |
| `consultant-postgres-dbt` | `ask` |
| `dbt-existing-stack` | `ask` |
| `governed-team` | `commit` |
| `demo` | `commit` |

Documentation explains trade-offs:

* ignoring generated files keeps repositories smaller and treats output as derived state
* committing generated files improves pull request review and deployment reproducibility
* teams should choose intentionally

---

## 10. Metadata model

## 10.1 Metadata files

Users can organize metadata across multiple YAML files under `metadata/`. The engine auto-discovers files and resolves cross-file references.

Example:

```yaml
version: "1.0"
project:
  id: ecommerce_demo
  name: E-commerce Demo
  description: Demo model for customers, orders, products, suppliers, inventory, and returns.

target_database:
  kind: postgres
  version: "18"
  compatibility_floor: "16"

policy:
  pack: dc_standard
  version: "1.0"
```

## 10.2 YAML authoring subset and parser requirements

know-now intentionally supports a safe, boring YAML authoring subset. The metadata format should be pleasant for humans, predictable for editors and CI, and constrained enough to support source-aware diagnostics, deterministic merging, and future targeted metadata patching.

Parser decision:

* Primary parser/deserializer: `serde-saphyr`.
* Fallback candidate: `marked-yaml`.
* Direct `saphyr-parser` usage is reserved for a future custom parser layer if the primary and fallback options cannot meet know-now requirements.
* `serde_yaml`, `serde_yml`, and unmaintained YAML parser stacks are not allowed.
* No C/FFI parser dependency is allowed unless explicitly approved through dependency policy.

Parser requirements:

* source spans for diagnostics
* typed Serde-compatible deserialization into Rust metadata structs
* deterministic mapping and sequence handling
* configurable file-size, nesting, and resource-exhaustion limits
* duplicate-key detection and rejection
* clear errors for unsupported YAML features
* maintained dependency posture
* no dependency on an unmaintained YAML parser without an explicit documented risk exception
* parser options, pre-scan validation, or event-level validation that rejects unsupported YAML features before metadata reaches semantic validation
* diagnostic output that includes file, line, column, YAML path where available, and suggested remediation for common unsupported syntax

Phase 1/2 YAML subset:

* top-level mapping documents
* scalar string keys
* strings, numbers, booleans, nulls, sequences, and mappings
* no anchors
* no aliases
* no merge keys
* no custom YAML tags
* no include directives
* no multi-document metadata files
* duplicate keys are errors
* excessive nesting is rejected
* excessive file sizes are rejected
* unsupported scalar forms that create ambiguous metadata semantics are rejected

Implementation shape:

```text
YAML text
  -> syntax parse with serde-saphyr
  -> know-now YAML subset validation
  -> typed authoring metadata structs with source spans
  -> semantic validation
  -> ProjectGraph
```

The parser dependency is isolated to `know_now_metadata`. Generators must never depend on parser crates or raw YAML nodes.

Recommended internal model:

```rust
pub struct ParsedMetadataDocument {
    pub source_id: SourceId,
    pub path: Utf8PathBuf,
    pub version: MetadataSchemaVersion,
    pub document: AuthoringMetadata,
    pub spans: SourceSpanIndex,
}

pub struct SourceSpanIndex {
    // Maps metadata object IDs, YAML paths, and field names to source spans.
}
```

Fallback rule:

* If the Phase 1 parser spike cannot make `serde-saphyr` reject anchors, aliases, merge keys, custom tags, include directives, multi-document files, duplicate keys, excessive nesting, and excessive file sizes with high-quality source-aware diagnostics, switch to `marked-yaml`.

Parser diagnostic fixtures must include:

* `valid_minimal.yml`
* `valid_multifile_project.yml`
* `duplicate_key.yml`
* `anchor.yml`
* `alias.yml`
* `merge_key.yml`
* `custom_tag.yml`
* `include_directive.yml`
* `multi_document.yml`
* `deep_nesting.yml`
* `large_file.yml`
* `bad_scalar_type.yml`
* `unknown_field.yml`

Each invalid fixture snapshots both text and JSON diagnostics.

Rationale:

* `serde-saphyr` aligns with typed Rust metadata through Serde-compatible deserialization while providing source-span and parser-budget capabilities needed by know-now.
* The constrained subset keeps metadata portable across editors, CI, documentation tooling, and future UI-assisted patching.
* Rejecting advanced YAML features improves safety, diagnostics, and deterministic merging.
* Future metadata-writing commands are easier to implement when the authoring subset is constrained.
* The fallback rule prevents the parser choice from blocking the product if the Phase 1 spike reveals diagnostic or subset-enforcement gaps.

## 10.3 Domains and modules

Large projects can organize metadata into domains and modules.

Example:

```yaml
domains:
  - id: dom_commercial
    name: commercial
    display_name: Commercial
    owner: sales_ops
    description: Customer, order, product, and revenue concepts.

modules:
  - id: mod_orders
    domain: commercial
    name: orders
    description: Order capture, fulfillment, returns, and order financials.
```

Entities may belong to a domain and module:

```yaml
entities:
  - id: ent_order
    name: order
    domain: commercial
    module: orders
```

Rules:

* Domain and module membership is optional in small projects.
* Policy packs may require it for governed projects.
* Dashboard navigation groups entities by domain/module where available.
* Diff and issue reports include domain/module context.

## 10.4 Entity definition

```yaml
entities:
  - id: ent_customer
    name: customer
    display_name: Customer
    domain: commercial
    module: customers
    owner: sales_ops
    steward: data_governance
    classification: internal
    retention_policy: standard_customer_retention
    description: A person or organization that places orders.
    type: dimension
    tags: [core, commercial]
    business_key: [customer_id]
    attributes:
      - id: attr_customer_id
        name: customer_id
        logical_type: string
        semantic_type: identifier
        required: true
        unique: true
        description: Stable customer identifier from the source system.
      - id: attr_customer_email
        name: email
        logical_type: string
        semantic_type: email
        sensitivity: personal_data
        pii: true
        required: false
        constraints:
          max_length: 320
        description: Customer email address.
```

## 10.5 Relationship definition

```yaml
relationships:
  - id: rel_order_customer
    name: order_customer
    from_entity: order
    to_entity: customer
    cardinality: many_to_one
    from_attributes: [customer_id]
    to_attributes: [customer_id]
    description: Each order belongs to one customer.
```

## 10.6 Source mapping definition

```yaml
sources:
  - id: src_shopify
    name: shopify
    type: application_export
    freshness:
      expected_interval: 24h
      warn_after: 36h
      error_after: 72h
    tables:
      - id: src_tbl_shopify_orders
        name: orders
        target_entity: order
        grain: one_row_per_order
        load_pattern: incremental
        incremental_key: updated_at
        transformation_intent: staging_to_entity
        columns:
          - source: id
            target: order_id
          - source: customer_id
            target: customer_id
          - source: total_price
            target: total_amount
```

Phase 2 supports simple generation from these fields. Later phases can use this richer metadata for lineage, freshness checks, incremental model generation, and source introspection.

## 10.7 Attribute type model

Attributes separate logical type, semantic type, and optional physical overrides.

Example:

```yaml
attributes:
  - id: attr_customer_email
    name: email
    logical_type: string
    semantic_type: email
    required: false
    constraints:
      max_length: 320
```

Rules:

* `logical_type` controls portable type behavior.
* `semantic_type` controls meaning, documentation, display, and quality expectations.
* Physical database types are derived from target dialect and policy pack unless explicitly overridden.
* Physical overrides are allowed only with policy approval.

Phase 2 logical types:

* `string`
* `integer`
* `decimal`
* `boolean`
* `date`
* `timestamp`
* `time`
* `json`
* `uuid`
* `binary`

Decimal attributes must support precision and scale. Timestamp attributes must support timezone semantics.

## 10.8 Semantic types

Phase 2 semantic types:

* `identifier`
* `name`
* `email`
* `phone`
* `url`
* `currency_amount`
* `percentage`
* `country_code`
* `postal_code`
* `status`
* `category`
* `free_text`
* `json_payload`
* `created_timestamp`
* `updated_timestamp`

Semantic types influence:

* database type mapping policy
* dbt tests
* quality contracts
* documentation
* dashboard display
* generated examples
* deterministic demo fixture generation
* future synthetic/test data generation

## 10.9 Governance metadata

Entities and attributes may include governance metadata:

* `owner`
* `steward`
* `classification`
* `sensitivity`
* `pii`
* `retention_policy`
* `access_policy_ref`
* `regulatory_tags`

Rules:

* Governance metadata is optional in minimal projects.
* Governed profiles may require owner, classification, and retention metadata.
* Policy packs can validate governance completeness.
* Generated docs and dashboard views clearly distinguish technical metadata from governance metadata.
* Support bundles redact or summarize sensitive governance fields where configured.

## 10.10 Open questions and assumptions

Metadata may include explicit open questions and assumptions.

Example:

```yaml
open_questions:
  - id: q_customer_lifetime_value
    related_objects: [ent_customer]
    question: Should customer lifetime value include refunded orders?
    status: needs_stakeholder_confirmation
    owner: marco
    due: 2026-05-15

assumptions:
  - id: asm_order_currency
    related_objects: [ent_order, attr_order_currency]
    statement: All order totals are stored in the transaction currency.
    confidence: medium
```

Rules:

* Open questions appear in generated docs and stakeholder review summaries.
* Blocking questions can prevent generation in strict governed profiles.
* Assumptions are clearly labeled as assumptions in dashboard and docs.

## 10.11 YAML preservation strategy

Early phases never rewrite metadata files during normal commands. This avoids accidental loss of comments, formatting, anchors, ordering, or editor-specific formatting.

Commands that must not modify metadata:

* `validate`
* `check`
* `generate`
* `generate --dry-run`
* `diff`
* `issues`
* `serve`
* `doctor`
* `schema`
* `version`
* `config inspect`

Future metadata-writing commands must:

* require explicit user action
* preview changes
* create backups
* apply targeted patches where possible
* preserve comments and formatting as much as possible
* clearly separate no-write suggestions, such as `id suggest`, from explicit write commands, such as `id backfill --apply`

---

## 11. Artifact generation

## 11.1 Generated artifact categories

Phase 2A generates:

1. PostgreSQL DDL.
2. Markdown entity documentation.
3. Deterministic generation manifest.
4. Traceability metadata.
5. Volatile run logs under `.knownow/`.

Phase 2B adds:

1. dbt project files.
2. dbt tests and custom generic tests.
3. Provider-neutral quality contract YAML.
4. Mermaid ER diagrams.
5. Optional deterministic demo fixture data for bundled examples.

Phase 3 adds:

1. Dashboard data JSON where needed.
2. Review summary Markdown and review-pack exports.
3. Admin/project-health exports.
4. Custom artifacts generated by restricted declarative template packs.

## 11.2 PostgreSQL DDL

Generated DDL includes:

* schemas
* tables
* columns
* primary keys
* foreign keys
* unique constraints
* not-null constraints
* check constraints
* indexes
* comments where supported

Target compatibility:

```yaml
target_database:
  kind: postgres
  version: "18"
  compatibility_floor: "16"
```

Compatibility policy:

* The greenfield default compatibility floor should be PostgreSQL 16 unless pilot constraints require another floor.
* PostgreSQL 18 is the default target for new demo projects.
* PostgreSQL 15 may remain supported through an explicit older-client profile if required by real deployments.
* The compatibility matrix must be refreshed before every public release.
* Generated DDL must be tested against the configured floor and current target where practical.

## 11.3 dbt generation

Generated dbt output includes:

* `dbt_project.yml`
* source definitions
* staging models
* mart models
* schema tests
* custom generic tests where needed
* generated documentation metadata

The generated project must pass configured dbt validation in supported environments. dbt validation is optional and external to know-now core operation unless project configuration explicitly requires it.

## 11.4 Quality generation

Phase 2 quality output has two layers:

1. **Executable dbt tests** for checks that map cleanly to dbt.
2. **Provider-neutral quality contracts** that record all generated quality expectations in a stable know-now schema.

Optional provider exporters may generate Great Expectations-compatible or other tool-specific configuration later.

## 11.5 Documentation generation

Generated documentation includes:

* project overview
* domain/module pages
* entity pages
* attribute tables
* semantic type descriptions
* business keys
* relationship lists
* source mapping summaries
* governance summaries
* quality rule summaries
* generation manifest link
* Mermaid ER diagram
* open questions
* assumptions and confidence labels
* documentation quality warnings

Markdown must render in GitHub and standard Markdown viewers.

Documentation quality requirements:

* Every entity page starts with a plain-language summary.
* Relationship descriptions should explain business meaning, not only foreign keys.
* Constraints are translated into stakeholder-friendly text where possible.
* Missing descriptions are surfaced as documentation quality warnings.
* Documentation includes a “questions to confirm” section for incomplete metadata.
* Generated docs distinguish confirmed metadata, assumptions, unresolved questions, inferred defaults, and policy-provided defaults.

## 11.6 ER diagrams

Mermaid diagrams include:

* all entities
* all relationships
* cardinality
* self-referencing relationships
* many-to-many relationship handling
* relationship labels where configured

The dashboard renders diagrams visually and supports navigation to related entity detail pages. For accessibility and large models, the dashboard must also provide a table-based relationship view.

## 11.7 Generated artifact definition of done

A generated artifact set is valid only when:

* all files are produced deterministically from the same metadata hash
* no user-owned files are modified
* manually edited generated files are detected before overwrite
* SQL parses successfully
* PostgreSQL DDL executes in configured CI targets
* dbt project compiles where dbt validation is configured
* generated dbt tests are syntactically valid
* generated documentation renders without broken Mermaid syntax
* manifest records every artifact path and hash
* warnings are classified as info/warn/error/blocking
* failed generation leaves previous generated output intact
* stale generated artifacts are detected and reported

## 11.8 Generated artifact ownership markers

Where the target format supports comments, generated files include a machine-readable ownership header:

```text
Generated by know-now.
Artifact ID: art_postgres_schema
Generator: know_now_gen_postgres
Input hash: sha256:...
Manifest path: generated/manifest.json
Do not edit directly unless you intend to fork this artifact.
```

Before replacing an existing generated file, know-now checks whether the previous artifact hash in the last deterministic manifest matches the existing file content after applying the documented canonical hash mode.

Hashing rules:

* The manifest records artifact hashes.
* Generated files do not embed their own full-file content hash by default.
* If a format requires embedded hashes, the hash mode must explicitly exclude the hash field itself.
* Manual edit detection uses the previous manifest, not only the file header.

If a generated file was manually edited:

* default behavior: block promotion and report the edited file
* `--accept-generated-overwrite`: replace after explicit confirmation
* `--adopt-generated-edits`: future command to convert safe changes back into metadata where possible

## 11.9 Deterministic demo fixture generation

Users can generate small deterministic fixture datasets for bundled examples and CI validation.

Acceptance:

1. Fixture generation is optional.
2. Fixture data is deterministic for identical metadata and seed.
3. Fixture generation respects logical types, semantic types, required fields, and simple relationships.
4. Fixture data is clearly marked as synthetic/demo data.
5. Production projects do not generate fixture data unless explicitly configured.

---

## 12. CLI product design

## 12.1 Command list

Phase 2A:

```bash
know-now init
know-now validate
know-now check
know-now schema
know-now generate
know-now lock update
know-now lock check
know-now id check
know-now id suggest
know-now id backfill --dry-run
know-now examples list
know-now config inspect
know-now version
know-now version --capabilities
```

Phase 2B:

```bash
know-now generate --target dbt
know-now generate --target quality
know-now generate --target docs
know-now generate --target diagrams
```

Phase 3:

```bash
know-now diff
know-now issues
know-now doctor
know-now serve
know-now explain
know-now support bundle
know-now policy status
know-now policy explain
know-now admin scan
know-now admin catalog check
know-now review export
know-now id backfill --apply
```

Phase 4+:

```bash
know-now migrate
know-now import
know-now introspect
know-now transform
```

## 12.2 Standard flags

All relevant commands support:

```bash
--format text|json|sarif|quiet
--verbose
--debug
--config <path>
--project <path>
--no-color
```

Generation commands support:

```bash
--dry-run
--target ddl|dbt|quality|docs|diagrams|review|fixtures|all
--strict
--fail-on-warnings
--locked
--no-cache
--changed
--prune stale|none
--accept-generated-overwrite
```

Diff commands support:

```bash
--impact
--scan-custom
--baseline last-generation|git:<ref>|manifest:<path>
--migration-safe
```

## 12.3 Project initialization profiles

`know-now init` supports profiles:

```bash
know-now init my-project --profile minimal
know-now init my-project --profile consultant-postgres-dbt
know-now init my-project --profile dbt-existing-stack
know-now init my-project --profile governed-team
know-now init my-project --profile demo
know-now init my-project --guided
know-now init --demo
```

Supported Phase 2 profiles:

* `minimal`
* `consultant-postgres-dbt`
* `dbt-existing-stack`
* `governed-team`
* `demo`

Rules:

* `init --demo` is an alias for `init --profile demo`.
* `init --guided` asks a small number of local-only questions and writes the initial config.
* Guided init never sends telemetry or contacts external services.
* `examples list` shows bundled examples available offline.
* `config inspect` shows resolved config, profile, policy, target versions, and lockfile state.

## 12.4 Error diagnostics

Errors should include:

* severity
* error code
* human-readable message
* file path
* line
* column
* YAML path
* related metadata object ID where available
* suggested fix where deterministic
* JSON representation for CI
* SARIF representation for CI/code-scanning integrations

Example:

```text
error[META-REL-001]: relationship references unknown entity `customers`
  --> metadata/relationships/orders.yml:12:18
   |
12 |     to_entity: customers
   |                ^^^^^^^^^ unknown entity
   |
help: did you mean `customer`?
```

## 12.5 `check`

`know-now check` is the recommended local and CI verification command.

It runs:

* metadata discovery
* YAML parsing
* schema validation
* semantic validation
* policy validation
* lockfile consistency check where `--locked` is used
* capability check
* generation planning
* dry-run artifact generation
* configured artifact validation

Rules:

* `check` writes no generated artifacts unless explicitly requested.
* `check --format json` is suitable for CI annotations.
* `check --format sarif` is suitable for code-scanning integrations.
* CI examples should prefer `know-now check --format json --locked`.

## 12.6 `doctor`

`know-now doctor` reports:

* know-now version
* OS and architecture
* project root
* config file status
* lockfile status
* metadata schema version
* generator contract version
* metadata discovery summary
* policy pack status
* policy drift status
* target database configuration
* dbt validation mode and availability
* generator capability status
* dashboard asset status
* unresolved issue count
* last generation status
* dependency/template warnings
* security warnings for network binding or unsafe config

It supports JSON output for automated support bundles.

## 12.7 `explain`

`know-now explain` helps users understand why artifacts and warnings exist.

Examples:

```bash
know-now explain generated/ddl/postgres/schema.sql
know-now explain ent_customer
know-now explain META-REL-001
```

The command reports:

* related metadata objects
* affected artifacts
* generator name and version
* policy rules involved
* template pack inputs where relevant
* validation gates
* manifest and lockfile hashes
* source YAML spans where available
* artifact spans where available
* inferred/defaulted metadata versus explicit metadata

## 12.8 `support bundle`

`know-now support bundle` creates a sanitized support bundle for debugging.

It includes:

* doctor output
* command logs
* manifests
* config summary
* compatibility status
* lockfile hash
* policy/template versions
* sanitized diagnostics
* recent run summaries

Rules:

* Metadata inclusion is opt-in.
* Secrets and environment variables are redacted.
* Redaction uses a documented allowlist/denylist model for environment variables, paths, config keys, and command logs.
* Users can preview bundle contents before writing.
* Bundles include a manifest of included file categories without exposing redacted values.
* Support bundles summarize sensitive governance metadata unless explicit inclusion is approved.

## 12.9 Offline behavior

The following commands must work without network access:

* `init` with bundled templates/demo
* `validate`
* `check`
* `schema`
* `generate`
* `generate --dry-run`
* `diff`
* `issues`
* `doctor`, excluding explicit update checks
* `examples list`
* `config inspect`
* `version`

No telemetry is sent by default.

External validation adapters may require separately installed tools. Core know-now validation and generation must remain available without those tools unless a project explicitly requires adapter validation.

---

## 13. Dashboard and local API

## 13.1 Dashboard scope

The Phase 3 dashboard is local-first and read-only by default. It is intended for stakeholder review, consultant demos, and project health visibility.

Core views:

* project overview
* domain/module overview
* entity list
* entity detail
* attribute detail
* relationship graph
* accessible relationship table
* generated documentation viewer
* generation manifest viewer
* artifact traceability view
* artifact explanation view
* deprecation issues list
* project health/admin view
* stakeholder review summary
* review checklist
* open-question register
* change approval summary

## 13.2 Local server

The local server is implemented in Rust and serves both the TypeScript dashboard and JSON API.

Defaults:

* bind to `127.0.0.1`
* read-only mode
* per-session access token required for browser/API access
* strict CORS and origin checks
* no generation trigger unless explicitly enabled
* no telemetry

When `serve` starts, it prints a local URL containing a one-time launch token. The token is exchanged for a short-lived local session and then removed from the browser-visible route.

```text
http://127.0.0.1:3827/__open?launch_token=...
```

Network exposure requires explicit configuration:

```bash
know-now serve --host 0.0.0.0
```

This must emit a warning that the Phase 3 server is not intended as an authenticated multi-user deployment.

Rules:

* API requests require a local session.
* The launch token is single-use.
* The browser session uses a same-site, HTTP-only cookie where practical.
* Query-string tokens are not used for normal API requests after initial launch.
* The dashboard clears token-bearing routes from browser history where practical.
* Write endpoints require CSRF protection in addition to the session.
* Write endpoints require `--allow-generate`.
* Write requests include an explicit request-level confirmation token or confirmation field; server-side flags alone are not sufficient.
* Write endpoints require a second explicit confirmation flag when bound to anything other than `127.0.0.1`.
* CORS allows only the served dashboard origin by default.
* Security warnings are displayed in the dashboard and `doctor`.

## 13.3 API behavior

Read endpoints:

* project summary
* domains/modules
* entities
* entity detail
* relationships
* graph data
* documentation
* diagrams
* manifests
* issues
* health diagnostics
* traceability data
* policy findings
* open questions
* review summary

Write endpoints:

* disabled by default
* `POST /generate` only available with `know-now serve --allow-generate`

The API schema is generated or validated from Rust server types and consumed by the TypeScript dashboard.

The launch-token exchange endpoint is intentionally separate from normal API routes. Normal API requests use the established local session, not the query-string launch token.

## 13.4 API contract rules

* Local API routes are versioned under `/api/v1`.
* The server exposes `/api/v1/version` with engine version, API contract version, dashboard asset version, and compatibility status.
* The TypeScript dashboard checks API compatibility on startup.
* Bundled dashboard assets and server API contract are tested together in CI.
* Future generated TypeScript clients are built from the documented API contract.

## 13.5 Dashboard UX principles

The dashboard should be useful to David, not only to Marco and Priya.

Design requirements:

* clear entity names and descriptions
* searchable/filterable entity list
* domain/module navigation
* relationship graph with pan/zoom
* graph table fallback for accessibility and large models
* click-through from entity to relationships and artifacts
* non-technical language for constraints where possible
* visible generation status
* visible unresolved issues
* visible last update timestamp
* clear labels for draft, generated, warning, deprecated, inferred, defaulted, assumption, and unresolved items
* no raw YAML required for stakeholder review
* WCAG 2.2 AA target for Phase 3 dashboard components
* keyboard navigable entity list, graph alternatives, and documentation views

Phase 3 review features:

* Review summary page listing entities, relationships, open questions, warnings, and changed items.
* Export review summary to Markdown.
* Copy stakeholder-safe link to local dashboard route.
* Graph table fallback for accessibility and large models.
* Mark review items as draft, needs-confirmation, confirmed, rejected, or deferred where local write mode is explicitly enabled.
* Store local review state under `.knownow/review_state.json` unless a future explicit metadata-writing workflow is enabled.
* Export a read-only review pack that can be committed or sent to stakeholders.

---

## 14. Policy packs

## 14.1 Purpose

Policy packs encode reusable project standards. They are especially important for consulting reuse and administrator governance.

Early policy packs are declarative and non-mutating. They may validate metadata, provide defaults through explicit resolution, and classify findings, but they cannot execute arbitrary code or rewrite metadata.

Policy packs can define:

* naming conventions
* required standard attributes
* allowed semantic types
* allowed logical types
* semantic type mappings
* default database type mappings
* default dbt materializations
* default quality rules
* documentation conventions
* domain/module requirements
* tag rules
* target version rules
* required owner/steward metadata
* classification and sensitivity rules
* retention policy requirements
* PII handling expectations
* warning/error severity profiles

## 14.2 Default policy pack

Phase 2A includes a built-in default policy profile. Phase 3 expands this into custom/project policy packs and drift workflows.

Default policy configuration:

```yaml
policy:
  pack: dc_standard
  version: "1.0"
```

The default pack should be useful but not overly strict.

## 14.3 Policy severity

Policy findings can be:

* info
* warning
* error
* blocking

Generation proceeds unless findings reach configured blocking thresholds.

## 14.4 Policy engine rules

* External policy packs are data-only in Phase 2 and Phase 3.
* Policy evaluation cannot mutate raw metadata or the canonical graph.
* Policy-provided defaults are applied through an explicit resolution step and recorded in traceability metadata.
* Every generated artifact can report which policy defaults or findings affected it.
* Unsafe or unknown policy pack features fail closed in `--locked` mode.

## 14.5 Policy traceability

The generation manifest records the policy pack name, version, and content hash used for generation.

## 14.6 Policy drift detection

Administrators can detect whether a project is using approved policy and template versions.

`know-now policy status` reports:

* configured policy versions
* locked policy versions
* available policy versions
* approved policy versions
* drift status

Drift classifications:

* none
* patch drift
* minor drift
* major drift
* unknown
* unapproved

## 14.7 Policy explanation

`know-now policy explain <finding-code>` shows:

* rule name
* rule rationale
* severity
* affected object
* remediation examples
* organization-specific explanation text where configured

---

## 15. Extension model

## 15.1 Phase 3: declarative template packs

Phase 3 supports declarative template packs only. These are data-only templates rendered by know-now through `know-now-minijinja-v1`, a restricted MiniJinja-based renderer profile.

Template packs include a manifest:

```yaml
name: internal_api_docs
version: "1.0"
target: docs
renderer:
  kind: know-now-minijinja
  profile: "1"
output_dir: generated/custom/internal_api_docs
permissions:
  filesystem: output_only
  network: none
limits:
  max_templates: 100
  max_template_bytes: 262144
  max_output_files: 100
  max_output_bytes: 10485760
  max_fuel: 50000
  max_include_depth: 8
trust: untrusted
```

Rules:

* no arbitrary code execution
* no network access
* no writes outside declared output root
* no reads outside approved metadata context and template directory
* no reads of environment variables, local secrets, database connections, or host-specific paths
* failures are reported separately
* built-in generators remain isolated from custom template failures unless fail-fast is configured
* template packs are included in lockfile and manifest hashes
* template renderer profile names and versions are included in lockfile and manifest data
* template output paths are validated by the artifact writer
* symlinks are not followed during template output writes
* rendered output size, template count, template byte size, include depth, render fuel, and file count are limited
* template packs are classified as built-in, approved, experimental, or untrusted
* untrusted template packs cannot be used in `--locked` CI mode unless explicitly permitted by policy
* templates receive only the versioned generator contract and explicit render context
* templates never receive raw YAML nodes or internal Rust-only graph structures
* custom packs cannot register functions, filters, tests, or loaders in Phase 3
* only built-in pure know-now filters are available to custom packs
* undefined values are errors
* render fuel limits are enforced
* dynamic include paths are rejected in Phase 3
* static includes are allowed only inside the template pack root
* template inheritance is allowed only when all referenced templates are statically resolvable inside the template pack root
* template diagnostics include template path, line, column, pack name, renderer profile, and generator-contract version
* MiniJinja is an implementation detail of the renderer profile; compatibility is promised for `know-now-minijinja-v1`, not for unrestricted MiniJinja/Jinja behavior

## 15.1.1 Restricted template renderer architecture

The template renderer is implemented in the `know_now_templates` crate.

Pipeline:

```text
TemplatePack manifest
  -> manifest validation
  -> template discovery inside pack root
  -> static template safety validation
  -> renderer profile configuration
  -> MiniJinja compilation
  -> strict, fuel-limited rendering against GeneratorContract
  -> artifact descriptors
  -> artifact writer path-safety validation
  -> atomic promotion
```

Renderer profile defaults:

* renderer profile: `know-now-minijinja-v1`
* undefined behavior: strict
* custom functions: disabled for custom packs
* custom filters: disabled for custom packs
* custom tests: disabled for custom packs
* custom loaders: disabled for custom packs
* dynamic includes: disabled
* static includes: allowed only inside pack root
* network access: none
* environment access: none
* filesystem reads: template files inside pack root only
* filesystem writes: none; templates return artifact descriptors only
* output writes: artifact writer only
* render fuel: required
* output byte limit: required
* template byte limit: required
* include-depth limit: required
* file-count limit: required

Built-in pure filters may include:

* `snake_case`
* `kebab_case`
* `pascal_case`
* `upper_case`
* `lower_case`
* `indent`
* `sort_by`
* `join`
* `default`
* `json`
* `yaml`
* `markdown_escape`
* `html_escape`

Forbidden filters and helpers:

* current time
* random values
* environment lookup
* filesystem lookup
* network fetch
* process execution
* database access
* host-specific path expansion

Rules:

* Template rendering must be deterministic for identical input.
* Template packs cannot write directly to project files.
* The artifact writer owns path validation, ownership markers, manual-edit detection, stale artifact handling, and atomic promotion.
* Renderer diagnostics are available in text and JSON.
* Renderer-profile compatibility changes require fixture diffs and release notes.
* The renderer profile is the public compatibility surface, not MiniJinja's complete feature set.

## 15.2 Phase 4: external generator protocol

Future external generators may communicate with know-now via JSON stdin/stdout. They must be explicitly enabled and permissioned.

Rules:

* External generators receive versioned generator contract JSON.
* External generators return artifact descriptors and content through a restricted protocol.
* External generators cannot write directly to project files.
* The artifact writer remains responsible for path safety and atomic promotion.
* External generators are disabled by default.
* External generator execution is recorded in the manifest with command, version, contract version, input hash, output hash, and trust level.
* External generator stderr/stdout are captured with secret redaction for support bundles.

## 15.3 Phase 5: WASI plugin runtime

If plugin demand grows, know-now may add a WASI runtime for stronger sandboxing and cross-language generator support.

---

## 16. Functional requirements

Requirements use stable domain-prefixed IDs rather than one global FR list.

---

## 16.1 Metadata authoring and validation

### META-001 — YAML metadata definitions

Users can define domains, modules, entities, attributes, relationships, business rules, source mappings, semantic types, logical types, tags, descriptions, governance metadata, open questions, assumptions, and target settings in YAML.

Acceptance:

1. A valid multi-file metadata project passes validation.
2. All constructs are available to downstream generators.
3. Cross-file references resolve correctly.

### META-002 — Logical and semantic types

Users can assign logical and semantic types to attributes.

Acceptance:

1. Phase 2 logical and semantic types are recognized.
2. Logical types influence physical type generation.
3. Semantic types influence at least one generated quality, documentation, dashboard, or fixture artifact.
4. Unknown types produce clear validation errors.

### META-003 — Schema validation diagnostics

Invalid YAML or invalid metadata structure produces source-aware diagnostics.

Acceptance:

1. Error includes file, line, column, YAML path, and message.
2. Common errors include suggested fixes.
3. Exit code is 1 on validation failure.
4. Unsupported YAML features produce explicit diagnostics rather than vague deserialization failures.
5. Duplicate keys include the duplicate key location and the original key location where available.

### META-004 — Semantic validation

The engine validates cross-entity consistency.

Acceptance:

1. Unknown relationship references are detected.
2. Duplicate entity names are detected.
3. Duplicate stable IDs are detected.
4. Invalid business keys are detected.
5. Incomplete source mappings are detected.

### META-005 — Target database settings

Users can configure target database kind and version.

Acceptance:

1. PostgreSQL target version controls generated DDL compatibility.
2. Version-incompatible constructs produce warnings or errors.
3. Target settings are recorded in the manifest.

### META-006 — Metadata files are not rewritten in early phases

Early-phase commands do not modify metadata files.

Acceptance:

1. Running read-only and generation commands results in zero changes under `metadata/`.
2. Tests verify file hashes before and after commands.

### META-007 — Metadata schema versioning

Metadata includes a schema version.

Acceptance:

1. Unsupported major versions are rejected.
2. Supported versions are listed by `know-now version`.
3. Optional non-breaking fields are allowed within a major version.

### META-008 — JSON Schema export

Users can export JSON Schema for IDE validation and autocomplete.

Acceptance:

1. `know-now schema` writes or prints JSON Schema.
2. Schema validates example metadata.
3. Project init configures VS Code schema association where requested.

### META-009 — Stable object IDs

Users can assign stable IDs to metadata objects.

Acceptance:

1. Duplicate IDs fail validation.
2. Diffing uses IDs where available.
3. Manifest links artifacts to metadata object IDs.
4. `know-now id check` reports missing IDs required for migration-safe workflows.
5. `know-now id suggest` proposes deterministic IDs without modifying metadata.
6. `know-now id backfill --apply` is a future explicit metadata-writing command and must preview changes before writing.

### META-010 — Policy pack validation

Users can apply policy packs.

Acceptance:

1. Policy violations are reported with severity.
2. Policy defaults influence generation where configured.
3. Manifest records policy pack name, version, and hash.
4. Policy evaluation is side-effect-free.
5. Policy-provided defaults are traceable.
6. Unknown policy features fail validation unless explicitly allowed by an experimental profile.

### META-011 — Domains and modules

Users can group metadata into domains and modules.

Acceptance:

1. Entities can reference domains and modules.
2. Unknown domain/module references fail validation.
3. Dashboard and docs group objects by domain/module where available.

### META-012 — Governance metadata

Users can add governance metadata to entities and attributes.

Acceptance:

1. Governance fields validate against the metadata schema.
2. Governed profiles can require owner, steward, classification, or retention fields.
3. Dashboard and generated docs display governance metadata clearly.
4. Policy packs can produce governance completeness warnings.

### META-013 — Open questions and assumptions

Users can capture open questions and assumptions as first-class metadata.

Acceptance:

1. Open questions and assumptions validate.
2. Related object references resolve.
3. Generated docs and dashboard review pages include them.
4. Strict governed profiles can block generation on blocking unresolved questions.

---

## 16.2 Artifact generation

### GEN-001 — PostgreSQL DDL generation

Users can generate PostgreSQL DDL from metadata.

Acceptance:

1. DDL includes tables, columns, primary keys, foreign keys, not-null constraints, unique constraints, check constraints, and indexes where configured.
2. Generated DDL parses successfully.
3. Generated DDL executes against CI PostgreSQL targets.

### GEN-002 — dbt project generation

Users can generate a dbt project.

Acceptance:

1. Source mappings produce dbt source definitions and staging models.
2. Entities produce mart models.
3. Generated project passes configured dbt validation where enabled.

### GEN-003 — dbt tests and quality contracts

Users can generate dbt tests and provider-neutral quality contracts.

Acceptance:

1. Constraints produce dbt tests where possible.
2. Semantic types produce relevant checks.
3. Quality contracts include all generated checks.
4. Unsupported provider-specific checks are represented in the neutral contract.

### GEN-004 — Documentation generation

Users can generate Markdown documentation.

Acceptance:

1. Every entity has generated documentation.
2. Attribute tables include type, semantic type, constraints, governance metadata, and description.
3. Documentation renders correctly in standard Markdown viewers.
4. Missing descriptions create documentation quality warnings.
5. Open questions and assumptions are included where configured.

### GEN-005 — Mermaid ER diagram generation

Users can generate Mermaid ER diagrams.

Acceptance:

1. All entities appear.
2. All relationships appear.
3. Cardinality is represented.
4. Mermaid syntax validates.

### GEN-006 — Generate all artifacts in one command

Users can generate all configured artifacts with `know-now generate`.

Acceptance:

1. No target flags are required for default generation.
2. Output count matches the generation plan.
3. Manifest is written after successful generation.

### GEN-007 — Deterministic output

Identical metadata and configuration produce byte-identical deterministic generated output.

Acceptance:

1. Consecutive runs produce identical hashes for deterministic generated artifacts.
2. Cross-platform snapshot tests pass.
3. Volatile run details are stored outside deterministic generated output.

### GEN-008 — Validate before writing

Invalid generated output is never promoted.

Acceptance:

1. Malformed generated SQL blocks promotion.
2. Failing artifact validation leaves previous output unchanged.
3. Error report identifies generator, artifact, and metadata object where possible.

### GEN-009 — Dry run

Users can preview generation without writing files.

Acceptance:

1. `know-now generate --dry-run` writes no files.
2. Output includes planned paths and artifact summaries.
3. JSON format is parseable in CI.

### GEN-010 — Generation manifest

Every successful generation writes a deterministic manifest.

Acceptance:

1. Manifest includes engine version, metadata schema version, generator contract version, input hash, lockfile hash, target versions, policy pack, artifact paths, artifact hashes, and warnings.
2. Manifest is machine-readable.
3. Dashboard can read and display the manifest.
4. Manifest excludes volatile timestamps.

### GEN-011 — Atomic generation

Generation is atomic.

Acceptance:

1. Artifacts are written to staging first.
2. Final output is promoted only after full success.
3. Failure preserves prior generated output.

### GEN-012 — Explain generated artifacts

Users can understand why a generated artifact exists and what metadata affected it.

Acceptance:

1. `know-now explain generated/ddl/postgres/schema.sql` shows generator, metadata objects, policy rules, template inputs, validation gates, and relevant source spans.
2. `know-now explain ent_customer` shows affected artifacts.
3. Dashboard artifact pages expose the same traceability information.
4. JSON output is available for CI and editor integrations.

### GEN-013 — Manual edit detection

The engine detects manually edited generated files before replacement.

Acceptance:

1. Generated artifacts include ownership markers where supported.
2. Promotion blocks if existing generated file content does not match the previous manifest.
3. Users can explicitly accept overwrite.

### GEN-014 — Incremental generation

The engine can regenerate only affected artifacts.

Acceptance:

1. `generate --changed` regenerates artifacts affected by changed metadata.
2. Incremental and full generation produce identical final output.
3. Cache corruption is detected and recovered.

### GEN-015 — Deterministic demo fixture generation

Users can generate small deterministic fixture datasets for bundled examples and CI validation.

Acceptance:

1. Fixture generation is optional.
2. Fixture data is deterministic for identical metadata and seed.
3. Fixture generation respects logical types, semantic types, required fields, and simple relationships.
4. Fixture data is clearly marked as synthetic/demo data.
5. Production projects do not generate fixture data unless explicitly configured.

---

## 16.3 Project lifecycle and change safety

### LIFE-001 — Project initialization

Users can initialize a project.

Acceptance:

1. `know-now init` creates standard directories and config.
2. Generated config passes validation.
3. Git policy is configured.
4. Project profiles configure sensible defaults for common user types.

### LIFE-002 — Demo project

Users can initialize a demo project.

Acceptance:

1. `know-now init --demo` creates complete demo metadata.
2. Demo validates.
3. Demo generates all Phase 2 artifacts.
4. Demo includes at least eight entities, self-reference, many-to-many relationship, semantic types, logical types, source mappings, business rules, governance metadata, open questions, domains, and modules.

### LIFE-003 — Metadata diff

Users can compare metadata versions.

Acceptance:

1. `know-now diff` categorizes added, modified, removed, renamed, compatible, breaking, destructive, and ambiguous changes.
2. Output supports text and JSON.
3. Stable IDs are used where available.

### LIFE-004 — Migration SQL stubs

Users can generate migration SQL stubs for safe changes.

Acceptance:

1. Additive columns produce `ALTER TABLE ADD COLUMN` statements.
2. Confirmed renames produce rename statements.
3. Ambiguous/destructive changes require explicit confirmation and may produce manual stubs.

### LIFE-005 — Deprecation issues

Breaking metadata changes create tracked issues.

Acceptance:

1. Issues include affected object, change type, suggested fix, and status.
2. Issues persist under `.knownow/`.
3. `know-now issues` lists unresolved issues.

### LIFE-006 — Independent projects

Multiple projects operate independently.

Acceptance:

1. No shared mutable state between project directories.
2. Running generation in one project cannot affect another.
3. Project root detection is deterministic.

### LIFE-007 — Change summary

Users can see changes since last generation.

Acceptance:

1. Summary lists changed objects and affected artifacts.
2. Output supports text and JSON.
3. Dashboard displays latest summary.

### LIFE-008 — Lockfile consistency

Users can enforce reproducible generation through `know-now.lock`.

Acceptance:

1. `generate --locked` fails when resolved versions differ from lockfile.
2. `lock update` refreshes resolved versions explicitly.
3. Manifest records lockfile hash.
4. Lockfile excludes local paths and volatile machine-specific state.
5. Generator contract upgrades are classified as compatible, warning, or breaking.
6. CI can fail specifically on lockfile drift.

### LIFE-009 — Impact analysis

Users can identify generated artifacts and custom references affected by metadata changes.

Acceptance:

1. `diff --impact` reports affected generated artifacts.
2. `diff --impact --scan-custom` scans custom references.
3. Findings distinguish exact stable-ID references, name references, and heuristic matches.

---

## 16.4 Customization and extension

### EXT-001 — User-owned custom space

Files in `custom/` are never overwritten.

Acceptance:

1. Pre/post hashes prove custom files are unchanged by generation.
2. Engine write operations are blocked from `custom/`.
3. Custom dbt files can coexist with generated dbt project paths.

### EXT-002 — Quality rule overrides

Users can override generated quality rules.

Acceptance:

1. Same-named custom rule takes precedence where configured.
2. Override is recorded in manifest or generation warning.
3. Other generated rules are unaffected.

### EXT-003 — Reference scanning

Users can scan custom files for references affected by metadata changes.

Acceptance:

1. Renamed/removed objects are searched under `custom/`.
2. Report includes file, line, and matched reference.
3. Scan performance target is met.
4. `diff --impact --scan-custom` includes custom reference findings.
5. Findings distinguish exact stable-ID references, name references, and heuristic matches.

### EXT-004 — Declarative template packs

Users can add declarative template packs.

Acceptance:

1. Template pack manifest is required.
2. Output is confined to the declared generated subdirectory.
3. No arbitrary code execution is allowed.
4. Failures are isolated and reported clearly.
5. Template pack versions and hashes are recorded in lockfile and manifest.
6. Template packs use the `know-now-minijinja-v1` renderer profile.
7. Rendering uses strict undefined behavior.
8. Rendering enforces fuel, template-size, output-size, include-depth, and file-count limits.
9. Custom packs cannot register native functions, filters, tests, or loaders.
10. Dynamic include paths are rejected.
11. Static includes cannot escape the template pack root.
12. Renderer diagnostics include template file, line, column, template pack, renderer profile, and contract version.
13. Renderer output is passed to the artifact writer rather than written directly.
14. Trust level is visible in `doctor`, manifest, and dashboard health views.


---

## 16.5 Developer workflow

### CLI-001 — Installation

Users can install know-now via documented binary and Cargo-compatible paths.

Acceptance:

1. Prebuilt binary installation works on supported OSes.
2. Cargo source-build fallback works with `cargo install --locked`.
3. Installed CLI runs `know-now version` successfully.

### CLI-002 — Shell completion

Users can install shell completion.

Acceptance:

1. Completion is available for bash, zsh, fish, and PowerShell where supported.
2. Completion suggests commands and flags.

### CLI-003 — Help output

Every command has useful help.

Acceptance:

1. `--help` includes description, usage, options, and examples.
2. Help renders in standard terminals.

### CLI-004 — Verbosity controls

Users can control output verbosity.

Acceptance:

1. Default is concise.
2. `--verbose` adds pipeline steps and timings.
3. `--debug` adds diagnostic details.

### CLI-005 — Output formats

Users can choose text, JSON, SARIF, or quiet output.

Acceptance:

1. Text is human-readable.
2. JSON is stable and parseable.
3. SARIF is available for diagnostics-producing commands.
4. Quiet emits only errors.

### CLI-006 — CI validation

Users can run validation in CI.

Acceptance:

1. Exit code 0 on success.
2. Exit code 1 on validation errors.
3. JSON diagnostics are suitable for CI annotations.
4. SARIF diagnostics are suitable for supported code-scanning workflows.

### CLI-007 — Offline operation

Core CLI commands work offline.

Acceptance:

1. No network required for validate/check/generate/diff/issues/schema/version.
2. No telemetry is sent by default.

### CLI-008 — Programmatic API

Users can use know-now programmatically through Rust crates and JSON contracts.

Acceptance:

1. Rust examples compile.
2. CLI JSON schemas are documented.
3. TypeScript client can consume local API schemas.

### CLI-009 — Doctor command

Users can diagnose project health.

Acceptance:

1. `doctor` reports project/toolchain/config health.
2. Supports text and JSON.
3. Does not require network unless update checks are explicitly enabled.

### CLI-010 — Check command

Users can run one command that performs the recommended local or CI verification workflow.

Acceptance:

1. `know-now check` runs metadata validation, policy validation, generation planning, dry-run artifact generation, and configured artifact validation.
2. It writes no generated artifacts unless explicitly requested.
3. It supports `--format text|json|sarif|quiet`.
4. It exits with code 0 only when the project is safe to generate.
5. CI examples use `know-now check --format json --locked`.
6. Optional code-scanning examples use `know-now check --format sarif --locked`.

### CLI-011 — Configuration inspection

Users can inspect resolved project configuration.

Acceptance:

1. `config inspect` shows config, profile, policy, lockfile state, target versions, generator capabilities, and effective defaults.
2. Output supports text and JSON.
3. It does not modify files.

---

## 16.6 Dashboard and visibility

### DASH-001 — Entity list

Stakeholders can browse entities.

Acceptance:

1. List displays entity name, description, type, tags, domain/module, governance labels, and attribute count.
2. Search and filters are available.
3. Load performance target is met.

### DASH-002 — Entity detail

Stakeholders can view entity details.

Acceptance:

1. Attributes, constraints, semantic types, logical types, business keys, source mappings, governance metadata, open questions, and relationships are shown.
2. Non-technical explanations are used where possible.

### DASH-003 — Relationship graph

Stakeholders can explore relationships visually.

Acceptance:

1. Graph renders all entities and relationships.
2. Pan and zoom are supported.
3. Click and hover reveal relationship details.
4. Accessible table fallback is available.

### DASH-004 — Generation status

Stakeholders can view generation status.

Acceptance:

1. Last generation timestamp from volatile run state, engine version, metadata version, artifact counts, warnings, and errors are shown.
2. Failed generations show summary and links to diagnostics.

### DASH-005 — Documentation viewer

Stakeholders can read generated documentation.

Acceptance:

1. Markdown renders as HTML.
2. Mermaid diagrams render visually.
3. Navigation by entity/domain/module is available.

### DASH-006 — Artifact traceability

Users can trace artifacts back to metadata.

Acceptance:

1. Entity pages link to generated artifacts.
2. Artifact pages link to metadata object IDs.
3. Manifest data powers the traceability view.

### DASH-007 — Project health/admin view

Administrators can inspect project health.

Acceptance:

1. View shows metadata schema version, generator contract version, policy pack, target versions, template packs, template renderer profiles, issue counts, policy drift, last generation status, and warnings.
2. Security warnings are visible.

### DASH-008 — Local API

Dashboard consumes local API endpoints.

Acceptance:

1. GET endpoints return project, entity, graph, documentation, issue, policy, traceability, review, and manifest data.
2. API is documented.
3. Generation endpoint is disabled unless explicitly enabled.
4. API requests require a local session.
5. Write requests require explicit server opt-in and request-level confirmation.
6. Dashboard displays binding, token/session, and write-mode security state.

### DASH-009 — Stakeholder review summary

Stakeholders can review a concise project summary.

Acceptance:

1. Review summary lists entities, relationships, open questions, warnings, assumptions, and changed items.
2. Summary can be exported to Markdown.
3. View is understandable without reading YAML or SQL.
4. Review items have explicit status values.
5. Exported review packs include manifest hash, metadata hash, and generation status.
6. Review exports never require cloud services.

---

## 16.7 Administration and governance

### ADMIN-001 — Policy pack version pinning

Administrators can pin policy pack versions.

Acceptance:

1. Project config records policy pack and version.
2. Validation fails or warns if policy pack is unavailable.
3. Manifest records policy pack version and hash.

### ADMIN-002 — Template pack visibility

Administrators can see active template packs.

Acceptance:

1. `doctor` and dashboard list template pack name, version, target, renderer profile, trust level, and status.
2. Unsafe or invalid template packs are blocked.
3. Unsupported renderer profiles are reported before generation.

### ADMIN-003 — Audit log

CLI operations produce a local audit log.

Acceptance:

1. Audit log records command, timestamp, project root, metadata hash, lockfile hash, engine version, and result.
2. Secrets are never logged.
3. Audit log is local and can be disabled only with explicit configuration.

### ADMIN-004 — Compatibility matrix

Project health includes compatibility status.

Acceptance:

1. Supported dbt validation modes and PostgreSQL target versions are documented.
2. `doctor` warns about unsupported configured versions.
3. CI tests cover documented support combinations.

### ADMIN-005 — Policy drift detection

Administrators can detect whether a project is using approved policy and template versions.

Acceptance:

1. `know-now policy status` reports configured, locked, available, and approved policy/template versions.
2. Drift is classified as none, patch drift, minor drift, major drift, unknown, or unapproved.
3. Dashboard health view shows drift status.
4. JSON output can be aggregated across repositories by external scripts.

### ADMIN-006 — Policy explanation

Users can understand why a policy finding occurred.

Acceptance:

1. `know-now policy explain <finding-code>` shows the rule, rationale, severity, affected object, and remediation examples.
2. Dashboard findings link to explanations.
3. Policy packs can include organization-specific explanation text.

### ADMIN-007 — Support bundle

Users can create a sanitized support bundle for debugging.

Acceptance:

1. `know-now support bundle` creates a zip/tar archive with doctor output, command logs, manifests, config summary, compatibility status, and sanitized diagnostics.
2. Metadata inclusion is opt-in.
3. Secrets and environment variables are redacted.
4. The bundle includes know-now version, OS/architecture, lockfile hash, policy/template versions, and last generation result.
5. Users can preview bundle contents before writing.

### ADMIN-008 — Multi-project governance scan

Administrators can scan multiple know-now projects and aggregate governance status.

Acceptance:

1. `know-now admin scan <path>` discovers project roots recursively.
2. Output includes project ID, engine version, metadata schema version, lockfile status, policy pack status, template pack status, template renderer profile status, target versions, issue counts, and last generation status.
3. JSON output can be consumed by external dashboards or CI jobs.
4. Scan does not modify projects.
5. Scan can be configured with an approved-version catalog.

### ADMIN-009 — Approved-version catalog

Administrators can define approved policy, template, generator, target, and metadata schema versions.

Acceptance:

1. Catalog files are declarative and committed to a governance repository.
2. `admin catalog check` validates catalog syntax.
3. `policy status` and `admin scan` classify drift against the catalog.
4. Unknown or unapproved packs are clearly reported.

Example catalog shape:

```yaml
approved:
  engines:
    know-now: ["1.0.x"]
  metadata_schema_versions: ["1.0"]
  generator_contract_versions: ["1.0"]
  policies:
    dc_standard: ["1.0.x"]
  templates:
    internal_api_docs: ["1.0.x"]
  template_renderers:
    know-now-minijinja-v1: ["1"]
  targets:
    postgres:
      floor: "16"
      allowed: ["16", "17", "18"]
```

---

## 17. Non-functional requirements

## 17.1 Performance

| ID | Requirement | Target |
| -- | ----------- | ------ |
| NFR-P1 | CLI startup | <500ms |
| NFR-P2 | Help output | <200ms |
| NFR-P3 | Metadata validation, 100 entities | <2s |
| NFR-P4 | Full generation, 10 entities | <5s |
| NFR-P5 | Full generation, 100 entities | <60s |
| NFR-P6 | Incremental single-entity regeneration, 100-entity project | <10s |
| NFR-P7 | Custom reference scan, 100 files | <5s |
| NFR-P8 | Dashboard FCP | <1s |
| NFR-P9 | Entity list API, 100 entities | <200ms p95 |
| NFR-P10 | Benchmark regression | >20% regression fails CI unless approved |
| NFR-P11 | Peak memory, 100-entity project | <512MB excluding external toolchains |
| NFR-P12 | Pipeline timing breakdown | `--verbose` and JSON output include stage timings |
| NFR-P13 | Parallel generation safety | Independent generators may run concurrently but must produce deterministic artifact ordering |

Stage budgets should be tracked for:

* metadata discovery and parsing
* semantic validation
* policy validation
* contract projection
* generation planning
* artifact generation by target
* artifact validation
* manual edit detection
* atomic writing
* dashboard API serialization

## 17.2 Security

| ID | Requirement |
| -- | ----------- |
| NFR-S1 | SQL/DDL generation uses typed IR and emitters; raw interpolation of identifiers/literals is forbidden. |
| NFR-S2 | Metadata identifiers are validated before generation. |
| NFR-S3 | Invalid generated output is never promoted. |
| NFR-S4 | Secrets and connection strings are not stored in metadata. |
| NFR-S5 | Rust and TypeScript dependencies are audited for licenses, advisories, duplicate dependencies, and banned packages. |
| NFR-S6 | Templates producing generated output have license review metadata. |
| NFR-S7 | Dashboard and docs sanitize metadata strings against XSS. |
| NFR-S8 | Declarative templates have no arbitrary code execution. |
| NFR-S9 | Credentials used by future introspection features are environment-scoped and never logged. |
| NFR-S10 | Lockfiles are committed and verified for release builds. |
| NFR-S11 | Generated files include provenance headers where format supports comments. |
| NFR-S12 | CLI operations produce local audit log entries. |
| NFR-S13 | YAML parsing uses `serde-saphyr` as the primary parser/deserializer and enforces file-size, nesting, and resource-exhaustion limits. |
| NFR-S14 | Local server binds to localhost by default. Network binding requires explicit flag and warning. |
| NFR-S15 | Local API uses per-session token/session and strict CORS by default. |
| NFR-S16 | Write endpoints are disabled unless explicitly enabled. |
| NFR-S17 | Template output paths are validated by the artifact writer. |
| NFR-S18 | Support bundles redact secrets and sensitive local environment details. |
| NFR-S19 | YAML parser dependency choice is reviewed for maintenance, security, source-span support, deterministic behavior, and no C/FFI parser dependency unless explicitly approved. |
| NFR-S20 | Local dashboard uses secure response headers and a restrictive content security posture where practical. |
| NFR-S21 | YAML parser configuration or pre-scan validation rejects anchors, aliases, merge keys, custom tags, include directives, duplicate keys, multi-document files, excessive nesting, and excessive file sizes before semantic validation. |
| NFR-S22 | Template rendering uses the restricted `know-now-minijinja-v1` profile with strict undefined behavior and no custom native extension registration for custom packs. |
| NFR-S23 | Template rendering enforces fuel, output-file-count, template-size, include-depth, and output-size limits. |
| NFR-S24 | Template includes, inheritance, and partials cannot escape the template pack root. |
| NFR-S25 | Template packs cannot read environment variables, execute processes, open network connections, access databases, or write files directly. |

## 17.3 Reliability

| ID | Requirement |
| -- | ----------- |
| NFR-R1 | Identical input produces byte-identical deterministic output across supported OSes. |
| NFR-R2 | Test coverage minimum 90% overall, 95% for core metadata/generation crates. |
| NFR-R3 | Snapshot tests cover all built-in generation targets. |
| NFR-R4 | Generated DDL executes against real PostgreSQL in CI. |
| NFR-R5 | Generated quality checks are tested against valid/invalid fixture data. |
| NFR-R6 | Metadata files are not modified unless explicitly requested. |
| NFR-R7 | Atomic generation preserves previous output on failure. |
| NFR-R8 | All file I/O uses explicit UTF-8. |
| NFR-R9 | Generated dbt projects compile where dbt validation is configured. |
| NFR-R10 | Incremental generation and full generation produce equivalent output. |
| NFR-R11 | Manually edited generated files are detected before replacement. |
| NFR-R12 | Stale generated artifacts are detected and reported. |
| NFR-R13 | Volatile run state is isolated from deterministic generated artifacts. |
| NFR-R14 | Parser diagnostics are snapshot-tested for common syntax, structure, unsupported-feature, duplicate-key, and parser-budget errors. |

## 17.4 Integration

| ID | Requirement |
| -- | ----------- |
| NFR-I1 | Maintain documented compatibility matrix for supported PostgreSQL and dbt validation modes. |
| NFR-I2 | Generated dbt project includes explicit dbt version constraints where configured. |
| NFR-I3 | JSON Schema is distributed with the CLI and usable by major editors. |
| NFR-I4 | CI includes generated artifact behavioral tests, not only snapshot tests. |
| NFR-I5 | Docker Compose supports full local demo stack. |
| NFR-I6 | Generated artifacts include manifest and provenance for CI traceability. |
| NFR-I7 | dbt validation is adapter-based and capability-detected. |
| NFR-I8 | Local API contract is versioned and consumed by generated TypeScript client. |
| NFR-I9 | Frontend package-manager version is pinned in `web/package.json`; CI verifies installs using the pinned version and committed `pnpm-lock.yaml`. |

## 17.5 Scalability

| ID | Requirement |
| -- | ----------- |
| NFR-SC1 | Metadata model supports 200+ entities within performance targets. |
| NFR-SC2 | New built-in generation targets can be added without modifying existing generators. |
| NFR-SC3 | Multi-file parsing scales approximately linearly. |
| NFR-SC4 | Relationship graph supports up to 5x relationship-to-entity ratio within performance targets. |
| NFR-SC5 | Dashboard remains navigable for large projects through search, filters, domain grouping, and table fallbacks. |

## 17.6 Maintainability

| ID | Requirement |
| -- | ----------- |
| NFR-M1 | Generator modules have no direct cross-dependencies. |
| NFR-M2 | Rust public APIs are typed and documented. |
| NFR-M3 | TypeScript uses strict mode. |
| NFR-M4 | CI runs formatting, linting, type checking, tests, and dependency policy checks. |
| NFR-M5 | Architecture fitness tests verify generator independence. |
| NFR-M6 | Dashboard/server API contract compatibility is tested in CI. |
| NFR-M7 | Frontend dependency installation uses committed `pnpm-lock.yaml`, pinned pnpm version, and frozen-lockfile CI. |
| NFR-M8 | YAML parser dependency selection is verified during dependency policy checks, release readiness, and architecture fitness tests. |
| NFR-M9 | Template renderer profile compatibility is tested with fixtures and release-note diff summaries. |

Architecture fitness tests must verify:

* generator crates do not depend on YAML parser crates
* YAML parser dependencies are isolated to `know_now_metadata`
* generator crates only receive validated graph/generator contract inputs
* built-in generators do not write files directly
* all writes go through the artifact writer
* artifact writer does not depend on individual generator crates
* diagnostics can be emitted in text, JSON, and SARIF without generator-specific code
* lockfile resolution is isolated from artifact generation
* policy validation cannot mutate metadata
* local server write endpoints are disabled unless explicitly enabled
* TypeScript dashboard consumes documented API contracts only
* dashboard/server API contract compatibility is tested in CI
* template packs cannot write files directly
* `know_now_templates` produces artifact descriptors and does not bypass writer path-safety rules
* template rendering cannot access raw YAML parser types
* custom template packs cannot register native MiniJinja functions, filters, tests, or loaders
* unsupported renderer profile changes fail compatibility tests

## 17.7 Portability

| ID | Requirement |
| -- | ----------- |
| NFR-PO1 | CLI works on Linux, macOS, and Windows native/WSL. |
| NFR-PO2 | Path handling supports spaces, Unicode, and platform separators. |
| NFR-PO3 | Generated output uses stable line endings and UTF-8 encoding. |

## 17.8 Supply-chain security

| ID | Requirement |
| -- | ----------- |
| NFR-SS1 | Release binaries are built in CI from tagged commits. |
| NFR-SS2 | Checksums are published for every binary/archive. |
| NFR-SS3 | Artifact attestations are published for binaries and container images where practical. |
| NFR-SS4 | SBOMs are published for release artifacts where practical. |
| NFR-SS5 | Cargo metadata supports prebuilt binary discovery where practical. |
| NFR-SS6 | Source-build fallback is documented as `cargo install --locked know-now --version <version>`. |
| NFR-SS7 | Release CI verifies install paths produce a working `know-now version`. |

---

## 18. Distribution and installation

## 18.1 End-user installation

Primary installation should use prebuilt binaries for speed and reliability.

Supported paths:

```bash
# Recommended where available; requires cargo-binstall and published binary metadata
cargo binstall know-now

# Source-build fallback
cargo install --locked know-now

# Direct binary download
# Provided through release assets for Linux, macOS, and Windows.
```

Rules:

* `cargo binstall` is documented as an optional binary-install path, not as built-in Cargo behavior.
* Direct release downloads remain available for users who do not use cargo-binstall.
* Source-build fallback must work with `cargo install --locked`.
* Unsupported installation paths should not be documented in the PRD.

## 18.2 Contributor setup

Contributors use:

```bash
cargo build
cargo test
cargo xtask check
```

Dashboard contributors use:

```bash
cd web
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

The dashboard package manager is `pnpm`. The repository commits `web/pnpm-lock.yaml` and pins the exact package-manager version in `web/package.json` using the `packageManager` field.

Frontend package-manager policy:

* Use `pnpm`.
* Commit `web/pnpm-lock.yaml`.
* Do not commit `package-lock.json`, `yarn.lock`, `bun.lock`, or `bun.lockb` under `web/`.
* Pin the exact pnpm version in `web/package.json`.
* Use frozen-lockfile installs in CI.
* Keep the Node.js version requirement aligned with the active Vite requirement.
* Do not make Corepack availability a hard assumption; CI must activate or install the pinned pnpm version explicitly.

Example `web/package.json` baseline:

```json
{
  "name": "@know-now/dashboard",
  "private": true,
  "type": "module",
  "packageManager": "pnpm@<pinned-version>",
  "engines": {
    "node": ">=<node-version-required-by-current-vite>"
  },
  "scripts": {
    "dev": "vite",
    "typecheck": "tsc --noEmit",
    "test": "vitest run",
    "build": "vite build"
  }
}
```

## 18.3 Docker Compose

Docker Compose supports a full local demo stack:

* know-now local server
* dashboard
* PostgreSQL
* optional dbt runner image
* demo metadata/project mount

---

## 19. Documentation strategy

## 19.1 Repository documentation

* README quick start.
* Install guide.
* Demo walkthrough.
* Metadata YAML reference.
* YAML authoring subset guide.
* Logical type and semantic type guide.
* Governance metadata guide.
* Open questions and assumptions guide.
* Domain/module modeling guide.
* CLI reference.
* Generated output reference.
* Ownership boundary guide.
* Artifact writer and regeneration safety guide.
* dbt customization guide.
* dbt validation adapter guide.
* Policy pack guide.
* Template pack guide.
* CI/CD recipes.
* Lockfile and reproducibility guide.
* Troubleshooting and `doctor` guide.
* `explain` and traceability guide.
* Administrator scan and approved-version catalog guide.
* Architecture overview.
* Contributing guide.
* Agent / contributor invariants (`AGENTS.md`).
* Architecture Decision Records (`docs/adr/`).
* Repository layout reference (`docs/dev/repo-layout.md`).
* Versioning policy reference (`docs/dev/versioning.md`).
* Commit conventions reference (`docs/dev/commit-conventions.md`).
* Compatibility matrix.
* Changelog.

## 19.2 Documentation site

Phase 2 should include GitHub Markdown docs. A dedicated documentation site can be added if it does not slow the CLI launch.

## 19.3 Generated project documentation

Generated docs include:

* project summary
* domain/module pages
* entity pages
* attribute pages or tables
* relationship docs
* source mapping docs
* governance docs
* quality rule docs
* open questions
* assumptions
* documentation quality warnings
* ER diagrams
* artifact manifest
* generation provenance
* review summary

## 19.4 Repository and contributor documentation

Beyond the user-facing reference in §19.1, the repository maintains a small set of contributor and maintainer documents at well-known locations so that humans and AI assistants can orient quickly without re-reading the PRD.

* `README.md` (repository root): project overview, status, and entry-point links.
* `AGENTS.md` (repository root): invariants and conventions for humans and AI agents working in the repo. Lists architecture invariants derived from §4, §8, §9, and §17.6, banned dependencies, and the workflow rules used by maintainers (Beads issue tracking, BMAD Dev Story Workflow).
* `CONTRIBUTING.md` (repository root): human contributor guide covering setup, build/test/lint commands, commit conventions, and PR expectations.
* `docs/README.md`: documentation index.
* `docs/adr/` — Architecture Decision Records. Significant architectural choices are recorded as ADRs with status, context, alternatives, and consequences. The ADR process is documented in `docs/adr/README.md`. The PRD §24 decisions table summarizes high-level decisions; ADRs explain them and record supersession.
* `docs/dev/repo-layout.md`: the **repository** layout (distinct from the **generated consumer project** layout in §9.1).
* `docs/dev/versioning.md`: maintainer reference for the multiple versioned compatibility surfaces in this product (engine, metadata schema, generator contract, local API contract, lockfile schema, renderer profile, policy pack, template pack).
* `docs/dev/commit-conventions.md`: Conventional Commits guide tailored to this repo, including breaking-change marking and footer fields.

Repository-conventions rules:

* The PRD remains the single source of truth. Where another document conflicts with the PRD, the PRD wins, and the conflicting document is updated in the same change.
* New architectural decisions are recorded as ADRs before they become load-bearing in code. The PRD §24 table is updated when a decision is durable enough to affect product or scope.
* Commit messages follow Conventional Commits. Breaking changes to versioned compatibility surfaces use the `!` marker and a `BREAKING CHANGE:` footer.
* `AGENTS.md` is kept aligned with the architecture invariants enforced by the architecture fitness tests in §17.6. If a fitness test is added or changed, `AGENTS.md` is updated in the same PR.

---

## 20. CI/CD and release quality

## 20.1 CI checks

Required CI checks:

* Rust formatting.
* Rust linting.
* Rust tests.
* Rust coverage.
* TypeScript typecheck.
* TypeScript lint/test/build.
* Dependency policy checks.
* License checks.
* Snapshot tests.
* Property-based tests for metadata validation and diff classification.
* Mutation-style tests for policy pack rules.
* Generated DDL execution against PostgreSQL.
* dbt compile for generated demo project where dbt validation is configured.
* Mermaid syntax validation.
* JSON Schema validation.
* Benchmark suite.
* Architecture fitness tests.
* Cross-platform test matrix.
* Compatibility fixture diff suite.

Frontend CI uses pnpm:

```bash
cd web
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
```

Frontend CI rules:

* CI must use the pinned pnpm version from `web/package.json`.
* CI should install or activate that pinned pnpm version explicitly rather than assuming Corepack is bundled with the active Node.js version.
* `pnpm-lock.yaml` changes must be reviewed like other dependency updates.
* Frontend build artifacts are treated as dashboard assets and versioned alongside the Rust server API contract.

Property-based tests should cover:

* random valid metadata graphs
* unknown references
* duplicate IDs
* rename/add/remove ambiguity
* deterministic ordering
* path normalization across operating systems
* validator stability under multi-file metadata layouts

Parser fixture tests should cover:

* `valid_minimal.yml`
* `valid_multifile_project.yml`
* `duplicate_key.yml`
* `anchor.yml`
* `alias.yml`
* `merge_key.yml`
* `custom_tag.yml`
* `include_directive.yml`
* `multi_document.yml`
* `deep_nesting.yml`
* `large_file.yml`
* `bad_scalar_type.yml`
* `unknown_field.yml`

Each invalid parser fixture snapshots both text and JSON diagnostics.

## 20.2 Compatibility fixtures

Compatibility fixtures include:

* minimal project
* ecommerce demo
* governed project
* missing-ID project
* rename-heavy project
* large 100-entity project
* documentation-quality-warning project
* policy-drift project
* dbt existing-stack project
* template-pack project
* template-pack strict-undefined failure
* template-pack dynamic-include rejection
* template-pack path-escape rejection
* template-pack fuel-exhaustion failure
* template-pack output-size failure
* template-pack untrusted-in-locked-mode failure

For each release, CI produces a fixture diff summary when generated outputs change.

Release notes must classify generated output changes as:

* expected formatting change
* metadata schema change
* generator behavior change
* policy default change
* bug fix
* breaking change

## 20.3 Release artifacts

Release should produce:

* Linux binary.
* macOS binary.
* Windows binary.
* checksums.
* artifact attestations where practical.
* SBOM where practical.
* crate release where appropriate.
* Docker image where appropriate.
* changelog.
* migration notes for metadata/generator contract changes.
* generated fixture diff summary if outputs changed.
* cargo-binstall-compatible package metadata where practical.
* installation verification snippets for each supported platform.

Release requirements:

* Release binaries are built in CI from tagged commits.
* Checksums are published for every binary/archive.
* Artifact attestations are published for binaries and container images where practical.
* SBOMs are published for release artifacts where practical.
* Cargo metadata supports prebuilt binary discovery where practical.
* Source-build fallback is documented as `cargo install --locked know-now --version <version>`.
* Release CI verifies that direct binary install, cargo-binstall install, and Cargo source install all produce a working `know-now version`.

## 20.4 Compatibility matrix

The repository maintains a compatibility matrix for:

* know-now engine version
* metadata schema version
* generator contract version
* template renderer profile versions
* PostgreSQL target versions
* dbt validation modes, detected engine families, versions, adapters, and generated target compatibility
* dashboard browser support
* frontend build toolchain: Node.js version, pnpm version, Vite version, TypeScript version
* supported OS/architecture combinations

Compatibility policy:

* The default PostgreSQL compatibility floor should target actively supported versions with enough remaining support life for new projects.
* Older versions may remain test-covered during early pilots if client environments require it.
* Before public launch and every major release, the team must re-confirm the default compatibility floor against the current PostgreSQL support window.

---

## 21. Risk mitigation

| Category | Risk | Mitigation |
| -------- | ---- | ---------- |
| Technical | YAML round-trip editing can lose comments, formatting, anchors, or ordering if handled naively | Do not rewrite metadata in early phases; use source spans, a constrained YAML subset, and targeted future patches. |
| Technical | YAML parser dependency becomes a maintenance or diagnostics liability | Use `serde-saphyr` as primary parser/deserializer, keep `marked-yaml` as fallback, isolate parser dependencies to `know_now_metadata`, and require dependency-policy review for maintenance, security, source-span, safe-limit, and deterministic-behavior requirements. |
| Technical | Metadata schema becomes unstable after generators depend on it | Separate authoring schema, normalized graph, and versioned generator contract. |
| Technical | dbt ecosystem remains external and mixed-toolchain | Treat dbt as generated/external target; use configurable capability-detected toolchain adapters. |
| Technical | SQL dialect complexity grows | Use typed DDL IR, dialect emitters, compatibility matrix, and live DB tests. |
| Technical | Generated dbt quality is not production-ready | Snapshot tests, compile tests, consulting pilots, readable output guidelines. |
| Technical | Template system becomes unsafe | Declarative templates only in Phase 3; restricted `know-now-minijinja-v1` profile; strict undefined behavior; no custom native extensions; no arbitrary code execution; output path safety; fuel and output limits. |
| Technical | Dashboard scope creep | Keep Phase 3 dashboard read-only by default and local-first. |
| Technical | Incremental generation produces inconsistent output | Require full-vs-incremental equivalence tests. |
| Technical | Manifest determinism is broken by volatile run data | Separate deterministic manifest from `.knownow/` run logs. |
| Technical | Generated file hashing becomes self-referential | Store artifact hashes in manifest; avoid embedding full-file content hash in generated file header. |
| Product | Metadata model too complex for new users | Demo project, guided init, project profiles, JSON Schema autocomplete, great errors, docs, examples. |
| Product | Metadata model not expressive enough | Domains/modules, richer source mappings, governance metadata, open questions, custom space, template packs, policy packs, consulting validation. |
| Product | Stakeholder dashboard is too technical | Plain-language summaries, review mode, accessible relationship table, open-question sections. |
| Market | Weak open-source adoption | Consulting use still validates tool and provides baseline value. |
| Market | Competitors copy approach | Build credibility through real consulting use, examples, policy packs, and open-source community. |
| Resource | Solo developer bottleneck | Phase 1/2A/2B/3 split, Rust workspace boundaries, strict scope, automated tests. |
| Resource | Maintenance burden from many integrations | Keep GX/Snowflake/BigQuery/Data Vault out of early core. |
| IP | License contamination in generated output | Template license review and dependency policy checks. |
| Security | Unsafe local server exposure | Localhost default, session token, strict CORS, explicit network flag, visible warning. |
| Reliability | Failed generation corrupts output | Atomic staging/promote mechanism. |
| Reliability | Users edit generated files manually | Ownership markers and manual-edit detection. |
| Reproducibility | Different machines generate different output | `know-now.lock`, content hashes, deterministic ordering, stable line endings, `--locked` CI mode. |

---

## 22. Recommended build sequence

1. Rust workspace, crate boundaries, CI skeleton, and release binary skeleton.
2. Implement the `serde-saphyr` parser/deserializer spike, define the supported YAML subset, and assess `marked-yaml` as fallback if subset enforcement or diagnostics fail.
3. Build diagnostics model with source spans, error codes, text/JSON output.
4. Build artifact writer proof of concept: path safety, staging, promotion, rollback, and project locks.
5. Build deterministic manifest model and separate volatile run log.
6. Phase 1 architecture spike with minimal metadata model, graph, generator contract, deterministic writer, and one DDL/docs generator.
7. Project layout and `know-now init --demo`.
8. Metadata discovery, parsing, typed model, semantic validation, and canonical `ProjectGraph`.
9. Stable object ID validation and `id suggest`.
10. Generator contract projection and capability registry.
11. CLI foundation: `validate`, `schema`, `version`, `config inspect`, and built-in policy validation.
12. Generation planning and dry-run output.
13. PostgreSQL DDL generator.
14. Markdown documentation generator.
15. `know-now.lock`, `lock check`, `lock update`, and `generate --locked`.
16. Artifact ownership markers and manual-edit detection.
17. `check` command.
18. Snapshot, property, compatibility fixture, and integration test suite.
19. Phase 2A release.
20. dbt generator.
21. dbt tests and quality contract generator.
22. Mermaid docs generator.
23. dbt toolchain capability detection.
24. Generated artifact validation.
25. Phase 2B release.
26. `diff`, stable-ID change classification, and impact scanning.
27. Incremental generation and cache.
28. `issues`, `doctor`, `explain`, and `support bundle`.
29. Policy packs and `policy status`/`policy explain`.
30. Admin multi-project scan and approved-version catalog.
31. Rust local server with hardened token/session/CORS defaults.
32. Versioned local API and generated TypeScript client.
33. TypeScript dashboard.
34. Stakeholder review summary and review export.
35. Declarative template packs using restricted `know-now-minijinja-v1` rendering.
36. Pilot hardening from real Data Champions engagements.
37. Supply-chain release hardening: checksums, attestations, SBOMs, and install verification.

---

## 23. Phase exit criteria

## 23.1 Phase 1 exit criteria — architecture contract spike

Phase 1 is complete when:

* minimal project initializes or loads from fixture
* metadata parses with source spans
* `serde-saphyr` parser spike enforces the know-now YAML subset directly or through an explicit pre-scan/event validation layer
* unsupported YAML features produce source-aware diagnostics
* duplicate keys fail with source-aware diagnostics
* parser file-size, nesting, and resource-exhaustion limits are tested
* canonical graph is built
* versioned generator contract exists
* generator capability declaration exists
* deterministic manifest exists
* volatile run log is separate from deterministic generated output
* one PostgreSQL DDL artifact is generated
* one Markdown documentation artifact is generated
* manifest records artifact hashes and metadata hash
* output is deterministic across consecutive runs
* atomic write proof of concept preserves previous output on failure
* path safety prevents writes outside generated roots
* at least one consulting-style fixture is represented successfully

## 23.2 Phase 2A exit criteria — Rust CLI MVP

Phase 2A is complete when:

* demo project initializes, validates, checks, and generates successfully
* generated PostgreSQL DDL executes in CI
* Markdown documentation generates correctly
* manifest records artifact hashes, metadata hash, and lockfile hash
* `know-now.lock` supports reproducible generation
* no user-owned files are overwritten
* manually edited generated files are detected
* stale generated artifacts are detected
* CLI works on Linux, macOS, and Windows
* JSON Schema is exported and usable in VS Code
* performance targets are met for 10-entity and 100-entity fixtures
* at least one real consulting-style fixture has been modeled successfully

## 23.3 Phase 2B exit criteria — dbt, quality, diagrams, and artifact validation

Phase 2B is complete when:

* generated dbt project compiles in CI where validation is configured
* dbt tests and provider-neutral quality contracts generate correctly
* documentation and Mermaid diagrams generate correctly
* deterministic demo fixture data works for bundled examples
* generated artifact validation gates are usable
* compatibility fixture diff summaries are produced in CI
* dbt toolchain validation is capability-detected

## 23.4 Phase 3 exit criteria — change safety, visibility, and administrator support

Phase 3 is complete when:

* diff and issues workflows handle additive, rename, removal, and ambiguous changes
* impact scanning reports affected generated artifacts and possible custom references
* incremental generation matches full generation output
* `doctor` catches common setup and project health problems
* `explain` traces artifacts to metadata and policy rules
* `support bundle` creates sanitized diagnostic bundles
* `admin scan` aggregates project health across multiple repositories
* approved-version catalog checks are usable
* dashboard renders entity list, entity detail, graph, graph table fallback, docs, manifest, health, review summary, and traceability views
* local server defaults are safe
* custom declarative template packs work safely through the restricted `know-now-minijinja-v1` profile
* policy pack validation is usable
* policy drift detection is usable
* stakeholder review summary is usable in a real review meeting
* at least two real consulting engagements or realistic pilots use the tool successfully

## 23.5 Phase 4 entry criteria

Phase 4 should not start until:

* Phase 3 is stable
* CLI UX is validated by external users
* metadata model has survived real project feedback
* governance needs are clearer
* import/introspection requirements are based on actual workflows
* hosted/control-plane value has credible demand

---

## 24. Decisions and remaining open decisions

| Decision | Current stance | Status or timing |
| -------- | -------------- | ---------------- |
| Open-source license | Architecture remains license-agnostic; generated output must be unencumbered. | Before public launch. |
| Frontend package manager | Use `pnpm`, commit `web/pnpm-lock.yaml`, and pin the exact pnpm version in `web/package.json`. | Decided. |
| Rust YAML parser | Use `serde-saphyr` as the primary metadata parser/deserializer. Keep `marked-yaml` as the fallback candidate if the Phase 1 parser spike cannot enforce the know-now YAML subset with high-quality source-aware diagnostics. | Decided. |
| YAML authoring subset | Phase 1/2 support a constrained YAML subset: top-level mapping documents, scalar string keys, ordinary scalars/sequences/mappings, no anchors, no aliases, no merge keys, no custom tags, no include directives, no multi-document files, and duplicate keys rejected. | Decided. |
| Template renderer | Use `minijinja` through a restricted `know-now-minijinja-v1` renderer profile. Template packs are data-only, strict, fuel-limited, path-isolated, and cannot register arbitrary functions, execute code, access the network, or write files directly. | Decided. |
| Minimum PostgreSQL version | Default compatibility floor is PostgreSQL 16 unless pilot constraints require another floor; re-confirm before release. | Before first public release. |
| dbt validation default | Adapter-based; likely `none` locally and configured validation in CI. | Before Phase 2B release. |
| Great Expectations exporter | Optional provider adapter, not early core. | After dbt quality contracts are stable. |
| Hosted/control-plane architecture | Control-plane/data-plane separation likely. | Phase 4 planning. |
| Stable IDs required or recommended | Recommended in Phase 2A; required for migration-safe workflows. | After pilot feedback. |
| Lockfile default | Recommended for teams and CI; likely created by default. | Before Phase 2A release. |
| Artifact attestation depth | Publish attestations/SBOMs where practical. | Before first public binary release. |
| Review state persistence | Local issue/review state first; hosted review later if validated. | Phase 3 planning. |
| External generator protocol | JSON stdin/stdout, explicitly enabled, no direct writes. | Phase 4 planning. |

---
