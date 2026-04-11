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

- When creating a new git worktree, place it in '.local/git-worktrees/'.

## After Changing Code

Ensure all:

- Call-sites, benches, examples, READMEs, and tests, where required, are updated to reflect any changes made, where required.
- Code is covered above 80% coverage.

## File/Code Scanning

Ignore the following directories, and/or files, unless instructed otherwise:

- `_recycler/`
- `**/archived/`
- `**/.idea/`
