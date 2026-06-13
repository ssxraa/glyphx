//! Git version control via gitoxide (`gix`) — pure Rust, no system `git` or C
//! deps, so the client is OS-agnostic and can cross-compile to mobile later.
//!
//! v1 is local-first: detect/init a repo, read the current branch + history, and
//! (added incrementally) the working-tree status, staging, and commits. Remotes
//! (clone/fetch/push + auth) are a later phase.

use serde::Serialize;

fn open(root: &str) -> Result<gix::Repository, String> {
    gix::open(root).map_err(|e| e.to_string())
}

/// Whether `root` is the top of a Git repository.
#[tauri::command]
pub async fn git_is_repo(root: String) -> bool {
    gix::open(&root).is_ok()
}

/// Initialize a new Git repository at `root`.
#[tauri::command]
pub async fn git_init(root: String) -> Result<(), String> {
    gix::init(&root).map(|_| ()).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct GitHead {
    /// Short branch name (e.g. `main`), or `None` when detached / unborn.
    pub branch: Option<String>,
    /// True before the first commit (HEAD points at an unborn branch).
    pub unborn: bool,
}

/// The repository's current branch / HEAD state.
#[tauri::command]
pub async fn git_head(root: String) -> Result<GitHead, String> {
    let repo = open(&root)?;
    let branch = repo
        .head_name()
        .map_err(|e| e.to_string())?
        .map(|name| name.shorten().to_string());
    let unborn = repo.head_id().is_err();
    Ok(GitHead { branch, unborn })
}

#[derive(Serialize)]
pub struct GitCommit {
    /// Abbreviated commit hash.
    pub hash: String,
    /// First line of the commit message.
    pub summary: String,
    pub author: String,
    /// Author time, seconds since the Unix epoch.
    pub time: i64,
}

/// Recent commits reachable from HEAD (newest first), up to `limit`.
#[tauri::command]
pub async fn git_log(root: String, limit: Option<usize>) -> Result<Vec<GitCommit>, String> {
    let repo = open(&root)?;
    // Unborn branch (no commits yet) → empty history rather than an error.
    let Ok(head_id) = repo.head_id() else {
        return Ok(Vec::new());
    };
    let limit = limit.unwrap_or(50);

    let mut out = Vec::new();
    let infos = head_id.ancestors().all().map_err(|e| e.to_string())?;
    for info in infos.take(limit) {
        let info = info.map_err(|e| e.to_string())?;
        let commit = info.object().map_err(|e| e.to_string())?;
        let author = commit.author().map_err(|e| e.to_string())?;
        let summary = commit
            .message()
            .map_err(|e| e.to_string())?
            .summary()
            .to_string();
        out.push(GitCommit {
            hash: commit.id().to_hex_with_len(7).to_string(),
            summary,
            author: author.name.to_string(),
            time: author.time.seconds,
        });
    }
    Ok(out)
}
