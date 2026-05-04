use std::path::{Path, PathBuf};

use know_now_metadata::authoring::AuthoringMetadata;
use know_now_metadata::budgets::ParserBudgets;
use know_now_metadata::parser::{self, ParseError};
use know_now_metadata::span::{SourceId, SourceSpanIndex};

#[derive(Debug)]
pub struct LoadedProject {
    pub metadata: AuthoringMetadata,
    pub spans: SourceSpanIndex,
    pub file_count: usize,
}

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Parse(Vec<ParseError>),
    VersionMismatch {
        file_a: PathBuf,
        version_a: String,
        file_b: PathBuf,
        version_b: String,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse(errors) => {
                for (i, err) in errors.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{err}")?;
                }
                Ok(())
            }
            Self::VersionMismatch {
                file_a,
                version_a,
                file_b,
                version_b,
            } => write!(
                f,
                "META-VER-MISMATCH: version '{}' in {} conflicts with version '{}' in {}",
                version_a,
                file_a.display(),
                version_b,
                file_b.display()
            ),
        }
    }
}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// # Errors
/// Returns I/O error if the directory cannot be read.
pub fn discover_metadata_files(metadata_dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    discover_recursive(metadata_dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn discover_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }

        if ft.is_dir() {
            discover_recursive(&path, files)?;
        } else if ft.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy();
                if ext == "yml" || ext == "yaml" {
                    files.push(path);
                }
            }
        }
    }

    Ok(())
}

/// # Errors
/// Returns `LoadError` on I/O failure, parse errors, or version mismatch.
pub fn load_project(
    metadata_dir: &Path,
    budgets: &ParserBudgets,
) -> Result<LoadedProject, LoadError> {
    let files = discover_metadata_files(metadata_dir)?;

    if files.is_empty() {
        return Ok(LoadedProject {
            metadata: empty_metadata(),
            spans: SourceSpanIndex::new(),
            file_count: 0,
        });
    }

    let mut merged = empty_metadata();
    let merged_spans = SourceSpanIndex::new();
    let mut all_errors = Vec::new();
    let mut version_origin: Option<(PathBuf, String)> = None;

    for (idx, file_path) in files.iter().enumerate() {
        let source = std::fs::read_to_string(file_path)?;
        let source_id = SourceId(idx as u32);

        let parsed: parser::ParsedDocument<AuthoringMetadata> =
            match parser::parse_yaml(&source, file_path, source_id, budgets) {
                Ok(doc) => doc,
                Err(errors) => {
                    all_errors.extend(errors);
                    continue;
                }
            };

        let meta = parsed.document;

        if let Some(ref ver) = meta.version {
            if let Some((ref origin_path, ref origin_ver)) = version_origin {
                if ver != origin_ver {
                    return Err(LoadError::VersionMismatch {
                        file_a: origin_path.clone(),
                        version_a: origin_ver.clone(),
                        file_b: file_path.clone(),
                        version_b: ver.clone(),
                    });
                }
            } else {
                version_origin = Some((file_path.clone(), ver.clone()));
            }
        }

        merge_metadata(&mut merged, meta);
    }

    if !all_errors.is_empty() {
        return Err(LoadError::Parse(all_errors));
    }

    if let Some((_, ver)) = version_origin {
        merged.version = Some(ver);
    }

    Ok(LoadedProject {
        metadata: merged,
        spans: merged_spans,
        file_count: files.len(),
    })
}

/// # Errors
/// Returns `LoadError` on parse errors.
pub fn load_single_source(
    source: &str,
    file_path: &Path,
    budgets: &ParserBudgets,
) -> Result<LoadedProject, LoadError> {
    let source_id = SourceId(0);
    let parsed: parser::ParsedDocument<AuthoringMetadata> =
        parser::parse_yaml(source, file_path, source_id, budgets).map_err(LoadError::Parse)?;

    Ok(LoadedProject {
        metadata: parsed.document,
        spans: SourceSpanIndex::new(),
        file_count: 1,
    })
}

fn merge_metadata(target: &mut AuthoringMetadata, source: AuthoringMetadata) {
    if target.project.is_none() {
        target.project = source.project;
    }
    if target.target_database.is_none() {
        target.target_database = source.target_database;
    }
    if target.policy.is_none() {
        target.policy = source.policy;
    }
    if target.governance.is_none() {
        target.governance = source.governance;
    }
    target.domains.extend(source.domains);
    target.modules.extend(source.modules);
    target.entities.extend(source.entities);
    target.relationships.extend(source.relationships);
    target.sources.extend(source.sources);
    target.rules.extend(source.rules);
    target.open_questions.extend(source.open_questions);
    target.assumptions.extend(source.assumptions);
}

fn empty_metadata() -> AuthoringMetadata {
    AuthoringMetadata {
        version: None,
        project: None,
        target_database: None,
        policy: None,
        domains: Vec::new(),
        modules: Vec::new(),
        entities: Vec::new(),
        relationships: Vec::new(),
        sources: Vec::new(),
        rules: Vec::new(),
        governance: None,
        open_questions: Vec::new(),
        assumptions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("know_now_loader_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_yaml(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn discover_sorts_by_path() {
        let dir = temp_dir("sort");
        write_yaml(&dir, "z_entities.yml", "entities: []");
        write_yaml(&dir, "a_domains.yml", "domains: []");
        write_yaml(&dir, "m_sources.yaml", "sources: []");

        let files = discover_metadata_files(&dir).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            names,
            vec!["a_domains.yml", "m_sources.yaml", "z_entities.yml"]
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_skips_hidden_files() {
        let dir = temp_dir("hidden");
        write_yaml(&dir, "visible.yml", "entities: []");
        write_yaml(&dir, ".hidden.yml", "entities: []");

        let files = discover_metadata_files(&dir).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("visible"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_recurses_into_subdirs() {
        let dir = temp_dir("recurse");
        let sub = dir.join("entities");
        fs::create_dir_all(&sub).unwrap();
        write_yaml(&dir, "project.yml", "project:\n  name: test");
        write_yaml(
            &sub,
            "customer.yml",
            "entities:\n  - name: customer\n    attributes: []",
        );

        let files = discover_metadata_files(&dir).unwrap();
        assert_eq!(files.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_skips_non_yaml() {
        let dir = temp_dir("nonyaml");
        write_yaml(&dir, "valid.yml", "entities: []");
        fs::write(dir.join("readme.md"), "# readme").unwrap();
        fs::write(dir.join("data.json"), "{}").unwrap();

        let files = discover_metadata_files(&dir).unwrap();
        assert_eq!(files.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_merges_multiple_files() {
        let dir = temp_dir("merge");
        write_yaml(
            &dir,
            "01_project.yml",
            "version: \"1.0\"\nproject:\n  name: test\ndomains:\n  - id: sales\n    name: Sales",
        );
        write_yaml(
            &dir,
            "02_entities.yml",
            "version: \"1.0\"\nentities:\n  - name: customer\n    attributes:\n      - name: id",
        );

        let result = load_project(&dir, &ParserBudgets::default()).unwrap();
        assert_eq!(result.file_count, 2);
        assert_eq!(result.metadata.project.as_ref().unwrap().name, "test");
        assert_eq!(result.metadata.domains.len(), 1);
        assert_eq!(result.metadata.entities.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_detects_version_mismatch() {
        let dir = temp_dir("vermismatch");
        write_yaml(&dir, "a.yml", "version: \"1.0\"\nentities: []");
        write_yaml(&dir, "b.yml", "version: \"2.0\"\nentities: []");

        let result = load_project(&dir, &ParserBudgets::default());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("META-VER-MISMATCH"),
            "error should contain META-VER-MISMATCH: {msg}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_empty_dir_returns_empty_metadata() {
        let dir = temp_dir("empty");
        let result = load_project(&dir, &ParserBudgets::default()).unwrap();
        assert_eq!(result.file_count, 0);
        assert!(result.metadata.entities.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_same_version_across_files_ok() {
        let dir = temp_dir("samever");
        write_yaml(
            &dir,
            "a.yml",
            "version: \"1.0\"\nentities:\n  - name: a\n    attributes: []",
        );
        write_yaml(
            &dir,
            "b.yml",
            "version: \"1.0\"\nentities:\n  - name: b\n    attributes: []",
        );

        let result = load_project(&dir, &ParserBudgets::default()).unwrap();
        assert_eq!(result.metadata.entities.len(), 2);
        assert_eq!(result.metadata.version, Some("1.0".to_owned()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadata_dir_not_modified() {
        let dir = temp_dir("nomod");
        write_yaml(
            &dir,
            "test.yml",
            "entities:\n  - name: customer\n    attributes:\n      - name: id",
        );

        let before_content = fs::read_to_string(dir.join("test.yml")).unwrap();
        let before_hash = simple_hash(before_content.as_bytes());

        let _ = load_project(&dir, &ParserBudgets::default()).unwrap();

        let after_content = fs::read_to_string(dir.join("test.yml")).unwrap();
        let after_hash = simple_hash(after_content.as_bytes());

        assert_eq!(
            before_hash, after_hash,
            "metadata/ must not be modified by discovery"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    fn simple_hash(data: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &byte in data {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }

    #[test]
    fn discover_deterministic_ordering() {
        let dir = temp_dir("deterministic");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        write_yaml(&dir, "c.yml", "entities: []");
        write_yaml(&dir, "a.yml", "entities: []");
        write_yaml(&sub, "b.yml", "entities: []");

        let run1 = discover_metadata_files(&dir).unwrap();
        let run2 = discover_metadata_files(&dir).unwrap();
        assert_eq!(run1, run2, "discovery must be deterministic");

        let _ = fs::remove_dir_all(&dir);
    }
}
