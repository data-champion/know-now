use serde::{Deserialize, Serialize};

use crate::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlType {
    Integer,
    BigInt,
    SmallInt,
    Text,
    Boolean,
    Date,
    TimestampTz,
    Time,
    Numeric,
    Uuid,
    Jsonb,
    Bytea,
}

impl SqlType {
    #[must_use]
    pub fn sql_keyword(&self) -> &'static str {
        match self {
            Self::Integer => "INTEGER",
            Self::BigInt => "BIGINT",
            Self::SmallInt => "SMALLINT",
            Self::Text => "TEXT",
            Self::Boolean => "BOOLEAN",
            Self::Date => "DATE",
            Self::TimestampTz => "TIMESTAMPTZ",
            Self::Time => "TIME",
            Self::Numeric => "NUMERIC",
            Self::Uuid => "UUID",
            Self::Jsonb => "JSONB",
            Self::Bytea => "BYTEA",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: Identifier,
    pub sql_type: SqlType,
    pub nullable: bool,
    pub metadata_object_id: String,
}

#[derive(Debug, Clone)]
pub struct PrimaryKeyConstraint {
    pub columns: Vec<Identifier>,
}

#[derive(Debug, Clone)]
pub struct CreateTableStatement {
    pub schema: Option<Identifier>,
    pub name: Identifier,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Option<PrimaryKeyConstraint>,
    pub entity_id: String,
}

impl CreateTableStatement {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        self.schema.as_ref().map_or_else(
            || self.name.quoted(),
            |s| format!("{}.{}", s.quoted(), self.name.quoted()),
        )
    }
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

    #[test]
    fn sql_type_keywords() {
        assert_eq!(SqlType::Integer.sql_keyword(), "INTEGER");
        assert_eq!(SqlType::Text.sql_keyword(), "TEXT");
        assert_eq!(SqlType::TimestampTz.sql_keyword(), "TIMESTAMPTZ");
        assert_eq!(SqlType::Jsonb.sql_keyword(), "JSONB");
        assert_eq!(SqlType::Bytea.sql_keyword(), "BYTEA");
    }

    #[test]
    fn qualified_name_with_schema() {
        let stmt = CreateTableStatement {
            schema: Some(Identifier::new("public").unwrap()),
            name: Identifier::new("customer").unwrap(),
            columns: vec![],
            primary_key: None,
            entity_id: "ent_customer".into(),
        };
        assert_eq!(stmt.qualified_name(), "\"public\".\"customer\"");
    }

    #[test]
    fn qualified_name_without_schema() {
        let stmt = CreateTableStatement {
            schema: None,
            name: Identifier::new("customer").unwrap(),
            columns: vec![],
            primary_key: None,
            entity_id: "ent_customer".into(),
        };
        assert_eq!(stmt.qualified_name(), "\"customer\"");
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
            SqlType::Numeric,
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
}
