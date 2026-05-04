use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    pub target: String,
    pub renderer: RendererRef,
    pub output_dir: String,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub limits: Limits,
    #[serde(default)]
    pub trust: TrustLevel,
    #[serde(default)]
    pub licensing: Option<Licensing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererRef {
    pub kind: String,
    #[serde(default = "default_profile")]
    pub profile: u32,
}

fn default_profile() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Permissions {
    #[serde(default = "default_filesystem")]
    pub filesystem: String,
    #[serde(default = "default_network")]
    pub network: String,
}

fn default_filesystem() -> String {
    "output_only".into()
}

fn default_network() -> String {
    "none".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    #[serde(default = "default_max_templates")]
    pub max_templates: usize,
    #[serde(default = "default_max_template_bytes")]
    pub max_template_bytes: usize,
    #[serde(default = "default_max_output_files")]
    pub max_output_files: usize,
    #[serde(default = "default_max_output_bytes")]
    pub max_output_bytes: usize,
    #[serde(default = "default_max_fuel")]
    pub max_fuel: u64,
    #[serde(default = "default_max_include_depth")]
    pub max_include_depth: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_templates: default_max_templates(),
            max_template_bytes: default_max_template_bytes(),
            max_output_files: default_max_output_files(),
            max_output_bytes: default_max_output_bytes(),
            max_fuel: default_max_fuel(),
            max_include_depth: default_max_include_depth(),
        }
    }
}

fn default_max_templates() -> usize {
    100
}
fn default_max_template_bytes() -> usize {
    262_144
}
fn default_max_output_files() -> usize {
    100
}
fn default_max_output_bytes() -> usize {
    10_485_760
}
fn default_max_fuel() -> u64 {
    50_000
}
fn default_max_include_depth() -> usize {
    8
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    BuiltIn,
    Approved,
    Experimental,
    #[default]
    Untrusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Licensing {
    pub license: String,
    #[serde(default)]
    pub license_url: Option<String>,
    #[serde(default)]
    pub license_review: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("renderer kind must be 'know-now-minijinja'")]
    UnsupportedRenderer,
    #[error("renderer profile must be 1 (know-now-minijinja-v1)")]
    UnsupportedProfile,
    #[error("output_dir must not be empty")]
    EmptyOutputDir,
    #[error("output_dir must not escape pack root: {0}")]
    OutputDirEscape(String),
    #[error("licensing metadata required in locked mode")]
    MissingLicensing,
    #[error("max_fuel must be greater than zero")]
    ZeroFuel,
}

pub fn validate_manifest(
    manifest: &PackManifest,
    locked_mode: bool,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if manifest.renderer.kind != "know-now-minijinja" {
        errors.push(ValidationError::UnsupportedRenderer);
    }
    if manifest.renderer.profile != 1 {
        errors.push(ValidationError::UnsupportedProfile);
    }
    if manifest.output_dir.is_empty() {
        errors.push(ValidationError::EmptyOutputDir);
    }
    if manifest.output_dir.contains("..") {
        errors.push(ValidationError::OutputDirEscape(
            manifest.output_dir.clone(),
        ));
    }
    if manifest.limits.max_fuel == 0 {
        errors.push(ValidationError::ZeroFuel);
    }
    if locked_mode && manifest.licensing.is_none() {
        errors.push(ValidationError::MissingLicensing);
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> PackManifest {
        PackManifest {
            name: "test-pack".into(),
            version: "1.0.0".into(),
            target: "postgres".into(),
            renderer: RendererRef {
                kind: "know-now-minijinja".into(),
                profile: 1,
            },
            output_dir: "output".into(),
            permissions: Permissions::default(),
            limits: Limits::default(),
            trust: TrustLevel::Untrusted,
            licensing: Some(Licensing {
                license: "MIT".into(),
                license_url: None,
                license_review: None,
            }),
        }
    }

    #[test]
    fn valid_manifest_passes() {
        let errors = validate_manifest(&valid_manifest(), true);
        assert!(errors.is_empty());
    }

    #[test]
    fn wrong_renderer_rejected() {
        let mut m = valid_manifest();
        m.renderer.kind = "handlebars".into();
        let errors = validate_manifest(&m, false);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::UnsupportedRenderer)));
    }

    #[test]
    fn wrong_profile_rejected() {
        let mut m = valid_manifest();
        m.renderer.profile = 2;
        let errors = validate_manifest(&m, false);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::UnsupportedProfile)));
    }

    #[test]
    fn output_dir_escape_rejected() {
        let mut m = valid_manifest();
        m.output_dir = "../escape".into();
        let errors = validate_manifest(&m, false);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::OutputDirEscape(_))));
    }

    #[test]
    fn missing_license_in_locked_mode() {
        let mut m = valid_manifest();
        m.licensing = None;
        let errors = validate_manifest(&m, true);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::MissingLicensing)));
    }

    #[test]
    fn missing_license_allowed_in_unlocked_mode() {
        let mut m = valid_manifest();
        m.licensing = None;
        let errors = validate_manifest(&m, false);
        assert!(errors.is_empty());
    }

    #[test]
    fn zero_fuel_rejected() {
        let mut m = valid_manifest();
        m.limits.max_fuel = 0;
        let errors = validate_manifest(&m, false);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::ZeroFuel)));
    }
}
