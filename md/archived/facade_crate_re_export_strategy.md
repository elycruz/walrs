# Façade Crate Re-export Strategy for `walrs`

## Question

What is the strategy for making the root `walrs` crate export all sub-crates from `crates/`? Is this a common practice in the Rust community?

---

## Current State

- The workspace has **9 sub-crates**: `acl`, `digraph`, `filter`, `form`, `graph`, `inputfilter`, `navigation`, `rbac`, `validation`.
- The root `walrs` crate only depends on **5 of 9** sub-crates (`acl`, `graph`, `inputfilter`, `navigation`, `rbac`).
- `src/lib.rs` only re-exports two items from `walrs_fieldfilter`:

```rust
pub use walrs_fieldfilter::filters;
pub use walrs_fieldfilter::validators;
```

---

## Community Precedent

Yes, this is a well-known and common Rust pattern called a **"façade crate"** (or umbrella/meta-crate).

Notable examples:
- **`bevy`** — re-exports `bevy_core`, `bevy_render`, `bevy_ecs`, etc.
- **`embassy`** — re-exports `embassy-executor`, `embassy-time`, etc.
- **`tauri`** — re-exports internal sub-crates.
- **`aws-sdk-rust`** — SDK collection pattern.

The main tradeoff: the façade crate's compile time increases since it depends on everything. Feature-gating mitigates this.

---

## Proposed Strategy

### Step 1 — Add all sub-crates as dependencies in root `Cargo.toml`

```toml
[dependencies]
walrs_acl         = { path = "crates/acl" }
walrs_digraph     = { path = "crates/digraph" }
walrs_filter      = { path = "crates/filter" }
walrs_form        = { path = "crates/form" }
walrs_graph       = { path = "crates/graph" }
walrs_fieldfilter = { path = "crates/inputfilter" }
walrs_navigation  = { path = "crates/navigation" }
walrs_rbac        = { path = "crates/rbac" }
walrs_validation  = { path = "crates/validation" }
```

### Step 2 — Re-export in `src/lib.rs`

Two styles to choose from:

#### A) As named modules (recommended — preserves namespace)

```rust
pub use walrs_acl         as acl;
pub use walrs_digraph     as digraph;
pub use walrs_filter      as filter;
pub use walrs_form        as form;
pub use walrs_graph       as graph;
pub use walrs_fieldfilter as inputfilter;
pub use walrs_navigation  as navigation;
pub use walrs_rbac        as rbac;
pub use walrs_validation  as validation;
```

Users then write: `walrs::inputfilter::Field<String>`.

#### B) Flatten specific public items

```rust
pub use walrs_fieldfilter::Field;
pub use walrs_validation::Rule;
// ...etc.
```

Users write: `walrs::Field<String>` — flatter API, but harder to maintain and risks name collisions.

### Step 3 (Optional) — Feature-gate heavy or optional crates

```toml
[features]
default = ["inputfilter", "validation", "filter"]
full    = ["inputfilter", "validation", "filter", "form", "acl", "rbac", "navigation", "graph", "digraph"]

inputfilter = ["dep:walrs_fieldfilter"]
validation  = ["dep:walrs_validation"]
filter      = ["dep:walrs_filter"]
form        = ["dep:walrs_form"]
acl         = ["dep:walrs_acl"]
rbac        = ["dep:walrs_rbac"]
navigation  = ["dep:walrs_navigation"]
graph       = ["dep:walrs_graph"]
digraph     = ["dep:walrs_digraph"]

[dependencies]
walrs_acl         = { path = "crates/acl",         optional = true }
walrs_digraph     = { path = "crates/digraph",     optional = true }
walrs_filter      = { path = "crates/filter",      optional = true }
walrs_form        = { path = "crates/form",        optional = true }
walrs_graph       = { path = "crates/graph",       optional = true }
walrs_fieldfilter = { path = "crates/inputfilter", optional = true }
walrs_navigation  = { path = "crates/navigation",  optional = true }
walrs_rbac        = { path = "crates/rbac",        optional = true }
walrs_validation  = { path = "crates/validation",  optional = true }
```

With conditional re-exports in `src/lib.rs`:

```rust
#[cfg(feature = "acl")]
pub use walrs_acl as acl;

#[cfg(feature = "form")]
pub use walrs_form as form;

// ...etc.
```

---

## Recommendation

**Approach A (named modules) with optional feature gates** is the most idiomatic for a utility suite like `walrs`. It matches the patterns used by `bevy`, `embassy`, and similar projects.
