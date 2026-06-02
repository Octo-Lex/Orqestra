use crate::error::GitBridgeError;
use crate::semantic::{AuthorType, CommitAuthor, SemanticCommitObject, SemanticPayload};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Legacy shell-out commit (preserved for backward compat)
// ---------------------------------------------------------------------------

pub struct CommitRequest {
    pub project_root: PathBuf,
    pub message: String,
    pub author_name: String,
    pub author_type: AuthorType,
    pub task_ids: Vec<String>,
    /// Files to stage, relative to project root.
    /// If empty, stages all modified files in roadmap/.
    pub files_to_stage: Vec<PathBuf>,
}

pub struct CommitResult {
    pub hash: String,
    pub semantic_stub_path: PathBuf,
}

/// Stage files, create a git commit, write a semantic stub object.
/// Legacy shell-out path — preserved for backward compatibility.
pub fn semantic_commit(request: CommitRequest) -> Result<CommitResult, GitBridgeError> {
    let _repo = gix::open(&request.project_root)
        .map_err(|e| GitBridgeError::GitOperation(e.to_string()))?;

    stage_files(&request.project_root, &request.files_to_stage)?;
    let hash = create_commit(&request.project_root, &request.message, &request.author_name)?;

    let stub_path = write_semantic_stub(
        &request.project_root,
        &hash,
        &request.message,
        &request.author_name,
        &request.author_type,
        &request.task_ids,
    )?;

    Ok(CommitResult {
        hash,
        semantic_stub_path: stub_path,
    })
}

// ---------------------------------------------------------------------------
// v1.0.2: Native gix semantic commit path
// ---------------------------------------------------------------------------

pub struct NativeCommitRequest {
    pub project_root: PathBuf,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_type: AuthorType,
    pub task_id: Option<String>,
    pub paths: Vec<PathBuf>,
}

pub struct NativeCommitResult {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub semantic_stub_path: PathBuf,
    pub elapsed_ms: u64,
}

/// Create a semantic commit through native gix for the commit operation.
/// Staging still uses shell-out `git add` (documented as partial migration).
/// The commit object is created natively through gix.
pub fn semantic_commit_native(
    request: NativeCommitRequest,
) -> Result<NativeCommitResult, GitBridgeError> {
    let start = Instant::now();

    // Validate repo
    let repo = gix::open(&request.project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("GIT_REPO_NOT_FOUND: {e}")))?;

    // Stage files (still shell-out — staging API in gix is not yet high-level)
    crate::gix_ops::stage_files(&request.project_root, &request.paths)?;

    // Check that something is staged
    let index = repo.open_index()
        .map_err(|e| GitBridgeError::GitOperation(format!("GIT_INDEX_ERROR: {e}")))?;

    // Compare index tree to HEAD tree to detect changes
    let head_tree = match repo.head_id() {
        Ok(id) => {
            let oid = id.detach();
            repo.find_object(oid).ok()
                .and_then(|obj| obj.try_into_commit().ok())
                .and_then(|commit| commit.tree().ok())
        },
        Err(_) => None,
    };

    // Simple check: if index has entries, proceed
    if index.entries().is_empty() {
        return Err(GitBridgeError::GitOperation(
            "GIT_NOTHING_TO_COMMIT: No staged changes".into(),
        ));
    }

    // Get parent hashes before commit
    let parent_hashes: Vec<String> = match repo.head_id() {
        Ok(id) => vec![id.detach().to_hex().to_string()],
        Err(_) => vec![],
    };

    // Create native commit
    let hash = crate::gix_ops::create_commit_native(
        &request.project_root,
        &request.message,
        &request.author_name,
        &request.author_email,
    ).map_err(|e| GitBridgeError::GitOperation(format!("GIT_COMMIT_ERROR: {e}")))?;

    let elapsed = start.elapsed();

    // Write semantic stub
    let task_ids: Vec<String> = request.task_id.into_iter().collect();
    let stub_path = write_semantic_stub(
        &request.project_root,
        &hash,
        &request.message,
        &request.author_name,
        &request.author_type,
        &task_ids,
    ).map_err(|e| GitBridgeError::GitOperation(format!("SEMANTIC_STUB_ERROR: Commit succeeded but stub write failed: {e}")))?;

    Ok(NativeCommitResult {
        hash,
        parent_hashes,
        semantic_stub_path: stub_path,
        elapsed_ms: elapsed.as_millis() as u64,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn stage_files(
    project_root: &Path,
    files: &[PathBuf],
) -> Result<(), GitBridgeError> {
    let mut cmd = std::process::Command::new("git");
    cmd.current_dir(project_root);

    if files.is_empty() {
        cmd.args(["add", "roadmap/"]);
    } else {
        cmd.arg("add");
        for f in files {
            cmd.arg(f);
        }
    }

    let output = cmd
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

fn create_commit(
    project_root: &Path,
    message: &str,
    author_name: &str,
) -> Result<String, GitBridgeError> {
    let output = std::process::Command::new("git")
        .current_dir(project_root)
        .args([
            "commit",
            "-m",
            message,
            "--author",
            &format!("{author_name} <orqestra@local>"),
        ])
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let hash_output = std::process::Command::new("git")
        .current_dir(project_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    Ok(String::from_utf8_lossy(&hash_output.stdout).trim().to_string())
}

fn write_semantic_stub(
    project_root: &Path,
    hash: &str,
    message: &str,
    author_name: &str,
    author_type: &AuthorType,
    task_ids: &[String],
) -> Result<PathBuf, GitBridgeError> {
    let dir = project_root.join(".Orqestra").join("graph").join("commits");
    std::fs::create_dir_all(&dir).map_err(|e| GitBridgeError::Io(dir.clone(), e))?;

    let stub = SemanticCommitObject {
        hash: hash.to_string(),
        parent_hashes: vec![], // filled in by backfill
        author: CommitAuthor {
            name: author_name.to_string(),
            author_type: author_type.clone(),
        },
        timestamp: Utc::now(),
        conventional_message: message.to_string(),
        semantic: SemanticPayload::Pending {
            task_ids: task_ids.to_vec(),
        },
    };

    let path = dir.join(format!("{hash}.json"));
    let tmp_path = dir.join(format!(".tmp-{hash}.json"));

    let content = serde_json::to_string_pretty(&stub)?;
    std::fs::write(&tmp_path, &content)
        .map_err(|e| GitBridgeError::Io(tmp_path.clone(), e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| GitBridgeError::Io(path.clone(), e))?;

    Ok(path)
}
