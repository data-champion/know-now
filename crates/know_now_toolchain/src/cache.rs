use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationCache {
    pub schema_version: String,
    pub metadata_hash: String,
    pub graph_hash: String,
    pub contract_hash: String,
    pub artifacts: BTreeMap<String, CachedArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedArtifact {
    pub generator: String,
    pub input_hash: String,
    pub output_hash: String,
    pub metadata_object_ids: Vec<String>,
}

const CACHE_SCHEMA_VERSION: &str = "1";
const CACHE_FILE: &str = "generation_cache.json";

pub struct CacheStore {
    cache_dir: PathBuf,
}

impl CacheStore {
    pub fn new(knownow_dir: &Path) -> Self {
        Self {
            cache_dir: knownow_dir.join("cache"),
        }
    }

    pub fn load(&self) -> Option<GenerationCache> {
        let path = self.cache_dir.join(CACHE_FILE);
        let content = fs::read_to_string(&path).ok()?;
        let cache: GenerationCache = serde_json::from_str(&content).ok()?;
        if cache.schema_version != CACHE_SCHEMA_VERSION {
            return None;
        }
        Some(cache)
    }

    /// # Errors
    /// Returns an error if the cache directory cannot be created or written.
    pub fn save(&self, cache: &GenerationCache) -> io::Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        let path = self.cache_dir.join(CACHE_FILE);
        let bytes = serde_json::to_vec_pretty(cache)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        let tmp_path = self.cache_dir.join(format!(".{CACHE_FILE}.tmp"));
        fs::write(&tmp_path, bytes)?;
        fs::rename(tmp_path, path)?;
        Ok(())
    }

    /// # Errors
    /// Returns an error if the cache file exists but cannot be removed.
    pub fn invalidate(&self) -> io::Result<()> {
        let path = self.cache_dir.join(CACHE_FILE);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("sha256:{:x}", hasher.finalize())
}

#[derive(Debug)]
pub struct CacheDecision {
    pub skip_artifact_ids: Vec<String>,
    pub reason: CacheHitReason,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CacheHitReason {
    FullHit,
    PartialHit,
    Miss,
}

pub fn evaluate_cache(
    cache: &GenerationCache,
    metadata_hash: &str,
    graph_hash: &str,
    contract_hash: &str,
) -> CacheDecision {
    if cache.metadata_hash != metadata_hash
        || cache.graph_hash != graph_hash
        || cache.contract_hash != contract_hash
    {
        return CacheDecision {
            skip_artifact_ids: vec![],
            reason: CacheHitReason::Miss,
        };
    }

    CacheDecision {
        skip_artifact_ids: cache.artifacts.keys().cloned().collect(),
        reason: CacheHitReason::FullHit,
    }
}

pub fn evaluate_incremental_cache(
    cache: &GenerationCache,
    changed_object_ids: &[String],
) -> CacheDecision {
    let mut skip = Vec::new();

    for (artifact_id, cached) in &cache.artifacts {
        let affected = cached
            .metadata_object_ids
            .iter()
            .any(|id| changed_object_ids.contains(id));
        if !affected {
            skip.push(artifact_id.clone());
        }
    }

    let reason = if skip.len() == cache.artifacts.len() {
        CacheHitReason::FullHit
    } else if skip.is_empty() {
        CacheHitReason::Miss
    } else {
        CacheHitReason::PartialHit
    };

    CacheDecision {
        skip_artifact_ids: skip,
        reason,
    }
}

pub fn build_cache(
    metadata_hash: &str,
    graph_hash: &str,
    contract_hash: &str,
    artifacts: BTreeMap<String, CachedArtifact>,
) -> GenerationCache {
    GenerationCache {
        schema_version: CACHE_SCHEMA_VERSION.into(),
        metadata_hash: metadata_hash.into(),
        graph_hash: graph_hash.into(),
        contract_hash: contract_hash.into(),
        artifacts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cache() -> GenerationCache {
        let mut artifacts = BTreeMap::new();
        artifacts.insert(
            "ddl_postgres_schema".into(),
            CachedArtifact {
                generator: "know_now_gen_postgres".into(),
                input_hash: "sha256:input1".into(),
                output_hash: "sha256:output1".into(),
                metadata_object_ids: vec!["ent_customer".into(), "ent_order".into()],
            },
        );
        artifacts.insert(
            "docs_api_reference".into(),
            CachedArtifact {
                generator: "know_now_gen_docs".into(),
                input_hash: "sha256:input2".into(),
                output_hash: "sha256:output2".into(),
                metadata_object_ids: vec!["ent_customer".into()],
            },
        );
        GenerationCache {
            schema_version: "1".into(),
            metadata_hash: "sha256:meta1".into(),
            graph_hash: "sha256:graph1".into(),
            contract_hash: "sha256:contract1".into(),
            artifacts,
        }
    }

    #[test]
    fn full_cache_hit_when_all_hashes_match() {
        let cache = sample_cache();
        let decision = evaluate_cache(&cache, "sha256:meta1", "sha256:graph1", "sha256:contract1");
        assert_eq!(decision.reason, CacheHitReason::FullHit);
        assert_eq!(decision.skip_artifact_ids.len(), 2);
    }

    #[test]
    fn cache_miss_on_metadata_change() {
        let cache = sample_cache();
        let decision =
            evaluate_cache(&cache, "sha256:meta2", "sha256:graph1", "sha256:contract1");
        assert_eq!(decision.reason, CacheHitReason::Miss);
        assert!(decision.skip_artifact_ids.is_empty());
    }

    #[test]
    fn cache_miss_on_graph_change() {
        let cache = sample_cache();
        let decision =
            evaluate_cache(&cache, "sha256:meta1", "sha256:graph2", "sha256:contract1");
        assert_eq!(decision.reason, CacheHitReason::Miss);
    }

    #[test]
    fn cache_miss_on_contract_change() {
        let cache = sample_cache();
        let decision =
            evaluate_cache(&cache, "sha256:meta1", "sha256:graph1", "sha256:contract2");
        assert_eq!(decision.reason, CacheHitReason::Miss);
    }

    #[test]
    fn incremental_skips_unaffected_artifacts() {
        let cache = sample_cache();
        let changed = vec!["ent_order".to_owned()];
        let decision = evaluate_incremental_cache(&cache, &changed);
        assert_eq!(decision.reason, CacheHitReason::PartialHit);
        assert!(decision.skip_artifact_ids.contains(&"docs_api_reference".to_owned()));
        assert!(!decision.skip_artifact_ids.contains(&"ddl_postgres_schema".to_owned()));
    }

    #[test]
    fn incremental_full_hit_when_no_changes() {
        let cache = sample_cache();
        let changed: Vec<String> = vec!["ent_unrelated".into()];
        let decision = evaluate_incremental_cache(&cache, &changed);
        assert_eq!(decision.reason, CacheHitReason::FullHit);
    }

    #[test]
    fn incremental_miss_when_all_affected() {
        let cache = sample_cache();
        let changed = vec!["ent_customer".to_owned()];
        let decision = evaluate_incremental_cache(&cache, &changed);
        assert_eq!(decision.reason, CacheHitReason::Miss);
    }

    #[test]
    fn sha256_bytes_produces_hex() {
        let hash = sha256_bytes(b"hello world");
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), "sha256:".len() + 64);
    }

    #[test]
    fn cache_store_roundtrip() {
        let tmp = std::env::temp_dir().join(format!(
            "know_now_cache_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let store = CacheStore::new(&tmp);

        assert!(store.load().is_none());

        let cache = sample_cache();
        store.save(&cache).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.metadata_hash, "sha256:meta1");
        assert_eq!(loaded.artifacts.len(), 2);

        store.invalidate().unwrap();
        assert!(store.load().is_none());

        std::fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn build_cache_creates_valid_structure() {
        let artifacts = BTreeMap::new();
        let cache = build_cache("sha256:m", "sha256:g", "sha256:c", artifacts);
        assert_eq!(cache.schema_version, "1");
        assert_eq!(cache.metadata_hash, "sha256:m");
    }

    #[test]
    fn invalid_schema_version_returns_none() {
        let tmp = std::env::temp_dir().join(format!(
            "know_now_cache_schema_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let cache_dir = tmp.join("cache");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(
            cache_dir.join("generation_cache.json"),
            r#"{"schema_version":"999","metadata_hash":"","graph_hash":"","contract_hash":"","artifacts":{}}"#,
        ).unwrap();

        let store = CacheStore::new(&tmp);
        assert!(store.load().is_none());

        std::fs::remove_dir_all(tmp).unwrap();
    }
}
