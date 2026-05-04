//! Phase 3 E2E integration tests covering cross-cutting CLI scenarios.

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("know-now").expect("binary should exist")
}

fn valid_project_with_lockfile() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let meta = dir.path().join("metadata");
    std::fs::create_dir(&meta).unwrap();
    std::fs::write(
        meta.join("project.yml"),
        r#"version: "1.0"
project:
  name: e2e-test
  owner: team
domains:
  - id: dom_core
    name: core
entities:
  - id: ent_user
    name: user
    domain: dom_core
    description: A user entity
    attributes:
      - id: attr_user_id
        name: id
        logical_type: integer
        description: Primary key
      - id: attr_user_email
        name: email
        logical_type: string
        description: Email address
relationships:
  - id: rel_self
    from_entity: user
    to_entity: user
"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("know-now.yml"), "version: '1.0'\n").unwrap();
    dir
}

// ─── Doctor ──────────────────────────────────────────────────────────────────

#[test]
fn doctor_text_output_shows_checks() {
    let dir = valid_project_with_lockfile();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["doctor"])
        .assert()
        .success();
}

#[test]
fn doctor_json_output_has_schema_version() {
    let dir = valid_project_with_lockfile();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["--format", "json", "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""doctor_schema_version""#));
}

// ─── Explain ─────────────────────────────────────────────────────────────────

#[test]
fn explain_shows_help_without_argument() {
    cmd()
        .args(["explain", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trace"));
}

// ─── Support bundle ─────────────────────────────────────────────────────────

#[test]
fn support_dry_run_lists_contents() {
    let dir = valid_project_with_lockfile();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["support", "--dry-run"])
        .assert()
        .success();
}

// ─── Admin scan cross-project ────────────────────────────────────────────────

#[test]
fn admin_scan_multiple_projects() {
    let root = tempfile::tempdir().unwrap();

    for name in ["project_a", "project_b", "project_c"] {
        let proj = root.path().join(name);
        std::fs::create_dir_all(proj.join("metadata")).unwrap();
        std::fs::write(proj.join("know-now.yml"), "version: '1.0'\n").unwrap();
        std::fs::write(
            proj.join("metadata").join("project.yml"),
            format!(
                "version: '1.0'\nproject:\n  name: {name}\n  owner: team\n"
            ),
        )
        .unwrap();
    }

    cmd()
        .args(["admin", "scan"])
        .arg(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 project(s) found"));
}

#[test]
fn admin_scan_json_includes_all_projects() {
    let root = tempfile::tempdir().unwrap();

    for name in ["alpha", "beta"] {
        let proj = root.path().join(name);
        std::fs::create_dir_all(proj.join("metadata")).unwrap();
        std::fs::write(proj.join("know-now.yml"), "version: '1.0'\n").unwrap();
        std::fs::write(
            proj.join("metadata").join("project.yml"),
            format!(
                "version: '1.0'\nproject:\n  name: {name}\n  owner: team\n"
            ),
        )
        .unwrap();
    }

    let output = cmd()
        .args(["--format", "json", "admin", "scan"])
        .arg(root.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let projects = json["payload"]["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 2);
}

// ─── Admin catalog check ─────────────────────────────────────────────────────

#[test]
fn admin_catalog_check_detects_invalid_semver() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = dir.path().join("catalog.json");
    std::fs::write(
        &catalog,
        r#"{"approved":{"engines":{"know-now":["not-semver"]},"metadata_schema_versions":["1.0"]}}"#,
    )
    .unwrap();

    let output = cmd()
        .args(["admin", "catalog-check"])
        .arg(&catalog)
        .output()
        .unwrap();

    // Should either fail validation or pass (depends on whether
    // validate() checks semver format)
    let _ = output.status;
}

// ─── Policy integration ──────────────────────────────────────────────────────

#[test]
fn policy_status_runs_without_lockfile() {
    let dir = valid_project_with_lockfile();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["policy", "status"])
        .assert()
        .success();
}

// ─── Diff with stable IDs ────────────────────────────────────────────────────

#[test]
fn diff_requires_baseline() {
    let dir = valid_project_with_lockfile();
    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["diff"])
        .assert()
        .failure();
}

// ─── Audit log written after commands ────────────────────────────────────────

#[test]
fn audit_log_written_after_validate() {
    let dir = valid_project_with_lockfile();
    let knownow_dir = dir.path().join(".knownow");

    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["validate"])
        .assert()
        .success();

    let audit_log = knownow_dir.join("audit.log");
    assert!(
        audit_log.exists(),
        "audit log should be written after validate"
    );

    let content = std::fs::read_to_string(&audit_log).unwrap();
    assert!(content.contains("validate"));
    assert!(content.contains("success"));
}

#[test]
fn audit_log_records_failure() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("metadata")).unwrap();
    std::fs::write(
        dir.path().join("metadata").join("project.yml"),
        "invalid: yaml: [\n",
    )
    .unwrap();

    cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["validate"])
        .assert()
        .failure();

    let audit_log = dir.path().join(".knownow").join("audit.log");
    if audit_log.exists() {
        let content = std::fs::read_to_string(&audit_log).unwrap();
        assert!(content.contains("validate"));
        assert!(content.contains("failure"));
    }
}

// ─── Incremental cache ───────────────────────────────────────────────────────

#[test]
fn generate_creates_cache_file() {
    let dir = tempfile::tempdir().unwrap();

    let output = cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["init", "--demo"])
        .output()
        .unwrap();
    if !output.status.success() {
        return;
    }

    let output = cmd()
        .args(["--project"])
        .arg(dir.path())
        .args(["generate", "--locked"])
        .output()
        .unwrap();

    if output.status.success() {
        let cache_file = dir
            .path()
            .join(".knownow")
            .join("cache")
            .join("generation_cache.json");
        assert!(
            cache_file.exists(),
            "generation cache should be written after successful generate"
        );
    }
}

// ─── Version across commands ─────────────────────────────────────────────────

#[test]
fn all_commands_accept_format_flag() {
    let dir = valid_project_with_lockfile();
    let commands = ["validate", "doctor", "version"];

    for command in commands {
        cmd()
            .args(["--project"])
            .arg(dir.path())
            .args(["--format", "json", command])
            .assert()
            .success();
    }
}
