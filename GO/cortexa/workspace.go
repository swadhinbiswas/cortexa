package cortexa

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"gopkg.in/yaml.v3"
)

const (
	mainBranch = "main"
	gccDir     = ".GCC"
)

func nowISO() string {
	return time.Now().UTC().Format(time.RFC3339)
}

func shortID() string {
	b := make([]byte, 4)
	rand.Read(b)
	return hex.EncodeToString(b)
}

// Workspace manages the .GCC/ directory for one agent project.
//
// Implements the four GCC commands (arXiv:2508.00031v2):
//
//	COMMIT  (§3.2) — milestone checkpointing to commit.md
//	BRANCH  (§3.3) — isolated reasoning workspace
//	MERGE   (§3.4) — synthesise divergent paths
//	CONTEXT (§3.5) — hierarchical memory retrieval (K-commit window)
//
// File system layout:
//
//	.GCC/
//	├── main.md                   # Global roadmap / planning artifact
//	└── branches/
//	    ├── main/
//	    │   ├── log.md            # Continuous OTA trace
//	    │   ├── commit.md         # Milestone-level summaries
//	    │   └── metadata.yaml     # Branch intent & status
//	    └── <branch>/
//	        └── ...
type Workspace struct {
	root          string
	gccPath       string
	currentBranch string
}

// New returns a Workspace rooted at projectRoot.
func New(projectRoot string) *Workspace {
	return &Workspace{
		root:          projectRoot,
		gccPath:       filepath.Join(projectRoot, gccDir),
		currentBranch: mainBranch,
	}
}

// ------------------------------------------------------------------ //
// Paths                                                               //
// ------------------------------------------------------------------ //

func (w *Workspace) branchDir(branch string) string {
	return filepath.Join(w.gccPath, "branches", branch)
}
func (w *Workspace) logPath(branch string) string {
	return filepath.Join(w.branchDir(branch), "log.md")
}
func (w *Workspace) commitPath(branch string) string {
	return filepath.Join(w.branchDir(branch), "commit.md")
}
func (w *Workspace) metaPath(branch string) string {
	return filepath.Join(w.branchDir(branch), "metadata.yaml")
}
func (w *Workspace) mainMd() string {
	return filepath.Join(w.gccPath, "main.md")
}

// ------------------------------------------------------------------ //
// I/O helpers                                                         //
// ------------------------------------------------------------------ //

func (w *Workspace) read(path string) string {
	data, err := os.ReadFile(path)
	if err != nil {
		return ""
	}
	return string(data)
}

func (w *Workspace) write(path, content string) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	return os.WriteFile(path, []byte(content), 0o644)
}

func (w *Workspace) appendFile(path, content string) error {
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = f.WriteString(content)
	return err
}

func (w *Workspace) parseCommits(branch string) []CommitRecord {
	text := w.read(w.commitPath(branch))
	var records []CommitRecord
	for _, block := range strings.Split(text, "---\n") {
		block = strings.TrimSpace(block)
		if block == "" {
			continue
		}
		var (
			commitID, branchPurpose, prevSummary, contribution, ts string
		)
		for _, line := range strings.Split(block, "\n") {
			switch {
			case strings.HasPrefix(line, "## Commit `"):
				commitID = strings.TrimSuffix(strings.TrimPrefix(line, "## Commit `"), "`")
			case strings.HasPrefix(line, "**Timestamp:**"):
				ts = strings.TrimSpace(strings.TrimPrefix(line, "**Timestamp:**"))
			case strings.HasPrefix(line, "**Branch Purpose:**"):
				branchPurpose = strings.TrimSpace(strings.TrimPrefix(line, "**Branch Purpose:**"))
			case strings.HasPrefix(line, "**Previous Progress Summary:**"):
				prevSummary = strings.TrimSpace(strings.TrimPrefix(line, "**Previous Progress Summary:**"))
			case strings.HasPrefix(line, "**This Commit's Contribution:**"):
				contribution = strings.TrimSpace(strings.TrimPrefix(line, "**This Commit's Contribution:**"))
			}
		}
		if commitID != "" {
			records = append(records, CommitRecord{
				CommitID:                commitID,
				BranchName:              branch,
				BranchPurpose:           branchPurpose,
				PreviousProgressSummary: prevSummary,
				ThisCommitContribution:  contribution,
				Timestamp:               ts,
			})
		}
	}
	return records
}

func (w *Workspace) parseOTA(branch string) []OTARecord {
	text := w.read(w.logPath(branch))
	var records []OTARecord
	for _, block := range strings.Split(text, "---\n") {
		block = strings.TrimSpace(block)
		if block == "" {
			continue
		}
		var step int
		var ts, obs, thought, action string
		for _, line := range strings.Split(block, "\n") {
			switch {
			case strings.HasPrefix(line, "### Step "):
				parts := strings.SplitN(line, " — ", 2)
				fmt.Sscanf(parts[0], "### Step %d", &step)
				if len(parts) > 1 {
					ts = strings.TrimSpace(parts[1])
				}
			case strings.HasPrefix(line, "**Observation:**"):
				obs = strings.TrimSpace(strings.TrimPrefix(line, "**Observation:**"))
			case strings.HasPrefix(line, "**Thought:**"):
				thought = strings.TrimSpace(strings.TrimPrefix(line, "**Thought:**"))
			case strings.HasPrefix(line, "**Action:**"):
				action = strings.TrimSpace(strings.TrimPrefix(line, "**Action:**"))
			}
		}
		if obs != "" || thought != "" {
			records = append(records, OTARecord{
				Step:        step,
				Timestamp:   ts,
				Observation: obs,
				Thought:     thought,
				Action:      action,
			})
		}
	}
	return records
}

func (w *Workspace) parseMeta(branch string) (*BranchMetadata, error) {
	text := w.read(w.metaPath(branch))
	if text == "" {
		return nil, nil
	}
	var meta BranchMetadata
	if err := yaml.Unmarshal([]byte(text), &meta); err != nil {
		return nil, err
	}
	return &meta, nil
}

// ------------------------------------------------------------------ //
// Initialisation                                                      //
// ------------------------------------------------------------------ //

// Init initialises a new GCC workspace.
func (w *Workspace) Init(projectRoadmap string) error {
	if _, err := os.Stat(w.gccPath); err == nil {
		return fmt.Errorf("GCC workspace already exists at %s", w.gccPath)
	}
	if err := os.MkdirAll(w.branchDir(mainBranch), 0o755); err != nil {
		return err
	}

	roadmap := fmt.Sprintf("# Project Roadmap\n\n**Initialized:** %s\n\n%s\n", nowISO(), projectRoadmap)
	if err := w.write(w.mainMd(), roadmap); err != nil {
		return err
	}
	if err := w.write(w.logPath(mainBranch), fmt.Sprintf("# OTA Log — branch `%s`\n\n", mainBranch)); err != nil {
		return err
	}
	if err := w.write(w.commitPath(mainBranch), fmt.Sprintf("# Commit History — branch `%s`\n\n", mainBranch)); err != nil {
		return err
	}

	meta := BranchMetadata{
		Name:        mainBranch,
		Purpose:     "Primary reasoning trajectory",
		CreatedFrom: "",
		CreatedAt:   nowISO(),
		Status:      "active",
	}
	data, err := yaml.Marshal(meta)
	if err != nil {
		return err
	}
	if err := w.write(w.metaPath(mainBranch), string(data)); err != nil {
		return err
	}

	w.currentBranch = mainBranch
	return nil
}

// Load opens an existing GCC workspace.
func (w *Workspace) Load() error {
	if _, err := os.Stat(w.gccPath); os.IsNotExist(err) {
		return fmt.Errorf("no GCC workspace found at %s", w.gccPath)
	}
	w.currentBranch = mainBranch
	return nil
}

// ------------------------------------------------------------------ //
// GCC Commands                                                        //
// ------------------------------------------------------------------ //

// LogOTA appends an Observation–Thought–Action step to the current branch's log.md.
func (w *Workspace) LogOTA(observation, thought, action string) (OTARecord, error) {
	existing := w.parseOTA(w.currentBranch)
	record := OTARecord{
		Step:        len(existing) + 1,
		Timestamp:   nowISO(),
		Observation: observation,
		Thought:     thought,
		Action:      action,
	}
	if err := w.appendFile(w.logPath(w.currentBranch), record.ToMarkdown()); err != nil {
		return OTARecord{}, err
	}
	return record, nil
}

// Commit is the COMMIT command (paper §3.2).
// Checkpoints milestone with: Branch Purpose, Previous Progress Summary,
// This Commit's Contribution.
func (w *Workspace) Commit(contribution string, previousSummary *string, updateRoadmap *string) (CommitRecord, error) {
	meta, _ := w.parseMeta(w.currentBranch)
	branchPurpose := ""
	if meta != nil {
		branchPurpose = meta.Purpose
	}

	prevSummary := "Initial state — no prior commits."
	if previousSummary != nil {
		prevSummary = *previousSummary
	} else {
		commits := w.parseCommits(w.currentBranch)
		if len(commits) > 0 {
			prevSummary = commits[len(commits)-1].ThisCommitContribution
		}
	}

	record := CommitRecord{
		CommitID:                shortID(),
		BranchName:              w.currentBranch,
		BranchPurpose:           branchPurpose,
		PreviousProgressSummary: prevSummary,
		ThisCommitContribution:  contribution,
		Timestamp:               nowISO(),
	}

	if err := w.appendFile(w.commitPath(w.currentBranch), record.ToMarkdown()); err != nil {
		return CommitRecord{}, err
	}
	if updateRoadmap != nil {
		update := fmt.Sprintf("\n## Update (%s)\n%s\n", record.Timestamp, *updateRoadmap)
		if err := w.appendFile(w.mainMd(), update); err != nil {
			return CommitRecord{}, err
		}
	}
	return record, nil
}

// Branch is the BRANCH command (paper §3.3).
// Creates isolated workspace: B_t^(name) = BRANCH(M_{t-1}).
func (w *Workspace) Branch(name, purpose string) error {
	if _, err := os.Stat(w.branchDir(name)); err == nil {
		return fmt.Errorf("branch '%s' already exists", name)
	}
	if err := os.MkdirAll(w.branchDir(name), 0o755); err != nil {
		return err
	}
	if err := w.write(w.logPath(name), fmt.Sprintf("# OTA Log — branch `%s`\n\n", name)); err != nil {
		return err
	}
	if err := w.write(w.commitPath(name), fmt.Sprintf("# Commit History — branch `%s`\n\n", name)); err != nil {
		return err
	}
	meta := BranchMetadata{
		Name:        name,
		Purpose:     purpose,
		CreatedFrom: w.currentBranch,
		CreatedAt:   nowISO(),
		Status:      "active",
	}
	data, err := yaml.Marshal(meta)
	if err != nil {
		return err
	}
	if err := w.write(w.metaPath(name), string(data)); err != nil {
		return err
	}
	w.currentBranch = name
	return nil
}

// Merge is the MERGE command (paper §3.4).
// Integrates a completed branch into target, merging summaries and OTA traces.
func (w *Workspace) Merge(branchName string, summary *string, target string) (CommitRecord, error) {
	if _, err := os.Stat(w.branchDir(branchName)); os.IsNotExist(err) {
		return CommitRecord{}, fmt.Errorf("branch '%s' not found", branchName)
	}
	if _, err := os.Stat(w.branchDir(target)); os.IsNotExist(err) {
		return CommitRecord{}, fmt.Errorf("target branch '%s' not found", target)
	}

	branchCommits := w.parseCommits(branchName)
	branchOTA := w.parseOTA(branchName)
	meta, _ := w.parseMeta(branchName)

	var mergeSummary string
	if summary != nil {
		mergeSummary = *summary
	} else {
		contribs := make([]string, len(branchCommits))
		for i, c := range branchCommits {
			contribs[i] = c.ThisCommitContribution
		}
		mergeSummary = fmt.Sprintf("Merged branch `%s` (%d commits). Contributions: %s",
			branchName, len(branchCommits), strings.Join(contribs, " | "))
	}

	if len(branchOTA) > 0 {
		header := fmt.Sprintf("\n## Merged from `%s` (%s)\n\n", branchName, nowISO())
		if err := w.appendFile(w.logPath(target), header); err != nil {
			return CommitRecord{}, err
		}
		for _, rec := range branchOTA {
			if err := w.appendFile(w.logPath(target), rec.ToMarkdown()); err != nil {
				return CommitRecord{}, err
			}
		}
	}

	w.currentBranch = target
	metaPurpose := ""
	if meta != nil {
		metaPurpose = meta.Purpose
	}
	prevSummary := fmt.Sprintf("Merging branch `%s` with purpose: %s", branchName, metaPurpose)
	mergeCommit, err := w.Commit(mergeSummary, &prevSummary, &mergeSummary)
	if err != nil {
		return CommitRecord{}, err
	}

	if meta != nil {
		meta.Status = "merged"
		merged := target
		meta.MergedInto = &merged
		mergedAt := nowISO()
		meta.MergedAt = &mergedAt
		data, err := yaml.Marshal(meta)
		if err != nil {
			return CommitRecord{}, err
		}
		if err := w.write(w.metaPath(branchName), string(data)); err != nil {
			return CommitRecord{}, err
		}
	}

	return mergeCommit, nil
}

// Context is the CONTEXT command (paper §3.5).
// Retrieves history at K-commit resolution. Paper default: K=1.
func (w *Workspace) Context(branch *string, k int) (ContextResult, error) {
	target := w.currentBranch
	if branch != nil {
		target = *branch
	}
	if _, err := os.Stat(w.branchDir(target)); os.IsNotExist(err) {
		return ContextResult{}, fmt.Errorf("branch '%s' not found", target)
	}

	allCommits := w.parseCommits(target)
	start := len(allCommits) - k
	if start < 0 {
		start = 0
	}
	commits := allCommits[start:]
	otaRecords := w.parseOTA(target)
	mainRoadmap := w.read(w.mainMd())
	meta, _ := w.parseMeta(target)

	return ContextResult{
		BranchName:  target,
		K:           k,
		Commits:     commits,
		OTARecords:  otaRecords,
		MainRoadmap: mainRoadmap,
		Metadata:    meta,
	}, nil
}

// ------------------------------------------------------------------ //
// Helpers                                                             //
// ------------------------------------------------------------------ //

// CurrentBranch returns the active branch name.
func (w *Workspace) CurrentBranch() string {
	return w.currentBranch
}

// SwitchBranch changes the active branch without creating a new one.
func (w *Workspace) SwitchBranch(name string) error {
	if _, err := os.Stat(w.branchDir(name)); os.IsNotExist(err) {
		return fmt.Errorf("branch '%s' does not exist", name)
	}
	w.currentBranch = name
	return nil
}

// ListBranches returns all branch names in the workspace.
func (w *Workspace) ListBranches() ([]string, error) {
	branchesRoot := filepath.Join(w.gccPath, "branches")
	entries, err := os.ReadDir(branchesRoot)
	if err != nil {
		return nil, err
	}
	var branches []string
	for _, e := range entries {
		if e.IsDir() {
			branches = append(branches, e.Name())
		}
	}
	return branches, nil
}
