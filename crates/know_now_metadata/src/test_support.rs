use crate::authoring::AuthoringMetadata;

/// Parse YAML into `AuthoringMetadata` for use in tests outside this crate.
///
/// # Panics
///
/// Panics if the YAML fails to deserialize.
#[must_use]
pub fn parse_yaml_metadata(yaml: &str) -> AuthoringMetadata {
    serde_saphyr::from_str(yaml).expect("test YAML should be valid AuthoringMetadata")
}
