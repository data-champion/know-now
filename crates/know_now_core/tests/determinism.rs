use know_now_codegen::generator::Generator;
use know_now_core::projection::project_graph_to_contract;
use know_now_gen_docs::DocsGenerator;
use know_now_gen_postgres::PostgresGenerator;
use know_now_metadata::test_support::parse_yaml_metadata;
use know_now_validate::builder::build_project_graph;

const FIXTURE: &str = include_str!("../../../fixtures/validation/valid_full.yml");

fn run_full_pipeline() -> Vec<(String, String)> {
    let metadata = parse_yaml_metadata(FIXTURE);
    let result = build_project_graph(&metadata);
    let graph = result.graph.expect("fixture should produce a valid graph");
    let contract = project_graph_to_contract(&graph);

    let pg = PostgresGenerator::new();
    let docs = DocsGenerator::new();

    let pg_artifacts = pg
        .generate(&contract)
        .expect("postgres generator should succeed");
    let doc_artifacts = docs
        .generate(&contract)
        .expect("docs generator should succeed");

    let mut all: Vec<(String, String)> = pg_artifacts
        .into_iter()
        .chain(doc_artifacts)
        .map(|a| (a.path, a.content))
        .collect();
    all.sort_by(|a, b| a.0.cmp(&b.0));
    all
}

#[test]
fn byte_identical_across_two_runs() {
    let run_a = run_full_pipeline();
    let run_b = run_full_pipeline();

    assert!(!run_a.is_empty(), "pipeline should produce artifacts");
    assert_eq!(
        run_a.len(),
        run_b.len(),
        "both runs should produce the same number of artifacts"
    );

    for (a, b) in run_a.iter().zip(run_b.iter()) {
        assert_eq!(a.0, b.0, "artifact paths should match");
        assert_eq!(
            a.1, b.1,
            "artifact content should be byte-identical for {}",
            a.0
        );
    }
}

#[test]
fn pipeline_produces_expected_artifact_paths() {
    let artifacts = run_full_pipeline();
    let paths: Vec<&str> = artifacts.iter().map(|(p, _)| p.as_str()).collect();

    assert!(paths.contains(&"ddl/postgres/schema.sql"));
    assert!(paths.contains(&"docs/entities/customer.md"));
    assert!(paths.contains(&"docs/entities/order.md"));
}

#[test]
fn no_crlf_in_any_artifact() {
    let artifacts = run_full_pipeline();
    for (path, content) in &artifacts {
        assert!(!content.contains('\r'), "NFR-PO3: LF only in {path}");
    }
}

#[test]
fn no_timestamps_in_any_artifact() {
    let artifacts = run_full_pipeline();
    for (path, content) in &artifacts {
        assert!(!content.contains("2026"), "no year in {path}");
        assert!(!content.contains("Generated at"), "no timestamp in {path}");
    }
}

#[test]
fn contract_version_present_in_projection() {
    let metadata = parse_yaml_metadata(FIXTURE);
    let result = build_project_graph(&metadata);
    let graph = result.graph.expect("fixture should produce a valid graph");
    let contract = project_graph_to_contract(&graph);

    assert_eq!(contract.contract_version, "1.0");
}

#[test]
fn trace_covers_all_entities_and_attributes() {
    let metadata = parse_yaml_metadata(FIXTURE);
    let result = build_project_graph(&metadata);
    let graph = result.graph.expect("fixture should produce a valid graph");
    let contract = project_graph_to_contract(&graph);

    assert!(!contract.trace.entity_ids.is_empty());
    assert!(!contract.trace.attribute_ids.is_empty());
    assert!(!contract.trace.relationship_ids.is_empty());

    for entity in &contract.entities {
        assert!(
            contract.trace.entity_ids.contains(&entity.id),
            "trace should contain entity {}",
            entity.id
        );
        for attr in &entity.attributes {
            assert!(
                contract.trace.attribute_ids.contains(&attr.id),
                "trace should contain attribute {}",
                attr.id
            );
        }
    }
}
