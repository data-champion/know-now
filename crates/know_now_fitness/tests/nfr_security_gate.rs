//! Cross-cutting NFR-Security fitness tests.
//!
//! Verifies that no crate re-introduces banned security patterns.
//! Each test maps to an NFR-S item from PRD §17.2.

use std::path::Path;
use std::process::Command;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn source_files_in(crate_dir: &str) -> Vec<String> {
    let src = workspace_root().join("crates").join(crate_dir).join("src");
    let mut files = Vec::new();
    collect_rs_files(&src, &mut files);
    files
}

fn collect_rs_files(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                out.push(content);
            }
        }
    }
}

// NFR-S1: SQL generators use typed IR, no raw string interpolation
#[test]
fn nfr_s1_no_raw_sql_interpolation_in_generators() {
    let generator_crates = [
        "know_now_gen_postgres",
        "know_now_gen_dbt",
        "know_now_gen_quality",
    ];

    for crate_name in &generator_crates {
        let files = source_files_in(crate_name);
        for content in &files {
            for (_line_num, line) in content.lines().enumerate() {
                if line.contains("format!(") && line.contains("SELECT")
                    || line.contains("format!(") && line.contains("INSERT")
                    || line.contains("format!(") && line.contains("UPDATE")
                    || line.contains("format!(") && line.contains("DELETE")
                {
                    if !line.contains("//") && !line.trim().starts_with("//") {
                        let is_test_context = line.contains("test")
                            || line.contains("Test")
                            || line.contains("comment")
                            || line.contains("--");
                        assert!(
                            is_test_context,
                            "NFR-S1 VIOLATED — raw SQL format! in {crate_name}: {line}"
                        );
                    }
                }
            }
        }
    }
}

// NFR-S4: No secrets/connection strings stored in metadata fixtures
#[test]
fn nfr_s4_no_secrets_in_metadata_fixtures() {
    let metadata_dir = workspace_root().join("crates");
    let mut yaml_files = Vec::new();
    find_yaml_fixtures(&metadata_dir, &mut yaml_files);

    let secret_patterns = [
        "password=",
        "secret_key",
        "api_key=",
        "AWS_SECRET",
        "PRIVATE_KEY",
        "-----BEGIN",
    ];

    for path in &yaml_files {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for pattern in &secret_patterns {
            assert!(
                !content.contains(pattern),
                "NFR-S4 VIOLATED — potential secret in fixture {}: contains '{pattern}'",
                path.display()
            );
        }
    }
}

fn find_yaml_fixtures(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "target" || name.starts_with('.') {
                continue;
            }
            find_yaml_fixtures(&path, out);
        } else if path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
            out.push(path);
        }
    }
}

// NFR-S5: Cargo dependencies audited (cargo-deny check runs)
#[test]
fn nfr_s5_cargo_deny_config_exists() {
    let deny_toml = workspace_root().join("deny.toml");
    assert!(
        deny_toml.exists(),
        "NFR-S5 — deny.toml must exist for cargo-deny audit"
    );
}

// NFR-S13: YAML parser uses serde-saphyr (not yaml-rust or serde_yaml)
#[test]
fn nfr_s13_parser_uses_serde_saphyr() {
    let metadata_files = source_files_in("know_now_metadata");
    let uses_saphyr = metadata_files
        .iter()
        .any(|content| content.contains("serde_saphyr") || content.contains("serde-saphyr"));
    assert!(
        uses_saphyr,
        "NFR-S13 — know_now_metadata must use serde-saphyr for YAML parsing"
    );
}

// NFR-S14: Server default is localhost
#[test]
fn nfr_s14_server_default_localhost() {
    let server_files = source_files_in("know_now_server");
    let has_localhost_default = server_files.iter().any(|content| {
        content.contains("127.0.0.1") || content.contains("Ipv4Addr::LOCALHOST")
    });
    assert!(
        has_localhost_default,
        "NFR-S14 — server must reference 127.0.0.1 or LOCALHOST as default"
    );
}

// NFR-S16: Write endpoints are feature-gated
#[test]
fn nfr_s16_write_endpoints_feature_gated() {
    let server_files = source_files_in("know_now_server");
    let has_cfg_gate = server_files.iter().any(|content| {
        content.contains(r#"cfg(feature = "allow-generate")"#)
    });
    assert!(
        has_cfg_gate,
        "NFR-S16 — server write endpoints must be behind allow-generate feature gate"
    );
}

// NFR-S22..S25: Template renderer restrictions enforced
#[test]
fn nfr_s22_templates_use_strict_undefined() {
    let template_files = source_files_in("know_now_templates");
    let has_strict = template_files
        .iter()
        .any(|content| content.contains("UndefinedBehavior::Strict"));
    assert!(
        has_strict,
        "NFR-S22 — template renderer must use strict undefined behavior"
    );
}

#[test]
fn nfr_s23_templates_use_fuel() {
    let template_files = source_files_in("know_now_templates");
    let has_fuel = template_files.iter().any(|content| content.contains("set_fuel"));
    assert!(
        has_fuel,
        "NFR-S23 — template renderer must set a fuel limit"
    );
}

// Cross-cutting: no query-string tokens in server API routes
#[test]
fn no_query_string_auth_in_server() {
    let server_files = source_files_in("know_now_server");
    for content in &server_files {
        assert!(
            !content.contains("query_token") && !content.contains("bearer_token"),
            "Server must not accept auth tokens via query string (AGENTS.md §8)"
        );
    }
}

// Git hooks or CI config exists
#[test]
fn nfr_s10_lockfiles_tracked() {
    let cargo_lock = workspace_root().join("Cargo.lock");
    assert!(
        cargo_lock.exists(),
        "NFR-S10 — Cargo.lock must be committed"
    );

    let output = Command::new("git")
        .args(["ls-files", "--error-unmatch", "Cargo.lock"])
        .current_dir(workspace_root())
        .output();

    if let Ok(out) = output {
        assert!(
            out.status.success(),
            "NFR-S10 — Cargo.lock must be tracked by git"
        );
    }
}
