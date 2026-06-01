use loro::{ExportMode, LoroDoc, LoroValue, ValueOrContainer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("CRDT error: {0}")]
    Crdt(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Document not found: {0}")]
    NotFound(String),
}

impl From<loro::LoroError> for EngineError {
    fn from(e: loro::LoroError) -> Self {
        EngineError::Crdt(e.to_string())
    }
}

/// A field within a task CRDT document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskField {
    pub key: String,
    pub value: String,
}

/// Manages one LoroDoc per roadmap/*.md file.
/// Each client gets a unique peer ID for conflict-free merging.
pub struct LoroEngine {
    peer_id: u64,
    docs: HashMap<String, LoroDoc>,
    snapshot_dir: PathBuf,
}

impl LoroEngine {
    /// Create a new engine with a random peer ID.
    pub fn new(snapshot_dir: impl AsRef<Path>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let peer_id = ts ^ simple_hash();

        Self {
            peer_id,
            docs: HashMap::new(),
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
        }
    }

    /// Create engine with a specific peer ID (for testing determinism).
    pub fn with_peer_id(snapshot_dir: impl AsRef<Path>, peer_id: u64) -> Self {
        Self {
            peer_id,
            docs: HashMap::new(),
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
        }
    }

    pub fn peer_id(&self) -> u64 {
        self.peer_id
    }

    /// Open (or create) a CRDT document for a task file.
    pub fn open_doc(&mut self, file_path: &str) -> Result<(), EngineError> {
        if self.docs.contains_key(file_path) {
            return Ok(());
        }

        let doc = LoroDoc::new();
        doc.set_peer_id(self.peer_id).map_err(|e| EngineError::Crdt(e.to_string()))?;

        // Load existing snapshot if available
        let snapshot_path = self.snapshot_path(file_path);
        if snapshot_path.exists() {
            let data = std::fs::read(&snapshot_path)?;
            if !data.is_empty() {
                doc.import(&data).map_err(|e| EngineError::Crdt(e.to_string()))?;
            }
        }

        self.docs.insert(file_path.to_string(), doc);
        Ok(())
    }

    /// Set a string field on the task map.
    pub fn set_field(&self, file_path: &str, key: &str, value: &str) -> Result<(), EngineError> {
        let doc = self.docs.get(file_path)
            .ok_or_else(|| EngineError::NotFound(file_path.to_string()))?;

        let root = doc.get_map("task");
        root.insert(key, value).map_err(|e| EngineError::Crdt(e.to_string()))?;
        doc.commit();
        Ok(())
    }

    /// Get a string field from the task map.
    pub fn get_field(&self, file_path: &str, key: &str) -> Result<String, EngineError> {
        let doc = self.docs.get(file_path)
            .ok_or_else(|| EngineError::NotFound(file_path.to_string()))?;

        let root = doc.get_map("task");
        match root.get(key) {
            Some(ValueOrContainer::Value(LoroValue::String(s))) => Ok(s.to_string()),
            _ => Ok(String::new()),
        }
    }

    /// Get all fields as key-value pairs.
    pub fn get_all_fields(&self, file_path: &str) -> Result<Vec<TaskField>, EngineError> {
        let doc = self.docs.get(file_path)
            .ok_or_else(|| EngineError::NotFound(file_path.to_string()))?;

        let root = doc.get_map("task");
        let value = root.get_value();

        let mut fields = Vec::new();
        if let LoroValue::Map(map) = value {
            for (k, v) in map.iter() {
                let s = match v {
                    LoroValue::String(s) => s.to_string(),
                    LoroValue::Bool(b) => b.to_string(),
                    LoroValue::Double(n) => n.to_string(),
                    LoroValue::I64(n) => n.to_string(),
                    LoroValue::Null => String::new(),
                    other => format!("{:?}", other),
                };
                fields.push(TaskField { key: k.to_string(), value: s });
            }
        }

        Ok(fields)
    }

    /// Export all updates (full delta) for a document.
    pub fn export_delta(&self, file_path: &str) -> Result<Vec<u8>, EngineError> {
        let doc = self.docs.get(file_path)
            .ok_or_else(|| EngineError::NotFound(file_path.to_string()))?;
        doc.export(ExportMode::all_updates())
            .map_err(|e| EngineError::Crdt(e.to_string()))
    }

    /// Import remote updates and merge.
    pub fn import_delta(&self, file_path: &str, data: &[u8]) -> Result<(), EngineError> {
        let doc = self.docs.get(file_path)
            .ok_or_else(|| EngineError::NotFound(file_path.to_string()))?;
        doc.import(data).map_err(|e| EngineError::Crdt(e.to_string()))?;
        doc.commit();
        Ok(())
    }

    /// Import from snapshot bytes.
    pub fn import_snapshot(&self, file_path: &str, data: &[u8]) -> Result<(), EngineError> {
        let doc = self.docs.get(file_path)
            .ok_or_else(|| EngineError::NotFound(file_path.to_string()))?;
        doc.import(data).map_err(|e| EngineError::Crdt(e.to_string()))?;
        doc.commit();
        Ok(())
    }

    /// Persist a snapshot to disk (atomic write).
    pub fn save_snapshot(&self, file_path: &str) -> Result<(), EngineError> {
        let doc = self.docs.get(file_path)
            .ok_or_else(|| EngineError::NotFound(file_path.to_string()))?;

        let snapshot = doc.export(ExportMode::Snapshot)
            .map_err(|e| EngineError::Crdt(e.to_string()))?;
        let path = self.snapshot_path(file_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &snapshot)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// Load content from a markdown file into a CRDT document.
    pub fn load_from_markdown(&mut self, file_path: &str, content: &str) -> Result<(), EngineError> {
        self.open_doc(file_path)?;

        if let Some(frontmatter) = extract_frontmatter(content) {
            let fields: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&frontmatter)?;
            for (key, val) in fields.iter() {
                let value_str = yaml_value_to_string(val);
                self.set_field(file_path, key, &value_str)?;
            }
        }

        if let Some(body) = extract_body(content) {
            self.set_field(file_path, "body", body.trim())?;
        }

        Ok(())
    }

    /// Export current state to a markdown string.
    pub fn export_to_markdown(&self, file_path: &str) -> Result<String, EngineError> {
        let fields = self.get_all_fields(file_path)?;
        let mut md = String::from("---\n");
        for f in &fields {
            if f.key == "body" { continue; }
            // Quote strings that contain special chars
            if f.value.contains(':') || f.value.contains('#') || f.value.contains('\n') {
                md.push_str(&format!("{}: \"{}\"\n", f.key, f.value.replace('"', "\\\"")));
            } else {
                md.push_str(&format!("{}: {}\n", f.key, f.value));
            }
        }
        md.push_str("---\n");

        if let Ok(body) = self.get_field(file_path, "body") {
            if !body.is_empty() {
                md.push_str(&body);
            }
        }

        Ok(md)
    }

    /// List open document paths.
    pub fn open_docs(&self) -> Vec<String> {
        self.docs.keys().cloned().collect()
    }

    fn snapshot_path(&self, file_path: &str) -> PathBuf {
        let safe_name = file_path.replace('/', "_").replace('\\', "_");
        self.snapshot_dir.join(format!("{}.loro", safe_name))
    }
}

fn simple_hash() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::thread::current().id().hash(&mut h);
    h.finish()
}

fn yaml_value_to_string(val: &serde_yaml::Value) -> String {
    match val {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => String::new(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

fn extract_frontmatter(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return None; }
    let rest = &trimmed[3..];
    let end = rest.find("---")?;
    Some(rest[..end].trim().to_string())
}

fn extract_body(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return None; }
    let rest = &trimmed[3..];
    let end = rest.find("---")?;
    Some(rest[end + 3..].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_engine(peer_id: u64) -> (TempDir, LoroEngine) {
        let dir = TempDir::new().unwrap();
        let engine = LoroEngine::with_peer_id(dir.path(), peer_id);
        (dir, engine)
    }

    #[test]
    fn test_set_and_get_field() {
        let (_dir, mut engine) = make_engine(1);
        engine.open_doc("roadmap/TASK-001.md").unwrap();
        engine.set_field("roadmap/TASK-001.md", "title", "Build auth").unwrap();

        let val = engine.get_field("roadmap/TASK-001.md", "title").unwrap();
        assert_eq!(val, "Build auth");
    }

    #[test]
    fn test_get_all_fields() {
        let (_dir, mut engine) = make_engine(1);
        engine.open_doc("roadmap/TASK-001.md").unwrap();
        engine.set_field("roadmap/TASK-001.md", "title", "Build auth").unwrap();
        engine.set_field("roadmap/TASK-001.md", "status", "in-progress").unwrap();
        engine.set_field("roadmap/TASK-001.md", "priority", "high").unwrap();

        let fields = engine.get_all_fields("roadmap/TASK-001.md").unwrap();
        assert!(fields.len() >= 3);

        let title = fields.iter().find(|f| f.key == "title").unwrap();
        assert_eq!(title.value, "Build auth");
    }

    #[test]
    fn test_snapshot_persistence() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();

        {
            let mut engine = LoroEngine::with_peer_id(&path, 1);
            engine.open_doc("roadmap/TASK-001.md").unwrap();
            engine.set_field("roadmap/TASK-001.md", "title", "Persist test").unwrap();
            engine.save_snapshot("roadmap/TASK-001.md").unwrap();
        }

        {
            let mut engine = LoroEngine::with_peer_id(&path, 99);
            engine.open_doc("roadmap/TASK-001.md").unwrap();
            let val = engine.get_field("roadmap/TASK-001.md", "title").unwrap();
            assert_eq!(val, "Persist test");
        }
    }

    #[test]
    fn test_two_peer_merge_no_data_loss() {
        let (_dir_a, mut peer_a) = make_engine(1);
        let (_dir_b, mut peer_b) = make_engine(2);
        let file = "roadmap/TASK-001.md";

        peer_a.open_doc(file).unwrap();
        peer_b.open_doc(file).unwrap();

        // Peer A edits title + status
        peer_a.set_field(file, "title", "Auth module v2").unwrap();
        peer_a.set_field(file, "status", "in-progress").unwrap();

        // Peer B edits title + priority (concurrent)
        peer_b.set_field(file, "title", "Auth module v3").unwrap();
        peer_b.set_field(file, "priority", "high").unwrap();

        // Sync: A → B
        let delta_a = peer_a.export_delta(file).unwrap();
        peer_b.import_delta(file, &delta_a).unwrap();

        // Sync: B → A
        let delta_b = peer_b.export_delta(file).unwrap();
        peer_a.import_delta(file, &delta_b).unwrap();

        // Both must converge to identical state
        let fields_a = peer_a.get_all_fields(file).unwrap();
        let fields_b = peer_b.get_all_fields(file).unwrap();

        let mut fa: Vec<_> = fields_a.into_iter().map(|f| (f.key, f.value)).collect();
        let mut fb: Vec<_> = fields_b.into_iter().map(|f| (f.key, f.value)).collect();
        fa.sort_by(|a, b| a.0.cmp(&b.0));
        fb.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(fa, fb, "Both peers must have identical state after merge");

        // No data loss: all 3 fields present
        let keys: Vec<&str> = fa.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"status"), "status field preserved");
        assert!(keys.contains(&"priority"), "priority field preserved");
        assert!(keys.contains(&"title"), "title field preserved");
    }

    #[test]
    fn test_offline_edit_reconnect() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();
        let file = "roadmap/TASK-042.md";

        // Create base snapshot
        {
            let mut base = LoroEngine::with_peer_id(&path, 0);
            base.open_doc(file).unwrap();
            base.set_field(file, "title", "Rate limiter").unwrap();
            base.set_field(file, "status", "todo").unwrap();
            base.save_snapshot(file).unwrap();
        }

        // Load base snapshot to get bytes
        let base_data = {
            let dir2 = TempDir::new().unwrap();
            let mut loader = LoroEngine::with_peer_id(dir2.path(), 0);
            loader.open_doc(file).unwrap();
            // Read the snapshot file
            let safe_name = file.replace('/', "_").replace('\\', "_");
            std::fs::read(path.join(format!("{}.loro", safe_name))).unwrap()
        };

        // Peer A: loads base, goes offline, edits
        let (_dir_a, mut peer_a) = make_engine(10);
        peer_a.open_doc(file).unwrap();
        peer_a.import_snapshot(file, &base_data).unwrap();
        peer_a.set_field(file, "title", "Rate limiter v2").unwrap();
        peer_a.set_field(file, "assignee", "alice").unwrap();
        let delta_a = peer_a.export_delta(file).unwrap();

        // Peer B: loads base, goes offline, different edits
        let (_dir_b, mut peer_b) = make_engine(20);
        peer_b.open_doc(file).unwrap();
        peer_b.import_snapshot(file, &base_data).unwrap();
        peer_b.set_field(file, "title", "Rate limiter v3").unwrap();
        peer_b.set_field(file, "priority", "critical").unwrap();
        let delta_b = peer_b.export_delta(file).unwrap();

        // Reconnect: exchange deltas
        peer_b.import_delta(file, &delta_a).unwrap();
        peer_a.import_delta(file, &delta_b).unwrap();

        // Verify convergence
        let state_a = peer_a.get_all_fields(file).unwrap();
        let state_b = peer_b.get_all_fields(file).unwrap();

        let mut fa: Vec<_> = state_a.into_iter().map(|f| (f.key, f.value)).collect();
        let mut fb: Vec<_> = state_b.into_iter().map(|f| (f.key, f.value)).collect();
        fa.sort_by(|a, b| a.0.cmp(&b.0));
        fb.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(fa, fb, "States must converge after reconnect");

        // No data loss
        let keys: Vec<&str> = fa.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"status"), "status preserved");
        assert!(keys.contains(&"assignee"), "assignee preserved");
        assert!(keys.contains(&"priority"), "priority preserved");

        let title = fa.iter().find(|(k, _)| k == "title").unwrap();
        assert!(!title.1.is_empty(), "title not lost");
    }

    #[test]
    fn test_load_from_markdown() {
        let (_dir, mut engine) = make_engine(1);
        let md = r#"---
title: Implement auth
status: in-progress
priority: high
pm-task: true
---

## Description
Build the authentication module.
"#;
        engine.load_from_markdown("roadmap/TASK-001.md", md).unwrap();

        assert_eq!(engine.get_field("roadmap/TASK-001.md", "title").unwrap(), "Implement auth");
        assert_eq!(engine.get_field("roadmap/TASK-001.md", "status").unwrap(), "in-progress");

        let body = engine.get_field("roadmap/TASK-001.md", "body").unwrap();
        assert!(body.contains("Build the authentication module"));
    }

    #[test]
    fn test_export_to_markdown() {
        let (_dir, mut engine) = make_engine(1);
        engine.open_doc("roadmap/TASK-001.md").unwrap();
        engine.set_field("roadmap/TASK-001.md", "title", "Export test").unwrap();
        engine.set_field("roadmap/TASK-001.md", "status", "done").unwrap();
        engine.set_field("roadmap/TASK-001.md", "body", "Body content here").unwrap();

        let md = engine.export_to_markdown("roadmap/TASK-001.md").unwrap();
        assert!(md.starts_with("---"));
        assert!(md.contains("title: Export test"));
        assert!(md.contains("status: done"));
        assert!(md.contains("Body content here"));
    }
}
