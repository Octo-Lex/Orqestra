//! Evidence schema validation for docs/evidence/*.json files.
//!
//! Validates that evidence files conform to the expected schema before
//! embedding into orqestra-roadmap.json. Used by the export CLI and CI.
//!
//! Validation policy:
//!   - CLI normal mode: warn + omit invalid evidence
//!   - CI validation mode: fail
//!   - Dashboard runtime: graceful fallback

use serde_json::Value;
use std::path::Path;

/// Validation result with context.
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        ValidationResult { valid: true, errors: vec![] }
    }

    pub fn fail(errors: Vec<String>) -> Self {
        ValidationResult { valid: false, errors }
    }

    pub fn add_error(&mut self, msg: String) {
        self.valid = false;
        self.errors.push(msg);
    }
}

/// Validate all five evidence files in a directory.
pub fn validate_evidence_dir(dir: &Path) -> ValidationResult {
    let mut result = ValidationResult::ok();

    if !dir.is_dir() {
        result.add_error("Evidence directory does not exist".to_string());
        return result;
    }

    let files = [
        "release-history.json",
        "test-count-history.json",
        "security-boundaries.json",
        "autonomy-policy.json",
        "runtime-decision-matrix.json",
    ];

    for file in &files {
        let path = dir.join(file);
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let parsed: Result<Value, _> = serde_json::from_str(&content);
                match parsed {
                    Ok(v) => validate_evidence_file(file, &v, &mut result),
                    Err(e) => result.add_error(format!("{}: invalid JSON: {}", file, e)),
                }
            }
            Err(e) => result.add_error(format!("{}: {}", file, e)),
        }
    }

    result
}

/// Validate a single evidence file by name.
fn validate_evidence_file(name: &str, data: &Value, result: &mut ValidationResult) {
    // All files must have schema_version === 1
    let sv = data.get("schema_version").and_then(|v| v.as_u64());
    match sv {
        Some(1) => {}
        Some(v) => result.add_error(format!("{}: schema_version is {}, expected 1", name, v)),
        None => result.add_error(format!("{}: missing schema_version", name)),
    }

    // File-specific validation
    match name {
        "release-history.json" => validate_release_history(data, result),
        "test-count-history.json" => validate_test_counts(data, result),
        "security-boundaries.json" => validate_security_boundaries(data, result),
        "autonomy-policy.json" => validate_autonomy_policy(data, result),
        "runtime-decision-matrix.json" => validate_runtime_evidence(data, result),
        _ => {}
    }
}

fn validate_release_history(data: &Value, result: &mut ValidationResult) {
    let releases = data.get("releases");
    if releases.is_none() {
        result.add_error("release-history.json: missing 'releases' field".to_string());
        return;
    }
    let releases = releases.unwrap();

    if !releases.is_object() {
        result.add_error("release-history.json: 'releases' must be an object".to_string());
        return;
    }

    for (version, entry) in releases.as_object().unwrap() {
        if !entry.is_object() {
            result.add_error(format!("release-history.json: entry {} is not an object", version));
            continue;
        }
        let obj = entry.as_object().unwrap();
        for required in &["date", "type", "label"] {
            if !obj.contains_key(*required) {
                result.add_error(format!("release-history.json: {} missing field '{}'", version, required));
            }
        }
        // date should be a string
        if let Some(d) = obj.get("date") {
            if !d.is_string() {
                result.add_error(format!("release-history.json: {}.date is not a string", version));
            }
        }
        // type should be a string
        if let Some(t) = obj.get("type") {
            if !t.is_string() {
                result.add_error(format!("release-history.json: {}.type is not a string", version));
            }
        }
    }
}

fn validate_test_counts(data: &Value, result: &mut ValidationResult) {
    let history = data.get("history");
    if history.is_none() {
        result.add_error("test-count-history.json: missing 'history' field".to_string());
        return;
    }
    let history = history.unwrap();

    if !history.is_array() {
        result.add_error("test-count-history.json: 'history' must be an array".to_string());
        return;
    }

    for (i, entry) in history.as_array().unwrap().iter().enumerate() {
        if !entry.is_object() {
            result.add_error(format!("test-count-history.json: history[{}] is not an object", i));
            continue;
        }
        let obj = entry.as_object().unwrap();
        for required in &["version", "rust", "worker", "total"] {
            if !obj.contains_key(*required) {
                result.add_error(format!("test-count-history.json: history[{}] missing '{}'", i, required));
            }
        }
        // Numeric fields should be numbers
        for numeric in &["rust", "worker", "total"] {
            if let Some(v) = obj.get(*numeric) {
                if !v.is_number() {
                    result.add_error(format!("test-count-history.json: history[{}].{} is not a number", i, numeric));
                }
            }
        }
    }
}

fn validate_security_boundaries(data: &Value, result: &mut ValidationResult) {
    let boundaries = data.get("boundaries");
    if boundaries.is_none() {
        result.add_error("security-boundaries.json: missing 'boundaries' field".to_string());
        return;
    }
    if !boundaries.unwrap().is_object() {
        result.add_error("security-boundaries.json: 'boundaries' must be an object".to_string());
    }
}

fn validate_autonomy_policy(data: &Value, result: &mut ValidationResult) {
    // max_session_cap must be 10
    let cap = data.get("max_session_cap").and_then(|v| v.as_u64());
    match cap {
        Some(10) => {}
        Some(v) => result.add_error(format!(
            "autonomy-policy.json: max_session_cap is {}, expected 10 (no autonomy expansion)",
            v
        )),
        None => result.add_error("autonomy-policy.json: missing max_session_cap".to_string()),
    }

    // auto_commit must be false
    let auto_commit = data.get("auto_commit").and_then(|v| v.as_bool());
    match auto_commit {
        Some(false) => {}
        Some(true) => result.add_error(
            "autonomy-policy.json: auto_commit is true, expected false".to_string(),
        ),
        None => result.add_error("autonomy-policy.json: missing auto_commit".to_string()),
    }

    // allowed_paths must be an array
    let paths = data.get("allowed_paths");
    if paths.is_none() {
        result.add_error("autonomy-policy.json: missing allowed_paths".to_string());
    } else if !paths.unwrap().is_array() {
        result.add_error("autonomy-policy.json: allowed_paths must be an array".to_string());
    }
}

fn validate_runtime_evidence(data: &Value, result: &mut ValidationResult) {
    // evidence_type must be structural-runtime-decision-matrix
    let etype = data.get("evidence_type").and_then(|v| v.as_str());
    match etype {
        Some("structural-runtime-decision-matrix") => {}
        Some(v) => result.add_error(format!(
            "runtime-decision-matrix.json: evidence_type is '{}', expected 'structural-runtime-decision-matrix'",
            v
        )),
        None => result.add_error("runtime-decision-matrix.json: missing evidence_type".to_string()),
    }

    // external_beta_user_data must be false
    let ext = data.get("external_beta_user_data").and_then(|v| v.as_bool());
    match ext {
        Some(false) => {}
        Some(true) => result.add_error(
            "runtime-decision-matrix.json: external_beta_user_data is true, expected false".to_string(),
        ),
        None => result.add_error("runtime-decision-matrix.json: missing external_beta_user_data".to_string()),
    }

    // path_matrix_evaluated must be a number
    let paths = data.get("path_matrix_evaluated").and_then(|v| v.as_u64());
    if paths.is_none() {
        result.add_error("runtime-decision-matrix.json: missing path_matrix_evaluated".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_evidence_file(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    fn make_valid_evidence(dir: &Path) {
        write_evidence_file(dir, "release-history.json", r#"{"schema_version":1,"releases":{"2.9.1":{"date":"2026-06-10","type":"security-patch","label":"Test"}}}"#);
        write_evidence_file(dir, "test-count-history.json", r#"{"schema_version":1,"history":[{"version":"2.9.1","rust":442,"worker":24,"dashboard":12,"total":478}]}"#);
        write_evidence_file(dir, "security-boundaries.json", r#"{"schema_version":1,"boundaries":{"relay_auth":{"algorithm":"HMAC-SHA256"}}}"#);
        write_evidence_file(dir, "autonomy-policy.json", r#"{"schema_version":1,"status":"docs-only pilot","max_session_cap":10,"default_cap":5,"auto_commit":false,"allowed_paths":["docs/**"]}"#);
        write_evidence_file(dir, "runtime-decision-matrix.json", r#"{"schema_version":1,"evidence_type":"structural-runtime-decision-matrix","external_beta_user_data":false,"path_matrix_evaluated":50}"#);
    }

    #[test]
    fn test_valid_evidence_schema_passes() {
        let tmp = tempfile::tempdir().unwrap();
        make_valid_evidence(tmp.path());
        let result = validate_evidence_dir(tmp.path());
        assert!(result.valid, "Expected valid, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_missing_schema_version_fails() {
        let tmp = tempfile::tempdir().unwrap();
        // Write files without schema_version
        write_evidence_file(tmp.path(), "release-history.json", r#"{"releases":{}}"#);
        write_evidence_file(tmp.path(), "test-count-history.json", r#"{"history":[]}"#);
        write_evidence_file(tmp.path(), "security-boundaries.json", r#"{"boundaries":{}}"#);
        write_evidence_file(tmp.path(), "autonomy-policy.json", r#"{"max_session_cap":10,"auto_commit":false,"allowed_paths":[]}"#);
        write_evidence_file(tmp.path(), "runtime-decision-matrix.json", r#"{"evidence_type":"structural-runtime-decision-matrix","external_beta_user_data":false,"path_matrix_evaluated":50}"#);
        let result = validate_evidence_dir(tmp.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("missing schema_version")));
    }

    #[test]
    fn test_wrong_schema_version_fails() {
        let tmp = tempfile::tempdir().unwrap();
        write_evidence_file(tmp.path(), "release-history.json", r#"{"schema_version":2,"releases":{}}"#);
        write_evidence_file(tmp.path(), "test-count-history.json", r#"{"schema_version":1,"history":[]}"#);
        write_evidence_file(tmp.path(), "security-boundaries.json", r#"{"schema_version":1,"boundaries":{}}"#);
        write_evidence_file(tmp.path(), "autonomy-policy.json", r#"{"schema_version":1,"max_session_cap":10,"auto_commit":false,"allowed_paths":[]}"#);
        write_evidence_file(tmp.path(), "runtime-decision-matrix.json", r#"{"schema_version":1,"evidence_type":"structural-runtime-decision-matrix","external_beta_user_data":false,"path_matrix_evaluated":50}"#);
        let result = validate_evidence_dir(tmp.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("schema_version is 2, expected 1")));
    }

    #[test]
    fn test_runtime_evidence_must_be_structural() {
        let tmp = tempfile::tempdir().unwrap();
        make_valid_evidence(tmp.path());
        // Overwrite runtime evidence with wrong type
        write_evidence_file(tmp.path(), "runtime-decision-matrix.json", r#"{"schema_version":1,"evidence_type":"external-beta","external_beta_user_data":false,"path_matrix_evaluated":50}"#);
        let result = validate_evidence_dir(tmp.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("evidence_type") && e.contains("external-beta")));
    }

    #[test]
    fn test_runtime_evidence_must_not_claim_external_beta() {
        let tmp = tempfile::tempdir().unwrap();
        make_valid_evidence(tmp.path());
        write_evidence_file(tmp.path(), "runtime-decision-matrix.json", r#"{"schema_version":1,"evidence_type":"structural-runtime-decision-matrix","external_beta_user_data":true,"path_matrix_evaluated":50}"#);
        let result = validate_evidence_dir(tmp.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("external_beta_user_data is true")));
    }

    #[test]
    fn test_autonomy_cap_must_remain_10() {
        let tmp = tempfile::tempdir().unwrap();
        make_valid_evidence(tmp.path());
        write_evidence_file(tmp.path(), "autonomy-policy.json", r#"{"schema_version":1,"status":"docs-only pilot","max_session_cap":15,"default_cap":5,"auto_commit":false,"allowed_paths":["docs/**"]}"#);
        let result = validate_evidence_dir(tmp.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("max_session_cap is 15, expected 10")));
    }

    #[test]
    fn test_missing_evidence_dir_still_graceful() {
        let tmp = tempfile::tempdir().unwrap();
        let result = validate_evidence_dir(&tmp.path().join("nonexistent"));
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("does not exist")));
    }

    #[test]
    fn test_malformed_evidence_file_fails_with_clear_error() {
        let tmp = tempfile::tempdir().unwrap();
        make_valid_evidence(tmp.path());
        write_evidence_file(tmp.path(), "test-count-history.json", "not valid json {{{");
        let result = validate_evidence_dir(tmp.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("invalid JSON")));
    }
}
