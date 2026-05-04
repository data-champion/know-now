# Regeneration safety

know-now is designed so that running `generate` is always safe. This document explains the mechanisms that protect your work.

PRD refs: §9.3, §9.4, §11.7.

## Atomic promotion

Generation writes to a staging area first. Only after all generators succeed does the engine atomically promote the staged output to `generated/`. If any generator fails, the previous artifact set under `generated/` is preserved unchanged.

This means a failed generation never leaves `generated/` in a partial state.

## Manual-edit detection

The engine tracks content hashes for every file it writes. On the next `generate` run, it compares the current file content against the recorded hash. If you edited a generated file by hand, know-now warns you and refuses to overwrite it by default.

To regenerate anyway:

```bash
know-now generate --accept-generated-overwrite
```

This flag acknowledges that your manual edits will be lost.

## Stale artifact handling

When you rename or remove an entity from metadata, the generated files for the old entity become stale — they exist in the previous manifest but not in the new generation plan.

**Default behavior (`--prune none`):** stale artifacts are reported as warnings but not deleted.

**Explicit deletion (`--prune stale`):** stale artifacts are deleted. Only files recorded in the previous manifest are eligible for deletion. Untracked files you placed under `generated/` are never deleted.

```bash
# Report stale artifacts as warnings
know-now generate

# Delete stale artifacts
know-now generate --prune stale
```

Warning codes:
- `WRITER-STALE` — file exists on disk and was removed from the generation plan.
- `WRITER-STALE-MISSING` — file was in the previous manifest but is already missing from disk.
- `WRITER-UNTRACKED` — file exists under `generated/` but is not in the manifest. Never deleted by `--prune stale`.

## Project-scoped advisory lock

Commands that write to `generated/` or `.knownow/` state acquire an advisory lock under `.knownow/locks/`. This prevents two concurrent `generate` runs from corrupting output.

If a lock cannot be acquired within 30 seconds, the command fails with a `LOCK-TIMEOUT` error that includes the PID and command name of the current holder.

Read-only commands (`validate`, `check`, `schema`, `version`) do not acquire the lock.

## What is safe to delete

| Path | Safe to delete? | What happens |
|------|-----------------|--------------|
| `generated/` | Yes | Fully recreated on next `generate` |
| `.knownow/cache/` | Yes | Cache rebuilt on next run |
| `.knownow/locks/` | Yes | Lock files recreated on next write command |
| `.knownow/runs/` | Yes | Run logs are informational |
| `know-now.lock` | Yes, but re-run `lock update` | Regenerated; `--locked` checks will fail until updated |
| `metadata/` | No | Your source of truth; not recreated by the engine |
| `custom/` | No | Your files; not recreated by the engine |
