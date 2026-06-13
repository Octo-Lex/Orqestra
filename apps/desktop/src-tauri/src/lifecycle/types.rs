//! Orqestra Development Lifecycle — Type definitions (v0.2.0)
//!
//! Core enums and structs for the 13-stage lifecycle model.
//! All lifecycle state is derived from events. These types define
//! what events are valid, what artifacts exist, and what gates apply.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Lifecycle Stages
// ---------------------------------------------------------------------------

/// The 13 lifecycle stages from Orient through Evolve.
/// Order matters — stages advance sequentially.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    Orient,
    Discover,
    Define,
    Design,
    Plan,
    Prepare,
    Build,
    Verify,
    Review,
    Ship,
    Observe,
    Learn,
    Evolve,
}

impl LifecycleStage {
    /// All stages in order.
    pub fn all() -> &'static [LifecycleStage] {
        &[
            LifecycleStage::Orient,
            LifecycleStage::Discover,
            LifecycleStage::Define,
            LifecycleStage::Design,
            LifecycleStage::Plan,
            LifecycleStage::Prepare,
            LifecycleStage::Build,
            LifecycleStage::Verify,
            LifecycleStage::Review,
            LifecycleStage::Ship,
            LifecycleStage::Observe,
            LifecycleStage::Learn,
            LifecycleStage::Evolve,
        ]
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            LifecycleStage::Orient => "Orient",
            LifecycleStage::Discover => "Discover",
            LifecycleStage::Define => "Define",
            LifecycleStage::Design => "Design",
            LifecycleStage::Plan => "Plan",
            LifecycleStage::Prepare => "Prepare",
            LifecycleStage::Build => "Build",
            LifecycleStage::Verify => "Verify",
            LifecycleStage::Review => "Review",
            LifecycleStage::Ship => "Ship",
            LifecycleStage::Observe => "Observe",
            LifecycleStage::Learn => "Learn",
            LifecycleStage::Evolve => "Evolve",
        }
    }

    /// Index in the stage sequence (0-based).
    pub fn index(&self) -> usize {
        LifecycleStage::all()
            .iter()
            .position(|s| s == self)
            .unwrap_or(0)
    }

    /// Next stage in sequence, or None if at Evolve.
    pub fn next(&self) -> Option<LifecycleStage> {
        let all = LifecycleStage::all();
        let idx = self.index();
        if idx + 1 < all.len() {
            Some(all[idx + 1])
        } else {
            None
        }
    }

    /// Previous stage in sequence, or None if at Orient.
    pub fn prev(&self) -> Option<LifecycleStage> {
        if self.index() == 0 {
            None
        } else {
            Some(LifecycleStage::all()[self.index() - 1])
        }
    }

    /// Stages that are implemented in v2.15.0 (lifecycle foundation).
    /// Other stages are placeholders that render but can't advance.
    pub fn is_implemented(&self) -> bool {
        matches!(
            self,
            LifecycleStage::Orient | LifecycleStage::Discover | LifecycleStage::Define | LifecycleStage::Plan
        )
    }

    /// Human-readable description of what this stage answers.
    pub fn purpose(&self) -> &'static str {
        match self {
            LifecycleStage::Orient => "What is this project? What exists? What is safe to touch?",
            LifecycleStage::Discover => "What problem are we solving, for whom, and why now?",
            LifecycleStage::Define => "What exactly are we building, and what is out of scope?",
            LifecycleStage::Design => "What is the user experience, system shape, and technical approach?",
            LifecycleStage::Plan => "How do we build this safely, in what order, with what checks?",
            LifecycleStage::Prepare => "What files, commands, tests, and constraints apply to this slice?",
            LifecycleStage::Build => "How should the repo change?",
            LifecycleStage::Verify => "What evidence shows this works or does not work?",
            LifecycleStage::Review => "Is the work safe, simple, maintainable, and aligned with the spec?",
            LifecycleStage::Ship => "What exactly shipped, from which commit, with what artifacts and evidence?",
            LifecycleStage::Observe => "What happened after release?",
            LifecycleStage::Learn => "What did we learn, and what changes because of it?",
            LifecycleStage::Evolve => "What is the next most important improvement?",
        }
    }
}

// ---------------------------------------------------------------------------
// Artifact Types
// ---------------------------------------------------------------------------

/// Typed artifact identifiers — each maps to a specific file in `.Orqestra/lifecycle/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    // Orient
    ProjectProfile,
    ArchitectureMap,
    StackMap,
    RepoMap,
    Conventions,
    RiskMap,
    // Discover
    ProblemBrief,
    UserSegment,
    Constraints,
    Assumptions,
    OpenQuestions,
    // Define
    Prd,
    AcceptanceCriteria,
    NonScope,
    SuccessMeasures,
    // Design
    UxFlow,
    InterfaceContracts,
    Adr,
    TechnicalDesign,
    SecurityBoundaries,
    // Plan
    IssueGraph,
    TaskSlices,
    DependencyMap,
    TestPlan,
    RolloutPlan,
    // Prepare
    FileScope,
    CommandPlan,
    TestCommandMap,
    RollbackPlan,
    // Build
    PatchProposal,
    ImplementationNotes,
    ChangedFiles,
    Rationale,
    // Verify
    TestResults,
    QaReport,
    VerificationLog,
    KnownFailures,
    // Review
    ReviewReport,
    SecurityReview,
    SimplificationReport,
    UnresolvedRisks,
    // Ship
    ReleaseManifest,
    Checksums,
    ReleaseNotes,
    ArtifactList,
    Provenance,
    // Observe
    BetaFeedback,
    DiagnosticsSummary,
    FailureTaxonomy,
    SessionOutcome,
    UserFrictionReport,
    // Learn
    LearningSummary,
    AcceptedEvidence,
    RejectedEvidence,
    DecisionLog,
    RoadmapAdjustment,
    // Evolve
    NextCyclePlan,
    PrioritizedBacklog,
    DebtRegister,
    RiskRegister,
}

impl ArtifactType {
    /// Which stage this artifact belongs to.
    pub fn stage(&self) -> LifecycleStage {
        match self {
            // Orient
            ArtifactType::ProjectProfile
            | ArtifactType::ArchitectureMap
            | ArtifactType::StackMap
            | ArtifactType::RepoMap
            | ArtifactType::Conventions
            | ArtifactType::RiskMap => LifecycleStage::Orient,

            // Discover
            ArtifactType::ProblemBrief
            | ArtifactType::UserSegment
            | ArtifactType::Constraints
            | ArtifactType::Assumptions
            | ArtifactType::OpenQuestions => LifecycleStage::Discover,

            // Define
            ArtifactType::Prd
            | ArtifactType::AcceptanceCriteria
            | ArtifactType::NonScope
            | ArtifactType::SuccessMeasures => LifecycleStage::Define,

            // Design
            ArtifactType::UxFlow
            | ArtifactType::InterfaceContracts
            | ArtifactType::Adr
            | ArtifactType::TechnicalDesign
            | ArtifactType::SecurityBoundaries => LifecycleStage::Design,

            // Plan
            ArtifactType::IssueGraph
            | ArtifactType::TaskSlices
            | ArtifactType::DependencyMap
            | ArtifactType::TestPlan
            | ArtifactType::RolloutPlan => LifecycleStage::Plan,

            // Prepare
            ArtifactType::FileScope
            | ArtifactType::CommandPlan
            | ArtifactType::TestCommandMap
            | ArtifactType::RollbackPlan => LifecycleStage::Prepare,

            // Build
            ArtifactType::PatchProposal
            | ArtifactType::ImplementationNotes
            | ArtifactType::ChangedFiles
            | ArtifactType::Rationale => LifecycleStage::Build,

            // Verify
            ArtifactType::TestResults
            | ArtifactType::QaReport
            | ArtifactType::VerificationLog
            | ArtifactType::KnownFailures => LifecycleStage::Verify,

            // Review
            ArtifactType::ReviewReport
            | ArtifactType::SecurityReview
            | ArtifactType::SimplificationReport
            | ArtifactType::UnresolvedRisks => LifecycleStage::Review,

            // Ship
            ArtifactType::ReleaseManifest
            | ArtifactType::Checksums
            | ArtifactType::ReleaseNotes
            | ArtifactType::ArtifactList
            | ArtifactType::Provenance => LifecycleStage::Ship,

            // Observe
            ArtifactType::BetaFeedback
            | ArtifactType::DiagnosticsSummary
            | ArtifactType::FailureTaxonomy
            | ArtifactType::SessionOutcome
            | ArtifactType::UserFrictionReport => LifecycleStage::Observe,

            // Learn
            ArtifactType::LearningSummary
            | ArtifactType::AcceptedEvidence
            | ArtifactType::RejectedEvidence
            | ArtifactType::DecisionLog
            | ArtifactType::RoadmapAdjustment => LifecycleStage::Learn,

            // Evolve
            ArtifactType::NextCyclePlan
            | ArtifactType::PrioritizedBacklog
            | ArtifactType::DebtRegister
            | ArtifactType::RiskRegister => LifecycleStage::Evolve,
        }
    }

    /// File extension for this artifact type.
    pub fn file_extension(&self) -> &'static str {
        match self {
            ArtifactType::AcceptanceCriteria
            | ArtifactType::Assumptions
            | ArtifactType::IssueGraph
            | ArtifactType::TaskSlices
            | ArtifactType::DependencyMap
            | ArtifactType::FileScope
            | ArtifactType::CommandPlan
            | ArtifactType::TestCommandMap
            | ArtifactType::ChangedFiles
            | ArtifactType::TestResults
            | ArtifactType::VerificationLog
            | ArtifactType::UnresolvedRisks
            | ArtifactType::ReleaseManifest
            | ArtifactType::ArtifactList
            | ArtifactType::Provenance
            | ArtifactType::DiagnosticsSummary
            | ArtifactType::FailureTaxonomy
            | ArtifactType::SessionOutcome
            | ArtifactType::AcceptedEvidence
            | ArtifactType::RejectedEvidence
            | ArtifactType::PrioritizedBacklog
            | ArtifactType::ProjectProfile
            | ArtifactType::RepoMap
            | ArtifactType::InterfaceContracts => "json",

            ArtifactType::PatchProposal => "diff",
            ArtifactType::Checksums => "txt",

            _ => "md",
        }
    }
}

// ---------------------------------------------------------------------------
// Roles
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleId {
    ProductManager,
    UxDesigner,
    Architect,
    TechLead,
    ImplementationAgent,
    QaAgent,
    SecurityReviewer,
    ReleaseEvidenceAgent,
}

impl RoleId {
    pub fn display_name(&self) -> &'static str {
        match self {
            RoleId::ProductManager => "Product Manager",
            RoleId::UxDesigner => "UX Designer",
            RoleId::Architect => "Architect",
            RoleId::TechLead => "Tech Lead",
            RoleId::ImplementationAgent => "Implementation Agent",
            RoleId::QaAgent => "QA Agent",
            RoleId::SecurityReviewer => "Security Reviewer",
            RoleId::ReleaseEvidenceAgent => "Release/Evidence Agent",
        }
    }
}

// ---------------------------------------------------------------------------
// Gates
// ---------------------------------------------------------------------------

/// Approval gates — each stage has a gate that must be approved before advancing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateId {
    /// Orient: Human confirms project understanding
    OrientUnderstandingConfirmed,
    /// Discover: Assumptions and open questions are explicit
    DiscoverIntakeComplete,
    /// Define: Human approves PRD
    PrdApproval,
    /// Design: Design reviewed
    DesignReview,
    /// Plan: Human approves issue graph
    IssueGraphApproval,
    /// Prepare: File scope confirmed
    FileScopeConfirmed,
    /// Build: Patch proposal reviewed
    PatchApproval,
    /// Verify: Mechanical evidence captured
    QaReportAccepted,
    /// Review: Human accepts/rejects findings
    ReviewAccepted,
    /// Ship: Release claims match artifacts
    ReleaseIntegrityVerified,
    /// Observe: Consented, redacted evidence
    EvidenceAccepted,
    /// Learn: Human decides on evidence
    LearningDecisionMade,
    /// Evolve: Next cycle from accepted state
    CyclePlanApproved,
}

impl GateId {
    /// Which stage this gate guards the exit of.
    pub fn stage(&self) -> LifecycleStage {
        match self {
            GateId::OrientUnderstandingConfirmed => LifecycleStage::Orient,
            GateId::DiscoverIntakeComplete => LifecycleStage::Discover,
            GateId::PrdApproval => LifecycleStage::Define,
            GateId::DesignReview => LifecycleStage::Design,
            GateId::IssueGraphApproval => LifecycleStage::Plan,
            GateId::FileScopeConfirmed => LifecycleStage::Prepare,
            GateId::PatchApproval => LifecycleStage::Build,
            GateId::QaReportAccepted => LifecycleStage::Verify,
            GateId::ReviewAccepted => LifecycleStage::Review,
            GateId::ReleaseIntegrityVerified => LifecycleStage::Ship,
            GateId::EvidenceAccepted => LifecycleStage::Observe,
            GateId::LearningDecisionMade => LifecycleStage::Learn,
            GateId::CyclePlanApproved => LifecycleStage::Evolve,
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Lifecycle events — append-only, never mutated.
/// State is derived by replaying these.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum LifecycleEvent {
    /// Emitted when a lifecycle session is started for a project.
    #[serde(rename = "lifecycle.started")]
    Started {
        project_root: String,
        timestamp: String,
    },

    /// Emitted when entering a lifecycle stage.
    #[serde(rename = "lifecycle.stage.entered")]
    StageEntered {
        stage: LifecycleStage,
        feature_id: Option<String>,
        timestamp: String,
        actor: String,
    },

    /// Emitted when an artifact is created.
    #[serde(rename = "artifact.created")]
    ArtifactCreated {
        artifact_type: ArtifactType,
        path: String,
        feature_id: Option<String>,
        timestamp: String,
        actor: String,
    },

    /// Emitted when an artifact is updated.
    #[serde(rename = "artifact.updated")]
    ArtifactUpdated {
        artifact_type: ArtifactType,
        path: String,
        feature_id: Option<String>,
        timestamp: String,
        actor: String,
    },

    /// Emitted when a gate is requested (user asked to advance).
    #[serde(rename = "gate.requested")]
    GateRequested {
        gate: GateId,
        feature_id: Option<String>,
        timestamp: String,
        actor: String,
    },

    /// Emitted when a gate is approved (human accepted).
    #[serde(rename = "gate.approved")]
    GateApproved {
        gate: GateId,
        feature_id: Option<String>,
        timestamp: String,
        actor: String,
    },

    /// Emitted when a gate is rejected (human declined).
    #[serde(rename = "gate.rejected")]
    GateRejected {
        gate: GateId,
        feature_id: Option<String>,
        timestamp: String,
        actor: String,
        reason: String,
    },

    /// Emitted when advancing from one stage to the next.
    #[serde(rename = "lifecycle.stage.advanced")]
    StageAdvanced {
        from: LifecycleStage,
        to: LifecycleStage,
        feature_id: Option<String>,
        timestamp: String,
        actor: String,
    },
}

// ---------------------------------------------------------------------------
// Derived State
// ---------------------------------------------------------------------------

/// State derived from replaying events. Never persisted directly —
/// always recomputed from the event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleState {
    pub started: bool,
    pub current_stage: LifecycleStage,
    pub artifacts: Vec<ArtifactRecord>,
    pub gates: Vec<GateRecord>,
    pub events_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub artifact_type: ArtifactType,
    pub path: String,
    pub feature_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRecord {
    pub gate: GateId,
    pub feature_id: Option<String>,
    pub status: GateStatus,
    pub requested_at: Option<String>,
    pub decided_at: Option<String>,
    pub decided_by: Option<String>,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    /// Not yet requested
    Pending,
    /// User asked to advance — waiting for decision
    Requested,
    /// Human approved
    Approved,
    /// Human rejected
    Rejected,
}
