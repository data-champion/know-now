use know_now_diagnostics::diagnostic::{Diagnostic, Severity};
use know_now_metadata::authoring::AuthoringMetadata;

use crate::engine::{PolicyPack, PolicyPackInfo, PolicyRule};

pub const PACK_NAME: &str = "dc_standard";
pub const PACK_VERSION: &str = "1.0";
pub const PACK_HASH: &str = "sha256:embedded";

static RULES: &[PolicyRule] = &[
    PolicyRule {
        code: "POL-NAM-001",
        name: "entity_name_snake_case",
        rationale: "Consistent entity naming avoids ambiguity in generated DDL and docs.",
        remediation: "Rename entity to snake_case (e.g., 'CustomerOrder' -> 'customer_order').",
    },
    PolicyRule {
        code: "POL-NAM-002",
        name: "attribute_name_snake_case",
        rationale: "Consistent attribute naming maps cleanly to database columns.",
        remediation: "Rename attribute to snake_case (e.g., 'firstName' -> 'first_name').",
    },
    PolicyRule {
        code: "POL-NAM-003",
        name: "module_name_snake_case",
        rationale: "Consistent module naming keeps the project namespace predictable.",
        remediation: "Rename module id to snake_case (e.g., 'CoreModule' -> 'core_module').",
    },
    PolicyRule {
        code: "POL-NAM-004",
        name: "identifier_lowercase_ascii",
        rationale: "Non-ASCII identifiers risk encoding issues in downstream systems.",
        remediation: "Use only lowercase ASCII letters, digits, and underscores.",
    },
    PolicyRule {
        code: "POL-ENT-001",
        name: "entity_missing_primary_key",
        rationale: "Entities without a primary-key candidate cannot generate correct DDL.",
        remediation: "Add a 'required: true, unique: true' attribute or set business_key.",
    },
    PolicyRule {
        code: "POL-ENT-002",
        name: "entity_missing_business_key",
        rationale: "Business keys support diffing, rename detection, and data quality.",
        remediation: "Add a business_key list naming the natural key attributes.",
    },
    PolicyRule {
        code: "POL-DOC-001",
        name: "entity_missing_description",
        rationale: "Entity descriptions drive generated documentation quality.",
        remediation: "Add a 'description' field to the entity.",
    },
    PolicyRule {
        code: "POL-DOC-002",
        name: "required_attr_missing_description",
        rationale:
            "Required attributes appear prominently in docs; missing descriptions hurt clarity.",
        remediation: "Add a 'description' field to the required attribute.",
    },
];

pub struct DcStandard;

impl DcStandard {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DcStandard {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyPack for DcStandard {
    fn info(&self) -> PolicyPackInfo {
        PolicyPackInfo {
            pack: PACK_NAME.into(),
            version: PACK_VERSION.into(),
            hash: PACK_HASH.into(),
        }
    }

    fn rules(&self) -> &[PolicyRule] {
        RULES
    }

    fn evaluate(&self, metadata: &AuthoringMetadata) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_entity_names(metadata, &mut diagnostics);
        check_attribute_names(metadata, &mut diagnostics);
        check_module_names(metadata, &mut diagnostics);
        check_identifier_ascii(metadata, &mut diagnostics);
        check_entity_primary_key(metadata, &mut diagnostics);
        check_entity_business_key(metadata, &mut diagnostics);
        check_entity_descriptions(metadata, &mut diagnostics);
        check_required_attr_descriptions(metadata, &mut diagnostics);
        diagnostics
    }
}

fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    s.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        && !s.starts_with('_')
        && !s.ends_with('_')
        && !s.contains("__")
}

fn is_lowercase_ascii_identifier(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    s.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn check_entity_names(metadata: &AuthoringMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for (idx, entity) in metadata.entities.iter().enumerate() {
        if !is_snake_case(&entity.name) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-NAM-001",
                    format!("entity name '{}' is not snake_case", entity.name),
                )
                .with_yaml_path(format!("entities[{idx}].name"))
                .with_help("Rename to snake_case (e.g., 'CustomerOrder' -> 'customer_order')."),
            );
        }
    }
}

fn check_attribute_names(metadata: &AuthoringMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for (ent_idx, entity) in metadata.entities.iter().enumerate() {
        for (attr_idx, attr) in entity.attributes.iter().enumerate() {
            if !is_snake_case(&attr.name) {
                diagnostics.push(
                    Diagnostic::new(
                        Severity::Warning,
                        "POL-NAM-002",
                        format!(
                            "attribute '{}' on entity '{}' is not snake_case",
                            attr.name, entity.name
                        ),
                    )
                    .with_yaml_path(format!("entities[{ent_idx}].attributes[{attr_idx}].name"))
                    .with_help("Rename to snake_case (e.g., 'firstName' -> 'first_name')."),
                );
            }
        }
    }
}

fn check_module_names(metadata: &AuthoringMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for (idx, module) in metadata.modules.iter().enumerate() {
        if !is_snake_case(&module.id) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-NAM-003",
                    format!("module id '{}' is not snake_case", module.id),
                )
                .with_yaml_path(format!("modules[{idx}].id"))
                .with_help("Rename to snake_case (e.g., 'CoreModule' -> 'core_module')."),
            );
        }
    }
}

fn check_identifier_ascii(metadata: &AuthoringMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for (idx, entity) in metadata.entities.iter().enumerate() {
        if !is_lowercase_ascii_identifier(&entity.name) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-NAM-004",
                    format!(
                        "entity name '{}' contains non-lowercase-ASCII characters",
                        entity.name
                    ),
                )
                .with_yaml_path(format!("entities[{idx}].name"))
                .with_help("Use only lowercase ASCII letters, digits, and underscores."),
            );
        }
        for (attr_idx, attr) in entity.attributes.iter().enumerate() {
            if !is_lowercase_ascii_identifier(&attr.name) {
                diagnostics.push(
                    Diagnostic::new(
                        Severity::Warning,
                        "POL-NAM-004",
                        format!(
                            "attribute name '{}' on entity '{}' contains non-lowercase-ASCII characters",
                            attr.name, entity.name
                        ),
                    )
                    .with_yaml_path(format!("entities[{idx}].attributes[{attr_idx}].name"))
                    .with_help("Use only lowercase ASCII letters, digits, and underscores."),
                );
            }
        }
    }
    for (idx, domain) in metadata.domains.iter().enumerate() {
        if !is_lowercase_ascii_identifier(&domain.id) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-NAM-004",
                    format!(
                        "domain id '{}' contains non-lowercase-ASCII characters",
                        domain.id
                    ),
                )
                .with_yaml_path(format!("domains[{idx}].id"))
                .with_help("Use only lowercase ASCII letters, digits, and underscores."),
            );
        }
    }
    for (idx, module) in metadata.modules.iter().enumerate() {
        if !is_lowercase_ascii_identifier(&module.id) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-NAM-004",
                    format!(
                        "module id '{}' contains non-lowercase-ASCII characters",
                        module.id
                    ),
                )
                .with_yaml_path(format!("modules[{idx}].id"))
                .with_help("Use only lowercase ASCII letters, digits, and underscores."),
            );
        }
    }
}

fn check_entity_primary_key(metadata: &AuthoringMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for (idx, entity) in metadata.entities.iter().enumerate() {
        let has_pk_attr = entity
            .attributes
            .iter()
            .any(|a| a.required == Some(true) && a.is_unique == Some(true));
        let has_business_key = !entity.business_key.is_empty();
        if !has_pk_attr && !has_business_key {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-ENT-001",
                    format!(
                        "entity '{}' has no primary-key candidate (no required+unique attribute and no business_key)",
                        entity.name
                    ),
                )
                .with_yaml_path(format!("entities[{idx}]"))
                .with_help("Add a 'required: true, unique: true' attribute or set business_key."),
            );
        }
    }
}

fn check_entity_business_key(metadata: &AuthoringMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for (idx, entity) in metadata.entities.iter().enumerate() {
        if entity.business_key.is_empty() {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-ENT-002",
                    format!("entity '{}' has no business_key defined", entity.name),
                )
                .with_yaml_path(format!("entities[{idx}]"))
                .with_help("Add a business_key list naming the natural key attributes."),
            );
        }
    }
}

fn check_entity_descriptions(metadata: &AuthoringMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for (idx, entity) in metadata.entities.iter().enumerate() {
        if entity
            .description
            .as_ref()
            .is_none_or(|d| d.trim().is_empty())
        {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "POL-DOC-001",
                    format!("entity '{}' has no description", entity.name),
                )
                .with_yaml_path(format!("entities[{idx}].description"))
                .with_help("Add a 'description' field to the entity."),
            );
        }
    }
}

fn check_required_attr_descriptions(
    metadata: &AuthoringMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (ent_idx, entity) in metadata.entities.iter().enumerate() {
        for (attr_idx, attr) in entity.attributes.iter().enumerate() {
            if attr.required == Some(true)
                && attr
                    .description
                    .as_ref()
                    .is_none_or(|d| d.trim().is_empty())
            {
                diagnostics.push(
                    Diagnostic::new(
                        Severity::Warning,
                        "POL-DOC-002",
                        format!(
                            "required attribute '{}' on entity '{}' has no description",
                            attr.name, entity.name
                        ),
                    )
                    .with_yaml_path(format!(
                        "entities[{ent_idx}].attributes[{attr_idx}].description"
                    ))
                    .with_help("Add a 'description' field to the required attribute."),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use know_now_metadata::test_support::parse_yaml_metadata;

    use super::*;

    fn eval(yaml: &str) -> Vec<Diagnostic> {
        let meta = parse_yaml_metadata(yaml);
        DcStandard::new().evaluate(&meta)
    }

    fn has_code(diagnostics: &[Diagnostic], code: &str) -> bool {
        diagnostics.iter().any(|d| d.code == code)
    }

    #[test]
    fn clean_metadata_no_warnings() {
        let diags = eval(
            r"
entities:
  - name: customer
    description: A customer
    business_key: [email]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: Primary key
      - name: email
        logical_type: string
        semantic_type: email
",
        );
        assert!(diags.is_empty(), "expected no warnings, got: {diags:?}");
    }

    #[test]
    fn non_snake_case_entity_name() {
        let diags = eval(
            r"
entities:
  - name: CustomerOrder
    description: test
    business_key: [id]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: pk
",
        );
        assert!(has_code(&diags, "POL-NAM-001"));
    }

    #[test]
    fn non_snake_case_attribute_name() {
        let diags = eval(
            r"
entities:
  - name: customer
    description: test
    business_key: [firstName]
    attributes:
      - name: firstName
        logical_type: string
        required: true
        unique: true
        description: pk
",
        );
        assert!(has_code(&diags, "POL-NAM-002"));
    }

    #[test]
    fn non_snake_case_module_id() {
        let diags = eval(
            r"
modules:
  - id: CoreModule
    name: Core
entities: []
",
        );
        assert!(has_code(&diags, "POL-NAM-003"));
    }

    #[test]
    fn non_ascii_identifier() {
        let diags = eval(
            r#"
entities:
  - name: "café"
    description: test
    business_key: [id]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: pk
"#,
        );
        assert!(has_code(&diags, "POL-NAM-004"));
    }

    #[test]
    fn missing_primary_key_and_business_key() {
        let diags = eval(
            r"
entities:
  - name: customer
    description: test
    attributes:
      - name: id
        logical_type: integer
",
        );
        assert!(has_code(&diags, "POL-ENT-001"));
        assert!(has_code(&diags, "POL-ENT-002"));
    }

    #[test]
    fn business_key_satisfies_pk_check() {
        let diags = eval(
            r"
entities:
  - name: customer
    description: test
    business_key: [email]
    attributes:
      - name: email
        logical_type: string
",
        );
        assert!(!has_code(&diags, "POL-ENT-001"));
        assert!(!has_code(&diags, "POL-ENT-002"));
    }

    #[test]
    fn required_unique_satisfies_pk_check() {
        let diags = eval(
            r"
entities:
  - name: customer
    description: test
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: pk
",
        );
        assert!(!has_code(&diags, "POL-ENT-001"));
    }

    #[test]
    fn missing_entity_description() {
        let diags = eval(
            r"
entities:
  - name: customer
    business_key: [id]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: pk
",
        );
        assert!(has_code(&diags, "POL-DOC-001"));
    }

    #[test]
    fn empty_entity_description() {
        let diags = eval(
            r#"
entities:
  - name: customer
    description: "  "
    business_key: [id]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: pk
"#,
        );
        assert!(has_code(&diags, "POL-DOC-001"));
    }

    #[test]
    fn required_attr_missing_description() {
        let diags = eval(
            r"
entities:
  - name: customer
    description: test
    business_key: [id]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
",
        );
        assert!(has_code(&diags, "POL-DOC-002"));
    }

    #[test]
    fn non_required_attr_no_description_ok() {
        let diags = eval(
            r"
entities:
  - name: customer
    description: test
    business_key: [id]
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: pk
      - name: optional_field
        logical_type: string
",
        );
        assert!(!has_code(&diags, "POL-DOC-002"));
    }

    #[test]
    fn all_warnings_not_errors() {
        let diags = eval(
            r"
entities:
  - name: BadName
    attributes:
      - name: BadAttr
        logical_type: integer
",
        );
        for d in &diags {
            assert_eq!(
                d.severity,
                Severity::Warning,
                "dc_standard rules should produce warnings, not {:?} for {}",
                d.severity,
                d.code
            );
        }
    }

    #[test]
    fn each_rule_has_stable_code() {
        let pack = DcStandard::new();
        for rule in pack.rules() {
            assert!(
                rule.code.starts_with("POL-"),
                "rule code '{}' must start with POL-",
                rule.code
            );
            assert!(
                !rule.rationale.is_empty(),
                "rule {} missing rationale",
                rule.code
            );
            assert!(
                !rule.remediation.is_empty(),
                "rule {} missing remediation",
                rule.code
            );
        }
    }

    #[test]
    fn rule_codes_are_unique() {
        let pack = DcStandard::new();
        let mut codes: Vec<&str> = pack.rules().iter().map(|r| r.code).collect();
        let before = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), before, "duplicate policy rule codes");
    }

    #[test]
    fn pack_info_correct() {
        let pack = DcStandard::new();
        let info = pack.info();
        assert_eq!(info.pack, "dc_standard");
        assert_eq!(info.version, "1.0");
    }

    #[test]
    fn empty_metadata_no_warnings() {
        let diags = eval("{}");
        assert!(diags.is_empty());
    }

    #[test]
    fn evaluate_does_not_mutate_metadata() {
        let yaml = r"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
";
        let meta = parse_yaml_metadata(yaml);
        let json_before = serde_json::to_string(&meta).unwrap();
        let _diags = DcStandard::new().evaluate(&meta);
        let json_after = serde_json::to_string(&meta).unwrap();
        assert_eq!(
            json_before, json_after,
            "policy evaluation must not mutate metadata"
        );
    }

    #[test]
    fn all_diagnostics_have_yaml_path() {
        let diags = eval(
            r"
entities:
  - name: BadName
    attributes:
      - name: BadAttr
        logical_type: integer
        required: true
",
        );
        for d in &diags {
            assert!(
                d.yaml_path.is_some(),
                "diagnostic {} should have yaml_path",
                d.code
            );
        }
    }

    #[test]
    fn all_diagnostics_have_help() {
        let diags = eval(
            r"
entities:
  - name: BadName
    attributes:
      - name: BadAttr
        logical_type: integer
        required: true
",
        );
        for d in &diags {
            assert!(
                d.help.is_some(),
                "diagnostic {} should have help text",
                d.code
            );
        }
    }

    #[test]
    fn snake_case_check_edge_cases() {
        assert!(is_snake_case("customer"));
        assert!(is_snake_case("customer_order"));
        assert!(is_snake_case("a1"));
        assert!(!is_snake_case("CustomerOrder"));
        assert!(!is_snake_case("customer__order"));
        assert!(!is_snake_case("_customer"));
        assert!(!is_snake_case("customer_"));
        assert!(!is_snake_case("UPPER"));
        assert!(is_snake_case(""));
    }
}
