#![allow(
    clippy::cast_precision_loss,
    clippy::useless_format
)]

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use know_now_metadata::budgets::ParserBudgets;

fn generate_entity_yaml(count: usize) -> String {
    let mut lines = vec!["entities:".to_owned()];
    for i in 0..count {
        let name = format!("entity_{i:04}");
        lines.push(format!("  - id: ent_{name}"));
        lines.push(format!("    name: {name}"));
        lines.push("    business_key: [id]".to_owned());
        lines.push("    attributes:".to_owned());
        lines.push(format!("      - id: attr_{name}_id"));
        lines.push("        name: id".to_owned());
        lines.push("        logical_type: integer".to_owned());
        lines.push("        required: true".to_owned());
        lines.push(format!("      - id: attr_{name}_name"));
        lines.push(format!("        name: {name}_name"));
        lines.push("        logical_type: string".to_owned());
        lines.push("        required: true".to_owned());
        lines.push(format!("      - id: attr_{name}_created"));
        lines.push(format!("        name: {name}_created"));
        lines.push("        logical_type: timestamp".to_owned());
    }
    lines.join("\n")
}

fn temp_fixture(name: &str, count: usize) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("know_now_scale_{name}_{}", std::process::id()));
    let meta_dir = dir.join("metadata");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&meta_dir).unwrap();
    fs::write(meta_dir.join("entities.yml"), generate_entity_yaml(count)).unwrap();
    dir
}

fn parse_and_measure(count: usize) -> std::time::Duration {
    let dir = temp_fixture(&count.to_string(), count);
    let meta_dir = dir.join("metadata");
    let start = Instant::now();
    let result =
        know_now_core::project_loader::load_project(&meta_dir, &ParserBudgets::default());
    let elapsed = start.elapsed();
    assert!(result.is_ok(), "parse failed for {count} entities");
    let project = result.unwrap();
    assert_eq!(project.metadata.entities.len(), count);
    let _ = fs::remove_dir_all(&dir);
    elapsed
}

// NFR-SC1: metadata model supports 200+ entities within performance targets.
#[test]
fn nfr_sc1_200_entities_parse_within_budget() {
    let elapsed = parse_and_measure(200);
    assert!(
        elapsed.as_millis() < 5000,
        "parsing 200 entities took {elapsed:?} — budget is 5 s"
    );
}

// NFR-SC3: multi-file parsing scales approximately linearly.
#[test]
fn nfr_sc3_parsing_scales_approximately_linearly() {
    let t10 = parse_and_measure(10);
    let t100 = parse_and_measure(100);
    let t200 = parse_and_measure(200);

    let ratio_100_to_10 = t100.as_nanos() as f64 / t10.as_nanos().max(1) as f64;
    let ratio_200_to_100 = t200.as_nanos() as f64 / t100.as_nanos().max(1) as f64;

    assert!(
        ratio_100_to_10 < 50.0,
        "100/10 entity ratio was {ratio_100_to_10:.1}x — expected sub-linear or linear \
         (t10={t10:?}, t100={t100:?})"
    );

    assert!(
        ratio_200_to_100 < 10.0,
        "200/100 entity ratio was {ratio_200_to_100:.1}x — expected approximately linear \
         (t100={t100:?}, t200={t200:?})"
    );
}

// NFR-SC2: new built-in targets added without modifying existing generators.
// The architecture fitness test `generators_no_cross_deps` verifies that no
// generator crate depends on another, proving new generators can be added
// independently. This test confirms the stress fixture itself is valid input.
#[test]
fn nfr_sc2_stress_fixture_parses_and_validates() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/stress_large/metadata");
    if !fixture_path.exists() {
        eprintln!("stress_large fixture not found — skipping");
        return;
    }
    let result =
        know_now_core::project_loader::load_project(&fixture_path, &ParserBudgets::default());
    assert!(result.is_ok(), "stress fixture failed to parse: {result:?}");
    let project = result.unwrap();
    assert_eq!(project.metadata.entities.len(), 200);
    assert_eq!(project.metadata.relationships.len(), 1000);
}

// NFR-SC4: relationship graph supports 5x relationship-to-entity ratio.
#[test]
fn nfr_sc4_relationship_ratio() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/stress_large/metadata");
    if !fixture_path.exists() {
        eprintln!("stress_large fixture not found — skipping");
        return;
    }
    let project = know_now_core::project_loader::load_project(
        &fixture_path,
        &ParserBudgets::default(),
    )
    .unwrap();

    let entity_count = project.metadata.entities.len();
    let rel_count = project.metadata.relationships.len();
    let ratio = rel_count as f64 / entity_count as f64;

    assert!(
        ratio >= 5.0,
        "relationship-to-entity ratio is {ratio:.1} — expected >= 5.0 \
         ({rel_count} rels / {entity_count} entities)"
    );
}
