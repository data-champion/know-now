use know_now_contract::contract::GeneratorContract;
use know_now_diff::{Change, ChangeKind, DiffResult, ObjectKind};
use know_now_ir::ddl::SqlType;
use serde::Serialize;

use crate::type_map::map_logical_type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationCategory {
    Additive,
    Rename,
    Ambiguous,
    Destructive,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationStub {
    pub category: MigrationCategory,
    pub change_id: String,
    pub sql: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationPlan {
    pub stubs: Vec<MigrationStub>,
}

pub fn generate_migration_stubs(
    diff: &DiffResult,
    new_contract: &GeneratorContract,
) -> MigrationPlan {
    let added_entity_ids: Vec<&str> = diff
        .changes
        .iter()
        .filter(|c| c.object_kind == ObjectKind::Entity && c.kind == ChangeKind::Added)
        .map(|c| c.id.as_str())
        .collect();

    let mut stubs = Vec::new();

    for change in &diff.changes {
        if change.object_kind == ObjectKind::Attribute
            && change.kind == ChangeKind::Added
            && change
                .parent_id
                .as_deref()
                .is_some_and(|pid| added_entity_ids.contains(&pid))
        {
            continue;
        }
        if let Some(stub) = generate_stub(change, new_contract) {
            stubs.push(stub);
        }
    }

    MigrationPlan { stubs }
}

fn generate_stub(change: &Change, new_contract: &GeneratorContract) -> Option<MigrationStub> {
    match (change.object_kind, change.kind) {
        (ObjectKind::Entity, ChangeKind::Added) => Some(create_table_stub(change, new_contract)),
        (ObjectKind::Entity, ChangeKind::Renamed) => Some(rename_table_stub(change)),
        (ObjectKind::Entity, ChangeKind::Removed | ChangeKind::Destructive) => {
            Some(drop_table_stub(change))
        }
        (ObjectKind::Attribute, ChangeKind::Added) => {
            Some(add_column_stub(change, new_contract))
        }
        (ObjectKind::Attribute, ChangeKind::Renamed) => Some(rename_column_stub(change)),
        (ObjectKind::Attribute, ChangeKind::Removed) => Some(drop_column_stub(change)),
        (ObjectKind::Attribute, ChangeKind::Breaking) => {
            Some(alter_column_type_stub(change))
        }
        (ObjectKind::Relationship, ChangeKind::Added) => {
            Some(add_foreign_key_stub(change, new_contract))
        }
        (ObjectKind::Relationship, ChangeKind::Destructive) => {
            Some(drop_foreign_key_stub(change))
        }
        (_, ChangeKind::Ambiguous) => Some(ambiguous_stub(change)),
        _ => None,
    }
}

fn create_table_stub(change: &Change, contract: &GeneratorContract) -> MigrationStub {
    let sql = contract
        .entities
        .iter()
        .find(|e| e.id == change.id)
        .map_or_else(
            || format!("-- TODO: CREATE TABLE for entity '{}' (entity not found in contract)", change.name),
            |entity| {
                let table_name = &entity.name;
                let columns: Vec<String> = entity.attributes.iter().map(|attr| {
                    let sql_type = map_logical_type(attr.logical_type.as_deref())
                        .unwrap_or(SqlType::Text);
                    let not_null = if attr.required == Some(true) { " NOT NULL" } else { "" };
                    format!("    {} {}{}", attr.name, format_sql_type(&sql_type), not_null)
                }).collect();
                let cols = columns.join(",\n");
                format!("CREATE TABLE {table_name} (\n{cols}\n);")
            },
        );

    MigrationStub {
        category: MigrationCategory::Additive,
        change_id: change.id.clone(),
        sql,
    }
}

fn rename_table_stub(change: &Change) -> MigrationStub {
    let old_name = change
        .details
        .iter()
        .find(|d| d.field == "name")
        .and_then(|d| d.old.as_deref())
        .unwrap_or("unknown");
    let new_name = change
        .details
        .iter()
        .find(|d| d.field == "name")
        .and_then(|d| d.new.as_deref())
        .unwrap_or(&change.name);

    MigrationStub {
        category: MigrationCategory::Rename,
        change_id: change.id.clone(),
        sql: format!("ALTER TABLE {old_name} RENAME TO {new_name};"),
    }
}

fn drop_table_stub(change: &Change) -> MigrationStub {
    MigrationStub {
        category: MigrationCategory::Destructive,
        change_id: change.id.clone(),
        sql: format!(
            "-- DESTRUCTIVE: dropping table '{}'\n-- Requires --accept-destructive flag\nDROP TABLE IF EXISTS {} CASCADE;",
            change.name, change.name
        ),
    }
}

fn add_column_stub(change: &Change, contract: &GeneratorContract) -> MigrationStub {
    let sql = resolve_add_column(change, contract).unwrap_or_else(|| {
        format!(
            "ALTER TABLE <table> ADD COLUMN {} TEXT;\n-- TODO: verify type and constraints",
            change.name
        )
    });

    MigrationStub {
        category: MigrationCategory::Additive,
        change_id: change.id.clone(),
        sql,
    }
}

fn resolve_add_column(change: &Change, contract: &GeneratorContract) -> Option<String> {
    let parent_id = change.parent_id.as_deref()?;
    let entity = contract.entities.iter().find(|e| e.id == parent_id)?;
    let attr = entity.attributes.iter().find(|a| a.id == change.id)?;
    let sql_type = map_logical_type(attr.logical_type.as_deref()).unwrap_or(SqlType::Text);
    let not_null = if attr.required == Some(true) { " NOT NULL" } else { "" };
    Some(format!(
        "ALTER TABLE {} ADD COLUMN {} {}{};",
        entity.name,
        attr.name,
        format_sql_type(&sql_type),
        not_null
    ))
}

fn rename_column_stub(change: &Change) -> MigrationStub {
    let old_name = change
        .details
        .iter()
        .find(|d| d.field == "name")
        .and_then(|d| d.old.as_deref())
        .unwrap_or("unknown");
    let new_name = change
        .details
        .iter()
        .find(|d| d.field == "name")
        .and_then(|d| d.new.as_deref())
        .unwrap_or(&change.name);
    let table = change.parent_id.as_deref().unwrap_or("<table>");

    MigrationStub {
        category: MigrationCategory::Rename,
        change_id: change.id.clone(),
        sql: format!("ALTER TABLE {table} RENAME COLUMN {old_name} TO {new_name};"),
    }
}

fn drop_column_stub(change: &Change) -> MigrationStub {
    let table = change.parent_id.as_deref().unwrap_or("<table>");
    MigrationStub {
        category: MigrationCategory::Destructive,
        change_id: change.id.clone(),
        sql: format!(
            "-- DESTRUCTIVE: dropping column '{}' from '{}'\n-- Requires --accept-destructive flag\nALTER TABLE {} DROP COLUMN {};",
            change.name, table, table, change.name
        ),
    }
}

fn alter_column_type_stub(change: &Change) -> MigrationStub {
    let table = change.parent_id.as_deref().unwrap_or("<table>");
    let new_type = change
        .details
        .iter()
        .find(|d| d.field == "logical_type")
        .and_then(|d| d.new.as_deref())
        .unwrap_or("TEXT");

    let sql_type = map_logical_type(Some(new_type)).unwrap_or(SqlType::Text);

    MigrationStub {
        category: MigrationCategory::Ambiguous,
        change_id: change.id.clone(),
        sql: format!(
            "-- TODO: type change on column '{}' in '{}'\n-- Verify data compatibility before applying\nALTER TABLE {} ALTER COLUMN {} TYPE {} USING {}::{};",
            change.name, table, table, change.name,
            format_sql_type(&sql_type), change.name, format_sql_type(&sql_type)
        ),
    }
}

fn add_foreign_key_stub(change: &Change, contract: &GeneratorContract) -> MigrationStub {
    let sql = contract
        .relationships
        .iter()
        .find(|r| r.id == change.id)
        .map_or_else(
            || format!("-- TODO: add foreign key for relationship '{}'", change.id),
            |rel| {
                let fk_name = format!("fk_{}_{}", rel.from_entity, rel.to_entity);
                let from_key = rel.from_key.as_deref().unwrap_or("id");
                let to_key = rel.to_key.as_deref().unwrap_or("id");
                format!(
                    "ALTER TABLE {} ADD CONSTRAINT {fk_name} FOREIGN KEY ({from_key}) REFERENCES {} ({to_key});",
                    rel.from_entity, rel.to_entity
                )
            },
        );

    MigrationStub {
        category: MigrationCategory::Additive,
        change_id: change.id.clone(),
        sql,
    }
}

fn drop_foreign_key_stub(change: &Change) -> MigrationStub {
    MigrationStub {
        category: MigrationCategory::Destructive,
        change_id: change.id.clone(),
        sql: format!(
            "-- DESTRUCTIVE: dropping foreign key for '{}'\n-- Requires --accept-destructive flag\n-- ALTER TABLE <table> DROP CONSTRAINT <fk_name>;",
            change.name
        ),
    }
}

fn ambiguous_stub(change: &Change) -> MigrationStub {
    let obj = match change.object_kind {
        ObjectKind::Entity => "entity",
        ObjectKind::Attribute => "attribute",
        ObjectKind::Relationship => "relationship",
    };

    MigrationStub {
        category: MigrationCategory::Ambiguous,
        change_id: change.id.clone(),
        sql: format!(
            "-- TODO: ambiguous change on {obj} '{}' (id: {})\n-- This change could not be automatically classified.\n-- Review the diff and write the migration manually.",
            change.name, change.id
        ),
    }
}

fn format_sql_type(sql_type: &SqlType) -> String {
    match sql_type {
        SqlType::Text => "TEXT".into(),
        SqlType::Integer => "INTEGER".into(),
        SqlType::BigInt => "BIGINT".into(),
        SqlType::SmallInt => "SMALLINT".into(),
        SqlType::Boolean => "BOOLEAN".into(),
        SqlType::Date => "DATE".into(),
        SqlType::Time => "TIME".into(),
        SqlType::TimestampTz => "TIMESTAMPTZ".into(),
        SqlType::Uuid => "UUID".into(),
        SqlType::Jsonb => "JSONB".into(),
        SqlType::Bytea => "BYTEA".into(),
        SqlType::Varchar { length } => {
            length.as_ref().map_or_else(|| "VARCHAR".into(), |n| format!("VARCHAR({n})"))
        }
        SqlType::Char { length } => {
            length.as_ref().map_or_else(|| "CHAR".into(), |n| format!("CHAR({n})"))
        }
        SqlType::Numeric { precision, scale } => match (precision, scale) {
            (Some(p), Some(s)) => format!("NUMERIC({p},{s})"),
            (Some(p), None) => format!("NUMERIC({p})"),
            _ => "NUMERIC".into(),
        },
    }
}

pub fn format_migration_file(stubs: &[MigrationStub], accept_destructive: bool) -> String {
    let mut lines = Vec::new();
    lines.push("-- Migration generated by know-now".to_owned());
    lines.push(format!("-- {} statement(s)", stubs.len()));
    lines.push(String::new());

    for stub in stubs {
        if stub.category == MigrationCategory::Destructive && !accept_destructive {
            lines.push(format!(
                "-- SKIPPED (destructive, requires --accept-destructive):\n-- {}",
                stub.sql.replace('\n', "\n-- ")
            ));
        } else {
            lines.push(stub.sql.clone());
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use know_now_contract::contract::{
        ContractAttribute, ContractEntity, ContractRelationship, ContractTrace, GeneratorContract,
    };
    use know_now_diff::diff;

    fn empty_contract() -> GeneratorContract {
        GeneratorContract {
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
        }
    }

    fn entity(id: &str, name: &str, attrs: Vec<ContractAttribute>) -> ContractEntity {
        ContractEntity {
            id: id.into(),
            name: name.into(),
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
            attributes: attrs,
        }
    }

    fn attr(id: &str, name: &str, logical_type: &str) -> ContractAttribute {
        ContractAttribute {
            id: id.into(),
            name: name.into(),
            logical_type: Some(logical_type.into()),
            semantic_type: None,
            sensitivity: None,
            pii: None,
            required: None,
            is_unique: None,
            constraints: vec![],
            description: None,
            attr_type: None,
        }
    }

    fn rel(id: &str, from: &str, to: &str) -> ContractRelationship {
        ContractRelationship {
            id: id.into(),
            from_entity: from.into(),
            to_entity: to.into(),
            cardinality: Some("many_to_one".into()),
            from_key: Some("customer_id".into()),
            to_key: Some("id".into()),
            description: None,
        }
    }

    #[test]
    fn added_entity_produces_create_table() {
        let left = empty_contract();
        let mut right = empty_contract();
        right.entities.push(entity(
            "ent_1",
            "customer",
            vec![
                attr("a1", "id", "integer"),
                attr("a2", "name", "string"),
            ],
        ));

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1, "only CREATE TABLE, not redundant ADD COLUMNs");
        assert_eq!(plan.stubs[0].category, MigrationCategory::Additive);
        assert!(plan.stubs[0].sql.contains("CREATE TABLE customer"));
        assert!(plan.stubs[0].sql.contains("id INTEGER"));
        assert!(plan.stubs[0].sql.contains("name TEXT"));
    }

    #[test]
    fn added_column_produces_alter_table() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer", vec![attr("a1", "id", "integer")]));
        let mut right = empty_contract();
        right.entities.push(entity(
            "ent_1",
            "customer",
            vec![
                attr("a1", "id", "integer"),
                attr("a2", "email", "string"),
            ],
        ));

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1);
        assert_eq!(plan.stubs[0].category, MigrationCategory::Additive);
        assert!(plan.stubs[0].sql.contains("ALTER TABLE customer ADD COLUMN email TEXT"));
    }

    #[test]
    fn renamed_entity_produces_rename_table() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer", vec![]));
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "client", vec![]));

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1);
        assert_eq!(plan.stubs[0].category, MigrationCategory::Rename);
        assert!(plan.stubs[0].sql.contains("RENAME TO client"));
    }

    #[test]
    fn removed_entity_produces_destructive_drop() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer", vec![]));
        let right = empty_contract();

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1);
        assert_eq!(plan.stubs[0].category, MigrationCategory::Destructive);
        assert!(plan.stubs[0].sql.contains("DESTRUCTIVE"));
        assert!(plan.stubs[0].sql.contains("DROP TABLE"));
    }

    #[test]
    fn removed_attribute_produces_destructive_drop_column() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer", vec![attr("a1", "email", "string")]));
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "customer", vec![]));

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1);
        assert_eq!(plan.stubs[0].category, MigrationCategory::Destructive);
        assert!(plan.stubs[0].sql.contains("DROP COLUMN"));
    }

    #[test]
    fn type_change_produces_ambiguous_alter() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer", vec![attr("a1", "age", "string")]));
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "customer", vec![attr("a1", "age", "integer")]));

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1);
        assert_eq!(plan.stubs[0].category, MigrationCategory::Ambiguous);
        assert!(plan.stubs[0].sql.contains("ALTER TABLE"));
        assert!(plan.stubs[0].sql.contains("TYPE INTEGER"));
    }

    #[test]
    fn added_relationship_produces_foreign_key() {
        let left = empty_contract();
        let mut right = empty_contract();
        right.relationships.push(rel("rel_1", "orders", "customer"));

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1);
        assert_eq!(plan.stubs[0].category, MigrationCategory::Additive);
        assert!(plan.stubs[0].sql.contains("FOREIGN KEY"));
        assert!(plan.stubs[0].sql.contains("REFERENCES customer"));
    }

    #[test]
    fn removed_relationship_produces_destructive() {
        let mut left = empty_contract();
        left.relationships.push(rel("rel_1", "orders", "customer"));
        let right = empty_contract();

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        assert_eq!(plan.stubs.len(), 1);
        assert_eq!(plan.stubs[0].category, MigrationCategory::Destructive);
    }

    #[test]
    fn format_migration_file_skips_destructive_without_flag() {
        let stubs = vec![
            MigrationStub {
                category: MigrationCategory::Additive,
                change_id: "a1".into(),
                sql: "ALTER TABLE t ADD COLUMN x TEXT;".into(),
            },
            MigrationStub {
                category: MigrationCategory::Destructive,
                change_id: "a2".into(),
                sql: "DROP TABLE old_table;".into(),
            },
        ];

        let output = format_migration_file(&stubs, false);
        assert!(output.contains("ALTER TABLE t ADD COLUMN x TEXT;"));
        assert!(output.contains("SKIPPED (destructive"));
        assert!(!output.contains("\nDROP TABLE old_table;\n"));
    }

    #[test]
    fn format_migration_file_includes_destructive_with_flag() {
        let stubs = vec![MigrationStub {
            category: MigrationCategory::Destructive,
            change_id: "a2".into(),
            sql: "DROP TABLE old_table;".into(),
        }];

        let output = format_migration_file(&stubs, true);
        assert!(output.contains("DROP TABLE old_table;"));
        assert!(!output.contains("SKIPPED"));
    }

    #[test]
    fn empty_diff_produces_no_stubs() {
        let contract = empty_contract();
        let diff_result = diff(&contract, &contract);
        let plan = generate_migration_stubs(&diff_result, &contract);
        assert!(plan.stubs.is_empty());
    }

    #[test]
    fn migration_sql_is_parseable() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer", vec![attr("a1", "id", "integer")]));
        let mut right = empty_contract();
        right.entities.push(entity(
            "ent_1",
            "customer",
            vec![
                attr("a1", "id", "integer"),
                attr("a2", "email", "string"),
            ],
        ));

        let diff_result = diff(&left, &right);
        let plan = generate_migration_stubs(&diff_result, &right);

        for stub in &plan.stubs {
            if stub.category == MigrationCategory::Additive {
                let result = sqlparser::parser::Parser::parse_sql(
                    &sqlparser::dialect::PostgreSqlDialect {},
                    &stub.sql,
                );
                assert!(
                    result.is_ok(),
                    "SQL should parse: {} (error: {:?})",
                    stub.sql,
                    result.err()
                );
            }
        }
    }
}
