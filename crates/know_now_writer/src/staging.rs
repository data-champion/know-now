use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, thiserror::Error)]
pub enum StagingError {
    #[error("WRITER-STAGE-CREATE: failed to create staging directory {path}: {reason}")]
    CreateFailed { path: String, reason: String },

    #[error("WRITER-STAGE-XDEV: staging and target are on different filesystems: staging={staging}, target={target}")]
    CrossDevice { staging: String, target: String },

    #[error("WRITER-STAGE-RENAME: rename failed from {from} to {to}: {reason}")]
    RenameFailed {
        from: String,
        to: String,
        reason: String,
    },

    #[error("WRITER-STAGE-CLEANUP: failed to remove staging directory {path}: {reason}")]
    CleanupFailed { path: String, reason: String },

    #[error("WRITER-STAGE-TARGET: target directory error at {path}: {reason}")]
    TargetError { path: String, reason: String },
}

impl StagingError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::CreateFailed { .. } => "WRITER-STAGE-CREATE",
            Self::CrossDevice { .. } => "WRITER-STAGE-XDEV",
            Self::RenameFailed { .. } => "WRITER-STAGE-RENAME",
            Self::CleanupFailed { .. } => "WRITER-STAGE-CLEANUP",
            Self::TargetError { .. } => "WRITER-STAGE-TARGET",
        }
    }
}

#[derive(Debug)]
pub struct StagingDir {
    path: PathBuf,
}

impl StagingDir {
    /// # Errors
    /// Returns `StagingError::CreateFailed` if the staging directory cannot be created.
    pub fn create(knownow_dir: &Path, run_id: &str) -> Result<Self, StagingError> {
        let path = knownow_dir.join("staging").join(run_id);
        fs::create_dir_all(&path).map_err(|e| StagingError::CreateFailed {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;
        Ok(Self { path })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// # Errors
    /// Returns `StagingError` if the promote fails due to cross-device rename,
    /// rename failure, or target directory errors.
    pub fn promote(self, target: &Path) -> Result<(), StagingError> {
        check_same_filesystem(&self.path, target)?;

        if target.exists() {
            let old = target.with_extension("old");
            if old.exists() {
                fs::remove_dir_all(&old).map_err(|e| StagingError::TargetError {
                    path: old.display().to_string(),
                    reason: e.to_string(),
                })?;
            }

            fs::rename(target, &old).map_err(|e| StagingError::RenameFailed {
                from: target.display().to_string(),
                to: old.display().to_string(),
                reason: e.to_string(),
            })?;

            match fs::rename(&self.path, target) {
                Ok(()) => {
                    let _ = fs::remove_dir_all(&old);
                }
                Err(e) => {
                    let _ = fs::rename(&old, target);
                    return Err(StagingError::RenameFailed {
                        from: self.path.display().to_string(),
                        to: target.display().to_string(),
                        reason: e.to_string(),
                    });
                }
            }
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| StagingError::TargetError {
                    path: parent.display().to_string(),
                    reason: e.to_string(),
                })?;
            }
            fs::rename(&self.path, target).map_err(|e| StagingError::RenameFailed {
                from: self.path.display().to_string(),
                to: target.display().to_string(),
                reason: e.to_string(),
            })?;
        }

        self.cleanup_parent();
        Ok(())
    }

    /// # Errors
    /// Returns `StagingError::CleanupFailed` if the staging directory cannot be removed.
    pub fn discard(self) -> Result<(), StagingError> {
        fs::remove_dir_all(&self.path).map_err(|e| StagingError::CleanupFailed {
            path: self.path.display().to_string(),
            reason: e.to_string(),
        })?;
        self.cleanup_parent();
        Ok(())
    }

    fn cleanup_parent(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
}

#[cfg(unix)]
fn check_same_filesystem(staging: &Path, target: &Path) -> Result<(), StagingError> {
    use std::os::unix::fs::MetadataExt;

    let target_check = if target.exists() {
        target.to_path_buf()
    } else {
        target
            .parent()
            .filter(|p| p.exists())
            .map_or_else(|| PathBuf::from("/"), Path::to_path_buf)
    };

    let staging_dev = fs::metadata(staging)
        .map_err(|_| StagingError::CrossDevice {
            staging: staging.display().to_string(),
            target: target_check.display().to_string(),
        })
        .map(|m| m.dev())?;

    let target_dev = fs::metadata(&target_check)
        .map_err(|_| StagingError::CrossDevice {
            staging: staging.display().to_string(),
            target: target_check.display().to_string(),
        })
        .map(|m| m.dev())?;

    if staging_dev != target_dev {
        return Err(StagingError::CrossDevice {
            staging: staging.display().to_string(),
            target: target.display().to_string(),
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_same_filesystem(_staging: &Path, _target: &Path) -> Result<(), StagingError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("know_now_staging_{name}"))
    }

    fn setup(name: &str) -> PathBuf {
        let dir = test_dir(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn create_staging_dir() {
        let base = setup("create");
        let staging = StagingDir::create(&base, "run_001").unwrap();
        assert!(staging.path().exists());
        assert!(staging.path().ends_with("staging/run_001"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn promote_to_new_target() {
        let base = setup("promote_new");
        let staging = StagingDir::create(&base, "run_002").unwrap();
        fs::write(staging.path().join("schema.sql"), "CREATE TABLE t;").unwrap();

        let target = base.join("generated");
        staging.promote(&target).unwrap();

        assert!(target.join("schema.sql").exists());
        assert_eq!(
            fs::read_to_string(target.join("schema.sql")).unwrap(),
            "CREATE TABLE t;"
        );
        assert!(!base.join("staging").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn promote_replaces_existing_target() {
        let base = setup("promote_replace");
        let target = base.join("generated");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("old.sql"), "OLD").unwrap();

        let staging = StagingDir::create(&base, "run_003").unwrap();
        fs::write(staging.path().join("new.sql"), "NEW").unwrap();

        staging.promote(&target).unwrap();

        assert!(target.join("new.sql").exists());
        assert!(!target.join("old.sql").exists());
        assert!(!base.join("generated.old").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn discard_removes_staging() {
        let base = setup("discard");
        let staging = StagingDir::create(&base, "run_004").unwrap();
        fs::write(staging.path().join("file.txt"), "data").unwrap();
        let staging_path = staging.path().to_path_buf();

        staging.discard().unwrap();
        assert!(!staging_path.exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn promote_preserves_target_on_staging_rename_failure() {
        let base = setup("preserve");
        let target = base.join("generated");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("keep.sql"), "KEEP").unwrap();

        let staging = StagingDir::create(&base, "run_005").unwrap();
        let staging_path = staging.path().to_path_buf();

        fs::remove_dir_all(&staging_path).unwrap();

        let result = staging.promote(&target);
        assert!(result.is_err());
        assert!(target.join("keep.sql").exists());
        assert_eq!(fs::read_to_string(target.join("keep.sql")).unwrap(), "KEEP");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn error_codes_are_stable() {
        let errors = [
            StagingError::CreateFailed {
                path: String::new(),
                reason: String::new(),
            },
            StagingError::CrossDevice {
                staging: String::new(),
                target: String::new(),
            },
            StagingError::RenameFailed {
                from: String::new(),
                to: String::new(),
                reason: String::new(),
            },
            StagingError::CleanupFailed {
                path: String::new(),
                reason: String::new(),
            },
            StagingError::TargetError {
                path: String::new(),
                reason: String::new(),
            },
        ];

        let codes = [
            "WRITER-STAGE-CREATE",
            "WRITER-STAGE-XDEV",
            "WRITER-STAGE-RENAME",
            "WRITER-STAGE-CLEANUP",
            "WRITER-STAGE-TARGET",
        ];

        for (error, expected) in errors.iter().zip(codes.iter()) {
            assert_eq!(error.code(), *expected);
        }
    }

    #[cfg(unix)]
    #[test]
    fn same_filesystem_check_passes() {
        let base = setup("same_fs");
        let staging = StagingDir::create(&base, "run_fs").unwrap();
        let target = base.join("generated");
        let result = check_same_filesystem(staging.path(), &target);
        assert!(result.is_ok());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn multiple_promotes_work() {
        let base = setup("multi_promote");
        let target = base.join("generated");

        let s1 = StagingDir::create(&base, "run_a").unwrap();
        fs::write(s1.path().join("v1.sql"), "V1").unwrap();
        s1.promote(&target).unwrap();
        assert_eq!(fs::read_to_string(target.join("v1.sql")).unwrap(), "V1");

        let s2 = StagingDir::create(&base, "run_b").unwrap();
        fs::write(s2.path().join("v2.sql"), "V2").unwrap();
        s2.promote(&target).unwrap();
        assert!(target.join("v2.sql").exists());
        assert!(!target.join("v1.sql").exists());

        let _ = fs::remove_dir_all(&base);
    }
}
