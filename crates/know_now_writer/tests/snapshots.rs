use know_now_writer::manifest::{ManifestV1, PolicyRef, TargetDatabase};
use know_now_writer::manifest_builder::{ArtifactInput, ManifestBuilder, TraceInput};

/// Build a representative manifest via the builder API.
fn sample_manifest() -> ManifestV1 {
    let mut builder = ManifestBuilder::new("1.0.0", "1.0")
        .project_id("ecommerce_demo")
        .input_hash_from_bytes(b"stable-input-content")
        .lockfile_hash("sha256:lockfile000000000000000000000000000000000000000000000000000000");

    builder = builder.target_database(TargetDatabase {
        kind: "postgres".into(),
        version: "18".into(),
        compatibility_floor: "16".into(),
    });

    builder = builder.policy(PolicyRef {
        pack: "dc_standard".into(),
        version: "1.0".into(),
        hash: "sha256:pol000000000000000000000000000000000000000000000000000000000000".into(),
    });

    builder.add_artifact(ArtifactInput {
        path: "ddl/postgres/schema.sql".into(),
        kind: "postgres_ddl".into(),
        artifact_id: "art_postgres_schema".into(),
        generator: "know_now_gen_postgres".into(),
        generator_version: "1.0.0".into(),
        content: b"CREATE TABLE customer (\n    id INTEGER NOT NULL\n);\n".to_vec(),
        metadata_object_ids: vec!["ent_customer".into(), "attr_customer_id".into()],
        trace: vec![TraceInput {
            line_start: 1,
            line_end: 3,
            metadata_object_ids: vec!["ent_customer".into()],
            policy_rule_ids: vec!["pol_naming_convention".into()],
        }],
    });

    builder.add_artifact(ArtifactInput {
        path: "docs/entities/customer.md".into(),
        kind: "markdown_doc".into(),
        artifact_id: "art_docs_customer".into(),
        generator: "know_now_gen_docs".into(),
        generator_version: "1.0.0".into(),
        content: b"# Customer\n".to_vec(),
        metadata_object_ids: vec!["ent_customer".into()],
        trace: vec![],
    });

    builder = builder.warning("Entity `order` has no business key defined.".into());

    builder.build()
}

#[test]
fn manifest_json_snapshot() {
    let manifest = sample_manifest();
    let json = manifest.to_json_pretty();
    insta::assert_snapshot!(json);
}

#[test]
fn manifest_minimal_snapshot() {
    // A minimal manifest with no artifacts and no warnings.
    let builder = ManifestBuilder::new("0.1.0", "1.0")
        .project_id("empty_project")
        .input_hash_from_bytes(b"empty")
        .lockfile_hash("sha256:empty000000000000000000000000000000000000000000000000000000000")
        .target_database(TargetDatabase {
            kind: "postgres".into(),
            version: "16".into(),
            compatibility_floor: "14".into(),
        })
        .policy(PolicyRef {
            pack: "none".into(),
            version: "0.0.0".into(),
            hash: "sha256:none0000000000000000000000000000000000000000000000000000000000".into(),
        });

    let manifest = builder.build();
    let json = manifest.to_json_pretty();
    insta::assert_snapshot!(json);
}
