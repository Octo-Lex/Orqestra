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
        "external-beta-evidence.json",
        "external-beta-review.json",
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
        "external-beta-evidence.json" => validate_external_beta_evidence(data, result),
        "external-beta-review.json" => validate_external_beta_review(data, result),
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

fn validate_external_beta_evidence(data: &Value, result: &mut ValidationResult) {
    // schema_version
    match data.get("schema_version").and_then(|v| v.as_u64()) {
        Some(1) => {}
        Some(v) => result.add_error(format!(
            "external-beta-evidence.json: schema_version is {}, expected 1",
            v
        )),
        None => result.add_error("external-beta-evidence.json: missing schema_version".to_string()),
    }

    // status must be "none" unless real evidence exists
    match data.get("status").and_then(|v| v.as_str()) {
        Some(s) if s == "none" || s == "present" => {}
        Some(s) => result.add_error(format!(
            "external-beta-evidence.json: invalid status '{}', expected 'none' or 'present'",
            s
        )),
        None => result.add_error("external-beta-evidence.json: missing status".to_string()),
    }

    // external_beta_user_data must be false until real data
    match data.get("external_beta_user_data").and_then(|v| v.as_bool()) {
        Some(false) => {}
        Some(true) => {
            // Allowed only if status is "present"
            if data.get("status").and_then(|v| v.as_str()) != Some("present") {
                result.add_error(
                    "external-beta-evidence.json: external_beta_user_data is true but status is not 'present'"
                        .to_string(),
                );
            }
        }
        None => result.add_error("external-beta-evidence.json: missing external_beta_user_data".to_string()),
    }

    // automatic_upload must be false
    match data.get("automatic_upload").and_then(|v| v.as_bool()) {
        Some(false) => {}
        Some(true) => result.add_error(
            "external-beta-evidence.json: automatic_upload must be false"
                .to_string(),
        ),
        None => result.add_error("external-beta-evidence.json: missing automatic_upload".to_string()),
    }

    // consent_required must be true
    match data.get("consent_required").and_then(|v| v.as_bool()) {
        Some(true) => {}
        Some(false) => result.add_error(
            "external-beta-evidence.json: consent_required must be true"
                .to_string(),
        ),
        None => result.add_error("external-beta-evidence.json: missing consent_required".to_string()),
    }
}

fn validate_external_beta_review(data: &Value, result: &mut ValidationResult) {
    // schema_version
    match data.get("schema_version").and_then(|v| v.as_u64()) {
        Some(1) => {}
        Some(v) => result.add_error(format!(
            "external-beta-review.json: schema_version is {}, expected 1",
            v
        )),
        None => result.add_error("external-beta-review.json: missing schema_version".to_string()),
    }

    // status must be one of the allowed values
    let status = match data.get("status").and_then(|v| v.as_str()) {
        Some(s) if ["none", "present", "insufficient", "rejected"].contains(&s) => s,
        Some(s) => {
            result.add_error(format!(
                "external-beta-review.json: invalid status '{}', expected none/present/insufficient/rejected",
                s
            ));
            ""
        }
        None => {
            result.add_error("external-beta-review.json: missing status".to_string());
            ""
        }
    };

    // external_beta_user_data must match status
    let has_user_data = data.get("external_beta_user_data").and_then(|v| v.as_bool());
    match has_user_data {
        Some(false) => {
            if status == "present" {
                result.add_error(
                    "external-beta-review.json: status is 'present' but external_beta_user_data is false"
                        .to_string(),
                );
            }
        }
        Some(true) => {
            if status != "present" {
                result.add_error(format!(
                    "external-beta-review.json: external_beta_user_data is true but status is '{}' (not 'present')",
                    status
                ));
            }
        }
        None => result.add_error("external-beta-review.json: missing external_beta_user_data".to_string()),
    }

    // --- status-specific enforcement ---

    if status == "present" {
        // reviewed_bundle_count must be > 0
        match data.get("reviewed_bundle_count").and_then(|v| v.as_u64()) {
            Some(n) if n > 0 => {}
            Some(0) => result.add_error(
                "external-beta-review.json: reviewed_bundle_count must be > 0 when status is 'present'"
                    .to_string(),
            ),
            Some(_) => result.add_error(
                "external-beta-review.json: reviewed_bundle_count must be a non-negative integer"
                    .to_string(),
            ),
            None => result.add_error(
                "external-beta-review.json: reviewed_bundle_count is required when status is 'present'"
                    .to_string(),
            ),
        }

        // accepted_bundle_count must be > 0
        let reviewed = data.get("reviewed_bundle_count").and_then(|v| v.as_u64()).unwrap_or(0);
        match data.get("accepted_bundle_count").and_then(|v| v.as_u64()) {
            Some(n) if n > 0 => {
                if n > reviewed {
                    result.add_error(format!(
                        "external-beta-review.json: accepted_bundle_count ({}) must be <= reviewed_bundle_count ({})",
                        n, reviewed
                    ));
                }
            }
            Some(0) => result.add_error(
                "external-beta-review.json: accepted_bundle_count must be > 0 when status is 'present'"
                    .to_string(),
            ),
            Some(_) => result.add_error(
                "external-beta-review.json: accepted_bundle_count must be a non-negative integer"
                    .to_string(),
            ),
            None => result.add_error(
                "external-beta-review.json: accepted_bundle_count is required when status is 'present'"
                    .to_string(),
            ),
        }

        // aggregate_outcomes must exist with all 5 keys
        match data.get("aggregate_outcomes") {
            Some(obj) if obj.is_object() => {
                for key in &["completed", "completed_with_warnings", "blocked", "abandoned", "unknown"] {
                    match obj.get(*key).and_then(|v| v.as_u64()) {
                        Some(_) => {}
                        None => result.add_error(format!(
                            "external-beta-review.json: aggregate_outcomes.{} must be a non-negative integer",
                            key
                        )),
                    }
                }
            }
            Some(_) => result.add_error(
                "external-beta-review.json: aggregate_outcomes must be an object".to_string(),
            ),
            None => result.add_error(
                "external-beta-review.json: aggregate_outcomes is required when status is 'present'"
                    .to_string(),
            ),
        }

        // aggregate_failure_codes must exist and be an object
        match data.get("aggregate_failure_codes") {
            Some(obj) if obj.is_object() => {}
            Some(_) => result.add_error(
                "external-beta-review.json: aggregate_failure_codes must be an object".to_string(),
            ),
            None => result.add_error(
                "external-beta-review.json: aggregate_failure_codes is required when status is 'present'"
                    .to_string(),
            ),
        }
    }

    if status == "none" {
        // reviewed/accepted/rejected/follow-up counts must be zero if present
        for field in &["reviewed_bundle_count", "accepted_bundle_count", "rejected_bundle_count", "needs_follow_up_count"] {
            if let Some(n) = data.get(*field).and_then(|v| v.as_u64()) {
                if n > 0 {
                    result.add_error(format!(
                        "external-beta-review.json: {} must be 0 when status is 'none'",
                        field
                    ));
                }
            }
        }
    }

    // privacy section must exist and be honest
    let privacy = data.get("privacy");
    if privacy.is_none() {
        result.add_error("external-beta-review.json: missing privacy section".to_string());
    } else {
        let p = privacy.unwrap();
        for flag in &["raw_paths_published", "raw_tokens_published", "raw_file_contents_published", "raw_bundle_committed"] {
            match p.get(*flag).and_then(|v| v.as_bool()) {
                Some(false) => {}
                Some(true) => result.add_error(format!(
                    "external-beta-review.json: privacy.{} must be false",
                    flag
                )),
                None => result.add_error(format!(
                    "external-beta-review.json: missing privacy.{}",
                    flag
                )),
            }
        }
        match p.get("aggregate_only").and_then(|v| v.as_bool()) {
            Some(true) => {}
            Some(false) => result.add_error(
                "external-beta-review.json: privacy.aggregate_only must be true".to_string(),
            ),
            None => result.add_error(
                "external-beta-review.json: missing privacy.aggregate_only".to_string(),
            ),
        }
    }

    // Must not contain validation claim without explicit threshold
    let serialized = serde_json::to_string(data).unwrap_or_default();
    if serialized.contains("external_beta_validated") {
        result.add_error(
            "external-beta-review.json: must not contain 'external_beta_validated' field".to_string(),
        );
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
        write_evidence_file(dir, "external-beta-evidence.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        write_evidence_file(dir, "external-beta-review.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"privacy":{"raw_paths_published":false,"raw_tokens_published":false,"raw_file_contents_published":false,"raw_bundle_committed":false,"aggregate_only":true}}"#);
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
        write_evidence_file(tmp.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        write_evidence_file(tmp.path(), "external-beta-review.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"privacy":{"raw_paths_published":false,"raw_tokens_published":false,"raw_file_contents_published":false,"raw_bundle_committed":false,"aggregate_only":true}}"#);
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
        write_evidence_file(tmp.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        write_evidence_file(tmp.path(), "external-beta-review.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"privacy":{"raw_paths_published":false,"raw_tokens_published":false,"raw_file_contents_published":false,"raw_bundle_committed":false,"aggregate_only":true}}"#);
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
        write_evidence_file(tmp.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        write_evidence_file(tmp.path(), "external-beta-review.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"privacy":{"raw_paths_published":false,"raw_tokens_published":false,"raw_file_contents_published":false,"raw_bundle_committed":false,"aggregate_only":true}}"#);
        let result = validate_evidence_dir(tmp.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("evidence_type") && e.contains("external-beta")));
    }

    #[test]
    fn test_runtime_evidence_must_not_claim_external_beta() {
        let tmp = tempfile::tempdir().unwrap();
        make_valid_evidence(tmp.path());
        write_evidence_file(tmp.path(), "runtime-decision-matrix.json", r#"{"schema_version":1,"evidence_type":"structural-runtime-decision-matrix","external_beta_user_data":true,"path_matrix_evaluated":50}"#);
        write_evidence_file(tmp.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        write_evidence_file(tmp.path(), "external-beta-review.json", r#"{"schema_version":1,"status":"none","external_beta_user_data":false,"privacy":{"raw_paths_published":false,"raw_tokens_published":false,"raw_file_contents_published":false,"raw_bundle_committed":false,"aggregate_only":true}}"#);
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

    // --- v2.13.0: external beta review schema tests ---

    #[test]
    fn external_beta_review_schema_accepts_none_status() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        let result = validate_evidence_dir(dir.path());
        assert!(result.valid, "Expected valid: {:?}", result.errors);
    }

    #[test]
    fn external_beta_review_schema_accepts_present_status_with_aggregate() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        // Override review with valid present status
        write_evidence_file(dir.path(), "external-beta-review.json", r#"{
            "schema_version": 1,
            "status": "present",
            "external_beta_user_data": true,
            "reviewed_bundle_count": 2,
            "accepted_bundle_count": 1,
            "rejected_bundle_count": 1,
            "needs_follow_up_count": 0,
            "aggregate_outcomes": {
                "completed": 1,
                "completed_with_warnings": 0,
                "blocked": 1,
                "abandoned": 0,
                "unknown": 0
            },
            "aggregate_failure_codes": {"INSTALL_BLOCKED": 1},
            "privacy": {
                "raw_paths_published": false,
                "raw_tokens_published": false,
                "raw_file_contents_published": false,
                "raw_bundle_committed": false,
                "aggregate_only": true
            }
        }"#);
        // Also override external-beta-evidence to match
        write_evidence_file(dir.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"present","external_beta_user_data":true,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        let result = validate_evidence_dir(dir.path());
        assert!(result.valid, "Expected valid: {:?}", result.errors);
    }

    #[test]
    fn external_beta_review_rejects_present_without_accepted_count() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        write_evidence_file(dir.path(), "external-beta-review.json", r#"{
            "schema_version": 1,
            "status": "present",
            "external_beta_user_data": true,
            "privacy": {
                "raw_paths_published": false,
                "raw_tokens_published": false,
                "raw_file_contents_published": false,
                "raw_bundle_committed": false,
                "aggregate_only": true
            }
        }"#);
        write_evidence_file(dir.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"present","external_beta_user_data":true,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        let result = validate_evidence_dir(dir.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("reviewed_bundle_count is required when status is 'present'")));
        assert!(result.errors.iter().any(|e| e.contains("accepted_bundle_count is required when status is 'present'")));
        assert!(result.errors.iter().any(|e| e.contains("aggregate_outcomes is required when status is 'present'")));
        assert!(result.errors.iter().any(|e| e.contains("aggregate_failure_codes is required when status is 'present'")));
    }

    #[test]
    fn external_beta_review_rejects_present_with_zero_accepted() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        write_evidence_file(dir.path(), "external-beta-review.json", r#"{
            "schema_version": 1,
            "status": "present",
            "external_beta_user_data": true,
            "reviewed_bundle_count": 1,
            "accepted_bundle_count": 0,
            "aggregate_outcomes": {"completed":0,"completed_with_warnings":0,"blocked":0,"abandoned":0,"unknown":0},
            "aggregate_failure_codes": {},
            "privacy": {
                "raw_paths_published": false,
                "raw_tokens_published": false,
                "raw_file_contents_published": false,
                "raw_bundle_committed": false,
                "aggregate_only": true
            }
        }"#);
        write_evidence_file(dir.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"present","external_beta_user_data":true,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        let result = validate_evidence_dir(dir.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("accepted_bundle_count must be > 0 when status is 'present'")));
    }

    #[test]
    fn external_beta_review_rejects_present_with_accepted_exceeding_reviewed() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        write_evidence_file(dir.path(), "external-beta-review.json", r#"{
            "schema_version": 1,
            "status": "present",
            "external_beta_user_data": true,
            "reviewed_bundle_count": 1,
            "accepted_bundle_count": 2,
            "aggregate_outcomes": {"completed":2,"completed_with_warnings":0,"blocked":0,"abandoned":0,"unknown":0},
            "aggregate_failure_codes": {},
            "privacy": {
                "raw_paths_published": false,
                "raw_tokens_published": false,
                "raw_file_contents_published": false,
                "raw_bundle_committed": false,
                "aggregate_only": true
            }
        }"#);
        write_evidence_file(dir.path(), "external-beta-evidence.json", r#"{"schema_version":1,"status":"present","external_beta_user_data":true,"automatic_upload":false,"consent_required":true,"redaction_required":true}"#);
        let result = validate_evidence_dir(dir.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("accepted_bundle_count (2) must be <= reviewed_bundle_count (1)")));
    }

    #[test]
    fn external_beta_review_rejects_none_with_user_data_true() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        write_evidence_file(dir.path(), "external-beta-review.json", r#"{
            "schema_version": 1,
            "status": "none",
            "external_beta_user_data": true,
            "privacy": {
                "raw_paths_published": false,
                "raw_tokens_published": false,
                "raw_file_contents_published": false,
                "raw_bundle_committed": false,
                "aggregate_only": true
            }
        }"#);
        let result = validate_evidence_dir(dir.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("external_beta_user_data is true but status is 'none'")));
    }

    #[test]
    fn external_beta_review_rejects_none_with_nonzero_counts() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        write_evidence_file(dir.path(), "external-beta-review.json", r#"{
            "schema_version": 1,
            "status": "none",
            "external_beta_user_data": false,
            "reviewed_bundle_count": 1,
            "accepted_bundle_count": 0,
            "rejected_bundle_count": 0,
            "needs_follow_up_count": 0,
            "privacy": {
                "raw_paths_published": false,
                "raw_tokens_published": false,
                "raw_file_contents_published": false,
                "raw_bundle_committed": false,
                "aggregate_only": true
            }
        }"#);
        let result = validate_evidence_dir(dir.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("reviewed_bundle_count must be 0 when status is 'none'")));
    }

    #[test]
    fn external_beta_review_rejects_validation_claim() {
        let dir = tempfile::tempdir().unwrap();
        make_valid_evidence(dir.path());
        write_evidence_file(dir.path(), "external-beta-review.json", r#"{
            "schema_version": 1,
            "status": "none",
            "external_beta_user_data": false,
            "external_beta_validated": true,
            "privacy": {
                "raw_paths_published": false,
                "raw_tokens_published": false,
                "raw_file_contents_published": false,
                "raw_bundle_committed": false,
                "aggregate_only": true
            }
        }"#);
        let result = validate_evidence_dir(dir.path());
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("must not contain 'external_beta_validated'")));
    }
}
