pub mod backfill;
pub mod commit;
pub mod commits;
pub mod diff;
pub mod error;
pub mod operational_risk;
pub mod gix_ops;
pub mod semantic;
pub mod semantic_prep;
pub mod snapshot;
pub mod status;

pub use backfill::{backfill_semantic_stub, BackfillRequest, BackfillResult};
pub use commit::{semantic_commit, semantic_commit_native, CommitRequest, CommitResult, NativeCommitRequest, NativeCommitResult};
pub use commits::{GitCommitSummary, RecentCommitsResult, recent_commits, recent_commits_with_provider};
pub use diff::{GitDiffStat, DiffStatResult, diff_stat, diff_stat_with_provider};
pub use error::GitBridgeError;
pub use semantic::{AuthorType, SemanticCommitObject, SemanticPayload};
pub use semantic_prep::{
    AgentContextV2, AgentGitContext, ChangedFileSummary, CommitGroup, ContentPolicy, DiffStatSummary,
    ProposalSummary, RiskSummary, SafeDiffContext, SafeDiffFile, SafeDiffHunk,
    SafeDiffPolicy, SafeDiffSummary,
    SemanticCommitInput, SemanticCommitProposal,
    build_agent_context, build_agent_context_v2, build_safe_diff_context,
    build_semantic_commit_input, check_diff_eligibility, parse_hunk_header,
    prepare_semantic_commit,
};
pub use snapshot::{GitRepositorySnapshot, GitChangedFile, GitHeadMetadata, repository_snapshot};
pub use status::update_task_status;
pub use gix_ops::{NativeGitStatus, native_git_status, GitProvider, GitProviderReport, GitOperationProvider, build_provider_report, get_head_hash};
