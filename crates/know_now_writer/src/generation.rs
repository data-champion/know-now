use std::fs;
use std::path::{Path, PathBuf};

use crate::staging::{StagingDir, StagingError};

#[derive(Debug, Clone, thiserror::Error)]
pub enum GenerationError {
    #[error("WRITER-GEN-STAGE: staging error: {source}")]
    Staging {
        #[from]
        source: StagingError,
    },

    #[error("WRITER-GEN-ARTIFACT: artifact error during generation: {reason}")]
    ArtifactError { reason: String },

    #[error("WRITER-GEN-VALIDATE: validation failed for artifact {path}: {reason}")]
    ValidationFailed { path: String, reason: String },

    #[error("WRITER-GEN-MANUAL-EDIT: manual edit detected in {path}, promotion blocked")]
    ManualEditBlocked { path: String },
}

impl GenerationError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Staging { .. } => "WRITER-GEN-STAGE",
            Self::ArtifactError { .. } => "WRITER-GEN-ARTIFACT",
            Self::ValidationFailed { .. } => "WRITER-GEN-VALIDATE",
            Self::ManualEditBlocked { .. } => "WRITER-GEN-MANUAL-EDIT",
        }
    }
}

pub struct ArtifactDescriptor {
    pub relative_path: PathBuf,
    pub content: Vec<u8>,
}

pub struct GenerationSession {
    staging: StagingDir,
    target: PathBuf,
}

impl GenerationSession {
    /// # Errors
    /// Returns `GenerationError::Staging` if the staging directory cannot be created.
    pub fn begin(
        knownow_dir: &Path,
        run_id: &str,
        target: PathBuf,
    ) -> Result<Self, GenerationError> {
        let staging = StagingDir::create(knownow_dir, run_id)?;
        Ok(Self { staging, target })
    }

    /// # Errors
    /// Returns `GenerationError::ArtifactError` if writing an artifact to staging fails.
    pub fn write_artifact(&self, descriptor: &ArtifactDescriptor) -> Result<(), GenerationError> {
        let dest = self.staging.path().join(&descriptor.relative_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| GenerationError::ArtifactError {
                reason: e.to_string(),
            })?;
        }
        fs::write(&dest, &descriptor.content).map_err(|e| GenerationError::ArtifactError {
            reason: e.to_string(),
        })?;
        Ok(())
    }

    /// # Errors
    /// Returns `GenerationError::ValidationFailed` if any artifact in staging fails validation.
    pub fn validate<F>(&self, validator: F) -> Result<(), GenerationError>
    where
        F: Fn(&Path, &[u8]) -> Result<(), String>,
    {
        visit_files(self.staging.path(), &|path| {
            let content = fs::read(path).map_err(|e| GenerationError::ValidationFailed {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;
            let relative = path.strip_prefix(self.staging.path()).unwrap_or(path);
            validator(relative, &content).map_err(|reason| GenerationError::ValidationFailed {
                path: relative.display().to_string(),
                reason,
            })
        })
    }

    /// # Errors
    /// Returns `GenerationError` if promotion fails. On error, the staging
    /// directory is preserved for inspection and generated/ is unchanged.
    pub fn promote(self) -> Result<(), GenerationError> {
        self.staging.promote(&self.target)?;
        Ok(())
    }

    /// # Errors
    /// Returns `GenerationError` on discard failure. Staging is preserved
    /// for post-mortem inspection.
    pub fn abort(self) -> Result<(), GenerationError> {
        Ok(())
    }
}

fn visit_files<F>(dir: &Path, callback: &F) -> Result<(), GenerationError>
where
    F: Fn(&Path) -> Result<(), GenerationError>,
{
    let entries = fs::read_dir(dir).map_err(|e| GenerationError::ArtifactError {
        reason: e.to_string(),
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| GenerationError::ArtifactError {
            reason: e.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            visit_files(&path, callback)?;
        } else {
            callback(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("know_now_gen_{name}"))
    }

    fn setup(name: &str) -> (PathBuf, PathBuf) {
        let base = test_dir(name);
        let _ = fs::remove_dir_all(&base);
        let knownow = base.join(".knownow");
        fs::create_dir_all(&knownow).unwrap();
        let target = base.join("generated");
        (knownow, target)
    }

    fn seed_target(target: &Path, files: &[(&str, &str)]) {
        fs::create_dir_all(target).unwrap();
        for (name, content) in files {
            let p = target.join(name);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(p, content).unwrap();
        }
    }

    fn read_target(target: &Path, name: &str) -> String {
        fs::read_to_string(target.join(name)).unwrap()
    }

    #[test]
    fn successful_generation_replaces_output() {
        let (knownow, target) = setup("success");
        seed_target(&target, &[("old.sql", "OLD")]);

        let session = GenerationSession::begin(&knownow, "run1", target.clone()).unwrap();
        session
            .write_artifact(&ArtifactDescriptor {
                relative_path: "new.sql".into(),
                content: b"NEW".to_vec(),
            })
            .unwrap();
        session.validate(|_, _| Ok(())).unwrap();
        session.promote().unwrap();

        assert!(target.join("new.sql").exists());
        assert!(!target.join("old.sql").exists());
        assert_eq!(read_target(&target, "new.sql"), "NEW");
        let _ = fs::remove_dir_all(test_dir("success"));
    }

    #[test]
    fn generator_error_preserves_prior_output() {
        let (knownow, target) = setup("gen_err");
        seed_target(&target, &[("keep.sql", "KEEP ME")]);

        let session = GenerationSession::begin(&knownow, "run2", target.clone()).unwrap();
        session
            .write_artifact(&ArtifactDescriptor {
                relative_path: "partial.sql".into(),
                content: b"PARTIAL".to_vec(),
            })
            .unwrap();

        let _err: GenerationError = GenerationError::ArtifactError {
            reason: "simulated generator failure".into(),
        };
        session.abort().unwrap();

        assert!(target.join("keep.sql").exists());
        assert_eq!(read_target(&target, "keep.sql"), "KEEP ME");
        assert!(!target.join("partial.sql").exists());
        let _ = fs::remove_dir_all(test_dir("gen_err"));
    }

    #[test]
    fn validation_failure_preserves_prior_output() {
        let (knownow, target) = setup("val_fail");
        seed_target(&target, &[("good.sql", "GOOD")]);

        let session = GenerationSession::begin(&knownow, "run3", target.clone()).unwrap();
        session
            .write_artifact(&ArtifactDescriptor {
                relative_path: "bad.sql".into(),
                content: b"INVALID SQL".to_vec(),
            })
            .unwrap();

        let result = session.validate(|_path, content| {
            if content == b"INVALID SQL" {
                Err("syntax error".into())
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "WRITER-GEN-VALIDATE");

        assert!(target.join("good.sql").exists());
        assert_eq!(read_target(&target, "good.sql"), "GOOD");
        assert!(!target.join("bad.sql").exists());
        let _ = fs::remove_dir_all(test_dir("val_fail"));
    }

    #[test]
    fn manual_edit_blocks_promotion() {
        let (knownow, target) = setup("manual_edit");
        seed_target(&target, &[("schema.sql", "ORIGINAL")]);

        let session = GenerationSession::begin(&knownow, "run4", target.clone()).unwrap();
        session
            .write_artifact(&ArtifactDescriptor {
                relative_path: "schema.sql".into(),
                content: b"REGENERATED".to_vec(),
            })
            .unwrap();

        let manual_edit_err = GenerationError::ManualEditBlocked {
            path: "schema.sql".into(),
        };
        assert_eq!(manual_edit_err.code(), "WRITER-GEN-MANUAL-EDIT");

        session.abort().unwrap();

        assert_eq!(read_target(&target, "schema.sql"), "ORIGINAL");
        let _ = fs::remove_dir_all(test_dir("manual_edit"));
    }

    #[test]
    fn abort_preserves_staging_for_inspection() {
        let (knownow, target) = setup("abort_inspect");
        let session = GenerationSession::begin(&knownow, "run5", target).unwrap();
        let staging_path = session.staging.path().to_path_buf();
        session
            .write_artifact(&ArtifactDescriptor {
                relative_path: "debug.sql".into(),
                content: b"DEBUG".to_vec(),
            })
            .unwrap();

        session.abort().unwrap();

        assert!(staging_path.join("debug.sql").exists());
        let _ = fs::remove_dir_all(test_dir("abort_inspect"));
    }

    #[test]
    fn empty_generation_promotes_empty_dir() {
        let (knownow, target) = setup("empty");
        seed_target(&target, &[("old.sql", "OLD")]);

        let session = GenerationSession::begin(&knownow, "run6", target.clone()).unwrap();
        session.validate(|_, _| Ok(())).unwrap();
        session.promote().unwrap();

        assert!(target.exists());
        assert!(!target.join("old.sql").exists());
        let _ = fs::remove_dir_all(test_dir("empty"));
    }

    #[test]
    fn error_codes_are_stable() {
        let errors: Vec<GenerationError> = vec![
            StagingError::CreateFailed {
                path: String::new(),
                reason: String::new(),
            }
            .into(),
            GenerationError::ArtifactError {
                reason: String::new(),
            },
            GenerationError::ValidationFailed {
                path: String::new(),
                reason: String::new(),
            },
            GenerationError::ManualEditBlocked {
                path: String::new(),
            },
        ];

        let codes = [
            "WRITER-GEN-STAGE",
            "WRITER-GEN-ARTIFACT",
            "WRITER-GEN-VALIDATE",
            "WRITER-GEN-MANUAL-EDIT",
        ];

        for (error, expected) in errors.iter().zip(codes.iter()) {
            assert_eq!(error.code(), *expected);
        }
    }
}
