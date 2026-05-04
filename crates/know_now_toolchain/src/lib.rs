//! External toolchain adapter crate for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

pub mod dbt;
pub mod project_lock;

use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const KNOWNOW_DIR: &str = ".knownow";
const RUNS_DIR: &str = "runs";
const LOCKS_DIR: &str = "locks";
const LAST_GENERATION_FILE: &str = "last_generation.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunRecord {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub command: String,
    pub result: RunResult,
    pub manifest_hash: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunResult {
    Success,
    Failure,
}

/// Maintains volatile runtime state under `.knownow/`.
///
/// Deterministic artifacts must not depend on this state. See PRD §8.11 and NFR-R13.
#[derive(Debug, Clone)]
pub struct VolatileStateStore {
    knownow_dir: PathBuf,
    runs_dir: PathBuf,
    locks_dir: PathBuf,
    max_runs: usize,
}

impl VolatileStateStore {
    /// Create (or attach to) a volatile state store under `<project_root>/.knownow/`.
    ///
    /// # Errors
    ///
    /// Returns an error when `max_runs` is zero or required directories cannot be created.
    pub fn new(project_root: impl AsRef<Path>, max_runs: usize) -> io::Result<Self> {
        if max_runs == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "max_runs must be greater than zero",
            ));
        }

        let project_root = project_root.as_ref();
        let knownow_dir = project_root.join(KNOWNOW_DIR);
        let runs_dir = knownow_dir.join(RUNS_DIR);
        let locks_dir = knownow_dir.join(LOCKS_DIR);

        fs::create_dir_all(&runs_dir)?;
        fs::create_dir_all(&locks_dir)?;

        Ok(Self {
            knownow_dir,
            runs_dir,
            locks_dir,
            max_runs,
        })
    }

    pub fn knownow_dir(&self) -> &Path {
        &self.knownow_dir
    }

    pub fn runs_dir(&self) -> &Path {
        &self.runs_dir
    }

    pub fn locks_dir(&self) -> &Path {
        &self.locks_dir
    }

    pub fn last_generation_path(&self) -> PathBuf {
        self.knownow_dir.join(LAST_GENERATION_FILE)
    }

    /// Persist one run record and update `.knownow/last_generation.json`.
    ///
    /// Run-pruning occurs only after writes succeed to keep cleanup crash-safe.
    ///
    /// # Errors
    ///
    /// Returns an error when the run ID is invalid, write operations fail, or pruning fails.
    pub fn persist_run(&self, record: &RunRecord) -> io::Result<()> {
        Self::validate_run_id(&record.run_id)?;

        let run_path = self.runs_dir.join(format!("{}.json", record.run_id));
        write_json_atomic(&run_path, record)?;
        write_json_atomic(&self.last_generation_path(), record)?;

        self.prune_old_runs()
    }

    fn validate_run_id(run_id: &str) -> io::Result<()> {
        if run_id.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "run_id must not be empty",
            ));
        }

        if run_id.contains('/') || run_id.contains('\\') || run_id == "." || run_id == ".." {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "run_id contains invalid path characters",
            ));
        }

        Ok(())
    }

    fn prune_old_runs(&self) -> io::Result<()> {
        let mut run_files = Vec::new();
        for entry in fs::read_dir(&self.runs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension() != Some(OsStr::new("json")) {
                continue;
            }

            run_files.push(path);
        }

        if run_files.len() <= self.max_runs {
            return Ok(());
        }

        // Run IDs are expected to be sortable; keep newest lexical entries.
        run_files.sort();
        let delete_count = run_files.len() - self.max_runs;

        for path in run_files.into_iter().take(delete_count) {
            fs::remove_file(path)?;
        }

        Ok(())
    }
}

fn write_json_atomic(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path has no parent: {}", path.display()),
        )
    })?;
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid target file name"))?;
    let tmp_path = parent.join(format!(".{file_name}.tmp"));

    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    fs::write(&tmp_path, bytes)?;
    fs::rename(tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project_root(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "know_now_toolchain_{test_name}_{}_{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn sample_run(run_id: &str) -> RunRecord {
        RunRecord {
            run_id: run_id.to_string(),
            started_at: "2026-05-02T12:00:00Z".to_string(),
            finished_at: "2026-05-02T12:00:04Z".to_string(),
            command: "generate --locked".to_string(),
            result: RunResult::Success,
            manifest_hash: "sha256:abc123".to_string(),
            duration_ms: 4210,
        }
    }

    #[test]
    fn creates_knownow_runs_and_locks_directories() {
        let root = temp_project_root("creates_dirs");
        let store = VolatileStateStore::new(&root, 50).expect("store should initialize");

        assert!(store.knownow_dir().is_dir());
        assert!(store.runs_dir().is_dir());
        assert!(store.locks_dir().is_dir());

        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    #[test]
    fn persists_run_and_updates_last_generation() {
        let root = temp_project_root("persist_run");
        let store = VolatileStateStore::new(&root, 50).expect("store should initialize");

        let run = sample_run("run_20260502_120000_abc123");
        store.persist_run(&run).expect("persist should succeed");

        let run_file = store.runs_dir().join("run_20260502_120000_abc123.json");
        assert!(run_file.exists());
        assert!(store.last_generation_path().exists());

        let last_generation = fs::read_to_string(store.last_generation_path())
            .expect("last generation file should be readable");
        assert!(last_generation.contains("run_20260502_120000_abc123"));
        assert!(last_generation.contains("duration_ms"));

        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    #[test]
    fn keeps_only_configured_number_of_run_files() {
        let root = temp_project_root("retention");
        let store = VolatileStateStore::new(&root, 2).expect("store should initialize");

        store
            .persist_run(&sample_run("run_20260502_120000_aaa111"))
            .expect("persist run 1 should succeed");
        store
            .persist_run(&sample_run("run_20260502_120001_bbb222"))
            .expect("persist run 2 should succeed");
        store
            .persist_run(&sample_run("run_20260502_120002_ccc333"))
            .expect("persist run 3 should succeed");

        let mut run_files = fs::read_dir(store.runs_dir())
            .expect("runs dir should be readable")
            .map(|entry| {
                entry
                    .expect("entry should be readable")
                    .file_name()
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<_>>();

        run_files.sort();
        assert_eq!(run_files.len(), 2);
        assert!(run_files.iter().any(|name| name.contains("bbb222")));
        assert!(run_files.iter().any(|name| name.contains("ccc333")));

        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    #[test]
    fn run_id_rejects_path_traversal() {
        let root = temp_project_root("path_safety");
        let store = VolatileStateStore::new(&root, 10).expect("store should initialize");

        let mut run = sample_run("../evil");
        let err = store
            .persist_run(&run)
            .expect_err("path traversal run_id must fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

        run.run_id = "run_20260502_120000_safe".to_string();
        store
            .persist_run(&run)
            .expect("safe run_id should still succeed");

        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    #[test]
    fn volatile_state_does_not_touch_generated_manifest() {
        let root = temp_project_root("deterministic_boundary");
        let generated_dir = root.join("generated");
        fs::create_dir_all(&generated_dir).expect("generated dir should exist");

        let manifest_path = generated_dir.join("manifest.json");
        let manifest_seed = "{\"engine_version\":\"1.0.0\"}";
        fs::write(&manifest_path, manifest_seed).expect("seed manifest should be writable");

        let store = VolatileStateStore::new(&root, 10).expect("store should initialize");
        store
            .persist_run(&sample_run("run_20260502_120000_boundary"))
            .expect("persist should succeed");

        let manifest_after =
            fs::read_to_string(&manifest_path).expect("manifest should remain readable");
        assert_eq!(manifest_after, manifest_seed);

        fs::remove_dir_all(root).expect("cleanup should succeed");
    }
}
