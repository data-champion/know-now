use know_now_diagnostics::diagnostic::Diagnostic;
use know_now_metadata::authoring::AuthoringMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyPackInfo {
    pub pack: String,
    pub version: String,
    pub hash: String,
}

pub struct PolicyRule {
    pub code: &'static str,
    pub name: &'static str,
    pub rationale: &'static str,
    pub remediation: &'static str,
}

pub trait PolicyPack: Send + Sync {
    fn info(&self) -> PolicyPackInfo;
    fn rules(&self) -> &[PolicyRule];
    fn evaluate(&self, metadata: &AuthoringMetadata) -> Vec<Diagnostic>;
}

pub fn evaluate_policy(pack: &dyn PolicyPack, metadata: &AuthoringMetadata) -> Vec<Diagnostic> {
    pack.evaluate(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_pack_info_serializes() {
        let info = PolicyPackInfo {
            pack: "dc_standard".into(),
            version: "1.0".into(),
            hash: "sha256:abc".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("dc_standard"));
        let parsed: PolicyPackInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pack, "dc_standard");
    }
}
