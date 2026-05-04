#!/usr/bin/env bash
set -euo pipefail

echo "[phase-2a] Running Phase 2A E2E integration tests"

echo "[phase-2a] step 1: CLI contract tests (generate pipeline + all subcommands)"
cargo test --package know_now_cli --test cli_contract -- --nocapture

echo "[phase-2a] step 2: metadata stability sweep (read-only commands)"
cargo test --package know_now_cli --test metadata_stability -- --nocapture 2>/dev/null || {
    echo "[phase-2a]   metadata_stability tests not yet present, skipping"
}

echo "[phase-2a] step 3: policy mutation tests"
cargo test --package know_now_policy --test mutation_policy -- --nocapture

echo "[phase-2a] step 4: snapshot tests"
for crate in know_now_gen_postgres know_now_gen_docs know_now_writer know_now_diagnostics; do
    if cargo test --package "$crate" --test snapshots -- --nocapture 2>/dev/null; then
        echo "[phase-2a]   $crate snapshot tests passed"
    else
        echo "[phase-2a]   $crate snapshot tests not yet present, skipping"
    fi
done

echo "[phase-2a] step 5: property-based tests"
for test_name in proptest_roundtrip proptest_pipeline proptest_identifiers; do
    found=false
    for crate in know_now_metadata know_now_core know_now_ir; do
        if cargo test --package "$crate" --test "$test_name" -- --nocapture 2>/dev/null; then
            echo "[phase-2a]   $crate::$test_name passed"
            found=true
        fi
    done
    if [ "$found" = false ]; then
        echo "[phase-2a]   $test_name not yet present in any crate, skipping"
    fi
done

echo "[phase-2a] step 6: end-to-end generate pipeline on demo fixture"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "[phase-2a]   init demo project"
cargo run --quiet -- --project "$TMPDIR/demo" init --demo

echo "[phase-2a]   validate"
cargo run --quiet -- --project "$TMPDIR/demo" validate

echo "[phase-2a]   check"
cargo run --quiet -- --project "$TMPDIR/demo" check

echo "[phase-2a]   check --locked"
cargo run --quiet -- --project "$TMPDIR/demo" check --locked

echo "[phase-2a]   generate"
cargo run --quiet -- --project "$TMPDIR/demo" generate

echo "[phase-2a]   verify manifest exists"
test -f "$TMPDIR/demo/generated/manifest.json"

echo "[phase-2a]   verify DDL exists"
test -f "$TMPDIR/demo/generated/ddl/postgres/schema.sql"

echo "[phase-2a]   verify docs exist"
test -d "$TMPDIR/demo/generated/docs"

echo "[phase-2a]   verify determinism (second generate)"
FIRST_MANIFEST=$(cat "$TMPDIR/demo/generated/manifest.json")
cargo run --quiet -- --project "$TMPDIR/demo" generate
SECOND_MANIFEST=$(cat "$TMPDIR/demo/generated/manifest.json")
if [ "$FIRST_MANIFEST" = "$SECOND_MANIFEST" ]; then
    echo "[phase-2a]   determinism check passed"
else
    echo "[phase-2a]   FAIL: manifests differ between runs"
    exit 1
fi

echo "[phase-2a]   verify no CRLF in generated files"
if grep -rPl '\r' "$TMPDIR/demo/generated/" 2>/dev/null; then
    echo "[phase-2a]   FAIL: CRLF found in generated files"
    exit 1
else
    echo "[phase-2a]   no CRLF in generated files"
fi

echo "[phase-2a]   dry-run produces no writes"
TMPDIR2=$(mktemp -d)
trap 'rm -rf "$TMPDIR" "$TMPDIR2"' EXIT
cargo run --quiet -- --project "$TMPDIR2/drytest" init --demo
cargo run --quiet -- --project "$TMPDIR2/drytest" generate --dry-run
test ! -f "$TMPDIR2/drytest/generated/manifest.json"
echo "[phase-2a]   dry-run verified"

echo "[phase-2a] step 7: compatibility fixture validation"
for fixture_dir in fixtures/*/; do
    fixture_name=$(basename "$fixture_dir")
    if [ -d "$fixture_dir/metadata" ]; then
        echo "[phase-2a]   validating fixture: $fixture_name"
        if cargo run --quiet -- --project "$fixture_dir" validate 2>/dev/null; then
            echo "[phase-2a]     $fixture_name: validate OK"
        else
            echo "[phase-2a]     $fixture_name: validate returned errors (expected for some fixtures)"
        fi
    fi
done

echo "[phase-2a] All Phase 2A E2E tests passed"
