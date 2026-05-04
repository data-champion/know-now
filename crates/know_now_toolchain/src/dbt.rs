use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbtValidationMode {
    #[default]
    None,
    Dbt,
    DbtCore,
    DbtFusion,
    Docker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbtConfig {
    #[serde(default)]
    pub mode: DbtValidationMode,
    #[serde(default = "default_executable")]
    pub executable: String,
    #[serde(default)]
    pub required_in_ci: bool,
    pub docker_image: Option<String>,
}

fn default_executable() -> String {
    "dbt".to_owned()
}

impl Default for DbtConfig {
    fn default() -> Self {
        Self {
            mode: DbtValidationMode::None,
            executable: default_executable(),
            required_in_ci: false,
            docker_image: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbtIdentity {
    Core,
    Fusion,
    Compatible,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbtDetection {
    pub identity: DbtIdentity,
    pub version: Option<String>,
    pub adapter: Option<String>,
    pub supported_commands: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbtValidationResult {
    pub success: bool,
    pub commands_run: Vec<String>,
    pub diagnostics: Vec<DbtDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbtDiagnostic {
    pub severity: DbtDiagnosticSeverity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbtDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, thiserror::Error)]
pub enum DbtError {
    #[error("dbt executable not found: {0}")]
    NotFound(String),
    #[error("dbt validation failed: {0}")]
    ValidationFailed(String),
    #[error("unsupported dbt identity {identity:?} for mode {mode:?}")]
    IdentityMismatch {
        identity: DbtIdentity,
        mode: DbtValidationMode,
    },
    #[error("docker image must be pinned by digest in locked mode: {0}")]
    UnpinnedDockerImage(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct DbtAdapter {
    config: DbtConfig,
}

impl DbtAdapter {
    #[must_use]
    pub fn new(config: DbtConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn config(&self) -> &DbtConfig {
        &self.config
    }

    /// Detect the installed dbt identity and version.
    ///
    /// # Errors
    ///
    /// Returns `DbtError::NotFound` when the configured executable is absent.
    pub fn detect(&self) -> Result<DbtDetection, DbtError> {
        if self.config.mode == DbtValidationMode::None {
            return Ok(DbtDetection {
                identity: DbtIdentity::Unknown,
                version: None,
                adapter: None,
                supported_commands: vec![],
                limitations: vec!["mode=none: dbt detection skipped".into()],
            });
        }

        if self.config.mode == DbtValidationMode::Docker {
            return Ok(Self::detect_docker());
        }

        let output = Command::new(&self.config.executable)
            .arg("--version")
            .output()
            .map_err(|_| DbtError::NotFound(self.config.executable.clone()))?;

        if !output.status.success() {
            return Err(DbtError::NotFound(self.config.executable.clone()));
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        Ok(parse_dbt_version(&version_output))
    }

    /// Run dbt validation (parse + compile) against a generated project.
    ///
    /// # Errors
    ///
    /// Returns `DbtError` when detection fails, identity mismatches the
    /// configured mode, or docker image is unpinned in locked mode.
    pub fn validate(
        &self,
        project_dir: &Path,
        locked: bool,
    ) -> Result<DbtValidationResult, DbtError> {
        if self.config.mode == DbtValidationMode::None {
            return Ok(DbtValidationResult {
                success: true,
                commands_run: vec![],
                diagnostics: vec![],
            });
        }

        if self.config.mode == DbtValidationMode::Docker {
            if locked {
                self.validate_docker_image_pinned()?;
            }
            return self.validate_docker(project_dir);
        }

        let detection = self.detect()?;
        self.validate_identity(&detection)?;

        let mut commands_run = Vec::new();
        let mut diagnostics = Vec::new();
        let mut overall_success = true;

        for cmd in &["parse", "compile"] {
            commands_run.push(format!("dbt {cmd}"));
            match run_dbt_command(&self.config.executable, cmd, project_dir) {
                Ok(result) => {
                    diagnostics.extend(result.diagnostics);
                    if !result.success {
                        overall_success = false;
                    }
                }
                Err(e) => {
                    diagnostics.push(DbtDiagnostic {
                        severity: DbtDiagnosticSeverity::Error,
                        message: e.to_string(),
                        file: None,
                        line: None,
                    });
                    overall_success = false;
                    break;
                }
            }
        }

        Ok(DbtValidationResult {
            success: overall_success,
            commands_run,
            diagnostics,
        })
    }

    fn validate_identity(&self, detection: &DbtDetection) -> Result<(), DbtError> {
        match self.config.mode {
            DbtValidationMode::DbtCore => {
                if detection.identity != DbtIdentity::Core
                    && detection.identity != DbtIdentity::Compatible
                {
                    return Err(DbtError::IdentityMismatch {
                        identity: detection.identity.clone(),
                        mode: self.config.mode.clone(),
                    });
                }
            }
            DbtValidationMode::DbtFusion => {
                if detection.identity != DbtIdentity::Fusion {
                    return Err(DbtError::IdentityMismatch {
                        identity: detection.identity.clone(),
                        mode: self.config.mode.clone(),
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_docker_image_pinned(&self) -> Result<(), DbtError> {
        if let Some(ref image) = self.config.docker_image {
            if !image.contains('@') {
                return Err(DbtError::UnpinnedDockerImage(image.clone()));
            }
        }
        Ok(())
    }

    fn detect_docker() -> DbtDetection {
        DbtDetection {
            identity: DbtIdentity::Unknown,
            version: None,
            adapter: None,
            supported_commands: vec!["parse".into(), "compile".into()],
            limitations: vec!["docker mode: identity detected at runtime".into()],
        }
    }

    fn validate_docker(&self, project_dir: &Path) -> Result<DbtValidationResult, DbtError> {
        let image = self
            .config
            .docker_image
            .as_deref()
            .ok_or_else(|| DbtError::ValidationFailed("docker_image not configured".into()))?;

        let mut commands_run = Vec::new();
        let mut diagnostics = Vec::new();
        let mut overall_success = true;

        for cmd in &["parse", "compile"] {
            commands_run.push(format!("docker run {image} dbt {cmd}"));
            match run_docker_dbt_command(image, cmd, project_dir) {
                Ok(result) => {
                    diagnostics.extend(result.diagnostics);
                    if !result.success {
                        overall_success = false;
                    }
                }
                Err(e) => {
                    diagnostics.push(DbtDiagnostic {
                        severity: DbtDiagnosticSeverity::Error,
                        message: e.to_string(),
                        file: None,
                        line: None,
                    });
                    overall_success = false;
                    break;
                }
            }
        }

        Ok(DbtValidationResult {
            success: overall_success,
            commands_run,
            diagnostics,
        })
    }
}

fn parse_dbt_version(version_output: &str) -> DbtDetection {
    let trimmed = version_output.trim();
    let version = extract_version_number(trimmed);

    let identity = if trimmed.contains("dbt-fusion") || trimmed.contains("dbt Fusion") {
        DbtIdentity::Fusion
    } else if trimmed.contains("dbt-core") || trimmed.contains("Core") {
        version
            .as_ref()
            .and_then(|v| parse_major_minor(v))
            .map_or(DbtIdentity::Compatible, |(major, minor)| {
                if major >= 1 && minor >= 7 {
                    DbtIdentity::Core
                } else {
                    DbtIdentity::Compatible
                }
            })
    } else {
        DbtIdentity::Unknown
    };

    let adapter = extract_adapter(trimmed);

    let supported_commands = if identity == DbtIdentity::Fusion {
        vec![
            "parse".into(),
            "compile".into(),
            "build".into(),
            "test".into(),
        ]
    } else {
        vec!["parse".into(), "compile".into(), "build".into()]
    };

    let mut limitations = Vec::new();
    if identity == DbtIdentity::Unknown {
        limitations.push("unrecognized dbt distribution".into());
    }
    if identity == DbtIdentity::Compatible {
        limitations.push("dbt-core version <1.7 detected; some features may be unavailable".into());
    }

    DbtDetection {
        identity,
        version,
        adapter,
        supported_commands,
        limitations,
    }
}

fn extract_version_number(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let candidate = &text[start..i];
            if candidate.contains('.') {
                return Some(candidate.to_owned());
            }
        } else {
            i += 1;
        }
    }
    None
}

fn extract_adapter(text: &str) -> Option<String> {
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("adapter") || lower.contains("plugin") {
            if let Some(name) = lower
                .split_whitespace()
                .find(|w| w.starts_with("postgres") || w.starts_with("snowflake") || w.starts_with("bigquery") || w.starts_with("redshift") || w.starts_with("duckdb") || w.starts_with("databricks"))
            {
                return Some(name.to_owned());
            }
        }
    }
    None
}

fn parse_major_minor(version: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        Some((major, minor))
    } else {
        None
    }
}

fn run_dbt_command(
    executable: &str,
    subcommand: &str,
    project_dir: &Path,
) -> Result<DbtCommandResult, DbtError> {
    let output = Command::new(executable)
        .arg(subcommand)
        .arg("--project-dir")
        .arg(project_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let diagnostics = parse_dbt_output(&stdout, &stderr);

    Ok(DbtCommandResult {
        success: output.status.success(),
        diagnostics,
    })
}

fn run_docker_dbt_command(
    image: &str,
    subcommand: &str,
    project_dir: &Path,
) -> Result<DbtCommandResult, DbtError> {
    let mount = format!("{}:/usr/app", project_dir.display());
    let output = Command::new("docker")
        .args(["run", "--rm", "-v", &mount, "-w", "/usr/app", image, "dbt", subcommand])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let diagnostics = parse_dbt_output(&stdout, &stderr);

    Ok(DbtCommandResult {
        success: output.status.success(),
        diagnostics,
    })
}

struct DbtCommandResult {
    success: bool,
    diagnostics: Vec<DbtDiagnostic>,
}

fn parse_dbt_output(stdout: &str, stderr: &str) -> Vec<DbtDiagnostic> {
    let mut diagnostics = Vec::new();

    for line in stderr.lines().chain(stdout.lines()) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.contains("ERROR") || trimmed.contains("error") {
            let (file, line_no) = extract_file_line(trimmed);
            diagnostics.push(DbtDiagnostic {
                severity: DbtDiagnosticSeverity::Error,
                message: trimmed.to_owned(),
                file,
                line: line_no,
            });
        } else if trimmed.contains("WARNING") || trimmed.contains("warning") {
            let (file, line_no) = extract_file_line(trimmed);
            diagnostics.push(DbtDiagnostic {
                severity: DbtDiagnosticSeverity::Warning,
                message: trimmed.to_owned(),
                file,
                line: line_no,
            });
        }
    }

    diagnostics
}

fn extract_file_line(message: &str) -> (Option<String>, Option<u32>) {
    for word in message.split_whitespace() {
        if word.contains(".sql:") || word.contains(".yml:") || word.contains(".yaml:") {
            let parts: Vec<&str> = word.rsplitn(2, ':').collect();
            if parts.len() == 2 {
                if let Ok(line_no) = parts[0].parse::<u32>() {
                    return (Some(parts[1].to_owned()), Some(line_no));
                }
            }
            return (Some(word.trim_end_matches(':').to_owned()), None);
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_none() {
        let config = DbtConfig::default();
        assert_eq!(config.mode, DbtValidationMode::None);
        assert_eq!(config.executable, "dbt");
        assert!(!config.required_in_ci);
        assert!(config.docker_image.is_none());
    }

    #[test]
    fn mode_none_skips_detection() {
        let adapter = DbtAdapter::new(DbtConfig::default());
        let detection = adapter.detect().unwrap();
        assert_eq!(detection.identity, DbtIdentity::Unknown);
        assert!(detection.limitations[0].contains("mode=none"));
    }

    #[test]
    fn mode_none_validation_succeeds_immediately() {
        let adapter = DbtAdapter::new(DbtConfig::default());
        let result = adapter.validate(Path::new("/tmp"), false).unwrap();
        assert!(result.success);
        assert!(result.commands_run.is_empty());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn parse_dbt_core_version() {
        let output = "Core:\n  - installed: 1.8.2\n  - latest:    1.9.0\n";
        let detection = parse_dbt_version(output);
        assert_eq!(detection.identity, DbtIdentity::Core);
        assert_eq!(detection.version, Some("1.8.2".into()));
    }

    #[test]
    fn parse_dbt_core_old_version() {
        let output = "dbt-core 1.5.0\n";
        let detection = parse_dbt_version(output);
        assert_eq!(detection.identity, DbtIdentity::Compatible);
        assert_eq!(detection.version, Some("1.5.0".into()));
        assert!(detection.limitations.iter().any(|l| l.contains("<1.7")));
    }

    #[test]
    fn parse_dbt_fusion_version() {
        let output = "dbt Fusion v0.3.1\n";
        let detection = parse_dbt_version(output);
        assert_eq!(detection.identity, DbtIdentity::Fusion);
        assert_eq!(detection.version, Some("0.3.1".into()));
    }

    #[test]
    fn parse_unknown_dbt() {
        let output = "some-other-tool 2.0\n";
        let detection = parse_dbt_version(output);
        assert_eq!(detection.identity, DbtIdentity::Unknown);
        assert!(detection.limitations.iter().any(|l| l.contains("unrecognized")));
    }

    #[test]
    fn docker_mode_detect_returns_unknown_identity() {
        let config = DbtConfig {
            mode: DbtValidationMode::Docker,
            docker_image: Some("ghcr.io/dbt-labs/dbt-core:1.8@sha256:abc123".into()),
            ..Default::default()
        };
        let adapter = DbtAdapter::new(config);
        let detection = adapter.detect().unwrap();
        assert_eq!(detection.identity, DbtIdentity::Unknown);
        assert!(detection.limitations[0].contains("docker"));
    }

    #[test]
    fn docker_unpinned_image_rejected_in_locked_mode() {
        let config = DbtConfig {
            mode: DbtValidationMode::Docker,
            docker_image: Some("ghcr.io/dbt-labs/dbt-core:latest".into()),
            ..Default::default()
        };
        let adapter = DbtAdapter::new(config);
        let err = adapter.validate(Path::new("/tmp"), true).unwrap_err();
        assert!(matches!(err, DbtError::UnpinnedDockerImage(_)));
    }

    #[test]
    fn docker_pinned_image_accepted_in_locked_mode() {
        let config = DbtConfig {
            mode: DbtValidationMode::Docker,
            docker_image: Some("ghcr.io/dbt-labs/dbt-core@sha256:abc123".into()),
            ..Default::default()
        };
        let adapter = DbtAdapter::new(config);
        adapter.validate_docker_image_pinned().unwrap();
    }

    #[test]
    fn identity_mismatch_dbt_core_mode() {
        let adapter = DbtAdapter::new(DbtConfig {
            mode: DbtValidationMode::DbtCore,
            ..Default::default()
        });
        let detection = DbtDetection {
            identity: DbtIdentity::Fusion,
            version: Some("0.3.1".into()),
            adapter: None,
            supported_commands: vec![],
            limitations: vec![],
        };
        let err = adapter.validate_identity(&detection).unwrap_err();
        assert!(matches!(err, DbtError::IdentityMismatch { .. }));
    }

    #[test]
    fn identity_mismatch_dbt_fusion_mode() {
        let adapter = DbtAdapter::new(DbtConfig {
            mode: DbtValidationMode::DbtFusion,
            ..Default::default()
        });
        let detection = DbtDetection {
            identity: DbtIdentity::Core,
            version: Some("1.8.2".into()),
            adapter: None,
            supported_commands: vec![],
            limitations: vec![],
        };
        let err = adapter.validate_identity(&detection).unwrap_err();
        assert!(matches!(err, DbtError::IdentityMismatch { .. }));
    }

    #[test]
    fn compatible_identity_accepted_for_dbt_core_mode() {
        let adapter = DbtAdapter::new(DbtConfig {
            mode: DbtValidationMode::DbtCore,
            ..Default::default()
        });
        let detection = DbtDetection {
            identity: DbtIdentity::Compatible,
            version: Some("1.5.0".into()),
            adapter: None,
            supported_commands: vec![],
            limitations: vec![],
        };
        adapter.validate_identity(&detection).unwrap();
    }

    #[test]
    fn dbt_mode_accepts_any_identity() {
        let adapter = DbtAdapter::new(DbtConfig {
            mode: DbtValidationMode::Dbt,
            ..Default::default()
        });
        for identity in [
            DbtIdentity::Core,
            DbtIdentity::Fusion,
            DbtIdentity::Compatible,
            DbtIdentity::Unknown,
        ] {
            let detection = DbtDetection {
                identity,
                version: None,
                adapter: None,
                supported_commands: vec![],
                limitations: vec![],
            };
            adapter.validate_identity(&detection).unwrap();
        }
    }

    #[test]
    fn parse_dbt_error_output() {
        let stderr = "ERROR in model customer: column 'id' not found at models/marts/customer.sql:5\n";
        let diagnostics = parse_dbt_output("", stderr);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, DbtDiagnosticSeverity::Error);
        assert!(diagnostics[0].message.contains("column 'id' not found"));
    }

    #[test]
    fn parse_dbt_warning_output() {
        let stdout = "WARNING: Unused config detected\n";
        let diagnostics = parse_dbt_output(stdout, "");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, DbtDiagnosticSeverity::Warning);
    }

    #[test]
    fn extract_file_line_from_error() {
        let (file, line) = extract_file_line("ERROR at models/staging/stg_crm.sql:42 not found");
        assert_eq!(file, Some("models/staging/stg_crm.sql".into()));
        assert_eq!(line, Some(42));
    }

    #[test]
    fn extract_file_line_no_line_number() {
        let (file, line) = extract_file_line("ERROR in models/staging/stg_crm.sql: syntax");
        assert_eq!(file, Some("models/staging/stg_crm.sql".into()));
        assert_eq!(line, None);
    }

    #[test]
    fn extract_file_line_no_file() {
        let (file, line) = extract_file_line("ERROR: something went wrong");
        assert!(file.is_none());
        assert!(line.is_none());
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = DbtConfig {
            mode: DbtValidationMode::Dbt,
            executable: "dbt".into(),
            required_in_ci: true,
            docker_image: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: DbtConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mode, DbtValidationMode::Dbt);
        assert!(parsed.required_in_ci);
    }

    #[test]
    fn detection_serde_roundtrip() {
        let detection = DbtDetection {
            identity: DbtIdentity::Core,
            version: Some("1.8.2".into()),
            adapter: Some("postgres".into()),
            supported_commands: vec!["parse".into(), "compile".into()],
            limitations: vec![],
        };
        let json = serde_json::to_string(&detection).unwrap();
        let parsed: DbtDetection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.identity, DbtIdentity::Core);
        assert_eq!(parsed.version, Some("1.8.2".into()));
    }

    #[test]
    fn validation_result_serde_roundtrip() {
        let result = DbtValidationResult {
            success: true,
            commands_run: vec!["dbt parse".into()],
            diagnostics: vec![DbtDiagnostic {
                severity: DbtDiagnosticSeverity::Warning,
                message: "test".into(),
                file: None,
                line: None,
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: DbtValidationResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.diagnostics.len(), 1);
    }

    #[test]
    fn adapter_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<DbtAdapter>();
    }

    #[test]
    fn extract_version_number_works() {
        assert_eq!(extract_version_number("1.8.2"), Some("1.8.2".into()));
        assert_eq!(extract_version_number("v0.3.1-beta"), Some("0.3.1".into()));
        assert_eq!(
            extract_version_number("installed: 1.8.2"),
            Some("1.8.2".into())
        );
        assert_eq!(extract_version_number("no version here"), None);
    }

    #[test]
    fn parse_major_minor_works() {
        assert_eq!(parse_major_minor("1.8.2"), Some((1, 8)));
        assert_eq!(parse_major_minor("0.3.1"), Some((0, 3)));
        assert_eq!(parse_major_minor("1"), None);
    }
}
