//! Lockfile schema and checks crate for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

pub mod check;
pub mod lockfile;
pub mod resolved;

pub const CURRENT_SCHEMA_VERSION: &str = "1.0";
pub const LOCKFILE_NAME: &str = "know-now.lock";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaVersion {
    V1,
}

impl SchemaVersion {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V1 => "1.0",
        }
    }

    #[must_use]
    pub fn from_version_str(s: &str) -> Option<Self> {
        match s {
            "1.0" => Some(Self::V1),
            _ => None,
        }
    }

    #[must_use]
    pub fn current() -> Self {
        Self::V1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_roundtrip() {
        let v = SchemaVersion::V1;
        assert_eq!(SchemaVersion::from_version_str(v.as_str()), Some(v));
    }

    #[test]
    fn current_schema_version_matches_constant() {
        assert_eq!(SchemaVersion::current().as_str(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn unknown_schema_version_returns_none() {
        assert_eq!(SchemaVersion::from_version_str("99.0"), None);
    }
}
