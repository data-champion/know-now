use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
    Quiet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonEnvelope<T> {
    pub version: String,
    pub command: String,
    pub result: String,
    pub payload: T,
}

impl<T: Serialize> JsonEnvelope<T> {
    pub fn success(command: &str, payload: T) -> Self {
        Self {
            version: crate::JSON_ENVELOPE_VERSION.to_owned(),
            command: command.to_owned(),
            result: "success".to_owned(),
            payload,
        }
    }

    pub fn error(command: &str, payload: T) -> Self {
        Self {
            version: crate::JSON_ENVELOPE_VERSION.to_owned(),
            command: command.to_owned(),
            result: "error".to_owned(),
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_envelope_success_roundtrip() {
        let env = JsonEnvelope::success("version", "0.1.0");
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains(r#""version":"1""#));
        assert!(json.contains(r#""result":"success""#));
        assert!(json.contains(r#""command":"version""#));
    }

    #[test]
    fn json_envelope_error_roundtrip() {
        let env = JsonEnvelope::<String>::error("validate", "failed".into());
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains(r#""result":"error""#));
    }

    #[test]
    fn output_format_default_is_text() {
        assert_eq!(OutputFormat::default(), OutputFormat::Text);
    }
}
