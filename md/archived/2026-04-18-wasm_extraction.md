# WASM Extraction Plan

- **Date**: 2026-04-18 (updated 2026-04-26)
- **Status**: Ready for execution
- **Crates affected**: `walrs_acl`, `walrs_rbac`, plus two new crates (`walrs_acl_wasm`, `walrs_rbac_wasm`). `walrs_form` resolved by #267 (crate removed).
- **Related issues**: #243 (umbrella), #43 (closed — ACL WASM support, prior art), #267 (closed — `walrs_form` removal)
- **Related docs**: `md/discussions/value_type_scope_and_cms_tradeoffs.md`

## Table of Contents

- [1. Goal](#1-goal)
- [2. Background](#2-background)
- [3. Current State (Audit)](#3-current-state-audit)
- [4. Recommended Approach](#4-recommended-approach)
- [5. Extraction Sequence (per crate)](#5-extraction-sequence-per-crate)
- [6. Handling the "Default Opted-In WASM" Concern](#6-handling-the-default-opted-in-wasm-concern)
- [7. Crate Layout](#7-crate-layout)
- [8. CI & Tooling](#8-ci--tooling)
- [9. Success Criteria](#9-success-criteria)
- [10. Open Questions](#10-open-questions)
- [11. Out of Scope](#11-out-of-scope)
- [12. Verification](#12-verification)

---

## 1. Goal

Move the `#[wasm_bindgen]` code currently bundled inside `walrs_acl` and `walrs_rbac` into standalone sibling crates (`walrs_acl_wasm`, `walrs_rbac_wasm`), so that:

- Pure-Rust consumers of the core crates pay **no WASM cost** in build time, dependency surface, or docs.rs clutter.
- WASM bindings can version independently from core logic.
- The `walrs_form` crate's orphan WASM plumbing (deps with no code) gets resolved.

## 2. Background

Three costs currently hit users who do not want WASM:

1. **Build surface**: `crate-type = ["cdylib", "rlib"]` on `walrs_acl` and `walrs_rbac` forces dylib output on every build.
2. **Dependency opacity**: optional `wasm-bindgen`, `serde-wasm-bindgen`, `js-sys` deps live in the core `Cargo.toml`, muddying the docs.rs landing page and `cargo tree` output.
3. **Unconditional target deps in `walrs_form`**: `[target.'cfg(target_arch = "wasm32")'.dependencies]` auto-activates four WASM crates for anyone compiling to wasm32 — the `wasm` feature has no off-switch for this block.
4. **Coupled release cadence**: a JS-side API tweak forces a core crate republish.

WASM support is affirmed as a keep-goal by `md/discussions/value_type_scope_and_cms_tradeoffs.md` — this plan does not remove functionality, only relocates it.

## 3. Current State (Audit)

| Crate | `wasm` feature | In defaults? | WASM LOC | Isolation | Extraction difficulty |
|---|---|---|---|---|---|
| `walrs_acl` | yes | no (default=`std`) | 363 (`src/wasm.rs`) | Clean — single file | Easy |
| `walrs_rbac` | yes | no (default=`std`) | 196 (`src/wasm.rs`) | Clean — single file | Easy |
| `walrs_form` | yes (empty) | no (default=`std`) | 0 (deps only, no code) | N/A | Decision needed |
| 8 other crates | no | — | 0 | — | — |

**Shared traits of `acl` + `rbac` today**:
- `crate-type = ["cdylib", "rlib"]` in `[lib]`.
- Optional deps: `wasm-bindgen = 0.2`, `serde-wasm-bindgen = 0.6`, `js-sys = 0.3`.
- `lib.rs` gates: `#[cfg(feature = "wasm")] pub mod wasm; #[cfg(feature = "wasm")] pub use wasm::*;`.
- Build script `ci-cd-wasm.sh` → `wasm-pack build --target nodejs --no-default-features --features wasm`.
- JS test suite in `tests-js/` (Node built-in test runner).
- **No GitHub Actions job** covers WASM builds — only local `ci-cd-wasm.sh`.

**WASM API surface**:
- `walrs_acl`: `JsAcl`, `JsAclBuilder`, convenience fns `createAclFromJson`, `checkPermission`.
- `walrs_rbac`: `JsRbac`, `JsRbacBuilder`, convenience fns `createRbacFromJson`, `checkPermission`.

## 4. Recommended Approach

**Sibling-crate extraction with clean break** (§6 Option B) — confirmed 2026-04-26 after audit showed no external WASM consumers.

**Naming**: `walrs_<name>_wasm` (underscore suffix). Rationale:
- Matches walrs crate conventions.
- Groups alphabetically with the core on crates.io.
- Clearly subordinate — `walrs_acl_wasm` depends on `walrs_acl`, never the reverse.

**Why sibling over alternatives**:
- Removes `cdylib` from core builds.
- Cleans core `Cargo.toml` and docs.rs.
- Independent versioning.
- Precedent: `md/plans/value_feature_gating.md` Phase 2 proposes the same pattern for `walrs_dyn`.
- Reversible — additive change, no core behavior moves.

## 5. Extraction Sequence (per crate)

Apply **ACL first as pilot**, then RBAC once validated.

Per crate (Option B — clean break, no deprecation window):

1. Create `crates/<name>-wasm/` with `Cargo.toml`:
   ```toml
   [package]
   name = "walrs_<name>_wasm"
   version = "0.1.0"
   edition = "..."
   [lib]
   crate-type = ["cdylib"]
   [dependencies]
   walrs_<name> = { path = "../<name>" }
   wasm-bindgen = "0.2"
   serde-wasm-bindgen = "0.6"
   js-sys = "0.3"
   ```
2. Move files: `src/wasm.rs` → new crate's `src/lib.rs`; move `ci-cd-wasm.sh`, `tests-js/`, relevant `examples/wasm_example.rs` into the new crate.
3. Update new crate's `ci-cd-wasm.sh` to drop `--no-default-features --features wasm` (no longer needed — the new crate is WASM-only).
4. In the **core** crate (`walrs_acl` / `walrs_rbac`) — clean break in same PR:
   - Remove `crate-type = ["cdylib", "rlib"]` from `[lib]` (defaults to `rlib`).
   - Remove `wasm` feature from `[features]`.
   - Remove optional deps: `wasm-bindgen`, `serde-wasm-bindgen`, `js-sys`.
   - Remove `pub mod wasm;` and `pub use wasm::*;` gates from `lib.rs`.
   - Delete `src/wasm.rs` (now relocated).
5. Add the new crate to the workspace `Cargo.toml` `members`.
6. Update `README.md` sub-crates table and feature-flag list (CLAUDE.md requirement).

## 6. Handling the "Default Opted-In WASM" Concern

Three graded options, recorded for discussion:

| Option | Risk | Speed | User-visible break |
|---|---|---|---|
| **A** Sibling extract + deprecate core `wasm` feature for one release | Low | ~2 days/crate | None on release N; break on N+1 |
| **B (chosen)** Extract + remove `wasm` feature from core immediately | Medium | ~1 day/crate | Breaking on release day |
| **C** Keep in place, remove `cdylib` only (conditional on feature) | Low | ~2 h/crate | None — but doesn't solve deps-in-Cargo.toml or versioning coupling |

**Decision (2026-04-26)**: Option B. Q3 audit confirmed no external WASM consumers, so the deprecation window has no value. Bumps to `0.3.0` per breaking-change policy.

## 7. Crate Layout

```
crates/
├── acl/                    (walrs_acl — no cdylib after follow-up release)
│   └── src/
│       ├── lib.rs          (wasm module removed in follow-up release)
│       └── simple/
├── acl-wasm/               (walrs_acl_wasm — NEW)
│   ├── Cargo.toml
│   ├── src/lib.rs          (moved from acl/src/wasm.rs, 363 LOC)
│   ├── ci-cd-wasm.sh       (moved)
│   ├── examples/wasm_example.rs (moved from acl/)
│   └── tests-js/           (moved)
├── rbac/                   (walrs_rbac — same treatment)
│   └── src/
├── rbac-wasm/              (walrs_rbac_wasm — NEW)
│   ├── Cargo.toml
│   ├── src/lib.rs          (moved from rbac/src/wasm.rs, 196 LOC)
│   ├── ci-cd-wasm.sh       (moved)
│   └── tests-js/           (moved)
└── form/                   (walrs_form — target-gated WASM deps removed; see §10)
```

## 8. CI & Tooling

Add a new GitHub Actions job to `.github/workflows/build-and-test.yml` (or a sibling `wasm.yml`) that:

1. Installs Rust + `wasm32-unknown-unknown` target.
2. Installs `wasm-pack` and `wasm-opt`.
3. For each `*-wasm` crate: runs `wasm-pack build --target nodejs`, then `cd tests-js && npm install && npm test`.
4. Runs on PR and on push to `main`.

This closes the current gap where WASM changes can regress silently between local runs of `ci-cd-wasm.sh`.

## 9. Success Criteria

- `cargo tree -p walrs_acl` shows **zero** WASM-adjacent deps.
- `cargo build -p walrs_acl` produces only `rlib`, not `cdylib`.
- `cargo build -p walrs_acl_wasm --target wasm32-unknown-unknown` succeeds.
- All pre-existing JS tests in `tests-js/` pass unchanged against the new crate.
- `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --check` all clean.
- Coverage per CLAUDE.md (>80%) holds — core coverage unchanged; WASM coverage via existing JS suite.
- New WASM CI job is green.
- `README.md` updated.

## 10. Open Questions

1. **What does "default opted-in WASM" specifically mean to the user?** Three readings, all addressed by this plan but with different emphasis. Confirm before execution so §6 landing is right.
2. **`walrs_form` WASM plumbing**: deps exist, no code. Three options:
   - Remove deps entirely; re-add when real code lands.
   - Pre-create `walrs_form_wasm` shell now (premature).
   - Defer to `md/plans/form_serde_design.md` — that plan already scopes WASM for form.
   - **Recommend**: remove deps, defer to form_serde. Sub-issue to track.
3. **External WASM consumers?** If none, Option B in §6 is cleaner than A. Confirm before release.
4. **Versioning starting point for `*_wasm` crates**: `0.1.0` (match core) or `0.0.1` (independent maturity signal)? Recommend `0.1.0`.
5. **Label creation**: repo lacks `wasm` and `refactor` labels — create them as part of issue setup.

## 11. Out of Scope

- Adding `tsify` / TypeScript type generation.
- `web-sys`-based browser work beyond existing `wasm-bindgen` bindings.
- Wrapping any other crate in WASM (validation, filter, fieldfilter, navigation, digraph, graph, fieldset_derive).
- Broader browser-target polish (loaders, bundler integrations).
- The follow-up release that actually removes `cdylib` and the `wasm` feature from core — tracked as a sub-issue, executed after deprecation window.

## 12. Verification

Per `*_wasm` crate, once extracted:

1. `cargo build -p walrs_acl` — must NOT pull `wasm-bindgen` (after follow-up release). `cargo tree -p walrs_acl | grep -i wasm` → empty.
2. `cargo build -p walrs_acl_wasm --target wasm32-unknown-unknown` — succeeds.
3. `cd crates/acl-wasm && bash ci-cd-wasm.sh` — wasm-pack build + JS tests pass identically to pre-extraction baseline.
4. `cargo test --workspace` — unchanged results.
5. `cargo clippy --workspace --all-targets -- -D warnings` — clean.
6. `cargo fmt --check` — clean.
7. `cargo doc --workspace --no-deps` — core docs no longer mention `wasm` module / WASM deps; `*_wasm` crate docs show the JS API.
8. New GitHub Actions WASM job green on the PR.
9. `README.md` sub-crates table includes the new crates.
