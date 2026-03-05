//! Data models for GCC (arXiv:2508.00031v2).

use serde::{Deserialize, Serialize};

/// Escape the `---` block separator so user content cannot break parsers.
pub fn sanitize(text: &str) -> String {
    text.replace("\n---\n", "\n\\---\n")
}

/// Reverse the escaping applied by [`sanitize`].
pub fn desanitize(text: &str) -> String {
    text.replace("\n\\---\n", "\n---\n")
}

/// Split markdown text on `---\n` separators while respecting escaped
/// separators (`\---\n` produced by [`sanitize`]).  After splitting,
/// the escaped backslash is left in place; callers should apply
/// [`desanitize`] on individual field values.
pub fn split_blocks(text: &str) -> Vec<String> {
    let raw: Vec<&str> = text.split("---\n").collect();
    let mut blocks: Vec<String> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        let mut block = raw[i].to_string();
        // If block ends with '\', the `---\n` was an escaped separator — rejoin.
        while block.ends_with('\\') && i + 1 < raw.len() {
            i += 1;
            block.push_str("---\n");
            block.push_str(raw[i]);
        }
        blocks.push(block);
        i += 1;
    }
    blocks
}

/// A single Observation–Thought–Action step logged to `log.md`.
/// The paper continuously logs OTA cycles as the agent executes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OTARecord {
    pub step: usize,
    pub timestamp: String,
    pub observation: String,
    pub thought: String,
    pub action: String,
}

impl OTARecord {
    pub fn to_markdown(&self) -> String {
        format!(
            "### Step {} — {}\n**Observation:** {}\n\n**Thought:** {}\n\n**Action:** {}\n\n---\n",
            self.step,
            self.timestamp,
            sanitize(&self.observation),
            sanitize(&self.thought),
            sanitize(&self.action),
        )
    }
}

/// A commit checkpoint as described in paper §3.2.
/// Fields: Branch Purpose, Previous Progress Summary, This Commit's Contribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRecord {
    pub commit_id: String,
    pub branch_name: String,
    pub branch_purpose: String,
    pub previous_progress_summary: String,
    pub this_commit_contribution: String,
    pub timestamp: String,
}

impl CommitRecord {
    pub fn to_markdown(&self) -> String {
        format!(
            "## Commit `{}`\n**Timestamp:** {}\n\n**Branch Purpose:** {}\n\n\
             **Previous Progress Summary:** {}\n\n\
             **This Commit's Contribution:** {}\n\n---\n",
            self.commit_id,
            self.timestamp,
            sanitize(&self.branch_purpose),
            sanitize(&self.previous_progress_summary),
            sanitize(&self.this_commit_contribution),
        )
    }
}

/// Branch metadata stored in `metadata.yaml` (paper §3.1).
/// Records the intent and motivation of each branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMetadata {
    pub name: String,
    pub purpose: String,
    pub created_from: String,
    pub created_at: String,
    pub status: String, // "active" | "merged" | "abandoned"
    pub merged_into: Option<String>,
    pub merged_at: Option<String>,
}

/// Result of the CONTEXT command (paper §3.5).
/// The paper fixes K=1 in experiments (most recent commit record revealed).
#[derive(Debug, Clone)]
pub struct ContextResult {
    pub branch_name: String,
    pub k: usize,
    pub commits: Vec<CommitRecord>,
    pub ota_records: Vec<OTARecord>,
    pub main_roadmap: String,
    pub metadata: Option<BranchMetadata>,
}

impl ContextResult {
    pub fn summary(&self) -> String {
        let mut out = format!(
            "# CONTEXT — branch `{}` (K={})\n\n",
            self.branch_name, self.k
        );
        out.push_str("## Global Roadmap\n");
        out.push_str(&self.main_roadmap);
        out.push_str("\n\n");
        out.push_str(&format!("## Last {} Commit(s)\n", self.k));
        for c in &self.commits {
            out.push_str(&c.to_markdown());
        }
        if !self.ota_records.is_empty() {
            let recent = self.ota_records.iter().rev().take(5).collect::<Vec<_>>();
            out.push_str(&format!(
                "\n## Recent OTA Steps (showing last {} of {})\n",
                recent.len(),
                self.ota_records.len()
            ));
            for r in recent.into_iter().rev() {
                out.push_str(&r.to_markdown());
            }
        }
        out
    }
}
