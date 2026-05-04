use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::doc::Doc;
use crate::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlType {
    SmallInt,
    Integer,
    BigInt,
    Text,
    Varchar {
        length: Option<u32>,
    },
    Char {
        length: Option<u32>,
    },
    Numeric {
        precision: Option<u8>,
        scale: Option<u8>,
    },
    Boolean,
    Date,
    TimestampTz,
    Time,
    Uuid,
    Jsonb,
    Bytea,
}

impl SqlType {
    #[must_use]
    pub fn sql_fragment(&self) -> Cow<'static, str> {
        match self {
            Self::SmallInt => "SMALLINT".into(),
            Self::Integer => "INTEGER".into(),
            Self::BigInt => "BIGINT".into(),
            Self::Text => "TEXT".into(),
            Self::Varchar { length: None } => "VARCHAR".into(),
            Self::Varchar {
                length: Some(n), ..
            } => format!("VARCHAR({n})").into(),
            Self::Char { length: None } => "CHAR".into(),
            Self::Char {
                length: Some(n), ..
            } => format!("CHAR({n})").into(),
            Self::Numeric {
                precision: None, ..
            } => "NUMERIC".into(),
            Self::Numeric {
                precision: Some(p),
                scale: None,
            } => format!("NUMERIC({p})").into(),
            Self::Numeric {
                precision: Some(p),
                scale: Some(s),
            } => format!("NUMERIC({p}, {s})").into(),
            Self::Boolean => "BOOLEAN".into(),
            Self::Date => "DATE".into(),
            Self::TimestampTz => "TIMESTAMPTZ".into(),
            Self::Time => "TIME".into(),
            Self::Uuid => "UUID".into(),
            Self::Jsonb => "JSONB".into(),
            Self::Bytea => "BYTEA".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Literal {
    Null,
    Bool(bool),
    Integer(i64),
    Decimal(String),
    String(String),
    CurrentTimestamp,
    CurrentDate,
}

impl Literal {
    #[must_use]
    pub fn sql_fragment(&self) -> Cow<'static, str> {
        match self {
            Self::Null => "NULL".into(),
            Self::Bool(true) => "TRUE".into(),
            Self::Bool(false) => "FALSE".into(),
            Self::Integer(n) => n.to_string().into(),
            Self::Decimal(s) => Cow::Owned(s.clone()),
            Self::String(s) => format!("'{}'", s.replace('\'', "''")).into(),
            Self::CurrentTimestamp => "CURRENT_TIMESTAMP".into(),
            Self::CurrentDate => "CURRENT_DATE".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: Identifier,
    pub sql_type: SqlType,
    pub nullable: bool,
    pub default: Option<Literal>,
    pub comment: Option<Doc>,
    pub metadata_object_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryKeyConstraint {
    pub name: Option<Identifier>,
    pub columns: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueConstraint {
    pub name: Option<Identifier>,
    pub columns: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferentialAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

impl ReferentialAction {
    #[must_use]
    pub fn sql_fragment(&self) -> &'static str {
        match self {
            Self::NoAction => "NO ACTION",
            Self::Restrict => "RESTRICT",
            Self::Cascade => "CASCADE",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeyConstraint {
    pub name: Option<Identifier>,
    pub columns: Vec<Identifier>,
    pub referenced_schema: Option<Identifier>,
    pub referenced_table: Identifier,
    pub referenced_columns: Vec<Identifier>,
    pub on_delete: Option<ReferentialAction>,
    pub on_update: Option<ReferentialAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckConstraint {
    pub name: Option<Identifier>,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDef {
    pub name: Identifier,
    pub columns: Vec<Identifier>,
    pub unique: bool,
}

#[derive(Debug, Clone)]
pub struct TableDef {
    pub name: Identifier,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Option<PrimaryKeyConstraint>,
    pub unique_constraints: Vec<UniqueConstraint>,
    pub foreign_keys: Vec<ForeignKeyConstraint>,
    pub check_constraints: Vec<CheckConstraint>,
    pub indexes: Vec<IndexDef>,
    pub comment: Option<Doc>,
    pub metadata_entity_id: String,
}

#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub name: Identifier,
    pub tables: Vec<TableDef>,
    pub comment: Option<Doc>,
}

#[derive(Debug, Clone)]
pub struct LogicalSchema {
    pub schemas: Vec<SchemaDef>,
}

#[derive(Debug, Clone)]
pub struct OwnershipHeader {
    pub artifact_id: String,
    pub generator: String,
    pub generator_version: String,
    pub input_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync_clone<T: Send + Sync + Clone>() {}

    #[test]
    fn ir_types_are_send_sync_clone() {
        assert_send_sync_clone::<SqlType>();
        assert_send_sync_clone::<Literal>();
        assert_send_sync_clone::<ColumnDef>();
        assert_send_sync_clone::<PrimaryKeyConstraint>();
        assert_send_sync_clone::<UniqueConstraint>();
        assert_send_sync_clone::<ForeignKeyConstraint>();
        assert_send_sync_clone::<CheckConstraint>();
        assert_send_sync_clone::<IndexDef>();
        assert_send_sync_clone::<ReferentialAction>();
        assert_send_sync_clone::<TableDef>();
        assert_send_sync_clone::<SchemaDef>();
        assert_send_sync_clone::<LogicalSchema>();
        assert_send_sync_clone::<OwnershipHeader>();
        assert_send_sync_clone::<Doc>();
    }

    #[test]
    fn sql_type_simple_fragments() {
        assert_eq!(&*SqlType::Integer.sql_fragment(), "INTEGER");
        assert_eq!(&*SqlType::Text.sql_fragment(), "TEXT");
        assert_eq!(&*SqlType::TimestampTz.sql_fragment(), "TIMESTAMPTZ");
        assert_eq!(&*SqlType::Jsonb.sql_fragment(), "JSONB");
        assert_eq!(&*SqlType::Bytea.sql_fragment(), "BYTEA");
    }

    #[test]
    fn sql_type_parameterized_fragments() {
        assert_eq!(
            &*SqlType::Varchar { length: None }.sql_fragment(),
            "VARCHAR"
        );
        assert_eq!(
            &*SqlType::Varchar { length: Some(255) }.sql_fragment(),
            "VARCHAR(255)"
        );
        assert_eq!(
            &*SqlType::Char { length: Some(1) }.sql_fragment(),
            "CHAR(1)"
        );
        assert_eq!(
            &*SqlType::Numeric {
                precision: None,
                scale: None
            }
            .sql_fragment(),
            "NUMERIC"
        );
        assert_eq!(
            &*SqlType::Numeric {
                precision: Some(10),
                scale: None
            }
            .sql_fragment(),
            "NUMERIC(10)"
        );
        assert_eq!(
            &*SqlType::Numeric {
                precision: Some(10),
                scale: Some(2)
            }
            .sql_fragment(),
            "NUMERIC(10, 2)"
        );
    }

    #[test]
    fn literal_sql_fragments() {
        assert_eq!(&*Literal::Null.sql_fragment(), "NULL");
        assert_eq!(&*Literal::Bool(true).sql_fragment(), "TRUE");
        assert_eq!(&*Literal::Bool(false).sql_fragment(), "FALSE");
        assert_eq!(&*Literal::Integer(42).sql_fragment(), "42");
        assert_eq!(&*Literal::Decimal("3.14".into()).sql_fragment(), "3.14");
        assert_eq!(&*Literal::String("hello".into()).sql_fragment(), "'hello'");
        assert_eq!(&*Literal::String("it's".into()).sql_fragment(), "'it''s'");
        assert_eq!(
            &*Literal::CurrentTimestamp.sql_fragment(),
            "CURRENT_TIMESTAMP"
        );
        assert_eq!(&*Literal::CurrentDate.sql_fragment(), "CURRENT_DATE");
    }

    #[test]
    fn referential_action_sql() {
        assert_eq!(ReferentialAction::Cascade.sql_fragment(), "CASCADE");
        assert_eq!(ReferentialAction::SetNull.sql_fragment(), "SET NULL");
        assert_eq!(ReferentialAction::NoAction.sql_fragment(), "NO ACTION");
        assert_eq!(ReferentialAction::Restrict.sql_fragment(), "RESTRICT");
        assert_eq!(ReferentialAction::SetDefault.sql_fragment(), "SET DEFAULT");
    }

    #[test]
    fn table_def_construction() {
        let table = TableDef {
            name: Identifier::new("customer").unwrap(),
            columns: vec![],
            primary_key: None,
            unique_constraints: vec![],
            foreign_keys: vec![],
            check_constraints: vec![],
            indexes: vec![],
            comment: None,
            metadata_entity_id: "ent_customer".into(),
        };
        assert_eq!(table.name.as_str(), "customer");
    }

    #[test]
    fn schema_def_with_tables_and_comments() {
        let schema = SchemaDef {
            name: Identifier::new("public").unwrap(),
            tables: vec![TableDef {
                name: Identifier::new("customer").unwrap(),
                columns: vec![],
                primary_key: None,
                unique_constraints: vec![],
                foreign_keys: vec![],
                check_constraints: vec![],
                indexes: vec![],
                comment: Some(Doc::new("Customer master table")),
                metadata_entity_id: "ent_customer".into(),
            }],
            comment: Some(Doc::new("Public schema")),
        };
        assert_eq!(schema.tables.len(), 1);
        assert_eq!(schema.comment.as_ref().unwrap().as_str(), "Public schema");
    }

    #[test]
    fn logical_schema_structure() {
        let logical = LogicalSchema {
            schemas: vec![SchemaDef {
                name: Identifier::new("public").unwrap(),
                tables: vec![],
                comment: None,
            }],
        };
        assert_eq!(logical.schemas.len(), 1);
    }

    #[test]
    fn sql_type_serde_roundtrip() {
        let types = [
            SqlType::Integer,
            SqlType::BigInt,
            SqlType::SmallInt,
            SqlType::Text,
            SqlType::Boolean,
            SqlType::Date,
            SqlType::TimestampTz,
            SqlType::Time,
            SqlType::Numeric {
                precision: None,
                scale: None,
            },
            SqlType::Numeric {
                precision: Some(10),
                scale: Some(2),
            },
            SqlType::Varchar { length: None },
            SqlType::Varchar { length: Some(255) },
            SqlType::Char { length: Some(1) },
            SqlType::Uuid,
            SqlType::Jsonb,
            SqlType::Bytea,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            let parsed: SqlType = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, t);
        }
    }

    #[test]
    fn literal_serde_roundtrip() {
        let literals = [
            Literal::Null,
            Literal::Bool(true),
            Literal::Integer(42),
            Literal::Decimal("3.14".into()),
            Literal::String("hello".into()),
            Literal::CurrentTimestamp,
            Literal::CurrentDate,
        ];
        for lit in &literals {
            let json = serde_json::to_string(lit).unwrap();
            let parsed: Literal = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, lit);
        }
    }

    #[test]
    fn foreign_key_constraint_with_actions() {
        let fk = ForeignKeyConstraint {
            name: Some(Identifier::new("fk_order_customer").unwrap()),
            columns: vec![Identifier::new("customer_id").unwrap()],
            referenced_schema: Some(Identifier::new("public").unwrap()),
            referenced_table: Identifier::new("customer").unwrap(),
            referenced_columns: vec![Identifier::new("id").unwrap()],
            on_delete: Some(ReferentialAction::Cascade),
            on_update: Some(ReferentialAction::NoAction),
        };
        assert_eq!(fk.columns.len(), 1);
        assert_eq!(fk.on_delete.as_ref().unwrap().sql_fragment(), "CASCADE");
    }

    #[test]
    fn table_def_with_all_constraint_types() {
        let table = TableDef {
            name: Identifier::new("order_item").unwrap(),
            columns: vec![
                ColumnDef {
                    name: Identifier::new("id").unwrap(),
                    sql_type: SqlType::Integer,
                    nullable: false,
                    default: None,
                    comment: None,
                    metadata_object_id: "attr_oi_id".into(),
                },
                ColumnDef {
                    name: Identifier::new("price").unwrap(),
                    sql_type: SqlType::Numeric {
                        precision: Some(10),
                        scale: Some(2),
                    },
                    nullable: false,
                    default: Some(Literal::Decimal("0.00".into())),
                    comment: Some(Doc::new("Unit price in cents")),
                    metadata_object_id: "attr_oi_price".into(),
                },
            ],
            primary_key: Some(PrimaryKeyConstraint {
                name: Some(Identifier::new("pk_order_item").unwrap()),
                columns: vec![Identifier::new("id").unwrap()],
            }),
            unique_constraints: vec![UniqueConstraint {
                name: Some(Identifier::new("uq_order_item_sku").unwrap()),
                columns: vec![Identifier::new("sku").unwrap()],
            }],
            foreign_keys: vec![ForeignKeyConstraint {
                name: Some(Identifier::new("fk_order_item_order").unwrap()),
                columns: vec![Identifier::new("order_id").unwrap()],
                referenced_schema: None,
                referenced_table: Identifier::new("order_header").unwrap(),
                referenced_columns: vec![Identifier::new("id").unwrap()],
                on_delete: Some(ReferentialAction::Cascade),
                on_update: None,
            }],
            check_constraints: vec![CheckConstraint {
                name: Some(Identifier::new("ck_positive_price").unwrap()),
                expression: "price >= 0".into(),
            }],
            indexes: vec![IndexDef {
                name: Identifier::new("idx_order_item_order_id").unwrap(),
                columns: vec![Identifier::new("order_id").unwrap()],
                unique: false,
            }],
            comment: Some(Doc::new("Line items within an order")),
            metadata_entity_id: "ent_order_item".into(),
        };
        assert_eq!(table.unique_constraints.len(), 1);
        assert_eq!(table.foreign_keys.len(), 1);
        assert_eq!(table.check_constraints.len(), 1);
        assert_eq!(table.indexes.len(), 1);
        assert!(table.comment.is_some());
    }
}
