/**
 * GCC Workspace — core implementation. by @swadhinbiswas
 *
 * Paper: "Git Context Controller: Manage the Context of LLM-based Agents like Git"
 * arXiv:2508.00031v2 — Junde Wu et al., 2025
 *
 * Implements the four GCC commands:
 *   COMMIT  (§3.2) — milestone checkpointing to commit.md
 *   BRANCH  (§3.3) — isolated reasoning workspace
 *   MERGE   (§3.4) — synthesise divergent paths
 *   CONTEXT (§3.5) — hierarchical memory retrieval (K-commit window)
 *
 * File system layout:
 *   .GCC/
 *   ├── main.md                 # Global roadmap / planning artifact
 *   └── branches/
 *       ├── main/
 *       │   ├── log.md          # Continuous OTA trace
 *       │   ├── commit.md       # Milestone-level summaries
 *       │   └── metadata.yaml   # Branch intent & status
 *       └── <branch>/
 *           └── ...
 */

import * as fs from "fs";
import * as path from "path";
import * as yaml from "js-yaml";
import { randomBytes } from "crypto";
import type {
  BranchMetadata,
  CommitRecord,
  ContextResult,
  OTARecord,
} from "./types";

const MAIN_BRANCH = "main";
const GCC_DIR = ".GCC";

const now = (): string => new Date().toISOString();
const shortId = (): string => randomBytes(4).toString("hex");

function otaToMarkdown(r: OTARecord): string {
  return (
    `### Step ${r.step} — ${r.timestamp}\n` +
    `**Observation:** ${r.observation}\n\n` +
    `**Thought:** ${r.thought}\n\n` +
    `**Action:** ${r.action}\n\n` +
    `---\n`
  );
}

function commitToMarkdown(c: CommitRecord): string {
  return (
    `## Commit \`${c.commitId}\`\n` +
    `**Timestamp:** ${c.timestamp}\n\n` +
    `**Branch Purpose:** ${c.branchPurpose}\n\n` +
    `**Previous Progress Summary:** ${c.previousProgressSummary}\n\n` +
    `**This Commit's Contribution:** ${c.thisCommitContribution}\n\n` +
    `---\n`
  );
}

function makeContextResult(
  branchName: string,
  k: number,
  commits: CommitRecord[],
  otaRecords: OTARecord[],
  mainRoadmap: string,
  metadata?: BranchMetadata
): ContextResult {
  return {
    branchName,
    k,
    commits,
    otaRecords,
    mainRoadmap,
    metadata,
    summary(): string {
      const lines: string[] = [
        `# CONTEXT — branch \`${branchName}\` (K=${k})\n`,
        `## Global Roadmap\n${mainRoadmap}\n`,
        `## Last ${k} Commit(s)\n`,
      ];
      for (const c of commits) lines.push(commitToMarkdown(c));
      if (otaRecords.length > 0) {
        const recent = otaRecords.slice(-5);
        lines.push(
          `\n## Recent OTA Steps (showing last ${recent.length} of ${otaRecords.length})\n`
        );
        for (const r of recent) lines.push(otaToMarkdown(r));
      }
      return lines.join("\n");
    },
  };
}

/** Manages the `.GCC/` directory structure for one agent project. */
export class GCCWorkspace {
  private readonly root: string;
  private readonly gccDir: string;
  private _currentBranch: string = MAIN_BRANCH;

  constructor(projectRoot: string) {
    this.root = projectRoot;
    this.gccDir = path.join(projectRoot, GCC_DIR);
  }

  // ------------------------------------------------------------------ //
  // Paths                                                                //
  // ------------------------------------------------------------------ //

  private branchDir(branch: string): string {
    return path.join(this.gccDir, "branches", branch);
  }
  private logPath(branch: string): string {
    return path.join(this.branchDir(branch), "log.md");
  }
  private commitPath(branch: string): string {
    return path.join(this.branchDir(branch), "commit.md");
  }
  private metaPath(branch: string): string {
    return path.join(this.branchDir(branch), "metadata.yaml");
  }
  private mainMd(): string {
    return path.join(this.gccDir, "main.md");
  }

  // ------------------------------------------------------------------ //
  // I/O helpers                                                          //
  // ------------------------------------------------------------------ //

  private read(filePath: string): string {
    try {
      return fs.readFileSync(filePath, "utf8");
    } catch {
      return "";
    }
  }

  private write(filePath: string, content: string): void {
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    fs.writeFileSync(filePath, content, "utf8");
  }

  private append(filePath: string, content: string): void {
    fs.appendFileSync(filePath, content, "utf8");
  }

  private parseCommits(branch: string): CommitRecord[] {
    const text = this.read(this.commitPath(branch));
    const records: CommitRecord[] = [];
    for (const block of text.split("---\n")) {
      const trimmed = block.trim();
      if (!trimmed) continue;
      let commitId = "",
        branchPurpose = "",
        prevSummary = "",
        contribution = "",
        ts = "";
      for (const line of trimmed.split("\n")) {
        if (line.startsWith("## Commit `")) {
          commitId = line.replace(/^## Commit `|`\s*$/g, "");
        } else if (line.startsWith("**Timestamp:**")) {
          ts = line.replace("**Timestamp:**", "").trim();
        } else if (line.startsWith("**Branch Purpose:**")) {
          branchPurpose = line.replace("**Branch Purpose:**", "").trim();
        } else if (line.startsWith("**Previous Progress Summary:**")) {
          prevSummary = line
            .replace("**Previous Progress Summary:**", "")
            .trim();
        } else if (line.startsWith("**This Commit's Contribution:**")) {
          contribution = line
            .replace("**This Commit's Contribution:**", "")
            .trim();
        }
      }
      if (commitId) {
        records.push({
          commitId,
          branchName: branch,
          branchPurpose,
          previousProgressSummary: prevSummary,
          thisCommitContribution: contribution,
          timestamp: ts,
        });
      }
    }
    return records;
  }

  private parseOTA(branch: string): OTARecord[] {
    const text = this.read(this.logPath(branch));
    const records: OTARecord[] = [];
    for (const block of text.split("---\n")) {
      const trimmed = block.trim();
      if (!trimmed) continue;
      let step = 0,
        ts = "",
        obs = "",
        thought = "",
        action = "";
      for (const line of trimmed.split("\n")) {
        if (line.startsWith("### Step ")) {
          const parts = line.split(" — ");
          step = parseInt(parts[0]?.replace("### Step ", "") ?? "0", 10) || 0;
          ts = parts[1]?.trim() ?? "";
        } else if (line.startsWith("**Observation:**")) {
          obs = line.replace("**Observation:**", "").trim();
        } else if (line.startsWith("**Thought:**")) {
          thought = line.replace("**Thought:**", "").trim();
        } else if (line.startsWith("**Action:**")) {
          action = line.replace("**Action:**", "").trim();
        }
      }
      if (obs || thought) {
        records.push({ step, timestamp: ts, observation: obs, thought, action });
      }
    }
    return records;
  }

  private parseMeta(branch: string): BranchMetadata | undefined {
    const text = this.read(this.metaPath(branch));
    if (!text) return undefined;
    try {
      const d = yaml.load(text) as Record<string, unknown>;
      return {
        name: String(d["name"] ?? ""),
        purpose: String(d["purpose"] ?? ""),
        createdFrom: String(d["created_from"] ?? ""),
        createdAt: String(d["created_at"] ?? ""),
        status: (d["status"] as BranchMetadata["status"]) ?? "active",
        mergedInto: d["merged_into"] ? String(d["merged_into"]) : undefined,
        mergedAt: d["merged_at"] ? String(d["merged_at"]) : undefined,
      };
    } catch {
      return undefined;
    }
  }

  private dumpMeta(meta: BranchMetadata): string {
    return yaml.dump({
      name: meta.name,
      purpose: meta.purpose,
      created_from: meta.createdFrom,
      created_at: meta.createdAt,
      status: meta.status,
      merged_into: meta.mergedInto ?? null,
      merged_at: meta.mergedAt ?? null,
    });
  }

  // ------------------------------------------------------------------ //
  // Initialisation                                                       //
  // ------------------------------------------------------------------ //

  /** Initialise a new GCC workspace. */
  init(projectRoadmap = ""): void {
    if (fs.existsSync(this.gccDir)) {
      throw new Error(`GCC workspace already exists at ${this.gccDir}`);
    }
    fs.mkdirSync(this.branchDir(MAIN_BRANCH), { recursive: true });

    this.write(
      this.mainMd(),
      `# Project Roadmap\n\n**Initialized:** ${now()}\n\n${projectRoadmap}\n`
    );
    this.write(this.logPath(MAIN_BRANCH), `# OTA Log — branch \`${MAIN_BRANCH}\`\n\n`);
    this.write(this.commitPath(MAIN_BRANCH), `# Commit History — branch \`${MAIN_BRANCH}\`\n\n`);

    const meta: BranchMetadata = {
      name: MAIN_BRANCH,
      purpose: "Primary reasoning trajectory",
      createdFrom: "",
      createdAt: now(),
      status: "active",
    };
    this.write(this.metaPath(MAIN_BRANCH), this.dumpMeta(meta));
    this._currentBranch = MAIN_BRANCH;
  }

  /** Load an existing GCC workspace. */
  load(): void {
    if (!fs.existsSync(this.gccDir)) {
      throw new Error(`No GCC workspace found at ${this.gccDir}`);
    }
    this._currentBranch = MAIN_BRANCH;
  }

  // ------------------------------------------------------------------ //
  // GCC Commands                                                         //
  // ------------------------------------------------------------------ //

  /**
   * Append an OTA step to current branch's log.md.
   * The paper logs continuous Observation–Thought–Action cycles.
   */
  logOTA(observation: string, thought: string, action: string): OTARecord {
    const existing = this.parseOTA(this._currentBranch);
    const record: OTARecord = {
      step: existing.length + 1,
      timestamp: now(),
      observation,
      thought,
      action,
    };
    this.append(this.logPath(this._currentBranch), otaToMarkdown(record));
    return record;
  }

  /**
   * COMMIT command (paper §3.2).
   * Checkpoints milestone with: Branch Purpose, Previous Progress Summary,
   * This Commit's Contribution.
   */
  commit(
    contribution: string,
    previousSummary?: string,
    updateRoadmap?: string
  ): CommitRecord {
    const meta = this.parseMeta(this._currentBranch);
    const branchPurpose = meta?.purpose ?? "";

    let prevSummary = previousSummary;
    if (!prevSummary) {
      const commits = this.parseCommits(this._currentBranch);
      prevSummary =
        commits.at(-1)?.thisCommitContribution ??
        "Initial state — no prior commits.";
    }

    const record: CommitRecord = {
      commitId: shortId(),
      branchName: this._currentBranch,
      branchPurpose,
      previousProgressSummary: prevSummary,
      thisCommitContribution: contribution,
      timestamp: now(),
    };

    this.append(this.commitPath(this._currentBranch), commitToMarkdown(record));

    if (updateRoadmap) {
      this.append(this.mainMd(), `\n## Update (${record.timestamp})\n${updateRoadmap}\n`);
    }

    return record;
  }

  /**
   * BRANCH command (paper §3.3).
   * Creates isolated workspace: B_t^(name) = BRANCH(M_{t-1}).
   */
  branch(name: string, purpose: string): void {
    if (fs.existsSync(this.branchDir(name))) {
      throw new Error(`Branch '${name}' already exists.`);
    }
    fs.mkdirSync(this.branchDir(name), { recursive: true });
    this.write(this.logPath(name), `# OTA Log — branch \`${name}\`\n\n`);
    this.write(this.commitPath(name), `# Commit History — branch \`${name}\`\n\n`);

    const meta: BranchMetadata = {
      name,
      purpose,
      createdFrom: this._currentBranch,
      createdAt: now(),
      status: "active",
    };
    this.write(this.metaPath(name), this.dumpMeta(meta));
    this._currentBranch = name;
  }

  /**
   * MERGE command (paper §3.4).
   * Integrates branch into target, merging summaries and OTA traces.
   */
  merge(
    branchName: string,
    summary?: string,
    target: string = MAIN_BRANCH
  ): CommitRecord {
    if (!fs.existsSync(this.branchDir(branchName))) {
      throw new Error(`Branch '${branchName}' not found.`);
    }
    if (!fs.existsSync(this.branchDir(target))) {
      throw new Error(`Target branch '${target}' not found.`);
    }

    const branchCommits = this.parseCommits(branchName);
    const branchOTA = this.parseOTA(branchName);
    const meta = this.parseMeta(branchName);

    const mergeSummary =
      summary ??
      `Merged branch \`${branchName}\` (${branchCommits.length} commits). Contributions: ${branchCommits
        .map((c) => c.thisCommitContribution)
        .join(" | ")}`;

    if (branchOTA.length > 0) {
      this.append(
        this.logPath(target),
        `\n## Merged from \`${branchName}\` (${now()})\n\n`
      );
      for (const rec of branchOTA) {
        this.append(this.logPath(target), otaToMarkdown(rec));
      }
    }

    this._currentBranch = target;
    const mergeCommit = this.commit(
      mergeSummary,
      `Merging branch \`${branchName}\` with purpose: ${meta?.purpose ?? ""}`,
      mergeSummary
    );

    if (meta) {
      const updated: BranchMetadata = {
        ...meta,
        status: "merged",
        mergedInto: target,
        mergedAt: now(),
      };
      this.write(this.metaPath(branchName), this.dumpMeta(updated));
    }

    return mergeCommit;
  }

  /**
   * CONTEXT command (paper §3.5).
   * Retrieves history at K-commit resolution. Paper default: K=1.
   */
  context(branch?: string, k = 1): ContextResult {
    const target = branch ?? this._currentBranch;
    if (!fs.existsSync(this.branchDir(target))) {
      throw new Error(`Branch '${target}' not found.`);
    }

    const allCommits = this.parseCommits(target);
    const commits = allCommits.slice(-k);
    const otaRecords = this.parseOTA(target);
    const mainRoadmap = this.read(this.mainMd());
    const metadata = this.parseMeta(target);

    return makeContextResult(target, k, commits, otaRecords, mainRoadmap, metadata);
  }

  // ------------------------------------------------------------------ //
  // Helpers                                                              //
  // ------------------------------------------------------------------ //

  get currentBranch(): string {
    return this._currentBranch;
  }

  switchBranch(name: string): void {
    if (!fs.existsSync(this.branchDir(name))) {
      throw new Error(`Branch '${name}' does not exist.`);
    }
    this._currentBranch = name;
  }

  listBranches(): string[] {
    const branchesRoot = path.join(this.gccDir, "branches");
    try {
      return fs
        .readdirSync(branchesRoot)
        .filter((f) => fs.statSync(path.join(branchesRoot, f)).isDirectory());
    } catch {
      return [];
    }
  }
}

