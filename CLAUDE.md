## Working Principles

Behavioral guidelines to reduce common LLM coding mistakes. These apply on top of the project-specific rules below. For trivial tasks, use judgment — these bias toward caution over speed.

### 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: every changed line should trace directly to the user's request.

Note: this does not override the **After Changing Code** section below. Updating call-sites, benches, examples, READMEs, and tests that are *affected by your change* is part of completing the task, not scope creep. "Surgical" means don't wander off fixing unrelated code — it does not mean leaving your own change half-applied.

### 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

## Git

- Use local `gh` tool for working with github issues.
- Do not commit code directly to 'main' branch, ensure you are in a git worktree branch, before committing any work, instead.
- Ensure an issue ticket is already created before you carry out any work, unless asked otherwise.
- Include issue ticket number in commit messages (e.g., `feat(ez-button): #7-hello-world ....`) and feature branch name.  If the issue ticket doesn't yet exist, ask the user if they would like you to create one.
- Before committing code ensure the following, on your branch:
  - fmt and clippy (with fix flag) are run.
  - Build, and Tests pass.
  - Code is covered above 80% coverage.

### Worktrees

- When creating a new git worktree, place it in '.claude/worktrees/'.  

## After Changing Code

Ensure all:

- Call-sites, benches, examples, READMEs, and tests, where required, are updated to reflect any changes made, where required.
- When a crate is added, renamed, or removed (in `crates/` and/or the workspace `Cargo.toml`), update the main repository `README.md` — sub-crates table, feature flags list, and any umbrella `walrs` crate re-exports / features — so it stays in sync with the current crate roster.
- Code is covered above 80% coverage.

## File/Code Scanning

Ignore the following directories, and/or files, unless instructed otherwise:

- `_recycler/`
- `**/archived/`
- `**/.idea/`
