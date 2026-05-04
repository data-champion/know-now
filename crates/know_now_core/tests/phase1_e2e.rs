use std::fs;
use std::path::Path;

use know_now_codegen::artifact::ArtifactDescriptor as CodegenArtifact;
use know_now_codegen::generator::Generator;
use know_now_core::projection::project_graph_to_contract;
use know_now_gen_docs::DocsGenerator;
use know_now_gen_postgres::PostgresGenerator;
use know_now_metadata::test_support::parse_yaml_metadata;
use know_now_toolchain::{RunRecord, RunResult, VolatileStateStore};
use know_now_validate::builder::build_project_graph;
use know_now_writer::generation::{ArtifactDescriptor as WriterArtifact, GenerationSession};
use know_now_writer::manifest::{
    ArtifactEntry, ManifestV1, PolicyRef, TargetDatabase, MANIFEST_SCHEMA_VERSION,
};

const FIXTURE: &str = include_str!("../../../fixtures/validation/valid_full.yml");

struct PipelineOutput {
    artifacts: Vec<CodegenArtifact>,
    manifest: ManifestV1,
}

fn run_pipeline() -> PipelineOutput {
    let metadata = parse_yaml_metadata(FIXTURE);
    let result = build_project_graph(&metadata);
    let graph = result.graph.expect("fixture should produce a valid graph");
    let contract = project_graph_to_contract(&graph);

    let pg = PostgresGenerator::new();
    let docs = DocsGenerator::new();

    let mut artifacts = pg
        .generate(&contract)
        .expect("postgres generator should succeed");
    artifacts.extend(
        docs.generate(&contract)
            .expect("docs generator should succeed"),
    );
    artifacts.sort_by(|a, b| a.path.cmp(&b.path));

    let manifest_artifacts: Vec<ArtifactEntry> = artifacts
        .iter()
        .map(|a| ArtifactEntry {
            path: a.path.clone().into(),
            kind: format!("{:?}", a.kind),
            artifact_id: a.artifact_id.clone(),
            generator: a.generator.clone(),
            generator_version: a.generator_version.clone(),
            hash: format!("sha256:{:064x}", simple_hash(a.content.as_bytes())),
            metadata_object_ids: a.metadata_object_ids.clone(),
            trace: Vec::new(),
        })
        .collect();

    let manifest = ManifestV1 {
        engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        metadata_schema_version: MANIFEST_SCHEMA_VERSION.to_owned(),
        generator_contract_version: contract.contract_version.clone(),
        project_id: contract
            .project
            .as_ref()
            .map_or_else(String::new, |p| p.name.clone()),
        input_hash: format!("sha256:{:064x}", simple_hash(FIXTURE.as_bytes())),
        lockfile_hash: String::new(),
        target_database: TargetDatabase {
            kind: "postgres".into(),
            version: String::new(),
            compatibility_floor: String::new(),
        },
        policy: PolicyRef {
            pack: String::new(),
            version: String::new(),
            hash: String::new(),
        },
        template_renderers: Vec::new(),
        artifacts: manifest_artifacts,
        warnings: Vec::new(),
    };

    PipelineOutput {
        artifacts,
        manifest,
    }
}

fn simple_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in data {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn write_to_disk(project_root: &Path, output: &PipelineOutput, run_id: &str) {
    let knownow_dir = project_root.join(".knownow");
    let target = project_root.join("generated");

    let session =
        GenerationSession::begin(&knownow_dir, run_id, target).expect("session should begin");

    for artifact in &output.artifacts {
        session
            .write_artifact(&WriterArtifact {
                relative_path: artifact.path.clone().into(),
                content: artifact.content.as_bytes().to_vec(),
            })
            .expect("artifact write should succeed");
    }

    let manifest_json = output.manifest.to_json_pretty();
    session
        .write_artifact(&WriterArtifact {
            relative_path: "manifest.json".into(),
            content: manifest_json.into_bytes(),
        })
        .expect("manifest write should succeed");

    session
        .validate(|_, _| Ok(()))
        .expect("validation should pass");
    session.promote().expect("promotion should succeed");

    let store =
        VolatileStateStore::new(project_root, 10).expect("volatile store should initialize");
    store
        .persist_run(&RunRecord {
            run_id: run_id.to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            command: "generate --locked".to_string(),
            result: RunResult::Success,
            manifest_hash: format!(
                "sha256:{:064x}",
                simple_hash(output.manifest.to_json_pretty().as_bytes())
            ),
            duration_ms: 42,
        })
        .expect("run record should persist");
}

fn temp_project(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("know_now_e2e_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

#[test]
fn full_pipeline_writes_artifacts_to_disk() {
    let root = temp_project("writes");
    let output = run_pipeline();
    write_to_disk(&root, &output, "run_001");

    let generated = root.join("generated");
    assert!(generated.join("ddl/postgres/schema.sql").exists());
    assert!(generated.join("docs/entities/customer.md").exists());
    assert!(generated.join("docs/entities/order.md").exists());
    assert!(generated.join("manifest.json").exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn volatile_state_recorded_separately() {
    let root = temp_project("volatile");
    let output = run_pipeline();
    write_to_disk(&root, &output, "run_002");

    let last_gen = root.join(".knownow/last_generation.json");
    assert!(
        last_gen.exists(),
        ".knownow/last_generation.json should exist"
    );

    let content = fs::read_to_string(&last_gen).expect("should read last_generation.json");
    assert!(content.contains("run_002"), "should contain run_id");
    assert!(content.contains("started_at"), "should contain started_at");
    assert!(
        content.contains("finished_at"),
        "should contain finished_at"
    );
    assert!(
        content.contains("duration_ms"),
        "should contain duration_ms"
    );

    let run_file = root.join(".knownow/runs/run_002.json");
    assert!(run_file.exists(), "run file should exist");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn manifest_contains_no_volatile_data() {
    let output = run_pipeline();
    let manifest_json = output.manifest.to_json_pretty();

    assert!(
        !manifest_json.contains("timestamp"),
        "manifest should not contain 'timestamp'"
    );
    assert!(
        !manifest_json.contains("hostname"),
        "manifest should not contain 'hostname'"
    );
    assert!(!manifest_json.contains('\r'), "manifest should use LF only");
}

#[test]
fn manifest_artifacts_sorted_by_path() {
    let output = run_pipeline();
    let paths: Vec<&str> = output
        .manifest
        .artifacts
        .iter()
        .map(|a| a.path.to_str().unwrap())
        .collect();

    let mut sorted = paths.clone();
    sorted.sort_unstable();
    assert_eq!(paths, sorted, "manifest artifacts should be sorted by path");
}

#[test]
fn two_runs_produce_identical_generated_dir() {
    let root_a = temp_project("ident_a");
    let root_b = temp_project("ident_b");

    let output_a = run_pipeline();
    let output_b = run_pipeline();

    write_to_disk(&root_a, &output_a, "run_a");
    write_to_disk(&root_b, &output_b, "run_b");

    let gen_a = root_a.join("generated");
    let gen_b = root_b.join("generated");

    let files_a = collect_files(&gen_a);
    let files_b = collect_files(&gen_b);

    assert_eq!(
        files_a.len(),
        files_b.len(),
        "both runs should produce the same number of files"
    );

    for ((path_a, content_a), (path_b, content_b)) in files_a.iter().zip(files_b.iter()) {
        assert_eq!(path_a, path_b, "paths should match");
        assert_eq!(
            content_a, content_b,
            "content should be byte-identical for {path_a}"
        );
    }

    let _ = fs::remove_dir_all(&root_a);
    let _ = fs::remove_dir_all(&root_b);
}

#[test]
fn volatile_state_differs_between_runs() {
    let root_a = temp_project("vol_a");
    let root_b = temp_project("vol_b");

    let output_a = run_pipeline();
    let output_b = run_pipeline();

    write_to_disk(&root_a, &output_a, "run_alpha");
    write_to_disk(&root_b, &output_b, "run_beta");

    let last_a = fs::read_to_string(root_a.join(".knownow/last_generation.json")).expect("read a");
    let last_b = fs::read_to_string(root_b.join(".knownow/last_generation.json")).expect("read b");

    assert!(last_a.contains("run_alpha"));
    assert!(last_b.contains("run_beta"));
    assert_ne!(last_a, last_b, "volatile state should differ by run_id");

    let _ = fs::remove_dir_all(&root_a);
    let _ = fs::remove_dir_all(&root_b);
}

#[test]
fn generated_files_contain_no_absolute_paths() {
    let root = temp_project("no_abs");
    let output = run_pipeline();
    write_to_disk(&root, &output, "run_abs");

    let generated = root.join("generated");
    for (path, content) in collect_files(&generated) {
        assert!(
            !content.contains(root.to_str().unwrap()),
            "file {path} should not contain absolute project path"
        );
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn promotion_failure_preserves_prior_output() {
    let root = temp_project("preserve");
    let output = run_pipeline();
    write_to_disk(&root, &output, "run_initial");

    let generated = root.join("generated");
    let schema_content =
        fs::read_to_string(generated.join("ddl/postgres/schema.sql")).expect("read schema");

    let knownow = root.join(".knownow");
    let session = GenerationSession::begin(&knownow, "run_fail", generated.clone())
        .expect("session should begin");
    session
        .write_artifact(&WriterArtifact {
            relative_path: "ddl/postgres/schema.sql".into(),
            content: b"BROKEN SQL".to_vec(),
        })
        .expect("write should succeed");

    let validation_result = session.validate(|_, content| {
        if content == b"BROKEN SQL" {
            Err("intentional failure".into())
        } else {
            Ok(())
        }
    });
    assert!(validation_result.is_err());

    let schema_after =
        fs::read_to_string(generated.join("ddl/postgres/schema.sql")).expect("read after");
    assert_eq!(
        schema_content, schema_after,
        "prior output should be preserved on validation failure"
    );

    let _ = fs::remove_dir_all(&root);
}

fn collect_files(dir: &Path) -> Vec<(String, String)> {
    let mut result = Vec::new();
    collect_files_recursive(dir, dir, &mut result);
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

fn collect_files_recursive(base: &Path, dir: &Path, result: &mut Vec<(String, String)>) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).expect("should read dir") {
        let entry = entry.expect("should read entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(base, &path, result);
        } else {
            let relative = path
                .strip_prefix(base)
                .expect("should strip prefix")
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(&path).expect("should read file");
            result.push((relative, content));
        }
    }
}
