//! GCC Workspace — core file-system operations (arXiv:2508.00031v2).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use fs2::FileExt;
use uuid::Uuid;

use crate::error::{GCCError, Result};
use crate::models::{
    desanitize, split_blocks, BranchMetadata, CommitRecord, ContextResult, OTARecord,
};

const MAIN_BRANCH: &str = "main";
const GCC_DIR: &str = ".GCC";

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn short_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

/// Return `Err(Validation)` if `value` is empty or whitespace-only.
fn validate_not_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(GCCError::Validation(format!("{field} must not be empty")));
    }
    Ok(())
}

/// Return `Err(Validation)` if `name` is not a valid branch identifier.
fn validate_branch_name(name: &str) -> Result<()> {
    validate_not_empty(name, "Branch name")?;
    if name.contains('/') || name.contains('\\') {
        return Err(GCCError::Validation(format!(
            "Branch name must not contain path separators: {name:?}"
        )));
    }
    if name == "." || name == ".." {
        return Err(GCCError::Validation(format!(
            "Branch name must not be '.' or '..': {name:?}"
        )));
    }
    Ok(())
}

/// Manages the `.GCC/` directory structure for one agent project.
///
/// Implements the four GCC commands from arXiv:2508.00031v2:
///   - COMMIT  (§3.2): milestone checkpointing
///   - BRANCH  (§3.3): isolated reasoning workspace
///   - MERGE   (§3.4): synthesise divergent paths
///   - CONTEXT (§3.5): hierarchical memory retrieval
pub struct GCCWorkspace {
    gcc_dir: PathBuf,
    current_branch: String,
}

/// RAII file lock guard. Acquires exclusive lock on creation, releases on drop.
struct FileLock {
    _file: fs::File,
}

impl FileLock {
    fn acquire(gcc_dir: &Path) -> Result<Self> {
        let lock_path = gcc_dir.join(".lock");
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)?;
        file.lock_exclusive()
            .map_err(|e| GCCError::Lock(e.to_string()))?;
        Ok(Self { _file: file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self._file.unlock();
    }
}

impl GCCWorkspace {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let gcc_dir = project_root.as_ref().join(GCC_DIR);
        Self {
            gcc_dir,
            current_branch: MAIN_BRANCH.to_string(),
        }
    }

    // ------------------------------------------------------------------ //
    // Paths                                                                //
    // ------------------------------------------------------------------ //

    fn branch_dir(&self, branch: &str) -> PathBuf {
        self.gcc_dir.join("branches").join(branch)
    }

    fn log_path(&self, branch: &str) -> PathBuf {
        self.branch_dir(branch).join("log.md")
    }

    fn commit_path(&self, branch: &str) -> PathBuf {
        self.branch_dir(branch).join("commit.md")
    }

    fn meta_path(&self, branch: &str) -> PathBuf {
        self.branch_dir(branch).join("metadata.yaml")
    }

    fn main_md(&self) -> PathBuf {
        self.gcc_dir.join("main.md")
    }

    // ------------------------------------------------------------------ //
    // I/O helpers                                                          //
    // ------------------------------------------------------------------ //

    fn read(&self, path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    fn write(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn append(&self, path: &Path, content: &str) -> Result<()> {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn read_commits(&self, branch: &str) -> Vec<CommitRecord> {
        let text = self.read(&self.commit_path(branch));
        let mut records = Vec::new();
        for block in split_blocks(&text) {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let mut commit_id = String::new();
            let mut ts = String::new();
            let mut current_field: Option<&str> = None;
            let mut fields: std::collections::HashMap<&str, String> =
                std::collections::HashMap::new();

            for line in block.lines() {
                if line.starts_with("## Commit `") {
                    commit_id = line
                        .trim_start_matches("## Commit `")
                        .trim_end_matches('`')
                        .to_string();
                    current_field = None;
                } else if line.starts_with("**Timestamp:**") {
                    ts = line.replace("**Timestamp:**", "").trim().to_string();
                    current_field = None;
                } else if line.starts_with("**Branch Purpose:**") {
                    let val = line.replace("**Branch Purpose:**", "").trim().to_string();
                    fields.insert("branch_purpose", val);
                    current_field = Some("branch_purpose");
                } else if line.starts_with("**Previous Progress Summary:**") {
                    let val = line
                        .replace("**Previous Progress Summary:**", "")
                        .trim()
                        .to_string();
                    fields.insert("prev_summary", val);
                    current_field = Some("prev_summary");
                } else if line.starts_with("**This Commit's Contribution:**") {
                    let val = line
                        .replace("**This Commit's Contribution:**", "")
                        .trim()
                        .to_string();
                    fields.insert("contribution", val);
                    current_field = Some("contribution");
                } else if let Some(field) = current_field {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Some(existing) = fields.get_mut(field) {
                            existing.push('\n');
                            existing.push_str(trimmed);
                        }
                    }
                }
            }
            if !commit_id.is_empty() {
                let get = |key: &str| -> String {
                    desanitize(fields.get(key).map(|s| s.trim()).unwrap_or(""))
                };
                records.push(CommitRecord {
                    commit_id,
                    branch_name: branch.to_string(),
                    branch_purpose: get("branch_purpose"),
                    previous_progress_summary: get("prev_summary"),
                    this_commit_contribution: get("contribution"),
                    timestamp: ts,
                });
            }
        }
        records
    }

    fn read_ota(&self, branch: &str) -> Vec<OTARecord> {
        let text = self.read(&self.log_path(branch));
        let mut records = Vec::new();
        for block in split_blocks(&text) {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let mut step = 0usize;
            let mut ts = String::new();
            let mut current_field: Option<&str> = None;
            let mut fields: std::collections::HashMap<&str, String> =
                std::collections::HashMap::new();

            for line in block.lines() {
                if line.starts_with("### Step ") {
                    let parts: Vec<&str> = line.splitn(2, '—').collect();
                    step = parts[0]
                        .replace("### Step ", "")
                        .trim()
                        .parse()
                        .unwrap_or(0);
                    ts = parts.get(1).unwrap_or(&"").trim().to_string();
                    current_field = None;
                } else if line.starts_with("**Observation:**") {
                    let val = line.replace("**Observation:**", "").trim().to_string();
                    fields.insert("obs", val);
                    current_field = Some("obs");
                } else if line.starts_with("**Thought:**") {
                    let val = line.replace("**Thought:**", "").trim().to_string();
                    fields.insert("thought", val);
                    current_field = Some("thought");
                } else if line.starts_with("**Action:**") {
                    let val = line.replace("**Action:**", "").trim().to_string();
                    fields.insert("action", val);
                    current_field = Some("action");
                } else if let Some(field) = current_field {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Some(existing) = fields.get_mut(field) {
                            existing.push('\n');
                            existing.push_str(trimmed);
                        }
                    }
                }
            }
            let get = |key: &str| -> String {
                desanitize(fields.get(key).map(|s| s.trim()).unwrap_or(""))
            };
            let obs = get("obs");
            let thought = get("thought");
            if !obs.is_empty() || !thought.is_empty() {
                records.push(OTARecord {
                    step,
                    timestamp: ts,
                    observation: obs,
                    thought,
                    action: get("action"),
                });
            }
        }
        records
    }

    fn read_meta(&self, branch: &str) -> Option<BranchMetadata> {
        let text = self.read(&self.meta_path(branch));
        if text.is_empty() {
            return None;
        }
        serde_yaml::from_str(&text).ok()
    }

    // ------------------------------------------------------------------ //
    // Initialisation                                                       //
    // ------------------------------------------------------------------ //

    /// Initialise a new GCC workspace.
    pub fn init(&mut self, project_roadmap: &str) -> Result<()> {
        if self.gcc_dir.exists() {
            return Err(GCCError::AlreadyExists {
                path: self.gcc_dir.display().to_string(),
            });
        }

        let main_dir = self.branch_dir(MAIN_BRANCH);
        fs::create_dir_all(&main_dir)?;

        let _lock = FileLock::acquire(&self.gcc_dir)?;

        // main.md — global roadmap
        let roadmap = format!(
            "# Project Roadmap\n\n**Initialized:** {}\n\n{}\n",
            now(),
            project_roadmap
        );
        self.write(&self.main_md(), &roadmap)?;

        // log.md and commit.md for main branch
        self.write(
            &self.log_path(MAIN_BRANCH),
            &format!("# OTA Log — branch `{MAIN_BRANCH}`\n\n"),
        )?;
        self.write(
            &self.commit_path(MAIN_BRANCH),
            &format!("# Commit History — branch `{MAIN_BRANCH}`\n\n"),
        )?;

        // metadata.yaml
        let meta = BranchMetadata {
            name: MAIN_BRANCH.to_string(),
            purpose: "Primary reasoning trajectory".to_string(),
            created_from: String::new(),
            created_at: now(),
            status: "active".to_string(),
            merged_into: None,
            merged_at: None,
        };
        self.write(&self.meta_path(MAIN_BRANCH), &serde_yaml::to_string(&meta)?)?;

        self.current_branch = MAIN_BRANCH.to_string();
        Ok(())
    }

    /// Load an existing GCC workspace.
    pub fn load(&mut self) -> Result<()> {
        if !self.gcc_dir.exists() {
            return Err(GCCError::NotFound {
                path: self.gcc_dir.display().to_string(),
            });
        }
        self.current_branch = MAIN_BRANCH.to_string();
        Ok(())
    }

    // ------------------------------------------------------------------ //
    // GCC Commands                                                         //
    // ------------------------------------------------------------------ //

    /// Append an OTA step to the current branch's `log.md`.
    /// The paper logs continuous Observation–Thought–Action cycles.
    pub fn log_ota(&self, observation: &str, thought: &str, action: &str) -> Result<OTARecord> {
        if observation.trim().is_empty() && thought.trim().is_empty() && action.trim().is_empty() {
            return Err(GCCError::Validation(
                "At least one of observation, thought, or action must be non-empty".to_string(),
            ));
        }
        let _lock = FileLock::acquire(&self.gcc_dir)?;
        let existing = self.read_ota(&self.current_branch);
        let step = existing.len() + 1;
        let record = OTARecord {
            step,
            timestamp: now(),
            observation: observation.to_string(),
            thought: thought.to_string(),
            action: action.to_string(),
        };
        self.append(&self.log_path(&self.current_branch), &record.to_markdown())?;
        Ok(record)
    }

    /// COMMIT command (paper §3.2).
    ///
    /// Persists a milestone checkpoint to `commit.md` with fields:
    /// Branch Purpose, Previous Progress Summary, This Commit's Contribution.
    pub fn commit(
        &self,
        contribution: &str,
        previous_summary: Option<&str>,
        update_roadmap: Option<&str>,
    ) -> Result<CommitRecord> {
        validate_not_empty(contribution, "Contribution")?;
        let _lock = FileLock::acquire(&self.gcc_dir)?;
        self.commit_inner(contribution, previous_summary, update_roadmap)
    }

    /// Internal commit logic, called with lock already held.
    fn commit_inner(
        &self,
        contribution: &str,
        previous_summary: Option<&str>,
        update_roadmap: Option<&str>,
    ) -> Result<CommitRecord> {
        let meta = self.read_meta(&self.current_branch);
        let branch_purpose = meta.as_ref().map(|m| m.purpose.clone()).unwrap_or_default();

        let prev = match previous_summary {
            Some(s) => s.to_string(),
            None => {
                let commits = self.read_commits(&self.current_branch);
                commits
                    .last()
                    .map(|c| c.this_commit_contribution.clone())
                    .unwrap_or_else(|| "Initial state — no prior commits.".to_string())
            }
        };

        let record = CommitRecord {
            commit_id: short_id(),
            branch_name: self.current_branch.clone(),
            branch_purpose,
            previous_progress_summary: prev,
            this_commit_contribution: contribution.to_string(),
            timestamp: now(),
        };

        self.append(
            &self.commit_path(&self.current_branch),
            &record.to_markdown(),
        )?;

        if let Some(roadmap_update) = update_roadmap {
            let update = format!("\n## Update ({})\n{}\n", record.timestamp, roadmap_update);
            self.append(&self.main_md(), &update)?;
        }

        Ok(record)
    }

    /// BRANCH command (paper §3.3).
    ///
    /// Creates isolated workspace: B_t^(name) = BRANCH(M_{t-1}).
    /// Initialises empty OTA trace and commit.md; metadata records intent.
    pub fn branch(&mut self, name: &str, purpose: &str) -> Result<()> {
        validate_branch_name(name)?;
        validate_not_empty(purpose, "Branch purpose")?;
        let branch_dir = self.branch_dir(name);
        if branch_dir.exists() {
            return Err(GCCError::BranchExists {
                name: name.to_string(),
            });
        }

        fs::create_dir_all(&branch_dir)?;

        let _lock = FileLock::acquire(&self.gcc_dir)?;
        self.write(
            &self.log_path(name),
            &format!("# OTA Log — branch `{name}`\n\n"),
        )?;
        self.write(
            &self.commit_path(name),
            &format!("# Commit History — branch `{name}`\n\n"),
        )?;

        let meta = BranchMetadata {
            name: name.to_string(),
            purpose: purpose.to_string(),
            created_from: self.current_branch.clone(),
            created_at: now(),
            status: "active".to_string(),
            merged_into: None,
            merged_at: None,
        };
        self.write(&self.meta_path(name), &serde_yaml::to_string(&meta)?)?;

        self.current_branch = name.to_string();
        Ok(())
    }

    /// MERGE command (paper §3.4).
    ///
    /// Integrates a completed branch back into `target` (default: main),
    /// merging summaries and execution traces into a unified state.
    pub fn merge(
        &mut self,
        branch_name: &str,
        summary: Option<&str>,
        target: &str,
    ) -> Result<CommitRecord> {
        validate_branch_name(branch_name)?;
        validate_branch_name(target)?;
        if !self.branch_dir(branch_name).exists() {
            return Err(GCCError::BranchNotFound {
                name: branch_name.to_string(),
            });
        }

        let _lock = FileLock::acquire(&self.gcc_dir)?;

        let branch_commits = self.read_commits(branch_name);
        let branch_ota = self.read_ota(branch_name);
        let meta = self.read_meta(branch_name);

        let merge_summary = match summary {
            Some(s) => s.to_string(),
            None => {
                let contribs: Vec<_> = branch_commits
                    .iter()
                    .map(|c| c.this_commit_contribution.as_str())
                    .collect();
                format!(
                    "Merged branch `{}` ({} commits). Contributions: {}",
                    branch_name,
                    branch_commits.len(),
                    contribs.join(" | ")
                )
            }
        };

        // Append branch OTA to target log
        if !branch_ota.is_empty() {
            let header = format!("\n## Merged from `{}` ({})\n\n", branch_name, now());
            self.append(&self.log_path(target), &header)?;
            for rec in &branch_ota {
                self.append(&self.log_path(target), &rec.to_markdown())?;
            }
        }

        // Create merge commit on target
        self.current_branch = target.to_string();
        let prev = format!(
            "Merging branch `{}` with purpose: {}",
            branch_name,
            meta.as_ref().map(|m| m.purpose.as_str()).unwrap_or("")
        );
        let merge_commit = self.commit_inner(&merge_summary, Some(&prev), Some(&merge_summary))?;

        // Mark branch as merged
        if let Some(mut m) = meta {
            m.status = "merged".to_string();
            m.merged_into = Some(target.to_string());
            m.merged_at = Some(now());
            self.write(&self.meta_path(branch_name), &serde_yaml::to_string(&m)?)?;
        }

        Ok(merge_commit)
    }

    /// CONTEXT command (paper §3.5).
    ///
    /// Retrieves historical context at K-commit resolution.
    /// Paper experiments use K=1 (only most recent commit revealed).
    pub fn context(&self, branch: Option<&str>, k: usize) -> Result<ContextResult> {
        if k < 1 {
            return Err(GCCError::Validation(format!("k must be >= 1, got {k}")));
        }
        let _lock = FileLock::acquire(&self.gcc_dir)?;
        let target = branch.unwrap_or(&self.current_branch);
        if !self.branch_dir(target).exists() {
            return Err(GCCError::BranchNotFound {
                name: target.to_string(),
            });
        }

        let all_commits = self.read_commits(target);
        let skip = if all_commits.len() > k {
            all_commits.len() - k
        } else {
            0
        };
        let commits = all_commits[skip..].to_vec();
        let ota_records = self.read_ota(target);
        let main_roadmap = self.read(&self.main_md());
        let metadata = self.read_meta(target);

        Ok(ContextResult {
            branch_name: target.to_string(),
            k,
            commits,
            ota_records,
            main_roadmap,
            metadata,
        })
    }

    // ------------------------------------------------------------------ //
    // Helpers                                                              //
    // ------------------------------------------------------------------ //

    pub fn current_branch(&self) -> &str {
        &self.current_branch
    }

    pub fn switch_branch(&mut self, name: &str) -> Result<()> {
        validate_branch_name(name)?;
        if !self.branch_dir(name).exists() {
            return Err(GCCError::BranchNotFound {
                name: name.to_string(),
            });
        }
        self.current_branch = name.to_string();
        Ok(())
    }

    pub fn list_branches(&self) -> Vec<String> {
        let branches_root = self.gcc_dir.join("branches");
        fs::read_dir(&branches_root)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn workspace() -> (TempDir, GCCWorkspace) {
        let dir = TempDir::new().unwrap();
        let mut ws = GCCWorkspace::new(dir.path());
        ws.init("Test project roadmap").unwrap();
        (dir, ws)
    }

    #[test]
    fn test_init_creates_gcc_structure() {
        let (dir, _) = workspace();
        assert!(dir.path().join(".GCC/main.md").exists());
        assert!(dir.path().join(".GCC/branches/main/log.md").exists());
        assert!(dir.path().join(".GCC/branches/main/commit.md").exists());
        assert!(dir.path().join(".GCC/branches/main/metadata.yaml").exists());
    }

    #[test]
    fn test_log_ota_increments_step() {
        let (_dir, ws) = workspace();
        let r1 = ws.log_ota("obs1", "thought1", "action1").unwrap();
        let r2 = ws.log_ota("obs2", "thought2", "action2").unwrap();
        assert_eq!(r1.step, 1);
        assert_eq!(r2.step, 2);
    }

    #[test]
    fn test_commit_writes_checkpoint() {
        let (_dir, ws) = workspace();
        let c = ws.commit("Initial scaffold done", None, None).unwrap();
        assert_eq!(c.this_commit_contribution, "Initial scaffold done");
        assert_eq!(c.branch_name, "main");
        assert_eq!(c.commit_id.len(), 8);
    }

    #[test]
    fn test_branch_creates_isolated_workspace() {
        let (_dir, mut ws) = workspace();
        ws.branch("experiment-a", "Try alternative algorithm")
            .unwrap();
        assert_eq!(ws.current_branch(), "experiment-a");
        let ota = ws.read_ota("experiment-a");
        assert!(ota.is_empty());
    }

    #[test]
    fn test_merge_integrates_branch() {
        let (_dir, mut ws) = workspace();
        ws.commit("Main first commit", None, None).unwrap();
        ws.branch("feature", "Add feature X").unwrap();
        ws.log_ota("feature obs", "feature thought", "feature action")
            .unwrap();
        ws.commit("Feature X done", None, None).unwrap();
        let merge_commit = ws.merge("feature", None, "main").unwrap();
        assert!(merge_commit.this_commit_contribution.contains("feature"));
        assert_eq!(ws.current_branch(), "main");
    }

    #[test]
    fn test_context_k1_returns_last_commit() {
        let (_dir, ws) = workspace();
        ws.commit("C1", None, None).unwrap();
        ws.commit("C2", None, None).unwrap();
        ws.commit("C3", None, None).unwrap();
        let ctx = ws.context(None, 1).unwrap();
        assert_eq!(ctx.commits.len(), 1);
        assert_eq!(ctx.commits[0].this_commit_contribution, "C3");
    }

    #[test]
    fn test_branch_metadata_records_purpose() {
        let (_dir, mut ws) = workspace();
        ws.branch("jwt-branch", "JWT auth experiment").unwrap();
        let meta = ws.read_meta("jwt-branch").unwrap();
        assert_eq!(meta.purpose, "JWT auth experiment");
        assert_eq!(meta.created_from, "main");
        assert_eq!(meta.status, "active");
    }

    #[test]
    fn test_merge_marks_branch_merged() {
        let (_dir, mut ws) = workspace();
        ws.branch("to-merge", "Will be merged").unwrap();
        ws.commit("Branch work", None, None).unwrap();
        ws.merge("to-merge", None, "main").unwrap();
        let meta = ws.read_meta("to-merge").unwrap();
        assert_eq!(meta.status, "merged");
        assert_eq!(meta.merged_into.unwrap(), "main");
    }
}
