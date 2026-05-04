use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub source_id: SourceId,
    pub byte_start: u32,
    pub byte_end: u32,
    pub line_start: u32,
    pub line_end: u32,
    pub col_start: u32,
    pub col_end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct YamlPath(String);

impl YamlPath {
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn child(&self, segment: &str) -> Self {
        if self.0.is_empty() {
            Self(segment.to_owned())
        } else {
            Self(format!("{}.{segment}", self.0))
        }
    }

    #[must_use]
    pub fn index(&self, i: usize) -> Self {
        Self(format!("{}[{i}]", self.0))
    }
}

impl std::fmt::Display for YamlPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceSpanIndex {
    by_object_id: BTreeMap<String, SourceSpan>,
    by_yaml_path: BTreeMap<YamlPath, SourceSpan>,
}

impl SourceSpanIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_by_object_id(&mut self, object_id: impl Into<String>, span: SourceSpan) {
        self.by_object_id.insert(object_id.into(), span);
    }

    pub fn insert_by_yaml_path(&mut self, path: YamlPath, span: SourceSpan) {
        self.by_yaml_path.insert(path, span);
    }

    #[must_use]
    pub fn lookup_by_object_id(&self, object_id: &str) -> Option<&SourceSpan> {
        self.by_object_id.get(object_id)
    }

    #[must_use]
    pub fn lookup_by_yaml_path(&self, path: &YamlPath) -> Option<&SourceSpan> {
        self.by_yaml_path.get(path)
    }

    #[must_use]
    pub fn object_id_count(&self) -> usize {
        self.by_object_id.len()
    }

    #[must_use]
    pub fn yaml_path_count(&self) -> usize {
        self.by_yaml_path.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_span() -> SourceSpan {
        SourceSpan {
            source_id: SourceId(0),
            byte_start: 10,
            byte_end: 50,
            line_start: 2,
            line_end: 5,
            col_start: 1,
            col_end: 20,
        }
    }

    #[test]
    fn source_id_ordering() {
        assert!(SourceId(0) < SourceId(1));
        assert_eq!(SourceId(42), SourceId(42));
    }

    #[test]
    fn yaml_path_construction() {
        let root = YamlPath::new("entities");
        assert_eq!(root.as_str(), "entities");

        let indexed = root.index(3);
        assert_eq!(indexed.as_str(), "entities[3]");

        let attr = indexed.child("attributes");
        assert_eq!(attr.as_str(), "entities[3].attributes");

        let name = attr.index(2).child("name");
        assert_eq!(name.as_str(), "entities[3].attributes[2].name");
    }

    #[test]
    fn yaml_path_empty_root() {
        let root = YamlPath::new("");
        let child = root.child("entities");
        assert_eq!(child.as_str(), "entities");
    }

    #[test]
    fn yaml_path_display() {
        let path = YamlPath::new("entities[0].name");
        assert_eq!(format!("{path}"), "entities[0].name");
    }

    #[test]
    fn span_json_roundtrip() {
        let span = sample_span();
        let json = serde_json::to_string(&span).unwrap();
        let parsed: SourceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span, parsed);
    }

    #[test]
    fn source_id_json_roundtrip() {
        let id = SourceId(42);
        let json = serde_json::to_string(&id).unwrap();
        let parsed: SourceId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn yaml_path_json_roundtrip() {
        let path = YamlPath::new("entities[3].attributes[2].name");
        let json = serde_json::to_string(&path).unwrap();
        let parsed: YamlPath = serde_json::from_str(&json).unwrap();
        assert_eq!(path, parsed);
    }

    #[test]
    fn index_lookup_by_object_id() {
        let mut index = SourceSpanIndex::new();
        let span = sample_span();
        index.insert_by_object_id("ent_customer", span.clone());

        assert_eq!(index.lookup_by_object_id("ent_customer"), Some(&span));
        assert_eq!(index.lookup_by_object_id("ent_missing"), None);
        assert_eq!(index.object_id_count(), 1);
    }

    #[test]
    fn index_lookup_by_yaml_path() {
        let mut index = SourceSpanIndex::new();
        let span = sample_span();
        let path = YamlPath::new("entities[0].name");
        index.insert_by_yaml_path(path.clone(), span.clone());

        assert_eq!(index.lookup_by_yaml_path(&path), Some(&span));
        assert_eq!(index.lookup_by_yaml_path(&YamlPath::new("missing")), None);
        assert_eq!(index.yaml_path_count(), 1);
    }

    #[test]
    fn index_json_roundtrip() {
        let mut index = SourceSpanIndex::new();
        index.insert_by_object_id("ent_customer", sample_span());
        index.insert_by_yaml_path(YamlPath::new("entities[0]"), sample_span());

        let json = serde_json::to_string(&index).unwrap();
        let parsed: SourceSpanIndex = serde_json::from_str(&json).unwrap();

        assert_eq!(index.object_id_count(), parsed.object_id_count());
        assert_eq!(index.yaml_path_count(), parsed.yaml_path_count());
        assert_eq!(
            index.lookup_by_object_id("ent_customer"),
            parsed.lookup_by_object_id("ent_customer")
        );
    }

    #[test]
    fn unicode_yaml_path() {
        let path = YamlPath::new("entités[0].données");
        let json = serde_json::to_string(&path).unwrap();
        let parsed: YamlPath = serde_json::from_str(&json).unwrap();
        assert_eq!(path, parsed);
        assert!(json.contains("entités"));
    }

    #[test]
    fn span_with_bom_offset() {
        let span = SourceSpan {
            source_id: SourceId(0),
            byte_start: 3,
            byte_end: 50,
            line_start: 1,
            line_end: 3,
            col_start: 1,
            col_end: 10,
        };
        let json = serde_json::to_string(&span).unwrap();
        let parsed: SourceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span, parsed);
        assert_eq!(span.byte_start, 3);
    }

    #[test]
    fn btree_ordering_is_deterministic() {
        let mut index = SourceSpanIndex::new();
        for i in 0..10 {
            index.insert_by_object_id(
                format!("obj_{i}"),
                SourceSpan {
                    source_id: SourceId(0),
                    byte_start: i * 10,
                    byte_end: i * 10 + 9,
                    line_start: i + 1,
                    line_end: i + 1,
                    col_start: 1,
                    col_end: 10,
                },
            );
        }

        let json1 = serde_json::to_string(&index).unwrap();
        let json2 = serde_json::to_string(&index).unwrap();
        assert_eq!(json1, json2);
    }
}
