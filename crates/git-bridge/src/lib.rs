pub mod backfill;
pub mod commit;
pub mod commits;
pub mod diff;
pub mod error;
pub mod gix_ops;
pub mod semantic;
pub mod semantic_prep;
pub mod snapshot;
pub mod status;

pub use backfill::{backfill_semantic_stub, BackfillRequest, BackfillResult};
pub use commit::{semantic_commit, semantic_commit_native, CommitRequest, CommitResult, NativeCommitRequest, NativeCommitResult};
pub use commits::{GitCommitSummary, recent_commits};
pub use diff::{GitDiffStat, diff_stat};
pub use error::GitBridgeError;
pub use semantic::{AuthorType, SemanticCommitObject, SemanticPayload};
pub use semantic_prep::{
    AgentGitContext, CommitGroup, DiffStatSummary, RiskSummary,
    SemanticCommitInput, SemanticCommitProposal,
    build_agent_context, build_semantic_commit_input, prepare_semantic_commit,
};
pub use snapshot::{GitRepositorySnapshot, GitChangedFile, GitHeadMetadata, repository_snapshot};
pub use status::update_task_status;
pub use gix_ops::{NativeGitStatus, native_git_status};
