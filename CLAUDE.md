## Git

- Do not commit code directly to 'main' branch.
- Ensure you are on a feature branch before committing any work.
- Ensure a github issue ticket is already created before you carry out any work, unless asked otherwise.
- Include github issue ticket number in commit messages (e.g., `feat(ez-button): #7-hello-world ....`) and feature branch name.  If the issue is not known ask the user if they would like you to create an issue ticket for the changes.

## After Changing Code

Ensure all:

- Call-sites, benches, examples, READMEs, and tests, where required, are updated to reflect any changes made, where required.
- Code is covered above 80% coverage.

## File/Code Scanning

Ignore the following directories, and/or files, unless instructed otherwise:

- `_recycler/`
- `**/archived/`
- `**/.idea/`
