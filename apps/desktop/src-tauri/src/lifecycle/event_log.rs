//! Orqestra Development Lifecycle — Event log (v0.2.0)
//!
//! Append-only JSONL event log stored in `.Orqestra/lifecycle/`.
//! State is derived by replaying events. Past events are never mutated.
//!
//! File: `.Orqestra/lifecycle/events.jsonl`
//! Format: one JSON object per line, each a serialized `LifecycleEvent`.

use std::path::{Path, PathBuf};
use chrono::Utc;

use super::types::*;

// ---------------------------------------------------------------------------
// Directory structure
// ---------------------------------------------------------------------------

/// Root directory for lifecycle state within a project repo.
pub fn lifecycle_root(project_root: &Path) -> PathBuf {
    project_root.join(".Orqestra").join("lifecycle")
}

/// Full directory structure for lifecycle state.
pub fn ensure_lifecycle_dirs(project_root: &Path) -> std::io::Result<()> {
    let root = lifecycle_root(project_root);
    let dirs = [
        root.join("project"),
        root.join("features"),
        root.join("releases"),
        root.join("observations"),
        root.join("learnings"),
        root.join("team"),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    Ok(())
}

/// Check if lifecycle mode has been initialized for this project.
pub fn is_lifecycle_initialized(project_root: &Path) -> bool {
    lifecycle_root(project_root).exists()
}

// ---------------------------------------------------------------------------
// Migration from .Orqestra/product-team/ (PTM v0.1.2)
// ---------------------------------------------------------------------------

/// If `.Orqestra/product-team/` exists, rename it to `.Orqestra/lifecycle/`.
/// This is a one-way migration. If `.Orqestra/lifecycle/` already exists,
/// the old directory is left in place (migration already done).
pub fn migrate_from_product_team(project_root: &Path) -> std::io::Result<bool> {
    let old_path = project_root.join(".Orqestra").join("product-team");
    let new_path = lifecycle_root(project_root);

    if !old_path.exists() {
        return Ok(false); // Nothing to migrate
    }

    if new_path.exists() {
        // Lifecycle already exists — don't clobber it
        tracing::info!(
            "product-team/ exists alongside lifecycle/ — skipping migration (already done)"
        );
        return Ok(false);
    }

    // Rename old → new
    std::fs::rename(&old_path, &new_path)?;
    tracing::info!("Migrated .Orqestra/product-team/ → .Orqestra/lifecycle/");

    // Log migration as first event
    let event = LifecycleEvent::Started {
        project_root: project_root.to_string_lossy().to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };
    // Best-effort: if event append fails (unlikely since dir exists), log but don't fail
    if let Err(e) = append_event(project_root, &event) {
        tracing::warn!("Failed to log migration event: {}", e);
    };

    Ok(true)
}

// ---------------------------------------------------------------------------
// Event log
// ---------------------------------------------------------------------------

/// Path to the global event log.
fn events_log_path(project_root: &Path) -> PathBuf {
    lifecycle_root(project_root).join("events.jsonl")
}

/// Append an event to the event log. The event is validated before appending.
/// Returns Err if the event is invalid.
pub fn append_event(project_root: &Path, event: &LifecycleEvent) -> Result<(), LifecycleError> {
    // Validate: can't append to uninitialized lifecycle (except Started)
    let requires_init = !matches!(event, LifecycleEvent::Started { .. });
    if requires_init && !is_lifecycle_initialized(project_root) {
        return Err(LifecycleError::NotInitialized);
    }

    // Validate gate logic
    validate_event(project_root, event)?;

    let path = events_log_path(project_root);
    let json = serde_json::to_string(event)
        .map_err(|e| LifecycleError::Serialization(e.to_string()))?;

    // Append (create if needed)
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| LifecycleError::Io(e.to_string()))?;

    writeln!(file, "{}", json).map_err(|e| LifecycleError::Io(e.to_string()))?;

    Ok(())
}

/// Read all events from the log.
pub fn read_events(project_root: &Path) -> Result<Vec<LifecycleEvent>, LifecycleError> {
    let path = events_log_path(project_root);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| LifecycleError::Io(e.to_string()))?;

    let mut events = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<LifecycleEvent>(line) {
            Ok(event) => events.push(event),
            Err(e) => {
                return Err(LifecycleError::CorruptEventLog {
                    line: line_num + 1,
                    message: e.to_string(),
                });
            }
        }
    }

    Ok(events)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate an event against current state before appending.
fn validate_event(project_root: &Path, event: &LifecycleEvent) -> Result<(), LifecycleError> {
    let state = derive_state(project_root)?;

    match event {
        LifecycleEvent::Started { .. } => {
            // Always allowed — starts the lifecycle
            // (duplicate starts are tolerated — the first one wins for derived state)
        }

        LifecycleEvent::StageEntered { stage, .. } => {
            // Must be a valid entry from current state
            // If lifecycle hasn't started, first entered must be Orient
            if !state.started && *stage != LifecycleStage::Orient {
                return Err(LifecycleError::InvalidStageEntry {
                    reason: "Lifecycle not started — first stage must be Orient".into(),
                });
            }
        }

        LifecycleEvent::StageAdvanced { from, to, .. } => {
            // Must advance from current stage
            if *from != state.current_stage {
                return Err(LifecycleError::InvalidStageAdvance {
                    from: from.display_name().to_string(),
                    expected_from: state.current_stage.display_name().to_string(),
                });
            }

            // Must advance to next stage
            match state.current_stage.next() {
                Some(next) if next == *to => {}
                Some(next) => {
                    return Err(LifecycleError::InvalidStageAdvance {
                        from: from.display_name().to_string(),
                        expected_from: format!(
                            "Expected advance to {}, got {}",
                            next.display_name(),
                            to.display_name()
                        ),
                    });
                }
                None => {
                    return Err(LifecycleError::InvalidStageAdvance {
                        from: from.display_name().to_string(),
                        expected_from: "Already at final stage (Evolve)".into(),
                    });
                }
            }

            // Gate for the stage being left must be approved
            let gate_for_stage = gate_for_stage(&state.current_stage);
            if let Some(gate) = gate_for_stage {
                let gate_record = state.gates.iter().find(|g| g.gate == gate);
                match gate_record {
                    Some(g) if g.status == GateStatus::Approved => { /* OK */ }
                    Some(g) => {
                        return Err(LifecycleError::GateNotApproved {
                            gate: gate.display_name().to_string(),
                            status: format!("{:?}", g.status),
                        });
                    }
                    None => {
                        return Err(LifecycleError::GateNotApproved {
                            gate: gate.display_name().to_string(),
                            status: "not requested".into(),
                        });
                    }
                }
            }
        }

        LifecycleEvent::GateApproved { gate, .. } => {
            // Gate must have been requested first (or be the first approval)
            // We allow direct approval without explicit request for UX simplicity
            let _ = gate; // validated by derive_state consistency
        }

        LifecycleEvent::GateRejected { gate, .. } => {
            let _ = gate;
        }

        LifecycleEvent::GateRequested { gate, .. } => {
            let _ = gate;
        }

        LifecycleEvent::ArtifactCreated { artifact_type, .. } => {
            // Artifact must belong to a stage that has been entered
            let stage = artifact_type.stage();
            let stage_entered = has_entered_stage(&state, &stage);
            if !stage_entered && state.current_stage != stage {
                return Err(LifecycleError::ArtifactBeforeStage {
                    artifact: format!("{:?}", artifact_type),
                    stage: stage.display_name().to_string(),
                    current: state.current_stage.display_name().to_string(),
                });
            }
        }

        LifecycleEvent::ArtifactUpdated { .. } => { /* Always allowed */ }
    }

    Ok(())
}

/// Get the gate that guards exit from a stage.
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

fn has_entered_stage(state: &LifecycleState, stage: &LifecycleStage) -> bool {
    // We consider a stage entered if it's the current stage or a previous stage
    stage.index() <= state.current_stage.index()
}

// ---------------------------------------------------------------------------
// State derivation
// ---------------------------------------------------------------------------

/// Derive current state by replaying all events.
/// This is the single source of truth for lifecycle state.
pub fn derive_state(project_root: &Path) -> Result<LifecycleState, LifecycleError> {
    let events = read_events(project_root)?;

    let mut state = LifecycleState {
        started: false,
        current_stage: LifecycleStage::Orient,
        artifacts: Vec::new(),
        gates: Vec::new(),
        events_count: events.len(),
    };

    for event in &events {
        match event {
            LifecycleEvent::Started { .. } => {
                state.started = true;
            }

            LifecycleEvent::StageEntered { stage, .. } => {
                state.current_stage = *stage;
            }

            LifecycleEvent::StageAdvanced { to, .. } => {
                state.current_stage = *to;
            }

            LifecycleEvent::ArtifactCreated {
                artifact_type,
                path,
                feature_id,
                timestamp,
                ..
            } => {
                state.artifacts.push(ArtifactRecord {
                    artifact_type: artifact_type.clone(),
                    path: path.clone(),
                    feature_id: feature_id.clone(),
                    created_at: timestamp.clone(),
                    updated_at: None,
                });
            }

            LifecycleEvent::ArtifactUpdated {
                artifact_type,
                path,
                feature_id,
                timestamp,
                ..
            } => {
                // Find matching artifact and update its updated_at
                for artifact in &mut state.artifacts {
                    if artifact.artifact_type == *artifact_type
                        && artifact.path == *path
                        && artifact.feature_id == *feature_id
                    {
                        artifact.updated_at = Some(timestamp.clone());
                    }
                }
            }

            LifecycleEvent::GateRequested {
                gate,
                feature_id,
                timestamp,
                ..
            } => {
                // Update or create gate record
                let existing = state
                    .gates
                    .iter_mut()
                    .find(|g| g.gate == *gate && g.feature_id == *feature_id);

                if let Some(g) = existing {
                    g.status = GateStatus::Requested;
                    g.requested_at = Some(timestamp.clone());
                } else {
                    state.gates.push(GateRecord {
                        gate: gate.clone(),
                        feature_id: feature_id.clone(),
                        status: GateStatus::Requested,
                        requested_at: Some(timestamp.clone()),
                        decided_at: None,
                        decided_by: None,
                        rejection_reason: None,
                    });
                }
            }

            LifecycleEvent::GateApproved {
                gate,
                feature_id,
                timestamp,
                actor,
                ..
            } => {
                let existing = state
                    .gates
                    .iter_mut()
                    .find(|g| g.gate == *gate && g.feature_id == *feature_id);

                if let Some(g) = existing {
                    g.status = GateStatus::Approved;
                    g.decided_at = Some(timestamp.clone());
                    g.decided_by = Some(actor.clone());
                } else {
                    state.gates.push(GateRecord {
                        gate: gate.clone(),
                        feature_id: feature_id.clone(),
                        status: GateStatus::Approved,
                        requested_at: None,
                        decided_at: Some(timestamp.clone()),
                        decided_by: Some(actor.clone()),
                        rejection_reason: None,
                    });
                }
            }

            LifecycleEvent::GateRejected {
                gate,
                feature_id,
                timestamp,
                actor,
                reason,
                ..
            } => {
                let existing = state
                    .gates
                    .iter_mut()
                    .find(|g| g.gate == *gate && g.feature_id == *feature_id);

                if let Some(g) = existing {
                    g.status = GateStatus::Rejected;
                    g.decided_at = Some(timestamp.clone());
                    g.decided_by = Some(actor.clone());
                    g.rejection_reason = Some(reason.clone());
                } else {
                    state.gates.push(GateRecord {
                        gate: gate.clone(),
                        feature_id: feature_id.clone(),
                        status: GateStatus::Rejected,
                        requested_at: None,
                        decided_at: Some(timestamp.clone()),
                        decided_by: Some(actor.clone()),
                        rejection_reason: Some(reason.clone()),
                    });
                }
            }
        }
    }

    Ok(state)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl GateId {
    pub fn display_name(&self) -> &'static str {
        match self {
            GateId::OrientUnderstandingConfirmed => "Orient: Understanding Confirmed",
            GateId::DiscoverIntakeComplete => "Discover: Intake Complete",
            GateId::PrdApproval => "Define: PRD Approved",
            GateId::DesignReview => "Design: Review Complete",
            GateId::IssueGraphApproval => "Plan: Issue Graph Approved",
            GateId::FileScopeConfirmed => "Prepare: File Scope Confirmed",
            GateId::PatchApproval => "Build: Patch Approved",
            GateId::QaReportAccepted => "Verify: QA Report Accepted",
            GateId::ReviewAccepted => "Review: Findings Accepted",
            GateId::ReleaseIntegrityVerified => "Ship: Release Integrity Verified",
            GateId::EvidenceAccepted => "Observe: Evidence Accepted",
            GateId::LearningDecisionMade => "Learn: Decision Made",
            GateId::CyclePlanApproved => "Evolve: Cycle Plan Approved",
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum LifecycleError {
    NotInitialized,
    Io(String),
    Serialization(String),
    CorruptEventLog { line: usize, message: String },
    InvalidStageEntry { reason: String },
    InvalidStageAdvance { from: String, expected_from: String },
    GateNotApproved { gate: String, status: String },
    ArtifactBeforeStage {
        artifact: String,
        stage: String,
        current: String,
    },
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::NotInitialized => {
                write!(f, "Lifecycle not initialized for this project")
            }
            LifecycleError::Io(msg) => write!(f, "IO error: {}", msg),
            LifecycleError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            LifecycleError::CorruptEventLog { line, message } => {
                write!(f, "Corrupt event log at line {}: {}", line, message)
            }
            LifecycleError::InvalidStageEntry { reason } => {
                write!(f, "Invalid stage entry: {}", reason)
            }
            LifecycleError::InvalidStageAdvance { from, expected_from } => {
                write!(
                    f,
                    "Invalid stage advance from '{}': {}",
                    from, expected_from
                )
            }
            LifecycleError::GateNotApproved { gate, status } => {
                write!(f, "Gate '{}' not approved (status: {})", gate, status)
            }
            LifecycleError::ArtifactBeforeStage {
                artifact,
                stage,
                current,
            } => {
                write!(
                    f,
                    "Cannot create artifact {} for stage {} — current stage is {}",
                    artifact, stage, current
                )
            }
        }
    }
}

impl std::error::Error for LifecycleError {}
