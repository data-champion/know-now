use std::path::PathBuf;
use std::sync::LazyLock;

use know_now_fitness::workspace_metadata;

static ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| workspace_metadata().workspace_root.into_std_path_buf());

fn test_file_exists(rel: &str) {
    let path = ROOT.join(rel);
    assert!(
        path.exists(),
        "NFR-R coverage gate: expected {rel} to exist. \
         If moved or renamed, update the trace in bsf.3."
    );
}

fn test_dir_exists(rel: &str) {
    let path = ROOT.join(rel);
    assert!(
        path.is_dir(),
        "NFR-R coverage gate: expected directory {rel} to exist. \
         If moved or renamed, update the trace in bsf.3."
    );
}

#[test]
fn nfr_r1_determinism_tests_exist() {
    test_file_exists("crates/know_now_core/tests/determinism.rs");
}

#[test]
fn nfr_r3_snapshot_tests_exist() {
    test_file_exists("crates/know_now_gen_postgres/tests/snapshots.rs");
    test_file_exists("crates/know_now_gen_docs/tests/snapshots.rs");
    test_file_exists("crates/know_now_diagnostics/tests/snapshots.rs");
    test_file_exists("crates/know_now_writer/tests/snapshots.rs");
}

#[test]
fn nfr_r4_live_pg_test_exists() {
    test_file_exists("crates/know_now_core/tests/postgres_live.rs");
}

#[test]
fn nfr_r6_metadata_stability_sweep_exists() {
    test_file_exists("crates/know_now_cli/tests/metadata_stability.rs");
}

#[test]
fn nfr_r7_atomic_write_tests_exist() {
    test_file_exists("crates/know_now_core/tests/phase1_e2e.rs");
}

#[test]
fn nfr_r11_edit_detection_tests_exist() {
    test_file_exists("crates/know_now_writer/src/edit_detection.rs");
}

#[test]
fn nfr_r12_stale_detection_tests_exist() {
    test_file_exists("crates/know_now_writer/src/stale_detection.rs");
}

#[test]
fn nfr_r13_volatile_state_isolation() {
    test_file_exists("crates/know_now_core/tests/phase1_e2e.rs");
}

#[test]
fn nfr_r14_parser_fixtures_exist() {
    test_dir_exists("fixtures/parser");
}

#[test]
fn compatibility_fixtures_exist() {
    test_dir_exists("fixtures/minimal");
    test_dir_exists("fixtures/demo_ecommerce");
    test_dir_exists("fixtures/missing_ids");
    test_dir_exists("fixtures/rename_heavy");
    test_dir_exists("fixtures/large_100_entity");
    test_dir_exists("fixtures/doc_quality_warning");
}

#[test]
fn e2e_phase2a_driver_exists() {
    test_file_exists("tests/e2e/phase-2a.sh");
}

#[test]
fn policy_mutation_tests_exist() {
    test_file_exists("crates/know_now_policy/tests/mutation_policy.rs");
}

#[test]
fn proptest_tests_exist() {
    test_file_exists("crates/know_now_ir/tests/proptest_identifiers.rs");
    test_file_exists("crates/know_now_metadata/tests/proptest_roundtrip.rs");
    test_file_exists("crates/know_now_core/tests/proptest_pipeline.rs");
}
