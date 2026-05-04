use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafePath {
    normalized: PathBuf,
}

impl SafePath {
    pub fn as_path(&self) -> &Path {
        &self.normalized
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PathSafetyError {
    #[error("WRITER-PATH-ABS: absolute path not allowed: {path}")]
    AbsolutePath { path: String },

    #[error("WRITER-PATH-TRAVERSE: path traversal (.. component) not allowed: {path}")]
    Traversal { path: String },

    #[error("WRITER-PATH-ROOT: path escapes approved root: {path}")]
    OutsideRoot { path: String },

    #[error("WRITER-PATH-NULL: embedded NUL byte in path: {path}")]
    NullByte { path: String },

    #[error("WRITER-PATH-NORMAL: path contains backslash separators: {path}")]
    BackslashSeparator { path: String },

    #[error("WRITER-PATH-WINRES: reserved Windows device name: {name}")]
    WindowsReserved { name: String },

    #[error("WRITER-PATH-SYMLINK: symlink escape detected at: {path}")]
    SymlinkEscape { path: String },
}

impl PathSafetyError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::AbsolutePath { .. } => "WRITER-PATH-ABS",
            Self::Traversal { .. } => "WRITER-PATH-TRAVERSE",
            Self::OutsideRoot { .. } => "WRITER-PATH-ROOT",
            Self::NullByte { .. } => "WRITER-PATH-NULL",
            Self::BackslashSeparator { .. } => "WRITER-PATH-NORMAL",
            Self::WindowsReserved { .. } => "WRITER-PATH-WINRES",
            Self::SymlinkEscape { .. } => "WRITER-PATH-SYMLINK",
        }
    }
}

/// # Errors
/// Returns `PathSafetyError` if the path is absolute, contains traversal, null bytes,
/// backslash separators, reserved Windows names, or escapes the approved root.
pub fn validate_artifact_path(raw: &str, root: &Path) -> Result<SafePath, PathSafetyError> {
    if raw.contains('\0') {
        return Err(PathSafetyError::NullByte {
            path: raw.replace('\0', "\\0"),
        });
    }

    if raw.contains('\\') {
        return Err(PathSafetyError::BackslashSeparator {
            path: raw.to_owned(),
        });
    }

    let path = Path::new(raw);

    if path.is_absolute() {
        return Err(PathSafetyError::AbsolutePath {
            path: raw.to_owned(),
        });
    }

    let mut depth: i32 = 0;
    for component in path.components() {
        match component {
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(PathSafetyError::Traversal {
                        path: raw.to_owned(),
                    });
                }
            }
            Component::Normal(_) => {
                depth += 1;
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err(PathSafetyError::AbsolutePath {
                    path: raw.to_owned(),
                });
            }
        }
    }

    check_windows_reserved(raw)?;

    let resolved = root.join(path);
    let normalized = normalize_components(&resolved);
    let root_normalized = normalize_components(root);

    if !normalized.starts_with(&root_normalized) {
        return Err(PathSafetyError::OutsideRoot {
            path: raw.to_owned(),
        });
    }

    Ok(SafePath {
        normalized: PathBuf::from(raw),
    })
}

/// # Errors
/// Returns `PathSafetyError::SymlinkEscape` if the canonicalized path escapes the root.
pub fn check_symlink_escape(full_path: &Path, root: &Path) -> Result<(), PathSafetyError> {
    let canonical_root = root
        .canonicalize()
        .map_err(|_| PathSafetyError::OutsideRoot {
            path: root.display().to_string(),
        })?;

    let canonical = full_path
        .canonicalize()
        .map_err(|_| PathSafetyError::SymlinkEscape {
            path: full_path.display().to_string(),
        })?;

    if !canonical.starts_with(&canonical_root) {
        return Err(PathSafetyError::SymlinkEscape {
            path: full_path.display().to_string(),
        });
    }

    Ok(())
}

fn check_windows_reserved(raw: &str) -> Result<(), PathSafetyError> {
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    for segment in raw.split('/') {
        let stem = segment.split('.').next().unwrap_or(segment);
        let upper = stem.to_ascii_uppercase();
        if RESERVED.contains(&upper.as_str()) {
            return Err(PathSafetyError::WindowsReserved {
                name: segment.to_owned(),
            });
        }
    }

    Ok(())
}

fn normalize_components(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(component),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn root() -> PathBuf {
        PathBuf::from("/project/generated")
    }

    #[test]
    fn valid_relative_path() {
        let result = validate_artifact_path("ddl/postgres/schema.sql", &root());
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().as_path(),
            Path::new("ddl/postgres/schema.sql")
        );
    }

    #[test]
    fn rejects_absolute_path() {
        let err = validate_artifact_path("/etc/passwd", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-ABS");
    }

    #[test]
    fn rejects_traversal() {
        let err = validate_artifact_path("../../etc/passwd", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-TRAVERSE");
    }

    #[test]
    fn rejects_sneaky_traversal() {
        let err = validate_artifact_path("a/b/../../../outside", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-TRAVERSE");
    }

    #[test]
    fn allows_benign_parent_component() {
        let result = validate_artifact_path("a/b/../c/file.sql", &root());
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_null_byte() {
        let err = validate_artifact_path("file\0.sql", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-NULL");
    }

    #[test]
    fn rejects_backslash_separator() {
        let err = validate_artifact_path("ddl\\postgres\\schema.sql", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-NORMAL");
    }

    #[test]
    fn rejects_windows_reserved_con() {
        let err = validate_artifact_path("ddl/CON", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-WINRES");
    }

    #[test]
    fn rejects_windows_reserved_with_extension() {
        let err = validate_artifact_path("ddl/nul.txt", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-WINRES");
    }

    #[test]
    fn rejects_windows_reserved_com1() {
        let err = validate_artifact_path("com1", &root()).unwrap_err();
        assert_eq!(err.code(), "WRITER-PATH-WINRES");
    }

    #[test]
    fn allows_non_reserved_names() {
        let result = validate_artifact_path("console/output.sql", &root());
        assert!(result.is_ok());
    }

    #[test]
    fn error_codes_are_stable() {
        let errors = [
            PathSafetyError::AbsolutePath {
                path: String::new(),
            },
            PathSafetyError::Traversal {
                path: String::new(),
            },
            PathSafetyError::OutsideRoot {
                path: String::new(),
            },
            PathSafetyError::NullByte {
                path: String::new(),
            },
            PathSafetyError::BackslashSeparator {
                path: String::new(),
            },
            PathSafetyError::WindowsReserved {
                name: String::new(),
            },
            PathSafetyError::SymlinkEscape {
                path: String::new(),
            },
        ];

        let codes = [
            "WRITER-PATH-ABS",
            "WRITER-PATH-TRAVERSE",
            "WRITER-PATH-ROOT",
            "WRITER-PATH-NULL",
            "WRITER-PATH-NORMAL",
            "WRITER-PATH-WINRES",
            "WRITER-PATH-SYMLINK",
        ];

        for (error, expected_code) in errors.iter().zip(codes.iter()) {
            assert_eq!(error.code(), *expected_code);
        }
    }

    #[test]
    fn symlink_escape_rejects_outside_root() {
        let temp = std::env::temp_dir().join("know_now_path_safety_test");
        let root_dir = temp.join("root");
        let outside = temp.join("outside");
        let link = root_dir.join("escape_link");

        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.txt"), "secret").unwrap();

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            let target = link.join("secret.txt");
            let result = check_symlink_escape(&target, &root_dir);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code(), "WRITER-PATH-SYMLINK");
        }

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn symlink_within_root_is_ok() {
        let temp = std::env::temp_dir().join("know_now_path_safety_ok");
        let root_dir = temp.join("root");
        let subdir = root_dir.join("subdir");
        let link = root_dir.join("internal_link");

        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&subdir).unwrap();
        std::fs::write(subdir.join("file.txt"), "data").unwrap();

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&subdir, &link).unwrap();
            let target = link.join("file.txt");
            let result = check_symlink_escape(&target, &root_dir);
            assert!(result.is_ok());
        }

        let _ = std::fs::remove_dir_all(&temp);
    }
}
