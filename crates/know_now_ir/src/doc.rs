use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Doc(String);

impl Doc {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Doc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_display() {
        let doc = Doc::new("Customer master table");
        assert_eq!(doc.to_string(), "Customer master table");
    }

    #[test]
    fn doc_as_str() {
        let doc = Doc::new("hello");
        assert_eq!(doc.as_str(), "hello");
    }

    #[test]
    fn doc_serde_roundtrip() {
        let doc = Doc::new("Customer table");
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Doc = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, doc);
    }
}
