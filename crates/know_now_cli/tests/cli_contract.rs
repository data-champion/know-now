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
    cmd()
        .arg("nonexistent")
        .assert()
        .code(predicate::eq(2));
}

#[test]
fn stub_commands_exit_with_validation_error() {
    let stubs = [
        vec!["init"],
        vec!["validate"],
        vec!["check"],
        vec!["schema"],
        vec!["generate"],
        vec!["lock", "update"],
        vec!["lock", "check"],
        vec!["id", "check"],
        vec!["id", "suggest"],
        vec!["id", "backfill", "--dry-run"],
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
    for flag in &["--format", "--verbose", "--debug", "--config", "--project", "--no-color"] {
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
