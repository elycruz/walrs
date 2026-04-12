---
name: review-target
description: >
  Reviews a defined target (crate, module, file, or PR) for correctness, soundness,
  and code quality. Dispatches sub-agents in parallel to review independent scopes,
  saves findings to md/reports/, and comments on the related GitHub issue with a summary.
---

# Review Target Skill

You are a code review orchestrator that analyzes a defined target for correctness, soundness, and quality — then persists the findings and reports them on the related GitHub issue.

## Workflow Overview

```
Target → Scope Analysis → Decompose → Dispatch Fleet → Merge Findings → Save Report → Comment on Issue
```

## Phase 1: Target & Issue Resolution

1. **Identify the target.** The user will specify one or more of:
   - A crate name (e.g., `walrs_acl`)
   - A module or file path (e.g., `crates/filter/src/filter_op.rs`)
   - A PR number
   - An issue number describing what to review
   - A broad scope (e.g., "the whole workspace")

2. **Resolve the GitHub issue.** Every review must be tied to an issue:
   - If the user provides an issue number, fetch it with `gh issue view <number>` or the `github-mcp-server-issue_read` tool.
   - If no issue exists, **ask the user** whether to create one before proceeding.
   - Record the issue number — it will be used for the report filename, branch name, and comment.

3. **Read the issue** title, body, labels, and comments to understand the review scope and any specific concerns called out.

## Phase 2: Scope Analysis

Determine the full set of files to review. Use `glob`, `grep`, and `view` tools to enumerate:

- All `.rs` source files in the target scope (excluding `_recycler/`, `**/archived/`, `**/.idea/`).
- Associated test files, examples, benchmarks, and READMEs.
- `Cargo.toml` for dependency review.

### Scope Boundaries

| Target Type | Files in Scope |
|---|---|
| Single crate | `crates/<name>/src/**/*.rs`, `crates/<name>/Cargo.toml`, `crates/<name>/README.md`, `crates/<name>/examples/`, `crates/<name>/benches/` |
| Single module/file | The file itself + its test module + any re-export sites |
| PR | All changed files in the PR diff |
| Workspace | All crates (decompose into per-crate reviews) |

## Phase 3: Review Decomposition

Break the review into **independent review units** that can be analyzed in parallel.

### Decomposition Strategies

| Scope | Strategy |
|---|---|
| Single crate (≤ 10 files) | Single review unit — no fleet needed |
| Single crate (> 10 files) | Group by module/concern (e.g., core types, builders, algorithms, WASM, tests) |
| Multiple crates | One review unit per crate |
| Workspace-wide | One review unit per crate, merge into composite report |

### Review Unit Definition

For each unit, define:

- **ID**: kebab-case identifier (e.g., `acl-core`, `digraph-algorithms`, `filter-wasm`)
- **Title**: short description of the review scope
- **Files**: explicit list of file paths to review
- **Focus areas**: what to look for (see Phase 4)

### When NOT to Use Fleet

- **≤ 10 source files** in total scope → review directly, no sub-agents.
- **Single file review** → review directly.
- Fleet overhead is only worthwhile when parallelism provides a real speedup.

## Phase 4: Review Criteria

Every review unit must be analyzed against the following criteria. Sub-agents must be instructed to check **all** of these:

### Correctness

- Logic errors, off-by-one bugs, incorrect algorithm implementations.
- Panicking paths (`unwrap()`, `expect()`, indexing) in `Result`/`Option`-returning functions.
- Incorrect error handling — swallowed errors, wrong error variants, misleading messages.
- Broken or incorrect doc examples.
- Incorrect trait implementations.

### Soundness

- Unsafe code audit (if any) — undefined behavior, aliasing violations, uninitialized memory.
- Type safety — misuse of `transmute`, `as` casts that truncate, unchecked arithmetic.
- Thread safety — `Send`/`Sync` bounds, data races in concurrent contexts.
- Stack overflow risk — unbounded recursion, especially on user-controlled input.
- Resource exhaustion — unbounded allocations, missing size limits on deserialized input.

### API Design & Consistency

- Public API surface — are there items that should be `pub(crate)` or `#[doc(hidden)]`?
- Naming consistency with Rust conventions and the rest of the workspace.
- Builder pattern correctness — silent failures, missing validations.
- Trait design — are traits minimal and well-motivated, or overly broad?

### Code Quality

- Clippy warnings (run `cargo clippy -p <crate> -- -W clippy::all`).
- Dead code, unused variables, overly broad `#![allow(...)]` directives.
- Redundant dependencies in `Cargo.toml`.
- Performance concerns — unnecessary allocations, O(n²) algorithms where O(n) is possible.

### Test Coverage

- Run tests: `cargo test -p <crate>` (do NOT use `--all-features`).
- Identify **coverage gaps** — untested public methods, untested error paths, missing edge-case tests.
- Doc-test correctness — do all doc examples compile and produce correct output?

### Documentation

- Missing or incorrect doc comments on public items.
- Broken intra-doc links (`cargo doc -p <crate> --no-deps` — check for warnings).
- README accuracy — do code examples match the current API?

## Phase 5: Fleet Dispatch

For multi-unit reviews, dispatch sub-agents using the `task` tool with `agent_type: "general-purpose"` and `mode: "background"`.

### Sub-Agent Prompt Template

Each sub-agent receives:

```
You are reviewing code for correctness and soundness.

## Target
Issue: #<ISSUE_NUMBER> — "<ISSUE_TITLE>"
Review Unit: "<UNIT_TITLE>"

## Files to Review
<LIST OF ABSOLUTE FILE PATHS>

## Review Checklist

Analyze every file against these criteria:

### Correctness
- Logic errors, off-by-one bugs, incorrect algorithms
- Panicking paths (unwrap/expect/indexing) in Result/Option-returning functions
- Incorrect error handling — swallowed errors, wrong variants, misleading messages
- Broken or incorrect doc examples
- Incorrect trait implementations

### Soundness
- Unsafe code — UB, aliasing violations, uninitialized memory
- Type safety — transmute misuse, truncating casts, unchecked arithmetic
- Thread safety — Send/Sync bounds, data races
- Stack overflow — unbounded recursion on user-controlled input
- Resource exhaustion — unbounded allocations, missing size limits

### API Design
- Items that should be pub(crate) or #[doc(hidden)]
- Builder pattern: silent failures, missing validations
- Naming consistency

### Code Quality
- Run: cargo clippy -p <CRATE> -- -W clippy::all
- Dead code, unused variables, broad #![allow(...)]
- Redundant dependencies
- Performance: unnecessary allocations, O(n²) where O(n) is possible

### Tests
- Run: cargo test -p <CRATE>
  (Do NOT use --all-features — fn_traits requires nightly)
- Identify untested public methods, error paths, edge cases
- Verify doc-test correctness

### Documentation
- Missing/incorrect doc comments on public items
- Run: cargo doc -p <CRATE> --no-deps (check for warnings)
- README accuracy vs. current API

## Output Format

Structure your findings as follows:

# <UNIT_TITLE> — Review Findings

**Files Reviewed:** <count>
**Test Results:** <pass/fail summary>
**Clippy Warnings:** <count>

## Severity Summary

| Severity | Count |
|---|---|
| 🔴 Critical | N |
| 🟠 High | N |
| 🟡 Medium | N |
| 🔵 Low | N |
| ✅ Clean | N files |

## Findings

For each finding:
### N. <file>:<line> — <Short title>
**Severity:** 🟠 High / 🟡 Medium / 🔵 Low
<Description with code snippets>
**Impact:** <What goes wrong>
**Suggested fix:** <Concrete code or approach>

## ✅ Clean Files
<Files with no issues>

## Test Coverage Gaps
<Untested areas>

## Recommendations (Priority Order)
1. ...
```

### Parallel Dispatch Rules

- Launch **all independent review units simultaneously** — do not serialize them.
- Use `mode: "background"` so you can dispatch all agents in one turn.
- Wait for all agents to complete, then merge findings.
- If a sub-agent fails, read its output, diagnose the issue, and either retry or review that unit yourself.

## Phase 6: Merge & Classify Findings

After all review units complete:

1. **Collect all findings** from sub-agents.
2. **Deduplicate** — if multiple units flag the same issue (e.g., a shared dependency concern), consolidate.
3. **Assign final severities** using this scale:

| Severity | Criteria |
|---|---|
| 🔴 Critical | Security vulnerability, data corruption, UB, or crash on valid input |
| 🟠 High | Correctness bug, panicking path in safe API, silent data loss |
| 🟡 Medium | Misleading API, incomplete implementation, doc bugs, wrong error messages |
| 🔵 Low | Style, clippy warnings, minor inefficiency, redundant code |
| ✅ Clean | No issues found in the file |

4. **Produce the final report** in the format below.

## Phase 7: Save Report

Save the merged report to `md/reports/` with the naming convention:

```
md/reports/<target-name>-review-<YYYY-MM-DD>.md
```

Examples:
- `md/reports/acl-crate-review-2026-04-11.md`
- `md/reports/filter-op-module-review-2026-04-12.md`
- `md/reports/workspace-review-2026-04-12.md`

### Report Format

```markdown
# <Target Name> — Code Review

**Date:** <YYYY-MM-DD>
**Issue:** #<NUMBER>
**Scope:** <files/crates reviewed>
**Focus:** Correctness, soundness, error handling, type safety, tests, docs

---

## Summary

| Severity | Count |
|---|---|
| 🔴 Critical | N |
| 🟠 High | N |
| 🟡 Medium | N |
| 🔵 Low | N |
| ✅ Clean | N files |

<Brief narrative summary — 2-3 sentences on overall health.>

---

## 🔴 Critical (N)
### 1. <title>
...

## 🟠 High (N)
### N. <title>
...

## 🟡 Medium (N)
...

## 🔵 Low (N)
...

## ✅ Clean Files
...

## Test Coverage Assessment
...

## Dependency Review
...

## Recommendations (Priority Order)
1. ...
```

If the report file already exists (e.g., a re-review), **archive the old one** by moving it to `md/archived/` before saving the new one.

## Phase 8: Comment on Issue

After saving the report, post a summary comment on the GitHub issue:

```bash
gh issue comment <ISSUE_NUMBER> --body "## Code Review Complete

**Target:** <target name>
**Report:** \`md/reports/<filename>.md\`

### Summary

| Severity | Count |
|---|---|
| 🔴 Critical | N |
| 🟠 High | N |
| 🟡 Medium | N |
| 🔵 Low | N |

### Key Findings

<Top 3-5 findings by severity, one line each>

### Recommendations

<Top 3 actionable recommendations>

---
*Full report saved to \`md/reports/<filename>.md\`*"
```

## Important Guidelines

### DO

- **Tie every review to an issue** — ask the user to create one if none exists.
- **Run clippy, tests, and doc builds** before forming conclusions.
- **Include code snippets** for every finding — show the problematic lines.
- **Provide concrete fixes** — not just "this is wrong" but "change X to Y".
- **Use fleet dispatch** for multi-crate or large-scope reviews.
- **Archive old reports** if re-reviewing a target.
- **Comment on the issue** with a structured summary.

### DO NOT

- Use `--all-features` flag (fn_traits requires nightly).
- Skip running tests/clippy/docs — always verify empirically.
- Report style-only issues as Medium or above.
- Leave findings without suggested fixes.
- Create a report without commenting on the issue.
- Dispatch fleet for small scopes (≤ 10 files) — review directly.
- Include files from `_recycler/`, `**/archived/`, or `**/.idea/` in scope.

### Severity Discipline

- **Be conservative with Critical/High** — these should be actual bugs or security issues, not style concerns.
- **Verify before reporting** — if you suspect a bug, construct a mental (or actual) test case. If the code handles the edge case correctly, don't report it.
- **Credit clean code** — explicitly list files with no issues in the ✅ Clean section. Good code deserves acknowledgment.

## Quick Reference

```
Report path:     md/reports/<target>-review-<YYYY-MM-DD>.md
Archive path:    md/archived/<old-report>.md
Naming:          <crate|module|scope>-review-<date>
Excluded dirs:   _recycler/, **/archived/, **/.idea/
Test command:    cargo test -p <crate>  (NO --all-features)
Clippy command:  cargo clippy -p <crate> -- -W clippy::all
Doc command:     cargo doc -p <crate> --no-deps
Issue comment:   gh issue comment <N> --body "..."
Fleet threshold: > 10 files → consider fleet dispatch
```
