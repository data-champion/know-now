//! PostgreSQL DDL generator crate for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

pub mod migrations;
pub mod type_map;

use know_now_codegen::artifact::{ArtifactDescriptor, ArtifactKind};
use know_now_codegen::generator::{GenerationError, Generator};
use know_now_contract::contract::GeneratorContract;
use know_now_ir::ddl::{ColumnDef, OwnershipHeader, PrimaryKeyConstraint, TableDef};
use know_now_ir::emitter;
use know_now_ir::identifier::{Identifier, IdentifierError};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use type_map::map_logical_type;

const GENERATOR_NAME: &str = "know_now_gen_postgres";
const ARTIFACT_PATH: &str = "ddl/postgres/schema.sql";

pub struct PostgresGenerator {
    version: String,
}

impl PostgresGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

impl Default for PostgresGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for PostgresGenerator {
    fn name(&self) -> &str {
        GENERATOR_NAME
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn generate(
        &self,
        contract: &GeneratorContract,
    ) -> Result<Vec<ArtifactDescriptor>, Vec<GenerationError>> {
        let mut errors = Vec::new();
        let mut tables = Vec::new();

        for entity in &contract.entities {
            match build_table(entity) {
                Ok(stmt) => tables.push(stmt),
                Err(errs) => errors.extend(errs),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let input_hash = compute_contract_hash(contract);
        let header = OwnershipHeader {
            artifact_id: "art_postgres_schema".into(),
            generator: GENERATOR_NAME.into(),
            generator_version: self.version.clone(),
            input_hash,
        };

        let emitted = emitter::emit_document(&mut tables, None, &header);

        if let Err(e) = validate_sql(&emitted.sql) {
            return Err(vec![GenerationError {
                code: "GEN-PG-PARSE".into(),
                message: format!("generated SQL failed parse validation: {e}"),
            }]);
        }

        let all_metadata_ids: Vec<String> = contract
            .entities
            .iter()
            .flat_map(|e| {
                let mut ids = vec![e.id.clone()];
                ids.extend(e.attributes.iter().map(|a| a.id.clone()));
                ids
            })
            .collect();

        Ok(vec![ArtifactDescriptor {
            path: ARTIFACT_PATH.into(),
            kind: ArtifactKind::PostgresDdl,
            artifact_id: "art_postgres_schema".into(),
            generator: GENERATOR_NAME.into(),
            generator_version: self.version.clone(),
            content: emitted.sql,
            metadata_object_ids: all_metadata_ids,
        }])
    }
}

fn build_table(
    entity: &know_now_contract::contract::ContractEntity,
) -> Result<TableDef, Vec<GenerationError>> {
    let mut errors = Vec::new();

    let table_name = match Identifier::new(&entity.name) {
        Ok(id) => id,
        Err(e) => {
            return Err(vec![GenerationError {
                code: identifier_error_code(&e).into(),
                message: format!("invalid table name '{}': {e}", entity.name),
            }]);
        }
    };

    let mut columns = Vec::new();
    for attr in &entity.attributes {
        let col_name = match Identifier::new(&attr.name) {
            Ok(id) => id,
            Err(e) => {
                errors.push(GenerationError {
                    code: identifier_error_code(&e).into(),
                    message: format!(
                        "invalid column name '{}' on entity '{}': {e}",
                        attr.name, entity.name
                    ),
                });
                continue;
            }
        };

        let sql_type = match map_logical_type(attr.logical_type.as_deref()) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        let nullable = !attr.required.unwrap_or(false);

        columns.push(ColumnDef {
            name: col_name,
            sql_type,
            nullable,
            default: None,
            comment: None,
            metadata_object_id: attr.id.clone(),
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let primary_key = build_primary_key(&entity.business_key, &columns, &entity.name)?;

    Ok(TableDef {
        name: table_name,
        columns,
        primary_key,
        unique_constraints: vec![],
        foreign_keys: vec![],
        check_constraints: vec![],
        indexes: vec![],
        comment: None,
        metadata_entity_id: entity.id.clone(),
    })
}

fn build_primary_key(
    business_key: &[String],
    columns: &[ColumnDef],
    entity_name: &str,
) -> Result<Option<PrimaryKeyConstraint>, Vec<GenerationError>> {
    if business_key.is_empty() {
        return Ok(None);
    }

    let mut pk_cols = Vec::new();
    let mut errors = Vec::new();

    for key in business_key {
        if columns.iter().any(|c| c.name.as_str() == key) {
            match Identifier::new(key) {
                Ok(id) => pk_cols.push(id),
                Err(e) => errors.push(GenerationError {
                    code: identifier_error_code(&e).into(),
                    message: format!(
                        "invalid business key column '{key}' on entity '{entity_name}': {e}"
                    ),
                }),
            }
        } else {
            errors.push(GenerationError {
                code: "GEN-PG-KEY".into(),
                message: format!("business key column '{key}' not found in entity '{entity_name}'"),
            });
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(Some(PrimaryKeyConstraint {
        name: None,
        columns: pk_cols,
    }))
}

fn identifier_error_code(error: &IdentifierError) -> &'static str {
    error.code()
}

fn validate_sql(sql: &str) -> Result<(), String> {
    let sql_body = sql
        .lines()
        .filter(|line| !line.starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");

    if sql_body.trim().is_empty() {
        return Ok(());
    }

    Parser::parse_sql(&PostgreSqlDialect {}, &sql_body)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn compute_contract_hash(contract: &GeneratorContract) -> String {
    let json = serde_json::to_string(contract).unwrap_or_default();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in json.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("fnv1a:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use know_now_contract::contract::{
        ContractAttribute, ContractEntity, ContractTrace, GeneratorContract,
    };

    fn minimal_contract() -> GeneratorContract {
        GeneratorContract {
            contract_version: "1.0".into(),
            project: None,
            target_database: None,
            entities: vec![ContractEntity {
                id: "ent_customer".into(),
                name: "customer".into(),
                display_name: None,
                domain: None,
                module: None,
                owner: None,
                steward: None,
                classification: None,
                retention_policy: None,
                description: None,
                entity_type: None,
                tags: vec![],
                business_key: vec!["id".into()],
                attributes: vec![
                    ContractAttribute {
                        id: "attr_customer_id".into(),
                        name: "id".into(),
                        logical_type: Some("integer".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: Some(true),
                        constraints: vec![],
                        description: None,
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_customer_email".into(),
                        name: "email".into(),
                        logical_type: Some("string".into()),
                        semantic_type: Some("email".into()),
                        sensitivity: None,
                        pii: Some(true),
                        required: None,
                        is_unique: None,
                        constraints: vec![],
                        description: None,
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_customer_active".into(),
                        name: "active".into(),
                        logical_type: Some("boolean".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: None,
                        constraints: vec![],
                        description: None,
                        attr_type: None,
                    },
                ],
            }],
            relationships: vec![],
            source_systems: vec![],
            quality_rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
            trace: ContractTrace::default(),
        }
    }

    #[test]
    fn generates_valid_sql() {
        let gen = PostgresGenerator::new();
        let contract = minimal_contract();
        let artifacts = gen.generate(&contract).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind, ArtifactKind::PostgresDdl);
        assert_eq!(artifacts[0].path, ARTIFACT_PATH);

        let sql = &artifacts[0].content;
        assert!(sql.contains("CREATE TABLE customer"));
        assert!(sql.contains("id INTEGER NOT NULL"));
        assert!(sql.contains("email TEXT"));
        assert!(sql.contains("active BOOLEAN NOT NULL"));
        assert!(sql.contains("PRIMARY KEY (id)"));
    }

    #[test]
    fn ownership_header_present() {
        let gen = PostgresGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let sql = &artifacts[0].content;
        assert!(sql.starts_with("-- Generated by know-now.\n"));
        assert!(sql.contains("-- Artifact ID: art_postgres_schema"));
        assert!(sql.contains("-- Generator: know_now_gen_postgres"));
    }

    #[test]
    fn deterministic_output() {
        let gen = PostgresGenerator::new();
        let contract = minimal_contract();
        let a = gen.generate(&contract).unwrap();
        let b = gen.generate(&contract).unwrap();
        assert_eq!(a[0].content, b[0].content);
    }

    #[test]
    fn sql_parse_validates() {
        let gen = PostgresGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let sql = &artifacts[0].content;
        let body: String = sql
            .lines()
            .filter(|l| !l.starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(Parser::parse_sql(&PostgreSqlDialect {}, &body).is_ok());
    }

    #[test]
    fn metadata_object_ids_collected() {
        let gen = PostgresGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let ids = &artifacts[0].metadata_object_ids;
        assert!(ids.contains(&"ent_customer".to_owned()));
        assert!(ids.contains(&"attr_customer_id".to_owned()));
        assert!(ids.contains(&"attr_customer_email".to_owned()));
        assert!(ids.contains(&"attr_customer_active".to_owned()));
    }

    #[test]
    fn rejects_invalid_identifier() {
        let gen = PostgresGenerator::new();
        let mut contract = minimal_contract();
        contract.entities[0].name = "invalid-name".into();
        let err = gen.generate(&contract).unwrap_err();
        assert!(err[0].code.starts_with("META-IDENT-"));
    }

    #[test]
    fn rejects_unknown_logical_type() {
        let gen = PostgresGenerator::new();
        let mut contract = minimal_contract();
        contract.entities[0].attributes[0].logical_type = Some("exotic_type".into());
        let err = gen.generate(&contract).unwrap_err();
        assert!(err[0].code.contains("GEN-PG-TYPE"));
    }

    #[test]
    fn rejects_missing_business_key_column() {
        let gen = PostgresGenerator::new();
        let mut contract = minimal_contract();
        contract.entities[0].business_key = vec!["nonexistent".into()];
        let err = gen.generate(&contract).unwrap_err();
        assert!(err[0].code.contains("GEN-PG-KEY"));
    }

    #[test]
    fn empty_contract_produces_empty_artifact() {
        let gen = PostgresGenerator::new();
        let contract = GeneratorContract {
            contract_version: "1.0".into(),
            project: None,
            target_database: None,
            entities: vec![],
            relationships: vec![],
            source_systems: vec![],
            quality_rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
            trace: ContractTrace::default(),
        };
        let artifacts = gen.generate(&contract).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert!(artifacts[0].content.contains("-- Generated by know-now."));
    }

    #[test]
    fn no_crlf_in_output() {
        let gen = PostgresGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        assert!(
            !artifacts[0].content.contains('\r'),
            "NFR-PO3: LF only, no CRLF"
        );
    }

    #[test]
    fn multiple_entities_sorted() {
        let gen = PostgresGenerator::new();
        let mut contract = minimal_contract();
        contract.entities.push(ContractEntity {
            id: "ent_account".into(),
            name: "account".into(),
            display_name: None,
            domain: None,
            module: None,
            owner: None,
            steward: None,
            classification: None,
            retention_policy: None,
            description: None,
            entity_type: None,
            tags: vec![],
            business_key: vec![],
            attributes: vec![ContractAttribute {
                id: "attr_account_name".into(),
                name: "name".into(),
                logical_type: Some("string".into()),
                semantic_type: None,
                sensitivity: None,
                pii: None,
                required: Some(true),
                is_unique: None,
                constraints: vec![],
                description: None,
                attr_type: None,
            }],
        });
        let artifacts = gen.generate(&contract).unwrap();
        let sql = &artifacts[0].content;
        let account_pos = sql.find("account").unwrap();
        let customer_pos = sql.find("customer").unwrap();
        assert!(account_pos < customer_pos, "tables must be sorted");
    }

    #[test]
    fn null_logical_type_defaults_to_text() {
        let gen = PostgresGenerator::new();
        let mut contract = minimal_contract();
        contract.entities[0].attributes[0].logical_type = None;
        contract.entities[0].business_key = vec![];
        let artifacts = gen.generate(&contract).unwrap();
        assert!(artifacts[0].content.contains("TEXT NOT NULL"));
    }

    #[test]
    fn generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PostgresGenerator>();
    }

    #[test]
    fn generator_name_and_version() {
        let gen = PostgresGenerator::new();
        assert_eq!(gen.name(), GENERATOR_NAME);
        assert!(!gen.version().is_empty());
    }

    #[test]
    fn parser_validation_gate_rejects_malformed_sql() {
        let bad_sql = "CREATE TABLE customer (id INTEGER NOT NULL,, email TEXT);";
        assert!(validate_sql(bad_sql).is_err());
    }

    #[test]
    fn parser_validation_gate_rejects_incomplete_sql() {
        let bad_sql = "CREATE TABLE customer (";
        assert!(validate_sql(bad_sql).is_err());
    }

    #[test]
    fn parser_validation_gate_accepts_valid_sql() {
        let good_sql = "CREATE TABLE customer (\n    id INTEGER NOT NULL\n);";
        assert!(validate_sql(good_sql).is_ok());
    }

    #[test]
    fn parser_validation_gate_skips_comment_lines() {
        let sql_with_comments =
            "-- Generated by know-now.\n-- Do not edit.\nCREATE TABLE t (id INTEGER);";
        assert!(validate_sql(sql_with_comments).is_ok());
    }

    #[test]
    fn parser_validation_gate_blocks_bad_output() {
        let gen = PostgresGenerator::new();
        let mut contract = minimal_contract();
        contract.entities[0].name = "invalid-name".into();
        let err = gen.generate(&contract).unwrap_err();
        assert!(
            err.iter().any(|e| e.code.starts_with("META-IDENT-")),
            "should reject invalid identifier before it reaches the writer"
        );
    }
}
