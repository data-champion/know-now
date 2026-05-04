use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("know-now").expect("binary should exist")
}

#[test]
fn help_shows_all_phase2a_subcommands() {
    let expected = [
        "init", "validate", "check", "schema", "generate", "lock", "id", "examples", "config",
        "version",
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
fn stub_commands_exit_with_validation_error() {
    let stubs = [
        vec!["generate"],
        vec!["examples", "list"],
        vec!["config", "inspect"],
    ];
    for args in &stubs {
        cmd()
            .args(args)
            .assert()
            .code(predicate::eq(1))
            .stderr(predicate::str::contains("not yet implemented"));
    }
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
fn id_backfill_dry_run() {
    let project = id_fixture_missing_ids();
    cmd()
        .args(["--project"])
        .arg(project.path())
        .args(["id", "backfill", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ent_customer"));
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
        .write_stdin("guided-project\npostgres\n\nask\n")
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
        .write_stdin("guided-project\npostgres\n\nask\n")
        .assert()
        .success();
    cmd()
        .args(["--project"])
        .arg(tmp.path().join("guided-project"))
        .args(["validate"])
        .assert()
        .success();
}
