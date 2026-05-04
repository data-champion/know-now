use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("know-now").expect("binary should exist")
}

#[test]
fn help_shows_all_phase2a_subcommands() {
    let expected = [
        "init", "validate", "check", "schema", "generate", "diff", "doctor", "explain", "issues",
        "lock", "id", "examples", "policy", "review", "support", "config", "version",
    ];
    let output = cmd().arg("--help").output().expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for sub in &expected {
        assert!(
            stdout.contains(sub),
            "--help should list subcommand '{sub}'"
        );
    }
}

#[test]
fn help_exit_code_is_zero() {
    cmd().arg("--help").assert().success();
}

#[test]
fn version_flag_exits_zero() {
    cmd().arg("--version").assert().success();
}

#[test]
fn version_subcommand_exits_zero() {
    cmd().arg("version").assert().success();
}

#[test]
fn version_shows_version_number() {
    cmd()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn version_json_has_envelope() {
    cmd()
        .args(["version", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""version": "1""#))
        .stdout(predicate::str::contains(r#""result": "success""#));
}

#[test]
fn version_capabilities_lists_generators() {
    cmd()
        .args(["version", "--capabilities"])
        .assert()
        .success()
        .stdout(predicate::str::contains("know_now_gen_postgres"))
        .stdout(predicate::str::contains("know_now_gen_docs"));
}

#[test]
fn unknown_subcommand_exits_with_usage_error() {
    cmd().arg("nonexistent").assert().code(predicate::eq(2));
}

#[test]
fn generate_without_metadata_exits_with_error() {
    cmd()
        .args(["generate"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("no metadata/ directory found"));
}

#[test]
fn generate_help_shows_all_flags() {
    let expected_flags = [
        "--dry-run",
        "--target",
        "--strict",
        "--fail-on-warnings",
        "--locked",
        "--no-cache",
        "--changed",
        "--prune",
        "--accept-generated-overwrite",
        "--migration-safe",
    ];
    let output = cmd()
        .args(["generate", "--help"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in &expected_flags {
        assert!(
            stdout.contains(flag),
            "generate --help should show flag '{flag}'"
        );
    }
}

#[test]
fn global_flags_present_on_subcommands() {
    let output = cmd()
        .args(["validate", "--help"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in &[
        "--format",
        "--verbose",
        "--debug",
        "--config",
        "--project",
        "--no-color",
    ] {
        assert!(
            stdout.contains(flag),
            "validate --help should show global flag '{flag}'"
        );
    }
}

#[test]
fn lock_subcommands_listed() {
    cmd()
        .args(["lock", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("check"));
}

#[test]
fn id_subcommands_listed() {
    cmd()
        .args(["id", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("suggest"))
        .stdout(predicate::str::contains("backfill"));
}

#[test]
fn version_quiet_produces_no_output() {
    cmd()
        .args(["version", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn schema_outputs_valid_json_schema() {
    cmd()
        .arg("schema")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#""$schema": "http://json-schema.org/draft-07/schema#""#,
        ))
        .stdout(predicate::str::contains(r#""title": "AuthoringMetadata""#));
}

#[test]
fn schema_defines_entities_and_relationships() {
    let output = cmd().arg("schema").output().expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let schema: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let defs = schema.get("definitions").expect("has definitions");
    assert!(defs.get("Entity").is_some(), "schema defines Entity");
    assert!(
        defs.get("Relationship").is_some(),
        "schema defines Relationship"
    );
    assert!(defs.get("Domain").is_some(), "schema defines Domain");
}

#[test]
fn schema_output_writes_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let schema_path = dir.path().join("schema.json");
    cmd()
        .args(["schema", "--output"])
        .arg(&schema_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Schema written to"));
    let content = std::fs::read_to_string(&schema_path).expect("file written");
    let schema: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(schema["title"], "AuthoringMetadata");
}

#[test]
fn schema_vscode_fragment() {
    let dir = tempfile::tempdir().expect("tempdir");
    let schema_path = dir.path().join("schema.json");
    let vscode_path = dir.path().join("settings.json");
    cmd()
        .args(["schema", "--output"])
        .arg(&schema_path)
        .arg("--vscode")
        .arg(&vscode_path)
        .assert()
        .success();
    let content = std::fs::read_to_string(&vscode_path).expect("file written");
    let fragment: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert!(
        fragment.get("yaml.schemas").is_some(),
        "fragment has yaml.schemas"
    );
}

#[test]
fn schema_quiet_produces_no_stdout() {
    cmd()
        .args(["schema", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn schema_is_deterministic() {
    let run1 = cmd().arg("schema").output().expect("run 1");
    let run2 = cmd().arg("schema").output().expect("run 2");
    assert_eq!(
        run1.stdout, run2.stdout,
        "schema output must be deterministic"
    );
}

fn id_fixture_all_ids() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let meta = dir.path().join("metadata");
    std::fs::create_dir(&meta).unwrap();
    std::fs::write(
        meta.join("project.yml"),
        r#"version: "1.0"
entities:
  - id: ent_customer
    name: customer
    attributes:
      - id: attr_customer_id
        name: id
        logical_type: integer
relationships:
  - id: rel_order_customer
    from_entity: order
    to_entity: customer
"#,
    )
    .unwrap();
    dir
}

fn id_fixture_missing_ids() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let meta = dir.path().join("metadata");
    std::fs::create_dir(&meta).unwrap();
    std::fs::write(
        meta.join("project.yml"),
        r#"version: "1.0"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
relationships:
  - from_entity: order
    to_entity: customer
"#,
    )
    .unwrap();
    dir
}

#[test]
fn id_check_succeeds_when_all_ids_present() {
    let project = id_fixture_all_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "All objects have valid stable IDs",
        ));
}

#[test]
fn id_check_json_envelope() {
    let project = id_fixture_all_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "check", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains(r#""command": "id check""#));
}

#[test]
fn id_check_reports_missing() {
    let project = id_fixture_missing_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("missing"));
}

#[test]
fn id_suggest_lists_missing_ids() {
    let project = id_fixture_missing_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ent_customer"));
}

#[test]
fn id_suggest_all_present_says_so() {
    let project = id_fixture_all_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "All objects already have stable IDs",
        ));
}

#[test]
fn id_backfill_default_is_dry_run() {
    let project = id_fixture_missing_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "backfill"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ent_customer"))
        .stdout(predicate::str::contains("Run with --apply"));
}

#[test]
fn id_backfill_apply_writes_ids() {
    let project = id_fixture_missing_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "backfill", "--apply"])
        .assert()
        .success()
        .stdout(predicate::str::contains("patched"));

    let content = std::fs::read_to_string(project.path().join("metadata/project.yml")).unwrap();
    assert!(
        content.contains("id: ent_customer"),
        "backfill must insert entity ID"
    );
}

#[test]
fn id_backfill_apply_creates_backup() {
    let project = id_fixture_missing_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "backfill", "--apply"])
        .assert()
        .success();

    let backups_dir = project.path().join(".knownow/backups");
    assert!(backups_dir.is_dir(), "backups directory must be created");
    let entries: Vec<_> = std::fs::read_dir(&backups_dir).unwrap().collect();
    assert_eq!(entries.len(), 1, "exactly one backup timestamp dir");
}

#[test]
fn id_backfill_apply_then_check_passes() {
    let project = id_fixture_missing_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "backfill", "--apply"])
        .assert()
        .success();

    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("All objects have valid stable IDs"));
}

#[test]
fn id_backfill_no_missing_says_so() {
    let project = id_fixture_all_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "backfill"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No missing IDs found"));
}

#[test]
fn id_check_no_metadata_dir_exits_with_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["id", "check"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("no metadata/ directory"));
}

#[test]
fn id_check_quiet_produces_no_stdout() {
    let project = id_fixture_all_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "check", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

fn valid_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let meta = dir.path().join("metadata");
    std::fs::create_dir(&meta).unwrap();
    std::fs::write(
        meta.join("project.yml"),
        r#"version: "1.0"
project:
  name: test
  owner: team
domains:
  - id: dom_sales
    name: sales
entities:
  - id: ent_customer
    name: customer
    domain: dom_sales
    description: A customer
    attributes:
      - id: attr_customer_id
        name: id
        logical_type: integer
        description: Primary key
      - id: attr_customer_email
        name: email
        logical_type: string
        description: Contact email
relationships:
  - id: rel_order_customer
    from_entity: customer
    to_entity: customer
"#,
    )
    .unwrap();
    dir
}

fn invalid_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let meta = dir.path().join("metadata");
    std::fs::create_dir(&meta).unwrap();
    std::fs::write(
        meta.join("project.yml"),
        r#"version: "1.0"
entities:
  - name: customer
    attributes: []
relationships:
  - from_entity: nonexistent
    to_entity: customer
"#,
    )
    .unwrap();
    dir
}

#[test]
fn validate_succeeds_on_valid_project() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn validate_fails_on_invalid_refs() {
    let project = invalid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["validate"])
        .assert()
        .code(predicate::eq(1));
}

#[test]
fn validate_json_envelope() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["validate", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains(r#""valid": true"#));
}

#[test]
fn validate_json_reports_errors() {
    let project = invalid_project();
    let output = cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["validate", "--format", "json"])
        .output()
        .expect("should run");
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(envelope["result"], "error");
    assert_eq!(envelope["payload"]["valid"], false);
    let diags = envelope["payload"]["diagnostics"].as_array().unwrap();
    assert!(!diags.is_empty());
}

#[test]
fn validate_sarif_output() {
    let project = valid_project();
    let output = cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["validate", "--format", "sarif"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout).expect("valid SARIF JSON");
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["runs"].is_array());
    let runs = sarif["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["tool"]["driver"]["name"], "know-now");
}

#[test]
fn validate_quiet_produces_no_stdout() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["validate", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn validate_no_metadata_dir_exits_with_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["validate"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("no metadata/ directory"));
}

#[test]
fn validate_text_shows_summary() {
    let project = invalid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["validate"])
        .assert()
        .code(predicate::eq(1))
        .stdout(predicate::str::contains("Validation failed"));
}

// ── Lock command tests ──────────────────────────────────────────────

#[test]
fn lock_update_creates_lockfile() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Lockfile updated"));
    assert!(dir.path().join("know-now.lock").exists());
}

#[test]
fn lock_update_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let first = std::fs::read_to_string(dir.path().join("know-now.lock")).unwrap();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let second = std::fs::read_to_string(dir.path().join("know-now.lock")).unwrap();
    assert_eq!(first, second, "lock update must be idempotent");
}

#[test]
fn lock_update_json_envelope() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains(r#""command": "lock update""#));
}

#[test]
fn lock_update_quiet_produces_no_stdout() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    assert!(dir.path().join("know-now.lock").exists());
}

#[test]
fn lock_check_succeeds_after_update() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("All versions match"));
}

#[test]
fn lock_check_fails_when_no_lockfile() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("LOCK-MISSING-004"));
}

#[test]
fn lock_check_fails_on_corrupt_lockfile() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("know-now.lock"), "not json").unwrap();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("LOCK-CORRUPT-005"));
}

#[test]
fn lock_check_fails_on_stale_lockfile() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let lock_path = dir.path().join("know-now.lock");
    let mut content = std::fs::read_to_string(&lock_path).unwrap();
    content = content.replace(
        r#""engine_version": "0.1.0""#,
        r#""engine_version": "99.0.0""#,
    );
    std::fs::write(&lock_path, content).unwrap();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check"])
        .assert()
        .code(predicate::eq(1))
        .stdout(predicate::str::contains("Drifted fields"));
}

#[test]
fn lock_check_json_envelope_on_success() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains(r#""passed": true"#));
}

#[test]
fn lock_check_json_envelope_on_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check", "--format", "json"])
        .assert()
        .code(predicate::eq(1))
        .stdout(predicate::str::contains(r#""result": "error""#))
        .stdout(predicate::str::contains(r#""passed": false"#));
}

#[test]
fn lock_update_lockfile_is_valid_json() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let content = std::fs::read_to_string(dir.path().join("know-now.lock")).unwrap();
    let lockfile: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(lockfile["lockfile_schema_version"], "1.0");
    assert!(lockfile["generators"].is_object());
    assert!(lockfile["policy"].is_object());
}

#[test]
fn lock_update_lockfile_contains_generators() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let content = std::fs::read_to_string(dir.path().join("know-now.lock")).unwrap();
    let lockfile: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    let generators = lockfile["generators"].as_object().unwrap();
    assert!(generators.contains_key("know_now_gen_postgres"));
    assert!(generators.contains_key("know_now_gen_docs"));
}

#[test]
fn lock_update_lockfile_contains_policy() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let content = std::fs::read_to_string(dir.path().join("know-now.lock")).unwrap();
    let lockfile: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(lockfile["policy"]["pack"], "dc_standard");
    assert_eq!(lockfile["policy"]["version"], "1.0");
}

#[test]
fn lock_check_quiet_produces_no_stdout() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn lock_update_shows_accept_contract_upgrade_flag() {
    let output = cmd()
        .args(["lock", "update", "--help"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--accept-contract-upgrade"),
        "lock update --help should show --accept-contract-upgrade"
    );
}

#[test]
fn lock_check_detects_unknown_schema_version() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let lock_path = dir.path().join("know-now.lock");
    let mut content = std::fs::read_to_string(&lock_path).unwrap();
    content = content.replace(
        r#""lockfile_schema_version": "1.0""#,
        r#""lockfile_schema_version": "99.0""#,
    );
    std::fs::write(&lock_path, content).unwrap();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["lock", "check"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("LOCK-SCHEMA-001"));
}

// ── Check command tests ─────────────────────────────────────────────

#[test]
fn check_succeeds_on_valid_project() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check passed"));
}

#[test]
fn check_fails_on_invalid_project() {
    let project = invalid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check"])
        .assert()
        .code(predicate::eq(1))
        .stdout(predicate::str::contains("Check failed"));
}

#[test]
fn check_json_envelope_on_success() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains(r#""passed": true"#))
        .stdout(predicate::str::contains(r#""command": "check""#));
}

#[test]
fn check_json_envelope_on_failure() {
    let project = invalid_project();
    let output = cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--format", "json"])
        .output()
        .expect("should run");
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(envelope["result"], "error");
    assert_eq!(envelope["payload"]["passed"], false);
}

#[test]
fn check_quiet_produces_no_stdout() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn check_sarif_output() {
    let project = valid_project();
    let output = cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--format", "sarif"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout).expect("valid SARIF JSON");
    assert_eq!(sarif["version"], "2.1.0");
}

#[test]
fn check_locked_succeeds_with_matching_lockfile() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["lock", "update"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--locked"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check passed"));
}

#[test]
fn check_locked_fails_without_lockfile() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--locked"])
        .assert()
        .code(predicate::eq(1))
        .stdout(predicate::str::contains("LOCK-MISSING-004"));
}

#[test]
fn check_locked_fails_on_stale_lockfile() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let lock_path = project.path().join("know-now.lock");
    let mut content = std::fs::read_to_string(&lock_path).unwrap();
    content = content.replace(
        r#""engine_version": "0.1.0""#,
        r#""engine_version": "99.0.0""#,
    );
    std::fs::write(&lock_path, content).unwrap();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--locked"])
        .assert()
        .code(predicate::eq(1))
        .stdout(predicate::str::contains("LOCK-STALE-003"));
}

#[test]
fn check_locked_json_includes_lock_status() {
    let project = valid_project();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["lock", "update"])
        .assert()
        .success();
    let output = cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--locked", "--format", "json"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(
        envelope["payload"]["lock_status"].is_object(),
        "JSON should include lock_status when --locked"
    );
    assert_eq!(envelope["payload"]["lock_status"]["passed"], true);
}

#[test]
fn check_without_locked_omits_lock_status() {
    let project = valid_project();
    let output = cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["check", "--format", "json"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(
        envelope["payload"]["lock_status"].is_null(),
        "JSON should not include lock_status without --locked"
    );
}

#[test]
fn check_no_metadata_dir_exits_with_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["check"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("no metadata/ directory"));
}

#[test]
fn check_help_shows_locked_flag() {
    let output = cmd()
        .args(["check", "--help"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--locked"),
        "check --help should show --locked flag"
    );
}

// ── Init command tests ──────────────────────────────────────────────

#[test]
fn init_demo_creates_project_structure() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created project"));
    let proj = tmp.path().join("demo-project");
    assert!(proj.join("metadata").is_dir());
    assert!(proj.join("generated").is_dir());
    assert!(proj.join("custom").is_dir());
    assert!(proj.join(".knownow").is_dir());
    assert!(proj.join("know-now.yml").is_file());
    assert!(proj.join("README.md").is_file());
    assert!(proj.join("generated/.gitkeep").is_file());
    assert!(proj.join("custom/.gitkeep").is_file());
}

#[test]
fn init_demo_creates_lockfile() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let lock_path = tmp.path().join("demo-project/know-now.lock");
    assert!(lock_path.is_file(), "demo should create know-now.lock");
    let content = std::fs::read_to_string(&lock_path).unwrap();
    let lockfile: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(lockfile["lockfile_schema_version"], "1.0");
}

#[test]
fn init_demo_validates_successfully() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("demo-project"))
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn init_demo_checks_successfully() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("demo-project"))
        .args(["check"])
        .assert()
        .success();
}

#[test]
fn init_demo_check_locked_succeeds() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("demo-project"))
        .args(["check", "--locked"])
        .assert()
        .success();
}

#[test]
fn init_demo_with_name() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "my-demo", "--profile", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-demo"));
    assert!(tmp.path().join("my-demo/metadata").is_dir());
}

#[test]
fn init_demo_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["--format", "json", "init", "--demo"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(envelope["result"], "success");
    assert_eq!(envelope["command"], "init");
    assert_eq!(envelope["payload"]["profile"], "demo");
}

#[test]
fn init_demo_quiet_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["--format", "quiet", "init", "--demo"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    assert!(tmp.path().join("demo-project/metadata").is_dir());
}

#[test]
fn init_minimal_creates_project() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "my-project", "--profile", "minimal"])
        .assert()
        .success();
    let proj = tmp.path().join("my-project");
    assert!(proj.join("metadata/project.yml").is_file());
    assert!(proj.join("know-now.yml").is_file());
    assert!(
        !proj.join("know-now.lock").exists(),
        "minimal should not create lockfile"
    );
}

#[test]
fn init_minimal_validates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "my-project", "--profile", "minimal"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("my-project"))
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn init_consultant_postgres_dbt_validates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "my-project", "--profile", "consultant-postgres-dbt"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("my-project"))
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn init_dbt_existing_stack_validates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "my-project", "--profile", "dbt-existing-stack"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("my-project"))
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn init_governed_team_validates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "my-project", "--profile", "governed-team"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("my-project"))
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn init_name_required_without_demo() {
    cmd()
        .args(["init"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("project name required"));
}

#[test]
fn init_existing_dir_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(tmp.path().join("exists")).unwrap();
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "exists"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn init_help_shows_options() {
    let output = cmd().args(["init", "--help"]).output().expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--profile"),
        "init help should show --profile"
    );
    assert!(stdout.contains("--demo"), "init help should show --demo");
    assert!(
        stdout.contains("--guided"),
        "init help should show --guided"
    );
    assert!(
        stdout.contains("--generated-git-policy"),
        "init help should show --generated-git-policy"
    );
}

#[test]
fn init_minimal_git_policy_ignore_creates_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "my-project", "--profile", "minimal"])
        .assert()
        .success();
    assert!(
        tmp.path().join("my-project/generated/.gitignore").is_file(),
        "minimal profile (git_policy=ignore) should create generated/.gitignore"
    );
}

#[test]
fn init_demo_no_generated_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    assert!(
        !tmp.path()
            .join("demo-project/generated/.gitignore")
            .exists(),
        "demo profile (git_policy=commit) should not create generated/.gitignore"
    );
}

#[test]
fn init_guided_reads_stdin() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--guided"])
        .write_stdin("guided-project\npostgres\nask\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("guided-project"));
    let proj = tmp.path().join("guided-project");
    assert!(proj.join("metadata").is_dir());
    assert!(proj.join("know-now.yml").is_file());
}

#[test]
fn init_guided_validates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--guided"])
        .write_stdin("guided-project\npostgres\nask\n")
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("guided-project"))
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn init_guided_none_db_creates_minimal() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--guided"])
        .write_stdin("guided-none\nnone\nignore\n")
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("guided-none"))
        .args(["validate"])
        .assert()
        .success();
}

#[test]
fn init_rejects_path_traversal() {
    cmd()
        .args(["init", "../../evil"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("path separators"));
}

#[test]
fn init_rejects_special_chars_in_name() {
    cmd()
        .args(["init", "foo:bar"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("ASCII letters"));
}

#[test]
fn init_rejects_dot_as_name() {
    cmd()
        .args(["init", "."])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("must not be '.' or '..'"));
}

// ── examples list ─────────────────────────────────────────────

#[test]
fn examples_list_shows_all_profiles() {
    let expected = [
        "minimal",
        "consultant-postgres-dbt",
        "dbt-existing-stack",
        "governed-team",
        "demo-ecommerce",
    ];
    let assert = cmd().args(["examples", "list"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for name in &expected {
        assert!(
            stdout.contains(name),
            "examples list should contain '{name}'"
        );
    }
}

#[test]
fn examples_list_json_envelope() {
    cmd()
        .args(["examples", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains("minimal"))
        .stdout(predicate::str::contains("demo-ecommerce"));
}

#[test]
fn examples_list_quiet_produces_no_output() {
    cmd()
        .args(["examples", "list", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ── config inspect ────────────────────────────────────────────

#[test]
fn config_inspect_without_project() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["config", "inspect"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file:  not found"))
        .stdout(predicate::str::contains("Metadata dir: not found"))
        .stdout(predicate::str::contains("Lockfile:      not found"));
}

#[test]
fn config_inspect_json_envelope() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["config", "inspect", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains("engine_version"))
        .stdout(predicate::str::contains("generators"));
}

#[test]
fn config_inspect_quiet_produces_no_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["config", "inspect", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn config_inspect_with_project_shows_metadata() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "inspect-test", "--profile", "demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("inspect-test"))
        .args(["config", "inspect"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file:  found"))
        .stdout(predicate::str::contains("Metadata dir: found"))
        .stdout(predicate::str::contains("Entities:"))
        .stdout(predicate::str::contains("Lockfile:      valid"));
}

#[test]
fn config_inspect_json_with_project_has_metadata() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "inspect-json-test", "--profile", "demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("inspect-json-test"))
        .args(["config", "inspect", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entity_count"))
        .stdout(predicate::str::contains("relationship_count"))
        .stdout(predicate::str::contains("lockfile"));
}

// ── version (expanded) ───────────────────────────────────────

#[test]
fn version_json_includes_all_schema_versions() {
    cmd()
        .args(["version", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("engine_version"))
        .stdout(predicate::str::contains("metadata_schema_version"))
        .stdout(predicate::str::contains("generator_contract_version"))
        .stdout(predicate::str::contains("lockfile_schema_version"));
}

#[test]
fn version_capabilities_json_envelope() {
    cmd()
        .args(["version", "--capabilities", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result": "success""#))
        .stdout(predicate::str::contains("know_now_gen_postgres"))
        .stdout(predicate::str::contains("artifact_kinds"));
}

#[test]
fn version_capabilities_shows_dialects() {
    cmd()
        .args(["version", "--capabilities"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dialect: postgres"))
        .stdout(predicate::str::contains("14, 15, 16, 17"));
}

#[test]
fn version_capabilities_quiet_produces_no_output() {
    cmd()
        .args(["version", "--capabilities", "--format", "quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ── generate command tests ───────────────────────────────────

#[test]
fn generate_phase3_changed_flag_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("demo-project"))
        .args(["generate", "--changed"])
        .assert()
        .code(predicate::eq(2))
        .stderr(predicate::str::contains("Phase 3 feature"));
}

#[test]
fn generate_phase3_target_changed_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("demo-project"))
        .args(["generate", "--target", "changed"])
        .assert()
        .code(predicate::eq(2))
        .stderr(predicate::str::contains("Phase 3 feature"));
}

#[test]
fn generate_phase3_migration_safe_flag_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("demo-project"))
        .args(["generate", "--migration-safe"])
        .assert()
        .code(predicate::eq(2))
        .stderr(predicate::str::contains("Phase 3 feature"));
}

#[test]
fn generate_fixtures_target_produces_csv() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "fixtures"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fixtures/"));
    assert!(project.join("generated/fixtures/README.md").exists());
    assert!(project.join("generated/fixtures/customer.csv").exists());
}

#[test]
fn generate_dbt_target_produces_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "dbt"])
        .assert()
        .success();
    assert!(project.join("generated/dbt/dbt_project.yml").exists());
    assert!(project.join("generated/dbt/models/staging").exists());
    assert!(project.join("generated/dbt/models/marts").exists());
}

#[test]
fn generate_dry_run_writes_no_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));
    assert!(
        !project.join("generated/manifest.json").exists(),
        "dry run must not write manifest.json"
    );
    assert!(
        !project.join("generated/ddl").exists(),
        "dry run must not write DDL artifacts"
    );
}

#[test]
fn generate_dry_run_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    let output = cmd()
        .args(["--project"])
        .arg(&project)
        .args(["--format", "json", "generate", "--dry-run"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["result"], "success");
    assert_eq!(json["command"], "generate");
    assert_eq!(json["payload"]["dry_run"], true);
    assert!(json["payload"]["planned_artifacts"].is_array());
}

#[test]
fn generate_produces_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generation complete"));

    let generated = project.join("generated");
    assert!(generated.exists(), "generated/ should exist");
    assert!(
        generated.join("manifest.json").exists(),
        "manifest.json should exist"
    );
    assert!(
        generated.join("ddl/postgres/schema.sql").exists(),
        "DDL artifact should exist"
    );
    assert!(
        generated.join("docs/README.md").exists(),
        "docs overview should exist"
    );
}

#[test]
fn generate_manifest_is_valid_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();
    let manifest_path = project.join("generated/manifest.json");
    let content = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert!(json["artifacts"].is_array());
    assert!(json["engine_version"].is_string());
}

#[test]
fn generate_locked_fails_without_lockfile() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "no-lock", "--profile", "minimal"])
        .assert()
        .success();
    let project = tmp.path().join("no-lock");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--locked"])
        .assert()
        .code(predicate::eq(1))
        .stderr(predicate::str::contains("LOCK-MISSING-004"));
}

#[test]
fn generate_locked_succeeds_with_lockfile() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--locked"])
        .assert()
        .success();
}

#[test]
fn generate_ddl_contains_sql() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();
    let ddl_path = project.join("generated/ddl/postgres/schema.sql");
    let content = std::fs::read_to_string(&ddl_path).expect("read DDL");
    assert!(content.contains("CREATE TABLE"));
    assert!(content.contains("Generated by know-now"));
}

#[test]
fn generate_is_deterministic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();
    let first_ddl =
        std::fs::read_to_string(project.join("generated/ddl/postgres/schema.sql")).unwrap();
    let first_manifest = std::fs::read_to_string(project.join("generated/manifest.json")).unwrap();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();
    let second_ddl =
        std::fs::read_to_string(project.join("generated/ddl/postgres/schema.sql")).unwrap();
    let second_manifest = std::fs::read_to_string(project.join("generated/manifest.json")).unwrap();

    assert_eq!(first_ddl, second_ddl, "DDL must be deterministic");
    assert_eq!(
        first_manifest, second_manifest,
        "manifest must be deterministic"
    );
}

#[test]
fn generate_target_ddl_only() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "ddl"])
        .assert()
        .success();
    let generated = project.join("generated");
    assert!(generated.join("ddl/postgres/schema.sql").exists());
    assert!(
        !generated.join("docs/README.md").exists(),
        "docs should not exist when targeting only DDL"
    );
}

#[test]
fn generate_target_docs_only() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "docs"])
        .assert()
        .success();
    let generated = project.join("generated");
    assert!(generated.join("docs/README.md").exists());
    assert!(
        !generated.join("ddl").exists(),
        "DDL should not exist when targeting only docs"
    );
}

#[test]
fn generate_target_comma_separated() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "ddl,docs"])
        .assert()
        .success();
    let generated = project.join("generated");
    assert!(generated.join("ddl/postgres/schema.sql").exists());
    assert!(generated.join("docs/README.md").exists());
}

#[test]
fn generate_target_all_alias_produces_supported_union() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "all"])
        .assert()
        .success();
    let generated = project.join("generated");
    assert!(generated.join("ddl/postgres/schema.sql").exists());
    assert!(generated.join("docs/README.md").exists());
}

#[test]
fn generate_failure_preserves_prior_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let ddl_path = project.join("generated/ddl/postgres/schema.sql");
    assert!(ddl_path.exists(), "first generate should create DDL");
    let first_ddl = std::fs::read_to_string(&ddl_path).unwrap();

    let entity_files: Vec<_> = std::fs::read_dir(project.join("metadata"))
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    if let Some(first_yaml) = entity_files.first() {
        std::fs::write(first_yaml.path(), "invalid: [yaml: content").unwrap();
    }

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .failure();

    let preserved_ddl = std::fs::read_to_string(&ddl_path).unwrap();
    assert_eq!(
        first_ddl, preserved_ddl,
        "prior generated output must be preserved on failure"
    );
}

#[test]
fn generate_no_crlf_in_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let ddl = std::fs::read_to_string(project.join("generated/ddl/postgres/schema.sql")).unwrap();
    assert!(
        !ddl.contains('\r'),
        "NFR-PO3: generated DDL must use LF only"
    );
    let manifest = std::fs::read_to_string(project.join("generated/manifest.json")).unwrap();
    assert!(
        !manifest.contains('\r'),
        "NFR-PO3: manifest must use LF only"
    );
}

#[test]
fn generate_quality_target_produces_contracts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "quality"])
        .assert()
        .success();
    let qc_dir = project.join("generated/quality_contracts");
    assert!(qc_dir.exists(), "quality_contracts/ dir should exist");
    assert!(
        qc_dir.join("customer.yml").exists(),
        "customer quality contract should exist"
    );
}

#[test]
fn generate_quality_contract_is_valid_yaml() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "quality"])
        .assert()
        .success();
    let qc_dir = project.join("generated/quality_contracts");
    for entry in std::fs::read_dir(&qc_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().is_some_and(|e| e == "yml") {
            let content = std::fs::read_to_string(entry.path()).unwrap();
            assert!(
                !content.is_empty(),
                "quality contract {} should not be empty",
                entry.path().display()
            );
            assert!(
                content.contains("checks:"),
                "quality contract {} should have checks section",
                entry.path().display()
            );
        }
    }
}

#[test]
fn generate_dbt_sources_yml_is_valid_yaml() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "dbt"])
        .assert()
        .success();
    let sources_path = project.join("generated/dbt/models/sources.yml");
    if sources_path.exists() {
        let content = std::fs::read_to_string(&sources_path).unwrap();
        assert!(content.contains("sources:"), "sources.yml must have sources key");
    }
}

#[test]
fn generate_dbt_mart_models_exist() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "dbt"])
        .assert()
        .success();
    let marts_dir = project.join("generated/dbt/models/marts");
    assert!(marts_dir.exists(), "marts directory should exist");
    let mart_files: Vec<_> = std::fs::read_dir(&marts_dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect();
    assert!(
        !mart_files.is_empty(),
        "at least one mart .sql model should exist"
    );
}

#[test]
fn generate_dbt_schema_yml_has_models() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "dbt"])
        .assert()
        .success();
    let schema_path = project.join("generated/dbt/models/marts/schema.yml");
    if schema_path.exists() {
        let content = std::fs::read_to_string(&schema_path).unwrap();
        assert!(content.contains("models:"), "marts schema.yml must have models key");
    }
}

#[test]
fn generate_all_targets_produces_all_phase2b_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "all"])
        .assert()
        .success();
    let generated = project.join("generated");
    assert!(generated.join("ddl/postgres/schema.sql").exists(), "DDL");
    assert!(generated.join("docs/README.md").exists(), "docs");
    assert!(generated.join("dbt/dbt_project.yml").exists(), "dbt");
    assert!(generated.join("quality_contracts").exists(), "quality");
    assert!(generated.join("fixtures/README.md").exists(), "fixtures");
    assert!(generated.join("diagrams/er/all.mmd").exists(), "ER diagram");
}

#[test]
fn generate_manifest_includes_all_generators() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "all"])
        .assert()
        .success();
    let manifest = std::fs::read_to_string(project.join("generated/manifest.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&manifest).expect("valid JSON");
    let artifacts = json["artifacts"].as_array().expect("artifacts array");
    let generators: std::collections::HashSet<&str> = artifacts
        .iter()
        .filter_map(|a| a["generator"].as_str())
        .collect();
    assert!(generators.contains("know_now_gen_postgres"), "postgres in manifest");
    assert!(generators.contains("know_now_gen_docs"), "docs in manifest");
    assert!(generators.contains("know_now_gen_dbt"), "dbt in manifest");
    assert!(generators.contains("know_now_gen_quality"), "quality in manifest");
    assert!(generators.contains("know_now_gen_fixtures"), "fixtures in manifest");
    assert!(generators.contains("know_now_gen_er"), "ER in manifest");
}

#[test]
fn generate_no_crlf_in_phase2b_output() {
    fn check_no_crlf(dir: &std::path::Path) {
        if !dir.exists() {
            return;
        }
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                check_no_crlf(&path);
            } else {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                assert!(
                    !content.contains('\r'),
                    "NFR-PO3: {} must use LF only",
                    path.display()
                );
            }
        }
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate", "--target", "all"])
        .assert()
        .success();
    let generated = project.join("generated");
    check_no_crlf(&generated);
}

#[test]
fn diff_help_shows_flags() {
    cmd()
        .args(["diff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--baseline"))
        .stdout(predicate::str::contains("--migration-safe"))
        .stdout(predicate::str::contains("--impact"))
        .stdout(predicate::str::contains("--scan-custom"));
}

#[test]
fn diff_without_baseline_fails_cleanly() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["diff"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("DIFF-NO-BASELINE-002"));
}

#[test]
fn diff_after_generate_shows_no_changes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let manifest_path = project.join("generated/manifest.json");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    if json.get("contract").is_none() {
        return;
    }

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["diff"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes"));
}

#[test]
fn diff_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let manifest_path = project.join("generated/manifest.json");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    if json.get("contract").is_none() {
        return;
    }

    let output = cmd()
        .args(["--project"])
        .arg(&project)
        .args(["--format", "json", "diff"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(result["schema_version"], "1");
    assert!(result["changes"].is_array());
}

#[test]
fn diff_unknown_baseline_format_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["diff", "--baseline", "bogus:foo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown baseline format"));
}

#[test]
fn diff_git_baseline_not_yet_implemented() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["diff", "--baseline", "git:main"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("DIFF-GIT-001"));
}

#[test]
fn diff_impact_flag_accepted() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let manifest_path = project.join("generated/manifest.json");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    if json.get("contract").is_none() {
        return;
    }

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["diff", "--impact"])
        .assert()
        .success();
}

#[test]
fn diff_impact_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let manifest_path = project.join("generated/manifest.json");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    if json.get("contract").is_none() {
        return;
    }

    let output = cmd()
        .args(["--project"])
        .arg(&project)
        .args(["--format", "json", "diff", "--impact"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(result["result"], "success");
    assert!(result["payload"]["diff"].is_object());
}

#[test]
fn diff_scan_custom_no_custom_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let manifest_path = project.join("generated/manifest.json");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    if json.get("contract").is_none() {
        return;
    }

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["diff", "--scan-custom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No custom/"));
}

#[test]
fn doctor_help_shows_flags() {
    cmd()
        .args(["doctor", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--check-updates"));
}

#[test]
fn doctor_on_demo_project_exits_zero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("know-now doctor"));
}

#[test]
fn doctor_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    let output = cmd()
        .args(["--project"])
        .arg(&project)
        .args(["--format", "json", "doctor"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["doctor_schema_version"], "1.0");
    assert!(json["engine"]["version"].is_string());
    assert!(json["project"]["config_present"].is_boolean());
    assert!(json["generators"].is_array());
}

#[test]
fn doctor_without_metadata_reports_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("empty-project")).unwrap();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("empty-project"))
        .args(["doctor"])
        .assert()
        .code(predicate::eq(2))
        .stdout(predicate::str::contains("DOC-META-001"));
}

#[test]
fn doctor_reports_generators() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    let output = cmd()
        .args(["--project"])
        .arg(&project)
        .args(["--format", "json", "doctor"])
        .output()
        .expect("should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let generators = json["generators"].as_array().expect("generators array");
    assert!(
        generators.len() >= 4,
        "should report at least 4 generators, got {}",
        generators.len()
    );
}

// ── explain ──────────────────────────────────────────────────────────

#[test]
fn explain_help_shows_flags() {
    cmd()
        .args(["explain", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--artifact"))
        .stdout(predicate::str::contains("--object-id"))
        .stdout(predicate::str::contains("--list"));
}

#[test]
fn explain_without_manifest_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["explain", "--list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("EXPLAIN-NO-MANIFEST-001"));
}

#[test]
fn explain_list_after_generate() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["explain", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("know-now explain"))
        .stdout(predicate::str::contains("artifact(s) in manifest"));
}

#[test]
fn explain_list_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let output = cmd()
        .args(["--project"])
        .arg(&project)
        .args(["--format", "json", "explain", "--list"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["payload"]["query"]["kind"], "list");
    assert!(json["payload"]["results"].is_array());
    assert!(json["payload"]["artifact_count"].as_u64().unwrap() > 0);
}

#[test]
fn explain_by_artifact_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["explain", "--artifact", "schema.sql"])
        .assert()
        .success()
        .stdout(predicate::str::contains("generator:"))
        .stdout(predicate::str::contains("kind:"));
}

#[test]
fn explain_by_artifact_no_match() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["explain", "--artifact", "nonexistent.sql"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching artifacts found"));
}

#[test]
fn explain_no_query_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["explain"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--artifact").or(predicate::str::contains("--list")));
}

// ── issues ───────────────────────────────────────────────────────────

#[test]
fn issues_list_help_shows_flags() {
    cmd()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--status"));
}

#[test]
fn issues_list_empty_project() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn issues_list_json_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["--format", "json", "issues", "list"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["result"], "success");
    assert!(json["payload"].is_array());
    assert_eq!(json["payload"].as_array().unwrap().len(), 0);
}

#[test]
fn issues_resolve_nonexistent_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "resolve", "no-such-id"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ISSUE-NOT-FOUND-001"));
}

#[test]
fn issues_snooze_nonexistent_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "snooze", "no-such-id", "--reason", "testing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ISSUE-NOT-FOUND-001"));
}

#[test]
fn issues_roundtrip_resolve() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let issues_dir = tmp.path().join(".knownow");
    std::fs::create_dir_all(&issues_dir).unwrap();
    let issues_json = issues_dir.join("issues.json");
    std::fs::write(
        &issues_json,
        r#"[{"id":"test-001","affected_object":"ent_customer","change_type":"breaking","description":"column removed","suggested_fix":"add column back","status":"open","snooze_reason":null,"created_at":"2026-05-04T00:00:00Z","updated_at":null}]"#,
    )
    .unwrap();

    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-001"))
        .stdout(predicate::str::contains("column removed"));

    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "resolve", "test-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved issue"));

    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));

    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "list", "--status", "all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[RESOLVED]"))
        .stdout(predicate::str::contains("test-001"));
}

#[test]
fn issues_roundtrip_snooze() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let issues_dir = tmp.path().join(".knownow");
    std::fs::create_dir_all(&issues_dir).unwrap();
    let issues_json = issues_dir.join("issues.json");
    std::fs::write(
        &issues_json,
        r#"[{"id":"test-002","affected_object":"attr_email","change_type":"ambiguous","description":"type changed","suggested_fix":"verify compatibility","status":"open","snooze_reason":null,"created_at":"2026-05-04T00:00:00Z","updated_at":null}]"#,
    )
    .unwrap();

    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "snooze", "test-002", "--reason", "waiting for review"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Snoozed issue"));

    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["issues", "list", "--status", "snoozed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-002"))
        .stdout(predicate::str::contains("waiting for review"));
}

// ── support ──────────────────────────────────────────────────────────

#[test]
fn support_help_shows_flags() {
    cmd()
        .args(["support", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--include-metadata"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn support_dry_run_lists_sections() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["support", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Support bundle would include"))
        .stdout(predicate::str::contains("doctor.json"))
        .stdout(predicate::str::contains("engine.json"));
}

#[test]
fn support_dry_run_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["--format", "json", "support", "--dry-run"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["result"], "success");
    assert!(json["payload"].is_array());
}

#[test]
fn support_creates_bundle_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["support", "--output"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Support bundle written to"));

    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("know-now-support-")
        })
        .collect();
    assert_eq!(entries.len(), 1, "should create exactly one bundle file");

    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON bundle");
    assert_eq!(json["bundle_version"], "1.0");
    assert!(json["engine"]["version"].is_string());
    assert!(json["environment"].is_object());
}

#[test]
fn support_bundle_on_demo_project() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["generate"])
        .assert()
        .success();

    let output_dir = tmp.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["support", "--output"])
        .arg(&output_dir)
        .assert()
        .success();

    let entries: Vec<_> = std::fs::read_dir(&output_dir)
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("know-now-support-")
        })
        .collect();
    assert_eq!(entries.len(), 1);

    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON bundle");
    assert!(json["manifest_summary"].is_object());
    assert!(json["manifest_summary"]["artifact_count"].as_u64().unwrap() > 0);
    assert!(json["generators"].is_array());
    assert!(json["doctor"].is_object());
}

#[test]
fn support_bundle_redacts_env() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let output = cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["--format", "json", "support", "--output"])
        .arg(tmp.path())
        .output()
        .expect("should run");
    assert!(output.status.success());

    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("know-now-support-")
        })
        .collect();
    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let env = json["environment"].as_object().unwrap();
    for (key, _) in env {
        assert!(
            ["USER", "LOGNAME", "SHELL", "TERM", "LANG", "LC_ALL", "HOME", "PATH",
             "EDITOR", "VISUAL", "CARGO_PKG_VERSION", "RUST_LOG", "NO_COLOR", "FORCE_COLOR"]
                .contains(&key.as_str()),
            "unexpected env var in bundle: {key}"
        );
    }
}

// ── review ───────────────────────────────────────────────────────────

#[test]
fn review_export_help_shows_flags() {
    cmd()
        .args(["review", "export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn review_export_dry_run_on_demo() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["review", "export", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Review pack would be written to"))
        .stdout(predicate::str::contains("summary.md"))
        .stdout(predicate::str::contains("manifest_summary.json"));
}

#[test]
fn review_export_dry_run_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");

    let output = cmd()
        .args(["--project"])
        .arg(&project)
        .args(["--format", "json", "review", "export", "--dry-run"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["result"], "success");
    assert!(json["payload"]["files"].is_array());
    let files = json["payload"]["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f.as_str() == Some("summary.md")));
}

#[test]
fn review_export_writes_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");

    let output_dir = tmp.path().join("reviews");
    std::fs::create_dir_all(&output_dir).unwrap();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["review", "export", "--output"])
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Review pack exported to"));

    let review_dirs: Vec<_> = std::fs::read_dir(&output_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("review_"))
        .collect();
    assert_eq!(review_dirs.len(), 1);

    let review_path = review_dirs[0].path();
    let summary = std::fs::read_to_string(review_path.join("summary.md")).unwrap();
    assert!(summary.contains("# Review Summary"));
    assert!(summary.contains("## Entities"));

    let manifest_summary =
        std::fs::read_to_string(review_path.join("manifest_summary.json")).unwrap();
    let ms: serde_json::Value = serde_json::from_str(&manifest_summary).expect("valid JSON");
    assert!(ms["generation_status"].is_string());

    assert!(review_path.join("entities").is_dir());
}

#[test]
fn review_export_entity_files_have_attributes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");

    let output_dir = tmp.path().join("reviews");
    std::fs::create_dir_all(&output_dir).unwrap();

    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["review", "export", "--output"])
        .arg(&output_dir)
        .assert()
        .success();

    let review_dirs: Vec<_> = std::fs::read_dir(&output_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("review_"))
        .collect();
    let review_path = review_dirs[0].path();
    let entities_dir = review_path.join("entities");
    let entity_files: Vec<_> = std::fs::read_dir(&entities_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert!(!entity_files.is_empty(), "should have entity markdown files");

    let first_entity = std::fs::read_to_string(entity_files[0].path()).unwrap();
    assert!(first_entity.contains("## Attributes"));
    assert!(first_entity.contains("**Status:**"));
}

// ── audit log ────────────────────────────────────────────────────────

#[test]
fn audit_log_created_on_version() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["version"])
        .assert()
        .success();

    let audit_log = tmp.path().join(".knownow").join("audit.log");
    assert!(audit_log.exists(), "audit.log should be created");
    let content = std::fs::read_to_string(&audit_log).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 1, "should have exactly one audit entry");
    let entry: serde_json::Value = serde_json::from_str(lines[0]).expect("valid JSONL");
    assert_eq!(entry["command"], "version");
    assert_eq!(entry["result"], "success");
    assert!(entry["timestamp"].is_string());
    assert!(entry["engine_version"].is_string());
}

#[test]
fn audit_log_records_failure() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["validate"])
        .assert()
        .failure();

    let audit_log = tmp.path().join(".knownow").join("audit.log");
    assert!(audit_log.exists(), "audit.log should exist even on failure");
    let content = std::fs::read_to_string(&audit_log).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 1);
    let entry: serde_json::Value = serde_json::from_str(lines[0]).expect("valid JSONL");
    assert_eq!(entry["command"], "validate");
    assert_eq!(entry["result"], "failure");
    assert!(entry["error_code"].is_string());
}

#[test]
fn audit_log_accumulates_entries() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["version"])
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["version"])
        .assert()
        .success();

    let audit_log = tmp.path().join(".knownow").join("audit.log");
    let content = std::fs::read_to_string(&audit_log).unwrap();
    assert_eq!(content.lines().count(), 2, "should accumulate entries across runs");
}

#[test]
fn audit_log_entries_are_valid_jsonl() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--project"])
        .arg(tmp.path())
        .args(["init", "--demo"])
        .assert()
        .success();
    let project = tmp.path().join("demo-project");
    cmd()
        .args(["--project"])
        .arg(&project)
        .args(["doctor"])
        .assert()
        .success();

    let audit_log = project.join(".knownow").join("audit.log");
    if !audit_log.exists() {
        return;
    }
    let content = std::fs::read_to_string(&audit_log).unwrap();
    for (i, line) in content.lines().enumerate() {
        let _: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("line {i} is not valid JSON: {e}"));
    }
}

// --- policy status tests ---

#[test]
fn policy_status_shows_builtin_pack() {
    let dir = valid_project();
    cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["policy", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dc_standard"));
}

#[test]
fn policy_status_json_has_packs_array() {
    let dir = valid_project();
    let output = cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["--format", "json", "policy", "status"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let packs = json["payload"]["packs"].as_array().unwrap();
    assert!(!packs.is_empty());
    assert_eq!(packs[0]["name"], "dc_standard");
}

#[test]
fn policy_status_with_catalog_shows_drift() {
    let dir = valid_project();
    let knownow_dir = dir.path().join(".knownow");
    std::fs::create_dir_all(&knownow_dir).unwrap();
    std::fs::write(
        knownow_dir.join("catalog.json"),
        r#"{
            "approved": {
                "engines": { "know-now": ["0.1.x"] },
                "policies": { "dc_standard": ["1.0"] }
            }
        }"#,
    )
    .unwrap();

    cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["policy", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Overall drift"));
}

#[test]
fn policy_status_with_missing_catalog_flag_errors() {
    let dir = valid_project();
    cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["policy", "status", "--catalog", "/nonexistent/catalog.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("catalog file not found"));
}

#[test]
fn policy_status_discovers_project_local_packs() {
    let dir = valid_project();
    let policy_dir = dir.path().join("policy");
    std::fs::create_dir(&policy_dir).unwrap();
    std::fs::write(
        policy_dir.join("corp.pack.json"),
        r#"{ "name": "corp_rules", "version": "2.0.0", "rules": [] }"#,
    )
    .unwrap();

    cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["policy", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("corp_rules"));
}

// --- policy explain tests ---

#[test]
fn policy_explain_builtin_rule() {
    let dir = valid_project();
    cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["policy", "explain", "POL-NAM-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entity_name_snake_case"))
        .stdout(predicate::str::contains("Remediation"));
}

#[test]
fn policy_explain_unknown_code() {
    let dir = valid_project();
    cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["policy", "explain", "NONEXISTENT-999"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unknown policy code"));
}

#[test]
fn policy_explain_json_output() {
    let dir = valid_project();
    let output = cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["--format", "json", "policy", "explain", "POL-ENT-001"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["payload"]["found"], true);
    assert_eq!(json["payload"]["rule"]["code"], "POL-ENT-001");
}

#[test]
fn policy_explain_declarative_rule() {
    let dir = valid_project();
    let policy_dir = dir.path().join("policy");
    std::fs::create_dir(&policy_dir).unwrap();
    std::fs::write(
        policy_dir.join("corp.pack.json"),
        r#"{
            "name": "corp_rules",
            "version": "1.0.0",
            "rules": [{
                "id": "CORP-001",
                "severity": "error",
                "applies_to": "entity",
                "expression": { "kind": "attribute_presence", "attribute": "description" },
                "rationale": "All entities must have a description for compliance",
                "remediation": "Add a description field to every entity"
            }]
        }"#,
    )
    .unwrap();

    cmd()
        .args(["--project", dir.path().to_str().unwrap()])
        .args(["policy", "explain", "CORP-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CORP-001"))
        .stdout(predicate::str::contains("compliance"));
}
