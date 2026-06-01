use std::fs;
use std::path::{Path, PathBuf};

use dashmap::DashMap;
use serde_json::Value;
use uuid::Uuid;

use crate::error::GraphStoreError;
use crate::types::Triple;

/// In-memory triple store backed by content-addressed files.
pub struct TripleStore {
    root: PathBuf,
    /// Key: (subject, predicate, object) → matching triples
    index: DashMap<(String, String, String), Vec<Triple>>,
    /// Secondary index: subject → all triples with that subject
    by_subject: DashMap<String, Vec<Triple>>,
}

impl TripleStore {
    /// Load all `.json` files from the `triples/` directory, building
    /// in-memory indices. Creates the directory if it doesn't exist.
    /// Gracefully skips corrupted files.
    pub fn load(root: PathBuf) -> Result<Self, GraphStoreError> {
        let index: DashMap<(String, String, String), Vec<Triple>> = DashMap::new();
        let by_subject: DashMap<String, Vec<Triple>> = DashMap::new();

        if !root.exists() {
            fs::create_dir_all(&root)
                .map_err(|e| GraphStoreError::Io(root.clone(), e))?;
        }

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Triple>(&content) {
                        Ok(triple) => {
                            Self::index_triple(&index, &by_subject, triple);
                        }
                        Err(e) => {
                            tracing::warn!("Skipping corrupted triple file {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Cannot read triple file {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(Self { root, index, by_subject })
    }

    /// Insert a new triple. Writes to disk atomically (write-then-rename)
    /// and updates both in-memory indices.
    pub fn insert(&self, triple: Triple) -> Result<(), GraphStoreError> {
        let filename = format!("{}.json", &triple.uuid);
        let final_path = self.root.join(&filename);
        let tmp_path = self.root.join(format!(".tmp-{}", filename));

        let content = serde_json::to_string_pretty(&triple)?;

        fs::write(&tmp_path, &content)
            .map_err(|e| GraphStoreError::Io(tmp_path.clone(), e))?;

        fs::rename(&tmp_path, &final_path)
            .map_err(|e| GraphStoreError::Io(final_path.clone(), e))?;

        Self::index_triple(&self.index, &self.by_subject, triple);
        Ok(())
    }

    /// Query by subject, predicate, and/or object.
    /// Passing `None` acts as a wildcard.
    pub fn query(
        &self,
        subject: Option<&str>,
        predicate: Option<&str>,
        object: Option<&str>,
    ) -> Vec<Triple> {
        // Fast path: all three specified → direct index lookup
        if let (Some(s), Some(p), Some(o)) = (subject, predicate, object) {
            return self.index
                .get(&(s.to_string(), p.to_string(), o.to_string()))
                .map(|v| v.value().clone())
                .unwrap_or_default();
        }

        // If only subject is specified, use the secondary index
        if let (Some(s), None, None) = (subject, predicate, object) {
            return self.by_subject
                .get(s)
                .map(|v| v.value().clone())
                .unwrap_or_default();
        }

        // Otherwise scan the full primary index
        let mut results = Vec::new();
        for entry in self.index.iter() {
            let (k, v) = entry.pair();
            let (s, p, o) = k;
            if subject.map_or(true, |q| s == q)
                && predicate.map_or(true, |q| p == q)
                && object.map_or(true, |q| o == q)
            {
                results.extend(v.iter().cloned());
            }
        }
        results
    }

    /// Return total number of indexed triples.
    pub fn len(&self) -> usize {
        self.index.iter().map(|e| e.value().len()).sum()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return all triples as a flat vector.
    pub fn all(&self) -> Vec<Triple> {
        self.index
            .iter()
            .flat_map(|e| e.value().clone())
            .collect()
    }

    /// Insert a triple into both in-memory indices.
    fn index_triple(
        index: &DashMap<(String, String, String), Vec<Triple>>,
        by_subject: &DashMap<String, Vec<Triple>>,
        triple: Triple,
    ) {
        let key = (
            triple.subject.clone(),
            triple.predicate.clone(),
            triple.object.clone(),
        );
        index.entry(key).or_default().push(triple.clone());
        by_subject
            .entry(triple.subject.clone())
            .or_default()
            .push(triple);
    }
}

/// Index all commit stubs in `.Orqestra/graph/commits/` into triples.
///
/// Generated triple patterns:
/// - `(commit_hash, "has_intent", intent_summary)`
/// - `(commit_hash, "affects_concept", concept)` per concept
/// - `(commit_hash, "affects_task", task_id)` per task
/// - `(commit_hash, "has_trace", trace_id)` if present
/// - `(commit_hash, "has_author", author_name)`
/// - `(task_id, "changed_by", commit_hash)` per task
/// - `(concept, "touched_in", commit_hash)` per concept
pub fn index_commits(
    store: &TripleStore,
    commits_dir: &Path,
) -> Result<usize, GraphStoreError> {
    if !commits_dir.exists() {
        return Err(GraphStoreError::NoCommitsDir(commits_dir.to_path_buf()));
    }

    let mut count = 0;
    let entries = fs::read_dir(commits_dir)
        .map_err(|e| GraphStoreError::Io(commits_dir.to_path_buf(), e))?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| GraphStoreError::Io(path.clone(), e))?;

        let stub: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Skipping corrupted commit stub {:?}: {}", path, e);
                continue;
            }
        };

        let hash = stub["hash"].as_str().unwrap_or("unknown").to_string();
        let timestamp = stub["timestamp"].as_str().unwrap_or("").to_string();
        let author = stub["author"]["name"].as_str().unwrap_or("unknown").to_string();

        let semantic = &stub["semantic"];
        let intent = semantic["intent_summary"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let confidence = semantic["confidence"].as_f64().unwrap_or(0.0);

        // (commit_hash, "has_intent", intent_summary)
        if !intent.is_empty() {
            store.insert(Triple {
                uuid: Uuid::new_v4().to_string(),
                subject: hash.clone(),
                predicate: "has_intent".into(),
                object: intent,
                commit: Some(hash.clone()),
                timestamp: Some(timestamp.clone()),
            })?;
            count += 1;
        }

        // (commit_hash, "has_author", author)
        store.insert(Triple {
            uuid: Uuid::new_v4().to_string(),
            subject: hash.clone(),
            predicate: "has_author".into(),
            object: author,
            commit: Some(hash.clone()),
            timestamp: Some(timestamp.clone()),
        })?;
        count += 1;

        // (commit_hash, "has_confidence", score)
        store.insert(Triple {
            uuid: Uuid::new_v4().to_string(),
            subject: hash.clone(),
            predicate: "has_confidence".into(),
            object: format!("{:.2}", confidence),
            commit: Some(hash.clone()),
            timestamp: Some(timestamp.clone()),
        })?;
        count += 1;

        // affected_concepts → (commit, "affects_concept", concept)
        if let Some(concepts) = semantic["affected_concepts"].as_array() {
            for c in concepts {
                if let Some(concept) = c.as_str() {
                    store.insert(Triple {
                        uuid: Uuid::new_v4().to_string(),
                        subject: hash.clone(),
                        predicate: "affects_concept".into(),
                        object: concept.to_string(),
                        commit: Some(hash.clone()),
                        timestamp: Some(timestamp.clone()),
                    })?;
                    count += 1;

                    // Reverse: (concept, "touched_in", commit)
                    store.insert(Triple {
                        uuid: Uuid::new_v4().to_string(),
                        subject: concept.to_string(),
                        predicate: "touched_in".into(),
                        object: hash.clone(),
                        commit: Some(hash.clone()),
                        timestamp: Some(timestamp.clone()),
                    })?;
                    count += 1;
                }
            }
        }

        // task_ids → (commit, "affects_task", task) + reverse
        if let Some(tasks) = semantic["task_ids"].as_array() {
            for t in tasks {
                if let Some(task) = t.as_str() {
                    store.insert(Triple {
                        uuid: Uuid::new_v4().to_string(),
                        subject: hash.clone(),
                        predicate: "affects_task".into(),
                        object: task.to_string(),
                        commit: Some(hash.clone()),
                        timestamp: Some(timestamp.clone()),
                    })?;
                    count += 1;

                    store.insert(Triple {
                        uuid: Uuid::new_v4().to_string(),
                        subject: task.to_string(),
                        predicate: "changed_by".into(),
                        object: hash.clone(),
                        commit: Some(hash.clone()),
                        timestamp: Some(timestamp.clone()),
                    })?;
                    count += 1;
                }
            }
        }

        // reasoning_trace_id → (commit, "has_trace", trace_id)
        if let Some(trace_id) = semantic["reasoning_trace_id"].as_str() {
            store.insert(Triple {
                uuid: Uuid::new_v4().to_string(),
                subject: hash.clone(),
                predicate: "has_trace".into(),
                object: trace_id.to_string(),
                commit: Some(hash.clone()),
                timestamp: Some(timestamp.clone()),
            })?;
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_empty_dir() {
        let dir = TempDir::new().unwrap();
        let store = TripleStore::load(dir.path().to_path_buf()).unwrap();
        assert!(store.is_empty());
    }

    #[test]
    fn insert_and_query() {
        let dir = TempDir::new().unwrap();
        let store = TripleStore::load(dir.path().to_path_buf()).unwrap();

        let triple = Triple {
            uuid: Uuid::new_v4().to_string(),
            subject: "TASK-2026-042".into(),
            predicate: "implements".into(),
            object: "ADR-011".into(),
            commit: Some("a1b2c3d".into()),
            timestamp: Some("2026-06-01T14:30:00Z".into()),
        };

        store.insert(triple.clone()).unwrap();
        assert_eq!(store.len(), 1);

        // Exact match
        let results = store.query(Some("TASK-2026-042"), Some("implements"), Some("ADR-011"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], triple);

        // Wildcard subject
        let results = store.query(None, Some("implements"), Some("ADR-011"));
        assert_eq!(results.len(), 1);

        // Wildcard all
        let results = store.query(None, None, None);
        assert_eq!(results.len(), 1);

        // No match
        let results = store.query(Some("nonexistent"), None, None);
        assert!(results.is_empty());
    }

    #[test]
    fn query_by_subject_secondary_index() {
        let dir = TempDir::new().unwrap();
        let store = TripleStore::load(dir.path().to_path_buf()).unwrap();

        for i in 0..5 {
            store.insert(Triple {
                uuid: Uuid::new_v4().to_string(),
                subject: "commit-abc".into(),
                predicate: format!("pred-{}", i),
                object: format!("obj-{}", i),
                commit: None,
                timestamp: None,
            }).unwrap();
        }

        let results = store.query(Some("commit-abc"), None, None);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn atomic_write_survives_rename() {
        let dir = TempDir::new().unwrap();
        let store = TripleStore::load(dir.path().to_path_buf()).unwrap();

        store.insert(Triple {
            uuid: "test-uuid-123".into(),
            subject: "s".into(),
            predicate: "p".into(),
            object: "o".into(),
            commit: None,
            timestamp: None,
        }).unwrap();

        // Verify file exists on disk
        let file_path = dir.path().join("test-uuid-123.json");
        assert!(file_path.exists());

        // Verify no tmp file leaked
        let tmp_path = dir.path().join(".tmp-test-uuid-123.json");
        assert!(!tmp_path.exists());
    }

    #[test]
    fn reload_preserves_data() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();

        {
            let store = TripleStore::load(path.clone()).unwrap();
            store.insert(Triple {
                uuid: "persist-test".into(),
                subject: "sub".into(),
                predicate: "pred".into(),
                object: "obj".into(),
                commit: None,
                timestamp: None,
            }).unwrap();
            assert_eq!(store.len(), 1);
        }

        // Reload from same directory
        let store2 = TripleStore::load(path).unwrap();
        assert_eq!(store2.len(), 1);
        let results = store2.query(Some("sub"), None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uuid, "persist-test");
    }

    #[test]
    fn skip_corrupted_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();

        // Write a valid triple
        let valid = Triple {
            uuid: "valid-1".into(),
            subject: "s".into(),
            predicate: "p".into(),
            object: "o".into(),
            commit: None,
            timestamp: None,
        };
        fs::write(path.join("valid-1.json"), serde_json::to_string(&valid).unwrap()).unwrap();

        // Write a corrupted file
        fs::write(path.join("bad.json"), "not json{{{").unwrap();

        let store = TripleStore::load(path).unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn index_commits_generates_triples() {
        let dir = TempDir::new().unwrap();
        let triples_dir = dir.path().join("triples");
        let commits_dir = dir.path().join("commits");
        fs::create_dir_all(&commits_dir).unwrap();

        // Write a commit stub
        let stub = serde_json::json!({
            "hash": "abc123",
            "parent_hashes": [],
            "author": { "name": "alice", "type": "human" },
            "timestamp": "2026-06-01T12:00:00Z",
            "conventional_message": "feat(auth): add rate limiting",
            "semantic": {
                "status": "complete",
                "intent_summary": "Add rate limiting to auth endpoints",
                "affected_concepts": ["rate limiting", "authentication"],
                "affected_apis": [],
                "risk_assessment": {
                    "breaking_change": false,
                    "migration_required": null,
                    "rollback_complexity": "low"
                },
                "confidence": 0.95,
                "reasoning_trace_id": "trace-001",
                "task_ids": ["TASK-2026-050"]
            }
        });
        fs::write(commits_dir.join("abc123.json"), serde_json::to_string_pretty(&stub).unwrap()).unwrap();

        let store = TripleStore::load(triples_dir).unwrap();
        let count = index_commits(&store, &commits_dir).unwrap();

        // Expected triples:
        // 1 has_intent, 1 has_author, 1 has_confidence,
        // 2 affects_concept + 2 touched_in, 1 affects_task + 1 changed_by,
        // 1 has_trace = 10
        assert_eq!(count, 10);
        assert_eq!(store.len(), 10);

        // Verify specific triples
        let intent = store.query(Some("abc123"), Some("has_intent"), None);
        assert_eq!(intent.len(), 1);
        assert_eq!(intent[0].object, "Add rate limiting to auth endpoints");

        let concepts = store.query(Some("abc123"), Some("affects_concept"), None);
        assert_eq!(concepts.len(), 2);

        let tasks = store.query(Some("abc123"), Some("affects_task"), None);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].object, "TASK-2026-050");

        // Reverse lookup: task → commit
        let changed_by = store.query(Some("TASK-2026-050"), Some("changed_by"), None);
        assert_eq!(changed_by.len(), 1);
        assert_eq!(changed_by[0].object, "abc123");

        // Concept → commit
        let touched = store.query(Some("rate limiting"), Some("touched_in"), None);
        assert_eq!(touched.len(), 1);
        assert_eq!(touched[0].object, "abc123");

        // Trace
        let trace = store.query(Some("abc123"), Some("has_trace"), None);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].object, "trace-001");
    }
}
