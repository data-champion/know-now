use know_now_diagnostics::diagnostic::Diagnostic;
use know_now_metadata::test_support::parse_yaml_metadata;
use know_now_policy::dc_standard::DcStandard;
use know_now_policy::engine::PolicyPack;

fn eval(yaml: &str) -> Vec<Diagnostic> {
    let meta = parse_yaml_metadata(yaml);
    DcStandard::new().evaluate(&meta)
}

fn has_code(diags: &[Diagnostic], code: &str) -> bool {
    diags.iter().any(|d| d.code == code)
}

const BASELINE: &str = r#"
entities:
  - name: customer
    description: "A registered customer account."
    business_key: [email]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: "Auto-incremented primary key."
      - name: email
        logical_type: string
        required: true
        description: "Contact email address."
"#;

#[test]
fn baseline_produces_zero_warnings() {
    let diags = eval(BASELINE);
    assert!(
        diags.is_empty(),
        "baseline must be warning-free, got: {diags:?}"
    );
}

#[test]
fn mutate_entity_name_non_snake_case() {
    let yaml = BASELINE.replace("name: customer", "name: Customer");
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-NAM-001"));
    assert!(has_code(&diags, "POL-NAM-004"));
}

#[test]
fn mutate_attribute_name_non_snake_case() {
    let yaml = BASELINE.replace("name: email", "name: emailAddress");
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-NAM-002"));
}

#[test]
fn mutate_remove_business_key() {
    let yaml = BASELINE.replace("business_key: [email]", "business_key: []");
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-ENT-002"));
}

#[test]
fn mutate_remove_pk_candidate_and_business_key() {
    let yaml = BASELINE
        .replace("business_key: [email]", "business_key: []")
        .replace("unique: true", "unique: false");
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-ENT-001"));
    assert!(has_code(&diags, "POL-ENT-002"));
}

#[test]
fn mutate_remove_entity_description() {
    let yaml = BASELINE.replace(
        "description: \"A registered customer account.\"",
        "description: \"\"",
    );
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-DOC-001"));
}

#[test]
fn mutate_remove_required_attr_description() {
    let yaml = BASELINE.replace(
        "description: \"Auto-incremented primary key.\"",
        "description: \"\"",
    );
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-DOC-002"));
}

#[test]
fn mutate_non_ascii_entity_name() {
    let yaml = BASELINE.replace("name: customer", "name: café");
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-NAM-004"));
}

#[test]
fn mutate_non_ascii_attribute_name() {
    let yaml = BASELINE.replace("name: email", "name: ëmail");
    let diags = eval(&yaml);
    assert!(has_code(&diags, "POL-NAM-004"));
}

#[test]
fn each_rule_code_has_at_least_one_triggering_mutation() {
    let pack = DcStandard::new();
    let expected_codes: Vec<&str> = pack.rules().iter().map(|r| r.code).collect();

    let all_mutations = vec![
        BASELINE.replace("name: customer", "name: Customer"),
        BASELINE.replace("name: email", "name: emailAddress"),
        BASELINE.replace("name: customer", "name: café"),
        BASELINE.replace("business_key: [email]", "business_key: []"),
        BASELINE
            .replace("business_key: [email]", "business_key: []")
            .replace("unique: true", "unique: false"),
        BASELINE.replace(
            "description: \"A registered customer account.\"",
            "description: \"\"",
        ),
        BASELINE.replace(
            "description: \"Auto-incremented primary key.\"",
            "description: \"\"",
        ),
        format!("{BASELINE}\nmodules:\n  - id: BadModule\n    name: Bad"),
    ];

    let mut triggered: Vec<String> = Vec::new();
    for mutation in &all_mutations {
        let diags = eval(mutation);
        for d in &diags {
            triggered.push(d.code.clone());
        }
    }
    triggered.sort_unstable();
    triggered.dedup();

    for code in &expected_codes {
        assert!(
            triggered.iter().any(|t| t == code),
            "rule {code} was never triggered by any mutation"
        );
    }
}

#[test]
fn single_mutation_triggers_warnings() {
    assert!(eval(BASELINE).is_empty());

    let mutations: Vec<(&str, &str)> = vec![
        ("name: customer", "name: Customer"),
        ("business_key: [email]", "business_key: []"),
        (
            "description: \"A registered customer account.\"",
            "description: \"\"",
        ),
    ];

    for (from, to) in mutations {
        let yaml = BASELINE.replace(from, to);
        let diags = eval(&yaml);
        assert!(
            !diags.is_empty(),
            "mutation '{from}' -> '{to}' should trigger at least one warning"
        );
    }
}
