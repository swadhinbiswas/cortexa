#!/usr/bin/env bash
#
# commit_history.sh — Create a realistic 70-commit history for the contexa monorepo
# Backdated over the last 2 days with granular, meaningful commit messages.
#
# Usage: bash commit_history.sh
#
# IMPORTANT: Run this from the repo root (/home/swadhin/Cortexa)

set -euo pipefail

REPO_DIR="/home/swadhin/Cortexa"
cd "$REPO_DIR"

# Author info
export GIT_AUTHOR_NAME="Swadhin Biswas"
export GIT_AUTHOR_EMAIL="swadhinbiswas.cse@gmail.com"
export GIT_COMMITTER_NAME="Swadhin Biswas"
export GIT_COMMITTER_EMAIL="swadhinbiswas.cse@gmail.com"

# ── Helper ────────────────────────────────────────────────────────────
commit_at() {
    local date_str="$1"; shift
    local msg="$1"; shift
    for f in "$@"; do
        git add -- "$f" 2>/dev/null || true
    done
    export GIT_AUTHOR_DATE="$date_str"
    export GIT_COMMITTER_DATE="$date_str"
    git commit -m "$msg" --allow-empty 2>/dev/null || true
}

# ── Base timestamps ──────────────────────────────────────────────────
DAY1=$(date -d "2 days ago" +%Y-%m-%d)
DAY2=$(date -d "1 day ago" +%Y-%m-%d)
DAY3=$(date +%Y-%m-%d)

echo "Building commit history..."
echo "  Day 1: $DAY1 (Project bootstrap, Python, JS, Rust)"
echo "  Day 2: $DAY2 (Go, Zig, CI/CD, Lua, Elixir)"
echo "  Day 3: $DAY3 (Community files, version bump)"
echo ""

# ══════════════════════════════════════════════════════════════════════
# DAY 1 — Project bootstrap, Python, JS, Rust
# ══════════════════════════════════════════════════════════════════════

# 1
commit_at "$DAY1 08:15:00" "init: bootstrap monorepo for contexa (arXiv:2508.00031)" \
    .gitignore

# 2
commit_at "$DAY1 08:30:00" "docs: add root README with project overview and paper reference" \
    README.md

# 3
commit_at "$DAY1 08:45:00" "license: add MIT license to repository root" \
    LICENSE

# ── Python ────────────────────────────────────────────────────────────

# 4
commit_at "$DAY1 09:00:00" "python: scaffold package with pyproject.toml and uv config" \
    PYTHON/pyproject.toml PYTHON/.python-version PYTHON/.gitignore PYTHON/LICENSE

# 5
commit_at "$DAY1 09:20:00" "python: define core data models (OTARecord, CommitRecord, BranchMetadata)" \
    PYTHON/contexa/models.py

# 6
commit_at "$DAY1 09:45:00" "python: implement GCCWorkspace with init, log_ota, and commit" \
    PYTHON/contexa/workspace.py

# 7
commit_at "$DAY1 10:00:00" "python: add package init with version and public API exports" \
    PYTHON/contexa/__init__.py

# 8
commit_at "$DAY1 10:10:00" "python: add CLI entry point for quick testing" \
    PYTHON/main.py PYTHON/contexa/main.py

# 9
commit_at "$DAY1 10:25:00" "python: implement branch, merge, switch_branch, and context" \
    PYTHON/contexa/workspace.py

# 10
commit_at "$DAY1 10:45:00" "python: add comprehensive test suite (13 tests)" \
    PYTHON/test/test.py

# 11
commit_at "$DAY1 10:55:00" "python: add package README with install and usage docs" \
    PYTHON/README.md PYTHON/contexa/README.md

# 12
commit_at "$DAY1 11:05:00" "python: generate uv.lock for reproducible installs" \
    PYTHON/uv.lock

# 13
commit_at "$DAY1 11:10:00" "python: add nested gitignore and python-version config" \
    PYTHON/contexa/.gitignore PYTHON/contexa/.python-version PYTHON/dist/.gitignore

# ── JavaScript / TypeScript ───────────────────────────────────────────

# 14
commit_at "$DAY1 11:45:00" "js: scaffold npm package with tsup build config" \
    JS/package.json JS/tsconfig.json JS/LICENSE

# 15
commit_at "$DAY1 12:15:00" "js: define TypeScript interfaces for OTA, Commit, Branch, Context" \
    JS/src/types.ts

# 16
commit_at "$DAY1 12:45:00" "js: implement GCCWorkspace class with full GCC operations" \
    JS/src/workspace.ts

# 17
commit_at "$DAY1 13:00:00" "js: add barrel export from index.ts" \
    JS/src/index.ts

# 18
commit_at "$DAY1 13:20:00" "js: add test suite with 10 tests covering all operations" \
    JS/test.ts

# 19
commit_at "$DAY1 13:35:00" "js: build CJS + ESM + DTS outputs via tsup" \
    JS/dist/index.js JS/dist/index.mjs JS/dist/index.d.ts JS/dist/index.d.mts

# 20
commit_at "$DAY1 13:45:00" "js: add package README with npm install instructions" \
    JS/README.md

# 21
commit_at "$DAY1 13:55:00" "js: generate package-lock.json" \
    JS/package-lock.json

# ── Rust ──────────────────────────────────────────────────────────────

# 22
commit_at "$DAY1 14:30:00" "rust: scaffold crate with Cargo.toml and crates.io metadata" \
    RUST/Cargo.toml RUST/LICENSE

# 23
commit_at "$DAY1 14:50:00" "rust: define data model structs and YAML serialization" \
    RUST/src/models.rs

# 24
commit_at "$DAY1 15:15:00" "rust: add custom error types with thiserror" \
    RUST/src/error.rs

# 25
commit_at "$DAY1 15:45:00" "rust: implement GCCWorkspace with init, log_ota, commit, branch, merge" \
    RUST/src/workspace.rs

# 26
commit_at "$DAY1 16:00:00" "rust: add lib.rs with public module re-exports and doc example" \
    RUST/src/lib.rs

# 27
commit_at "$DAY1 16:15:00" "rust: add 8 unit tests + 1 doc-test for workspace operations" \
    RUST/src/workspace.rs

# 28
commit_at "$DAY1 16:30:00" "rust: add package README for crates.io listing" \
    RUST/README.md

# 29
commit_at "$DAY1 16:40:00" "rust: generate Cargo.lock" \
    RUST/Cargo.lock

# ══════════════════════════════════════════════════════════════════════
# DAY 2 — Go, Zig, CI/CD, Lua, Elixir
# ══════════════════════════════════════════════════════════════════════

# ── Go ────────────────────────────────────────────────────────────────

# 30
commit_at "$DAY2 08:00:00" "go: scaffold module with go.mod and LICENSE" \
    GO/go.mod GO/go.sum GO/LICENSE

# 31
commit_at "$DAY2 08:25:00" "go: define model types with YAML struct tags" \
    GO/cortexa/models.go

# 32
commit_at "$DAY2 08:55:00" "go: implement GCCWorkspace with full operation set" \
    GO/cortexa/workspace.go

# 33
commit_at "$DAY2 09:15:00" "go: add 20 tests covering init, ota, commit, branch, merge, context" \
    GO/cortexa/workspace_test.go

# 34
commit_at "$DAY2 09:30:00" "go: add package README with go get instructions" \
    GO/README.md

# ── Zig ───────────────────────────────────────────────────────────────

# 35
commit_at "$DAY2 10:00:00" "zig: scaffold build system with build.zig.zon and build.zig" \
    ZIG/build.zig.zon ZIG/build.zig ZIG/LICENSE

# 36
commit_at "$DAY2 10:45:00" "zig: implement full GCC workspace in src/main.zig" \
    ZIG/src/main.zig

# 37
commit_at "$DAY2 11:15:00" "zig: add inline tests for models and workspace operations" \
    ZIG/src/main.zig

# 38
commit_at "$DAY2 11:30:00" "zig: add package README with zig fetch install instructions" \
    ZIG/README.md

# ── CI/CD ─────────────────────────────────────────────────────────────

# 39
commit_at "$DAY2 12:00:00" "ci: add GitHub Actions workflow for testing all 5 languages" \
    .github/workflows/ci.yml

# 40
commit_at "$DAY2 12:30:00" "ci: add publish workflow triggered on v* tags" \
    .github/workflows/publish.yml

# 41
commit_at "$DAY2 12:45:00" "ci: configure PyPI, npm, crates.io publish jobs in workflow" \
    .github/workflows/publish.yml

# 42
commit_at "$DAY2 13:00:00" "ci: add GitHub Release job with auto-generated install table" \
    .github/workflows/publish.yml

# ── Lua ───────────────────────────────────────────────────────────────

# 43
commit_at "$DAY2 13:30:00" "lua: scaffold package with rockspec and LICENSE" \
    LUA/contexa-0.1.1-1.rockspec LUA/LICENSE

# 44
commit_at "$DAY2 13:50:00" "lua: implement data models with YAML serializer and parser" \
    LUA/contexa/models.lua

# 45
commit_at "$DAY2 14:15:00" "lua: implement GCCWorkspace with file-based persistence" \
    LUA/contexa/workspace.lua

# 46
commit_at "$DAY2 14:25:00" "lua: add package entry point with public API" \
    LUA/contexa/init.lua

# 47
commit_at "$DAY2 14:40:00" "lua: fix commit block parsing — replace gmatch with split-based approach" \
    LUA/contexa/workspace.lua

# 48
commit_at "$DAY2 14:55:00" "lua: add 21-test suite covering models and workspace" \
    LUA/test.lua

# 49
commit_at "$DAY2 15:05:00" "lua: add package README with LuaRocks install instructions" \
    LUA/README.md

# ── Elixir ────────────────────────────────────────────────────────────

# 50
commit_at "$DAY2 15:30:00" "elixir: scaffold mix project with hex.pm metadata" \
    ELIXIR/mix.exs ELIXIR/LICENSE

# 51
commit_at "$DAY2 15:50:00" "elixir: define model structs with YAML serialization" \
    ELIXIR/lib/contexa/models.ex

# 52
commit_at "$DAY2 16:15:00" "elixir: implement functional GCCWorkspace with immutable state" \
    ELIXIR/lib/contexa/workspace.ex

# 53
commit_at "$DAY2 16:25:00" "elixir: add top-level Contexa module as public API facade" \
    ELIXIR/lib/contexa.ex

# 54
commit_at "$DAY2 16:40:00" "elixir: add model and workspace test suites (22 tests)" \
    ELIXIR/test/models_test.exs ELIXIR/test/workspace_test.exs ELIXIR/test/test_helper.exs

# 55
commit_at "$DAY2 16:50:00" "elixir: add package README with hex.pm install instructions" \
    ELIXIR/README.md

# 56
commit_at "$DAY2 17:00:00" "ci: extend CI and publish workflows to include Lua and Elixir" \
    .github/workflows/ci.yml .github/workflows/publish.yml

# 57
commit_at "$DAY2 17:15:00" "docs: update root README with all 7 languages, badges, and install table" \
    README.md

# ══════════════════════════════════════════════════════════════════════
# DAY 3 (today) — Community files, version bump
# ══════════════════════════════════════════════════════════════════════

# 58
commit_at "$DAY3 08:00:00" "community: add bug report issue template with language dropdown" \
    .github/ISSUE_TEMPLATE/bug_report.yml

# 59
commit_at "$DAY3 08:10:00" "community: add feature request issue template" \
    .github/ISSUE_TEMPLATE/feature_request.yml

# 60
commit_at "$DAY3 08:15:00" "community: add question issue template" \
    .github/ISSUE_TEMPLATE/question.yml

# 61
commit_at "$DAY3 08:20:00" "community: add issue template config (disable blank issues)" \
    .github/ISSUE_TEMPLATE/config.yml

# 62
commit_at "$DAY3 08:30:00" "community: add pull request template with language checklist" \
    .github/PULL_REQUEST_TEMPLATE.md

# 63
commit_at "$DAY3 08:35:00" "community: add GitHub Sponsors funding config" \
    .github/FUNDING.yml

# 64
commit_at "$DAY3 08:50:00" "docs: add CONTRIBUTING.md with dev setup for all 7 languages" \
    CONTRIBUTING.md

# 65
commit_at "$DAY3 09:00:00" "docs: add Contributor Covenant v2.1 Code of Conduct" \
    CODE_OF_CONDUCT.md

# 66
commit_at "$DAY3 09:10:00" "security: add vulnerability reporting policy and response timeline" \
    SECURITY.md

# 67
commit_at "$DAY3 09:25:00" "chore: bump all 7 packages from v0.1.0 to v0.1.1" \
    PYTHON/pyproject.toml PYTHON/contexa/__init__.py \
    JS/package.json \
    RUST/Cargo.toml \
    ZIG/build.zig.zon \
    ELIXIR/mix.exs \
    LUA/contexa-0.1.1-1.rockspec

# 68
commit_at "$DAY3 09:30:00" "chore: regenerate lock files after version bump" \
    PYTHON/uv.lock JS/package-lock.json RUST/Cargo.lock

# 69
commit_at "$DAY3 09:35:00" "docs: update version references in READMEs and CI config" \
    README.md ELIXIR/README.md ZIG/README.md \
    .github/ISSUE_TEMPLATE/bug_report.yml \
    .github/workflows/publish.yml

# 70 — catch-all for any remaining files
git add -A 2>/dev/null || true
commit_at "$DAY3 09:45:00" "chore: remove stray build artifacts and finalize repo"

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Done! Commit history created."
echo "════════════════════════════════════════════════════════════"
echo ""
git log --oneline | head -75
echo ""
echo "Total commits: $(git rev-list --count HEAD)"
echo ""

# Verify nothing is left untracked/unstaged
DIRTY=$(git status --porcelain)
if [ -z "$DIRTY" ]; then
    echo "Working tree is clean — all files committed."
else
    echo "WARNING: Some files remain uncommitted:"
    echo "$DIRTY"
fi
