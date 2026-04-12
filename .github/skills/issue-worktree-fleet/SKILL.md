---
name: issue-worktree-fleet
description: >
  Orchestrates work on a GitHub issue by decomposing it into independent work units,
  creating git worktrees for each, and dispatching sub-agents (fleet) to implement,
  test, commit, and open PRs in parallel. Use this when asked to work on a GitHub
  issue that involves multiple independent changes across files or crates.
---

# Issue Worktree Fleet Skill

You are an orchestrator that breaks a GitHub issue into parallel work units, assigns each to a sub-agent running in its own git worktree, and ensures every unit is committed and opened as a PR.

## Workflow Overview

```
Issue → Analyze → Decompose → Create Worktrees → Dispatch Fleet → Commit & PR
```

## Phase 1: Issue Analysis

1. **Fetch the issue** using the GitHub MCP server (`gh issue view <number>`) or the `github-mcp-server-issue_read` tool.
2. Read the issue title, body, labels, and any linked sub-issues or comments to fully understand the scope.
3. Identify the **acceptance criteria** — what must be true for the issue to be considered done.

## Phase 2: Work Decomposition

Break the issue into **logical work units** — independent, parallelizable chunks of work. Each work unit should:

- Be **self-contained**: completable without depending on another unit's uncommitted changes.
- Have a **clear boundary**: specific files, modules, crates, or concerns.
- Be **testable independently**: its changes can be validated in isolation.

### Decomposition Strategies

| Issue Type | Strategy |
|---|---|
| Multi-crate change | One unit per crate |
| Feature + tests + docs | Separate units for impl, tests, docs/examples |
| Refactor across modules | One unit per module or logical grouping |
| Bug fix + regression test | Single unit (tightly coupled) |
| Multiple independent fixes | One unit per fix |

### Work Unit Definition

For each unit, define:

- **ID**: kebab-case identifier (e.g., `add-filter-op-bool`, `update-readme`)
- **Title**: short description of the work
- **Description**: detailed instructions — files to touch, behavior to implement, tests to write
- **Branch name**: `<issue-number>-<unit-id>` (e.g., `42-add-filter-op-bool`)
- **Dependencies**: list of unit IDs this unit depends on (empty if independent)

### Dependency Handling

- Units with **no dependencies** on each other can run in parallel.
- If unit B depends on unit A's output, run A first, then create B's worktree from A's branch.
- Minimize dependencies — prefer independent units wherever possible.
- If the entire issue is tightly coupled, use a **single work unit** instead of forcing artificial splits.

## Phase 3: Worktree Setup

For each work unit, create a git worktree:

```bash
# From the repository root
git worktree add .claude/worktrees/<issue-number>-<unit-id> -b <issue-number>-<unit-id> main
```

### Rules

- All worktrees go in `.claude/worktrees/` (per project convention).
- Branch from `main` (or from a dependency's branch if sequential).
- Verify the worktree was created successfully before dispatching an agent.

## Phase 4: Fleet Dispatch

Dispatch sub-agents to work in their respective worktrees. Use fleet mode (`/fleet`) or the `task` tool with `agent_type: "general-purpose"`, `mode: "background"`, and `model: "claude-opus-4.6"`.

### Sub-Agent Prompt Template

Each sub-agent receives a prompt containing:

1. **Context**: The full issue description and acceptance criteria.
2. **Assignment**: The specific work unit's title, description, and scope.
3. **Working directory**: The absolute path to their worktree.
4. **Branch name**: The branch they are on.
5. **Conventions**: The project's coding and git conventions (below).
6. **Completion criteria**: What "done" looks like for this unit.

### Example Sub-Agent Prompt

```
You are working on issue #<NUMBER>: "<ISSUE TITLE>".

Your assignment is work unit "<UNIT TITLE>":
<UNIT DESCRIPTION>

## Working Directory
Your working directory is: <WORKTREE_PATH>
Change to this directory before doing any work: `cd <WORKTREE_PATH>`

## Branch
You are on branch: <BRANCH_NAME>

## Scope
Only modify files relevant to this work unit. Do not touch files outside your scope.

## Files in Scope
<LIST OF FILES>

## Conventions

### Code Quality (MUST pass before committing)
1. `cargo fmt --manifest-path <WORKTREE_PATH>/Cargo.toml`
2. `cargo clippy --fix --allow-dirty --manifest-path <WORKTREE_PATH>/Cargo.toml -- -D warnings`
   - If clippy --fix cannot auto-fix, manually fix the warnings.
3. `cargo build --workspace --manifest-path <WORKTREE_PATH>/Cargo.toml`
4. `cargo test --workspace --manifest-path <WORKTREE_PATH>/Cargo.toml`
5. Ensure code coverage is above 80% for changed code.

### Commit Messages
- Format: `<type>(<scope>): #<issue-number>-<unit-id> <description>`
- Types: feat, fix, refactor, test, docs, chore
- Example: `feat(walrs_filter): #42-add-filter-op-bool add bool support to FilterOp`
- Include trailer: `Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>`

### What to Update
- Implementation code
- Unit tests and doc tests for changed code
- Examples if behavior changes
- README/docs if public API changes
- Benchmarks if performance-sensitive

## When Done
After all changes are implemented, tested, and passing:

1. Stage and commit your changes:
   ```bash
   cd <WORKTREE_PATH>
   git add -A
   git commit -m "<COMMIT_MESSAGE>

   Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
   ```

2. Push the branch:
   ```bash
   git push origin <BRANCH_NAME>
   ```

3. Open a pull request:
   ```bash
   gh pr create \
     --title "<type>(<scope>): #<issue-number> <unit-title>" \
     --body "## Summary

   <Describe what this PR does>

   ## Related Issue

   Closes part of #<issue-number>

   ## Work Unit

   **ID**: <unit-id>
   **Scope**: <brief scope description>

   ## Changes

   - <list of changes>

   ## Testing

   - <how changes were tested>" \
     --base main \
     --head <BRANCH_NAME>
   ```

4. Report back with: the PR URL, a summary of changes made, and test results.
```

## Phase 5: Monitoring & Completion

1. **Monitor** sub-agents as they work. Use `read_agent` or `/tasks` to check progress.
2. **Collect results** — PR URLs, summaries, any failures.
3. **Handle failures**:
   - If a sub-agent fails, read its output, diagnose the issue, and either retry or handle manually.
   - If a worktree is in a bad state, clean it up: `git worktree remove .claude/worktrees/<name>`.
4. **Report** to the user:
   - List all PRs created with URLs.
   - Summarize what each PR does.
   - Note any units that failed or need manual attention.
   - If all units are complete, optionally suggest closing the issue or note which PR(s) should close it.

## Phase 6: Cleanup

After all work is done and reported:

```bash
# Remove worktrees (from repo root)
git worktree remove .claude/worktrees/<issue-number>-<unit-id>

# Prune stale worktree references
git worktree prune
```

## Important Guidelines

### Do NOT

- Commit directly to `main` — always use feature branches via worktrees.
- Force-split tightly coupled changes into separate units — keep them together.
- Leave worktrees behind after work is complete.
- Skip fmt, clippy, build, or test steps before committing.
- Use `--all-features` flag (fn_traits requires nightly and will fail on stable).

### DO

- Use `gh` CLI for all GitHub operations (issues, PRs).
- Include the issue number in every branch name and commit message.
- Run the full quality pipeline (fmt → clippy → build → test) before committing.
- Create worktrees in `.claude/worktrees/`.
- Prefer fewer, well-scoped units over many tiny ones.
- When in doubt about decomposition, ask the user.

### Single-Unit Fallback

If the issue is small or tightly coupled (e.g., a simple bug fix), skip fleet dispatch entirely. Instead:

1. Create one worktree.
2. Do the work directly (no sub-agent needed).
3. Commit and open one PR.

## Quick Reference

```
Worktree path:  .claude/worktrees/<issue>-<unit-id>
Branch pattern: <issue>-<unit-id>
Commit format:  <type>(<scope>): #<issue>-<unit-id> <message>
PR base:        main
Quality gate:   cargo fmt → clippy --fix → build → test
Coverage:       80%+ on changed code
```
