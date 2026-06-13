//! Orqestra Development Lifecycle — Tauri commands (v0.2.0)
//!
//! Commands exposed to the frontend for lifecycle operations.
//! All state changes go through the append-only event log.

use std::path::Path;
use tauri::command;
use chrono::Utc;

use super::event_log;
use super::types::*;

type CommandResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ---------------------------------------------------------------------------
// Lifecycle initialization
// ---------------------------------------------------------------------------

/// Initialize lifecycle mode for a project.
/// Creates `.Orqestra/lifecycle/` directory structure and emits Started event.
#[command]
pub fn lifecycle_init_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);

    // Check if already initialized
    if event_log::is_lifecycle_initialized(root) {
        let state = event_log::derive_state(root).map_err(err)?;
        return Ok(serde_json::json!({
            "ok": true,
            "already_initialized": true,
            "state": state,
        }));
    }

    // Check for migration from product-team/
    let migrated = event_log::migrate_from_product_team(root).map_err(err)?;

    // Create directory structure
    event_log::ensure_lifecycle_dirs(root).map_err(err)?;

    // Emit Started + StageEntered(Orient) events
    let timestamp = Utc::now().to_rfc3339();

    if !migrated {
        // Only emit Started if we didn't migrate (migration emits it)
        let started = LifecycleEvent::Started {
            project_root: project_root.clone(),
            timestamp: timestamp.clone(),
        };
        event_log::append_event(root, &started).map_err(err)?;
    }

    let entered = LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: Utc::now().to_rfc3339(),
        actor: "human".to_string(),
    };
    event_log::append_event(root, &entered).map_err(err)?;

    let state = event_log::derive_state(root).map_err(err)?;

    Ok(serde_json::json!({
        "ok": true,
        "already_initialized": false,
        "migrated_from_product_team": migrated,
        "state": state,
    }))
}

/// Get the current lifecycle state (derived from events).
#[command]
pub fn lifecycle_get_state_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);

    if !event_log::is_lifecycle_initialized(root) {
        return Ok(serde_json::json!({
            "initialized": false,
            "started": false,
            "current_stage": null,
        }));
    }

    let state = event_log::derive_state(root).map_err(err)?;

    Ok(serde_json::json!({
        "initialized": true,
        "started": state.started,
        "current_stage": state.current_stage,
        "current_stage_name": state.current_stage.display_name(),
        "current_stage_purpose": state.current_stage.purpose(),
        "stages": LifecycleStage::all().iter().map(|s| {
            serde_json::json!({
                "name": s.display_name(),
                "index": s.index(),
                "is_current": *s == state.current_stage,
                "is_implemented": s.is_implemented(),
            })
        }).collect::<Vec<_>>(),
        "artifacts": state.artifacts,
        "gates": state.gates,
        "events_count": state.events_count,
    }))
}

// ---------------------------------------------------------------------------
// Gate operations
// ---------------------------------------------------------------------------

/// Request to advance to the next stage (emits gate.requested).
#[command]
pub fn lifecycle_request_advance_cmd(
    project_root: String,
    feature_id: Option<String>,
) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);
    let state = event_log::derive_state(root).map_err(err)?;

    let current_gate = gate_for_stage(&state.current_stage).ok_or("Already at final stage")?;

    let event = LifecycleEvent::GateRequested {
        gate: current_gate.clone(),
        feature_id,
        timestamp: Utc::now().to_rfc3339(),
        actor: "human".to_string(),
    };
    event_log::append_event(root, &event).map_err(err)?;

    Ok(serde_json::json!({
        "ok": true,
        "gate": current_gate,
        "gate_name": current_gate.display_name(),
        "stage": state.current_stage,
    }))
}

/// Approve the current stage's gate and advance.
#[command]
pub fn lifecycle_approve_gate_cmd(
    project_root: String,
    feature_id: Option<String>,
) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);
    let state = event_log::derive_state(root).map_err(err)?;

    let current_stage = state.current_stage;
    let current_gate = gate_for_stage(&current_stage).ok_or("Already at final stage")?;

    let timestamp = Utc::now().to_rfc3339();

    // Approve gate
    let approve = LifecycleEvent::GateApproved {
        gate: current_gate.clone(),
        feature_id: feature_id.clone(),
        timestamp: timestamp.clone(),
        actor: "human".to_string(),
    };
    event_log::append_event(root, &approve).map_err(err)?;

    // Advance to next stage (if there is one)
    if let Some(next_stage) = current_stage.next() {
        let advance = LifecycleEvent::StageAdvanced {
            from: current_stage,
            to: next_stage,
            feature_id: feature_id.clone(),
            timestamp: timestamp.clone(),
            actor: "human".to_string(),
        };
        event_log::append_event(root, &advance).map_err(err)?;

        // Enter the next stage
        let enter = LifecycleEvent::StageEntered {
            stage: next_stage,
            feature_id,
            timestamp: Utc::now().to_rfc3339(),
            actor: "human".to_string(),
        };
        event_log::append_event(root, &enter).map_err(err)?;
    }

    let new_state = event_log::derive_state(root).map_err(err)?;

    Ok(serde_json::json!({
        "ok": true,
        "gate_approved": current_gate,
        "advanced_to": new_state.current_stage,
        "state": new_state,
    }))
}

/// Reject the current stage's gate.
#[command]
pub fn lifecycle_reject_gate_cmd(
    project_root: String,
    feature_id: Option<String>,
    reason: String,
) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);
    let state = event_log::derive_state(root).map_err(err)?;

    let current_gate = gate_for_stage(&state.current_stage).ok_or("Already at final stage")?;

    let event = LifecycleEvent::GateRejected {
        gate: current_gate.clone(),
        feature_id,
        timestamp: Utc::now().to_rfc3339(),
        actor: "human".to_string(),
        reason,
    };
    event_log::append_event(root, &event).map_err(err)?;

    Ok(serde_json::json!({
        "ok": true,
        "gate_rejected": current_gate,
        "stage_remains": state.current_stage,
    }))
}

// ---------------------------------------------------------------------------
// Artifact operations
// ---------------------------------------------------------------------------

/// Record that an artifact was created.
#[command]
pub fn lifecycle_record_artifact_cmd(
    project_root: String,
    artifact_type: String,
    path: String,
    feature_id: Option<String>,
    actor: String,
) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);

    let artifact = parse_artifact_type(&artifact_type)?;

    let event = LifecycleEvent::ArtifactCreated {
        artifact_type: artifact.clone(),
        path,
        feature_id,
        timestamp: Utc::now().to_rfc3339(),
        actor,
    };
    event_log::append_event(root, &event).map_err(err)?;

    Ok(serde_json::json!({
        "ok": true,
        "artifact_type": artifact,
    }))
}

/// Read an artifact file from the lifecycle directory.
#[command]
pub fn lifecycle_read_artifact_cmd(
    project_root: String,
    path: String,
) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);

    // Security: path must be within .Orqestra/lifecycle/
    let lifecycle_root = event_log::lifecycle_root(root);
    let target = lifecycle_root.join(&path);

    let canonical_lifecycle = lifecycle_root.canonicalize().map_err(err)?;
    let canonical_target = target.canonicalize().map_err(|e| {
        format!("Cannot read artifact {}: {}", path, e)
    })?;

    if !canonical_target.starts_with(&canonical_lifecycle) {
        return Err(format!(
            "Path traversal blocked: '{}' is outside lifecycle directory",
            path
        ));
    }

    let content = std::fs::read_to_string(&canonical_target).map_err(err)?;

    Ok(serde_json::json!({
        "ok": true,
        "path": path,
        "content": content,
    }))
}

/// Write an artifact file to the lifecycle directory.
#[command]
pub fn lifecycle_write_artifact_cmd(
    project_root: String,
    path: String,
    content: String,
    artifact_type: Option<String>,
    feature_id: Option<String>,
    actor: String,
) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);

    // Security: path must be within .Orqestra/lifecycle/
    let lifecycle_root = event_log::lifecycle_root(root);
    let target = lifecycle_root.join(&path);

    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(err)?;
    }

    // Canonicalize parent for traversal check
    let canonical_parent = target
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .ok_or("Cannot resolve target directory")?;
    let canonical_lifecycle = lifecycle_root.canonicalize().map_err(err)?;

    if !canonical_parent.starts_with(&canonical_lifecycle) {
        return Err(format!(
            "Path traversal blocked: '{}' is outside lifecycle directory",
            path
        ));
    }

    std::fs::write(&target, &content).map_err(err)?;

    // Record artifact creation/update in event log
    if let Some(art_str) = &artifact_type {
        if let Ok(art_type) = parse_artifact_type(art_str) {
            let is_update = target.exists();

            let event = if is_update {
                LifecycleEvent::ArtifactUpdated {
                    artifact_type: art_type,
                    path: path.clone(),
                    feature_id,
                    timestamp: Utc::now().to_rfc3339(),
                    actor,
                }
            } else {
                LifecycleEvent::ArtifactCreated {
                    artifact_type: art_type,
                    path: path.clone(),
                    feature_id,
                    timestamp: Utc::now().to_rfc3339(),
                    actor,
                }
            };

            // Best-effort event logging (file write succeeded)
            let _ = event_log::append_event(root, &event);
        }
    }

    Ok(serde_json::json!({
        "ok": true,
        "path": path,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gate_for_stage(stage: &LifecycleStage) -> Option<GateId> {
    match stage {
        LifecycleStage::Orient => Some(GateId::OrientUnderstandingConfirmed),
        LifecycleStage::Discover => Some(GateId::DiscoverIntakeComplete),
        LifecycleStage::Define => Some(GateId::PrdApproval),
        LifecycleStage::Design => Some(GateId::DesignReview),
        LifecycleStage::Plan => Some(GateId::IssueGraphApproval),
        LifecycleStage::Prepare => Some(GateId::FileScopeConfirmed),
        LifecycleStage::Build => Some(GateId::PatchApproval),
        LifecycleStage::Verify => Some(GateId::QaReportAccepted),
        LifecycleStage::Review => Some(GateId::ReviewAccepted),
        LifecycleStage::Ship => Some(GateId::ReleaseIntegrityVerified),
        LifecycleStage::Observe => Some(GateId::EvidenceAccepted),
        LifecycleStage::Learn => Some(GateId::LearningDecisionMade),
        LifecycleStage::Evolve => Some(GateId::CyclePlanApproved),
    }
}

fn parse_artifact_type(s: &str) -> CommandResult<ArtifactType> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| format!("Unknown artifact type '{}': {}", s, e))
}

// ---------------------------------------------------------------------------
// Orient stage — mechanical repo scan
// ---------------------------------------------------------------------------

/// Run the Orient stage: scan repo and generate project knowledge pack.
#[command]
pub fn lifecycle_run_orient_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);
    let profile = super::orient::run_orient(root).map_err(|e| e)?;
    Ok(serde_json::json!({
        "ok": true,
        "profile": profile,
    }))
}

// ---------------------------------------------------------------------------
// Discover stage — feature intake
// ---------------------------------------------------------------------------

/// Create a feature intake record (Discover stage).
#[command]
pub fn lifecycle_create_intake_cmd(
    project_root: String,
    feature_title: String,
    problem_brief: String,
    affected_users: String,
    repo_area: String,
    constraints: String,
    out_of_scope: String,
) -> CommandResult<serde_json::Value> {
    let root = Path::new(&project_root);

    // Generate feature ID
    let feature_id = format!("feat-{}", chrono::Utc::now().timestamp());

    // Ensure feature directory
    let lifecycle = super::event_log::lifecycle_root(root);
    let feature_dir = lifecycle.join("features").join(&feature_id).join("intake");
    std::fs::create_dir_all(&feature_dir).map_err(err)?;

    // Write problem-brief.md
    let brief = format!(
        "# Feature: {}\n\n## Problem Brief\n{}\n\n## Affected Users\n{}\n\n## Repo Area\n{}\n\n## Constraints\n{}\n\n## Out of Scope\n{}\n",
        feature_title, problem_brief, affected_users, repo_area, constraints, out_of_scope
    );
    std::fs::write(feature_dir.join("problem-brief.md"), &brief).map_err(err)?;

    // Write assumptions.json (user must fill)
    let assumptions = serde_json::json!({
        "schema_version": 1,
        "assumptions": [],
        "note": "Add explicit assumptions before requesting gate approval."
    });
    std::fs::write(
        feature_dir.join("assumptions.json"),
        serde_json::to_string_pretty(&assumptions).map_err(err)?,
    ).map_err(err)?;

    // Write open-questions.md (user must fill)
    std::fs::write(
        feature_dir.join("open-questions.md"),
        "# Open Questions\n\n- [ ] Add unresolved questions that block definition\n",
    ).map_err(err)?;

    // Record artifacts
    let timestamp = chrono::Utc::now().to_rfc3339();
    for (art_type, path) in [
        (ArtifactType::ProblemBrief, format!("features/{}/intake/problem-brief.md", feature_id)),
        (ArtifactType::Assumptions, format!("features/{}/intake/assumptions.json", feature_id)),
        (ArtifactType::OpenQuestions, format!("features/{}/intake/open-questions.md", feature_id)),
    ] {
        let event = LifecycleEvent::ArtifactCreated {
            artifact_type: art_type,
            path,
            feature_id: Some(feature_id.clone()),
            timestamp: timestamp.clone(),
            actor: "human".to_string(),
        };
        let _ = super::event_log::append_event(root, &event);
    }

    Ok(serde_json::json!({
        "ok": true,
        "feature_id": feature_id,
        "path": format!("features/{}/intake/", feature_id),
    }))
}
