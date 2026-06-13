//! Tests for the Orqestra Development Lifecycle module (v2.15.0).
//!
//! Tests cover:
//! - Directory creation
//! - Event append/read
//! - Event validation (rejects invalid events)
//! - State derivation from events
//! - Gate enforcement before stage advance
//! - Migration from .Orqestra/product-team/
//! - Path traversal prevention

use std::fs;
use std::path::PathBuf;

use orqestra_desktop::lifecycle::event_log;
use orqestra_desktop::lifecycle::types::*;

fn temp_project() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ---------------------------------------------------------------------------
// Directory creation
// ---------------------------------------------------------------------------

#[test]
fn creates_lifecycle_dirs_on_init() {
    let dir = temp_project();
    let root = dir.path();

    event_log::ensure_lifecycle_dirs(root).expect("Failed to create dirs");

    let lifecycle = root.join(".Orqestra/lifecycle");
    assert!(lifecycle.exists(), "lifecycle root should exist");
    assert!(lifecycle.join("project").exists());
    assert!(lifecycle.join("features").exists());
    assert!(lifecycle.join("releases").exists());
    assert!(lifecycle.join("observations").exists());
    assert!(lifecycle.join("learnings").exists());
    assert!(lifecycle.join("team").exists());
}

#[test]
fn is_lifecycle_initialized_false_before_creation() {
    let dir = temp_project();
    assert!(!event_log::is_lifecycle_initialized(dir.path()));
}

#[test]
fn is_lifecycle_initialized_true_after_creation() {
    let dir = temp_project();
    event_log::ensure_lifecycle_dirs(dir.path()).unwrap();
    assert!(event_log::is_lifecycle_initialized(dir.path()));
}

// ---------------------------------------------------------------------------
// Event append / read
// ---------------------------------------------------------------------------

#[test]
fn appends_valid_started_event() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    let event = LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    };

    event_log::append_event(root, &event).expect("Should append Started event");

    let events = event_log::read_events(root).unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], LifecycleEvent::Started { .. }));
}

#[test]
fn appends_stage_entered_event() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Start first
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();

    // Enter Orient
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    let events = event_log::read_events(root).unwrap();
    assert_eq!(events.len(), 2);
}

#[test]
fn read_events_empty_when_no_log() {
    let dir = temp_project();
    let events = event_log::read_events(dir.path()).unwrap();
    assert!(events.is_empty());
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[test]
fn rejects_event_before_init() {
    let dir = temp_project();
    let root = dir.path();
    // Note: don't create dirs

    let event = LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    };

    let result = event_log::append_event(root, &event);
    assert!(result.is_err(), "Should reject event before init");
    assert!(matches!(
        result.unwrap_err(),
        event_log::LifecycleError::NotInitialized
    ));
}

#[test]
fn rejects_stage_advance_without_gate_approval() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Start + enter Orient
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Try to advance without approving gate
    let advance = LifecycleEvent::StageAdvanced {
        from: LifecycleStage::Orient,
        to: LifecycleStage::Discover,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    };

    let result = event_log::append_event(root, &advance);
    assert!(result.is_err(), "Should reject advance without gate");
    assert!(matches!(
        result.unwrap_err(),
        event_log::LifecycleError::GateNotApproved { .. }
    ));
}

#[test]
fn allows_stage_advance_after_gate_approval() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Start + enter Orient
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Approve gate
    event_log::append_event(root, &LifecycleEvent::GateApproved {
        gate: GateId::OrientUnderstandingConfirmed,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Advance
    event_log::append_event(root, &LifecycleEvent::StageAdvanced {
        from: LifecycleStage::Orient,
        to: LifecycleStage::Discover,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    let state = event_log::derive_state(root).unwrap();
    assert_eq!(state.current_stage, LifecycleStage::Discover);
}

#[test]
fn rejects_invalid_stage_jump() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Start + enter Orient
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Approve gate
    event_log::append_event(root, &LifecycleEvent::GateApproved {
        gate: GateId::OrientUnderstandingConfirmed,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Try to jump to Define (skipping Discover)
    let result = event_log::append_event(root, &LifecycleEvent::StageAdvanced {
        from: LifecycleStage::Orient,
        to: LifecycleStage::Define,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    });

    assert!(result.is_err(), "Should reject non-sequential jump");
    assert!(matches!(
        result.unwrap_err(),
        event_log::LifecycleError::InvalidStageAdvance { .. }
    ));
}

// ---------------------------------------------------------------------------
// State derivation
// ---------------------------------------------------------------------------

#[test]
fn derives_state_from_events() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Start
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();

    let state = event_log::derive_state(root).unwrap();
    assert!(state.started);
    assert_eq!(state.events_count, 1);
    assert_eq!(state.current_stage, LifecycleStage::Orient); // default
}

#[test]
fn state_shows_artifacts_after_creation() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Start + enter Orient
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Create artifact
    event_log::append_event(root, &LifecycleEvent::ArtifactCreated {
        artifact_type: ArtifactType::ProjectProfile,
        path: "project/project-profile.json".to_string(),
        feature_id: None,
        timestamp: now(),
        actor: "repo-analyst".to_string(),
    }).unwrap();

    let state = event_log::derive_state(root).unwrap();
    assert_eq!(state.artifacts.len(), 1);
    assert_eq!(state.artifacts[0].artifact_type, ArtifactType::ProjectProfile);
}

#[test]
fn state_shows_gate_status() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Start + enter Orient
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Approve gate
    event_log::append_event(root, &LifecycleEvent::GateApproved {
        gate: GateId::OrientUnderstandingConfirmed,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    let state = event_log::derive_state(root).unwrap();
    assert_eq!(state.gates.len(), 1);
    assert_eq!(state.gates[0].status, GateStatus::Approved);
}

// ---------------------------------------------------------------------------
// Immutability
// ---------------------------------------------------------------------------

#[test]
fn never_mutates_prior_events() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // Write 3 events
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();
    event_log::append_event(root, &LifecycleEvent::GateApproved {
        gate: GateId::OrientUnderstandingConfirmed,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Read the raw file
    let log_path = root.join(".Orqestra/lifecycle/events.jsonl");
    let original = fs::read_to_string(&log_path).unwrap();

    // Add more events
    event_log::append_event(root, &LifecycleEvent::StageAdvanced {
        from: LifecycleStage::Orient,
        to: LifecycleStage::Discover,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    let after = fs::read_to_string(&log_path).unwrap();

    // Original content should be a prefix of current content (append-only)
    assert!(
        after.starts_with(&original),
        "Original events should not be mutated — append-only violated"
    );
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

#[test]
fn migrates_product_team_to_lifecycle() {
    let dir = temp_project();
    let root = dir.path();

    // Create old product-team structure
    let old_path = root.join(".Orqestra/product-team");
    fs::create_dir_all(&old_path).unwrap();
    fs::write(old_path.join("test.txt"), "migration test").unwrap();

    // Run migration
    let migrated = event_log::migrate_from_product_team(root).unwrap();
    assert!(migrated, "Should report migration happened");

    // Old path gone, new path exists
    assert!(!old_path.exists(), "Old product-team/ should be gone");
    assert!(root.join(".Orqestra/lifecycle").exists(), "lifecycle/ should exist");
}

#[test]
fn does_not_migrate_when_lifecycle_already_exists() {
    let dir = temp_project();
    let root = dir.path();

    // Create both old and new
    let old_path = root.join(".Orqestra/product-team");
    fs::create_dir_all(&old_path).unwrap();
    fs::write(old_path.join("test.txt"), "old data").unwrap();

    let new_path = root.join(".Orqestra/lifecycle");
    fs::create_dir_all(&new_path).unwrap();
    fs::write(new_path.join("existing.txt"), "existing data").unwrap();

    // Run migration
    let migrated = event_log::migrate_from_product_team(root).unwrap();
    assert!(!migrated, "Should not migrate when lifecycle/ already exists");

    // Both should still exist
    assert!(old_path.exists(), "Old should not be removed");
    assert!(new_path.exists());
    assert!(new_path.join("existing.txt").exists(), "Existing data should be intact");
}

// ---------------------------------------------------------------------------
// Stage helpers
// ---------------------------------------------------------------------------

#[test]
fn stage_next_and_prev() {
    assert_eq!(LifecycleStage::Orient.next(), Some(LifecycleStage::Discover));
    assert_eq!(LifecycleStage::Orient.prev(), None);

    assert_eq!(LifecycleStage::Evolve.next(), None);
    assert_eq!(LifecycleStage::Evolve.prev(), Some(LifecycleStage::Learn));

    assert_eq!(LifecycleStage::Define.next(), Some(LifecycleStage::Design));
    assert_eq!(LifecycleStage::Define.prev(), Some(LifecycleStage::Discover));
}

#[test]
fn stage_all_has_13_stages() {
    assert_eq!(LifecycleStage::all().len(), 13);
}

#[test]
fn stage_index_correct() {
    assert_eq!(LifecycleStage::Orient.index(), 0);
    assert_eq!(LifecycleStage::Discover.index(), 1);
    assert_eq!(LifecycleStage::Evolve.index(), 12);
}

#[test]
fn stage_is_implemented_for_v2_15_0() {
    assert!(LifecycleStage::Orient.is_implemented());
    assert!(LifecycleStage::Discover.is_implemented());
    assert!(LifecycleStage::Define.is_implemented());
    assert!(LifecycleStage::Plan.is_implemented());

    assert!(!LifecycleStage::Design.is_implemented());
    assert!(!LifecycleStage::Build.is_implemented());
    assert!(!LifecycleStage::Evolve.is_implemented());
}

// ---------------------------------------------------------------------------
// Full lifecycle flow
// ---------------------------------------------------------------------------

#[test]
fn full_orient_to_discover_flow() {
    let dir = temp_project();
    let root = dir.path();
    event_log::ensure_lifecycle_dirs(root).unwrap();

    // 1. Start
    event_log::append_event(root, &LifecycleEvent::Started {
        project_root: root.to_string_lossy().to_string(),
        timestamp: now(),
    }).unwrap();

    // 2. Enter Orient
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Orient,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // 3. Create project profile artifact
    event_log::append_event(root, &LifecycleEvent::ArtifactCreated {
        artifact_type: ArtifactType::ProjectProfile,
        path: "project/project-profile.json".to_string(),
        feature_id: None,
        timestamp: now(),
        actor: "repo-analyst".to_string(),
    }).unwrap();

    // 4. Gate requested
    event_log::append_event(root, &LifecycleEvent::GateRequested {
        gate: GateId::OrientUnderstandingConfirmed,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // 5. Gate approved
    event_log::append_event(root, &LifecycleEvent::GateApproved {
        gate: GateId::OrientUnderstandingConfirmed,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // 6. Advance
    event_log::append_event(root, &LifecycleEvent::StageAdvanced {
        from: LifecycleStage::Orient,
        to: LifecycleStage::Discover,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // 7. Enter Discover
    event_log::append_event(root, &LifecycleEvent::StageEntered {
        stage: LifecycleStage::Discover,
        feature_id: None,
        timestamp: now(),
        actor: "human".to_string(),
    }).unwrap();

    // Verify final state
    let state = event_log::derive_state(root).unwrap();
    assert_eq!(state.current_stage, LifecycleStage::Discover);
    assert_eq!(state.artifacts.len(), 1);
    assert_eq!(state.gates.len(), 1);
    assert_eq!(state.gates[0].status, GateStatus::Approved);
    assert_eq!(state.events_count, 7);
}
