# Local Development & Publishing Workflow for walrs

## Question 1: Working with walrs locally while crates depend on each other and need to be published to crates.io

### Summary

Do **not** publish first. Use path dependencies and/or the `[patch]` section for local development.

---

### Option 1: Path deps in the external project (simplest for dev)

```toml
# external-project/Cargo.toml
[dependencies]
walrs_fieldfilter = { path = "/path/to/walrs/crates/fieldfilter" }
walrs_filter      = { path = "/path/to/walrs/crates/filter" }
```

Changes to walrs are immediately reflected. No publish required.

---

### Option 2: `[patch.crates-io]` (best of both worlds)

Use published version strings normally, but override with local source during development:

```toml
# external-project/Cargo.toml
[dependencies]
walrs_fieldfilter = "0.1.0"

[patch.crates-io]
walrs_fieldfilter = { path = "/path/to/walrs/crates/fieldfilter" }
walrs_filter      = { path = "/path/to/walrs/crates/filter" }
```

Remove the `[patch]` block to switch back to crates.io versions.

---

### For publishing to crates.io

All intra-workspace dependencies need **both `path` and `version`**:

```toml
# crates/fieldfilter/Cargo.toml
walrs_filter     = { path = "../filter",     version = "0.1.0", features = ["validation"] }
walrs_validation = { path = "../validation", version = "0.1.0" }
```

Locally, `path` wins. On crates.io, `version` is used.

---

## Question 2: Can cargo automatically resolve and publish all workspace crates in dependency order?

### Summary

`cargo publish` is per-crate and does **not** support `--workspace`. Use `cargo-release` (de-facto standard).

---

### `cargo-release` (recommended)

```bash
cargo install cargo-release

# Dry-run (safe preview)
cargo release publish --workspace

# Actually publish
cargo release publish --workspace --execute
```

Automatically:
- Walks the dependency graph to determine correct publish order
- Verifies each crate before publishing
- Optionally bumps versions, creates git tags, and pushes

---

### `cargo-workspaces` (simpler alternative)

```bash
cargo install cargo-workspaces
cargo ws publish
```

Simpler but less full-featured. Good if version/tag management isn't needed.

---

### Prerequisites

Every intra-workspace dep must have `version` alongside `path` (see above) or crates.io will reject the publish.
