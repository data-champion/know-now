use know_now_codegen::generator::GenerationError;
use know_now_ir::ddl::SqlType;

/// # Errors
/// Returns `GenerationError` if the logical type is not a recognized PRD §10.7 type.
pub fn map_logical_type(logical: Option<&str>) -> Result<SqlType, GenerationError> {
    logical.map_or(Ok(SqlType::Text), |t| match t {
        "string" => Ok(SqlType::Text),
        "integer" => Ok(SqlType::Integer),
        "bigint" => Ok(SqlType::BigInt),
        "smallint" => Ok(SqlType::SmallInt),
        "decimal" | "numeric" => Ok(SqlType::Numeric),
        "boolean" => Ok(SqlType::Boolean),
        "date" => Ok(SqlType::Date),
        "timestamp" => Ok(SqlType::TimestampTz),
        "time" => Ok(SqlType::Time),
        "uuid" => Ok(SqlType::Uuid),
        "json" | "jsonb" => Ok(SqlType::Jsonb),
        "binary" => Ok(SqlType::Bytea),
        other => Err(GenerationError {
            code: "GEN-PG-TYPE".into(),
            message: format!("unsupported logical type '{other}'"),
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_prd_logical_types_map() {
        let cases = [
            ("string", SqlType::Text),
            ("integer", SqlType::Integer),
            ("decimal", SqlType::Numeric),
            ("boolean", SqlType::Boolean),
            ("date", SqlType::Date),
            ("timestamp", SqlType::TimestampTz),
            ("time", SqlType::Time),
            ("json", SqlType::Jsonb),
            ("uuid", SqlType::Uuid),
            ("binary", SqlType::Bytea),
        ];
        for (logical, expected) in cases {
            let result = map_logical_type(Some(logical)).unwrap();
            assert_eq!(
                result, expected,
                "logical type '{logical}' should map to {expected:?}"
            );
        }
    }

    #[test]
    fn none_defaults_to_text() {
        assert_eq!(map_logical_type(None).unwrap(), SqlType::Text);
    }

    #[test]
    fn unknown_type_errors() {
        let err = map_logical_type(Some("exotic")).unwrap_err();
        assert_eq!(err.code, "GEN-PG-TYPE");
        assert!(err.message.contains("exotic"));
    }

    #[test]
    fn aliases_work() {
        assert_eq!(map_logical_type(Some("numeric")).unwrap(), SqlType::Numeric);
        assert_eq!(map_logical_type(Some("jsonb")).unwrap(), SqlType::Jsonb);
        assert_eq!(map_logical_type(Some("bigint")).unwrap(), SqlType::BigInt);
        assert_eq!(
            map_logical_type(Some("smallint")).unwrap(),
            SqlType::SmallInt
        );
    }
}
