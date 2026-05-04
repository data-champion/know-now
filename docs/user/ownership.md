# Ownership boundaries

know-now enforces clear ownership over every file path in a project. This prevents accidental overwrites, clarifies who is responsible for each file, and makes regeneration safe.

PRD refs: §9.2, §9.3.

## Ownership table

| Path | Owner | What it means |
|------|-------|---------------|
| `metadata/` | **You** | Your source of truth. know-now reads this but never writes to it. Put your entity definitions, relationships, sources, rules, governance, and domain/module structures here. |
| `custom/` | **You** | Files you maintain that know-now will never touch. Use this for custom dbt macros, seeds, exposures, additional docs, or quality tests that go beyond what know-now generates. |
| `generated/` | **Engine** | know-now owns this directory. Every file here is produced by the generation pipeline and can be recreated from metadata at any time. Do not edit files here by hand — your changes will be overwritten on the next `generate` run (unless you use `--accept-generated-overwrite`). |
| `.knownow/` | **Engine state** | Internal working directory. Contains the cache, manifest, issue state, review state, audit logs, advisory locks, and run logs. You do not need to edit these files. They are safe to delete (the engine will recreate them). |
| `know-now.yml` | **You** | Project configuration. Created by `init`. Contains the project name, target database, policy pack, and git policy settings. |
| `know-now.lock` | **Engine + you** | The lockfile pins engine, contract, and policy versions for reproducible generation. The engine writes it; you review and commit it. See the lockfile guide for details. |
| `docs/exported/` | **You or engine** | Written only by explicit export commands. Not touched during normal generation. |

## Rules in plain language

### metadata/ is read-only to the engine

know-now will never modify, delete, or rewrite files under `metadata/`. When you run `validate`, `check`, or `generate`, the engine reads your YAML and reports diagnostics — it does not change what you wrote.

**Example:** If you misspell an entity name in `metadata/entities.yml`, you get a validation diagnostic. know-now does not auto-fix it.

### custom/ is never written by know-now

The `custom/` directory is yours. know-now will never create, modify, or delete files here. Use it for anything the engine should not manage: custom dbt models, macros, seeds, quality tests, or documentation.

**Example:** You add `custom/dbt/macros/my_macro.sql`. Running `generate` will never touch this file, even if it conflicts with a generated file name.

### generated/ is engine-owned and recreated atomically

Every file under `generated/` is produced by the generation pipeline. The engine writes output to a staging area first, then atomically promotes it to `generated/` only if generation succeeds. If generation fails, the previous artifact set is preserved.

**Example:** After adding a new entity to `metadata/entities.yml`, running `generate` recreates the DDL, docs, and manifest under `generated/`. Files for the new entity appear; existing files are regenerated.

### Manual-edit detection in generated/

If you edit a file under `generated/` by hand, know-now detects this on the next `generate` run and warns you. By default, it will not overwrite your manual edits. Use `--accept-generated-overwrite` to explicitly allow it.

**Example:** You edit `generated/ddl/customers.sql` to add a comment. On the next `generate`, you get a warning that the file was manually modified. Pass `--accept-generated-overwrite` to regenerate it anyway.

### Stale artifacts are reported, not silently deleted

When you rename or remove an entity, the generated files for the old entity become stale. know-now reports them as warnings but does not delete them by default.

Use `--prune stale` to delete stale artifacts. This only deletes files recorded in the previous manifest — it never deletes untracked files you placed under `generated/`.

**Example:** You rename `ent_customer` to `ent_client`. Running `generate` reports that `generated/ddl/customer.sql` is stale. Running `generate --prune stale` deletes it.

### Write operations acquire a project lock

Commands that write to `generated/` or mutable `.knownow/` state (`init`, `generate`, `lock update`) acquire an advisory lock under `.knownow/locks/`. This prevents two concurrent write operations from corrupting output. Read-only commands (`validate`, `check`, `schema`, `version`) do not acquire the lock.

### All writes go through the writer

Built-in generators and template packs do not write files directly. They produce artifact descriptors, and the central writer (`know_now_writer`) enforces path safety, ownership markers, manual-edit detection, stale handling, and atomic promotion. This means:

- Output paths must be relative and inside the approved generated root.
- Absolute paths are rejected.
- Path traversal (`../`) is rejected.
- Symlinks inside generated output are not followed during writes.
