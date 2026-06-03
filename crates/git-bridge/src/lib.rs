pub mod backfill;
pub mod commit;
pub mod error;
pub mod gix_ops;
pub mod semantic;
pub mod status;

pub use backfill::{backfill_semantic_stub, BackfillRequest, BackfillResult};
pub use commit::{semantic_commit, semantic_commit_native, CommitRequest, CommitResult, NativeCommitRequest, NativeCommitResult};
pub use error::GitBridgeError;
pub use semantic::{AuthorType, SemanticCommitObject, SemanticPayload};
pub use status::update_task_status;
pub use gix_ops::{NativeGitStatus, native_git_status};
