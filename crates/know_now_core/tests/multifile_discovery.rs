use std::path::Path;

use know_now_core::project_loader::{discover_metadata_files, load_project};
use know_now_metadata::budgets::ParserBudgets;

const FIXTURE_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/multifile/metadata"
);

#[test]
fn discovers_all_three_fixture_files() {
    let dir = Path::new(FIXTURE_DIR);
    let files = discover_metadata_files(dir).expect("discovery should succeed");
    assert_eq!(files.len(), 3, "should find 3 YAML files");
}

#[test]
fn files_are_sorted_deterministically() {
    let dir = Path::new(FIXTURE_DIR);
    let files_a = discover_metadata_files(dir).unwrap();
    let files_b = discover_metadata_files(dir).unwrap();
    assert_eq!(files_a, files_b, "discovery must be deterministic");

    let names: Vec<_> = files_a
        .iter()
        .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["01_project.yml", "02_entities.yml", "03_relationships.yml"]
    );
}

#[test]
fn load_merges_across_files_correctly() {
    let dir = Path::new(FIXTURE_DIR);
    let loaded = load_project(dir, &ParserBudgets::default()).expect("load should succeed");

    assert_eq!(loaded.file_count, 3);
    assert_eq!(loaded.metadata.version, Some("1.0".to_owned()));

    let project = loaded
        .metadata
        .project
        .as_ref()
        .expect("project should exist");
    assert_eq!(project.name, "multifile_demo");

    assert_eq!(loaded.metadata.domains.len(), 1);
    assert_eq!(loaded.metadata.modules.len(), 1);
    assert_eq!(loaded.metadata.entities.len(), 2);
    assert_eq!(loaded.metadata.relationships.len(), 1);
    assert_eq!(loaded.metadata.sources.len(), 1);
    assert!(loaded.metadata.governance.is_some());
}

#[test]
fn cross_file_entity_references_present() {
    let dir = Path::new(FIXTURE_DIR);
    let loaded = load_project(dir, &ParserBudgets::default()).unwrap();

    let rel = &loaded.metadata.relationships[0];
    assert_eq!(rel.from_entity, "order");
    assert_eq!(rel.to_entity, "customer");

    let entity_names: Vec<&str> = loaded
        .metadata
        .entities
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(entity_names.contains(&"customer"));
    assert!(entity_names.contains(&"order"));
}
