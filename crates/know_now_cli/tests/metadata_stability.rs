use assert_cmd::Command;
use sha2::{Digest, Sha256};
use std::path::Path;

fn cmd() -> Command {
    Command::cargo_bin("know-now").expect("binary should exist")
}

/// Recursively hash all file contents and relative paths under `dir`.
///
/// Returns a hex-encoded SHA-256 digest that captures both the file contents
/// and their relative paths, sorted for determinism.
fn hash_directory_tree(dir: &Path) -> String {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    collect_files(dir, dir, &mut entries);
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (rel_path, contents) in &entries {
        hasher.update(rel_path.as_bytes());
        hasher.update(b"\x00");
        hasher.update(contents);
        hasher.update(b"\x00");
    }
    format!("{:x}", hasher.finalize())
}

fn collect_files(base: &Path, current: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, out);
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if let Ok(contents) = std::fs::read(&path) {
                out.push((rel, contents));
            }
        }
    }
}

/// Create a demo project via `know-now init --demo`, returning the project dir.
fn create_demo_project(parent: &Path) -> std::path::PathBuf {
    cmd()
        .args(["--project"])
        .arg(parent)
        .args(["init", "--demo"])
        .assert()
        .success();
    parent.join("demo-project")
}

/// Assert that running the given CLI args against a demo project does not
/// modify the metadata/ directory.
fn assert_metadata_stable(command_args: &[&str], test_label: &str) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project = create_demo_project(tmp.path());
    let metadata_dir = project.join("metadata");
    assert!(metadata_dir.is_dir(), "metadata/ should exist after init");

    let hash_before = hash_directory_tree(&metadata_dir);

    // Run the command — we don't care about exit code, only that metadata is unchanged
    let mut c = cmd();
    c.args(["--project"]).arg(&project);
    for arg in command_args {
        c.arg(arg);
    }
    let _ = c.output().expect("command should run");

    let hash_after = hash_directory_tree(&metadata_dir);

    assert_eq!(
        hash_before, hash_after,
        "{test_label}: metadata/ directory was modified by a read-only command"
    );
}

#[test]
fn metadata_stable_after_validate() {
    assert_metadata_stable(&["validate"], "validate");
}

#[test]
fn metadata_stable_after_check() {
    assert_metadata_stable(&["check"], "check");
}

#[test]
fn metadata_stable_after_schema() {
    assert_metadata_stable(&["schema"], "schema");
}

#[test]
fn metadata_stable_after_id_check() {
    assert_metadata_stable(&["id", "check"], "id check");
}

#[test]
fn metadata_stable_after_id_suggest() {
    assert_metadata_stable(&["id", "suggest"], "id suggest");
}

#[test]
fn metadata_stable_after_lock_check() {
    assert_metadata_stable(&["lock", "check"], "lock check");
}

#[test]
fn metadata_stable_after_version() {
    assert_metadata_stable(&["version"], "version");
}

#[test]
fn metadata_stable_after_config_inspect() {
    assert_metadata_stable(&["config", "inspect"], "config inspect");
}
