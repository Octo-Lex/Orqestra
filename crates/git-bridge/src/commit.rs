use crate::error::GitBridgeError;
use crate::semantic::{AuthorType, CommitAuthor, SemanticCommitObject, SemanticPayload};
use chrono::Utc;
use std::path::{Path, PathBuf};

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
/// Never blocks on AI inference — the stub has status "pending".
/// The AI backfill is the caller's responsibility.
pub fn semantic_commit(request: CommitRequest) -> Result<CommitResult, GitBridgeError> {
    let _repo = gix::open(&request.project_root)
        .map_err(|e| GitBridgeError::GitOperation(e.to_string()))?;

    // Stage the specified files (or all roadmap/ changes if none specified)
    stage_files(&request.project_root, &request.files_to_stage)?;

    // Create the commit
    let hash = create_commit(&request.project_root, &request.message, &request.author_name)?;

    // Write the semantic stub immediately — never block on AI
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

fn stage_files(
    project_root: &Path,
    files: &[PathBuf],
) -> Result<(), GitBridgeError> {
    // Phase 1: shell out to git add.
    // Phase 2 will replace with native gix index ops.
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
    // Phase 1: shell out to git commit.
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

    // Read the hash of HEAD
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
