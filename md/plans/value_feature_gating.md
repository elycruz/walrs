# Plan: Feature-Gate the `Value` Type (Phase 1) & Extract Dynamic Path (Phase 2)

**Date:** 2026-04-18
**Status:** Planning
**Tracking issue:** #240 (Phase 1)
**Crates affected:** `walrs_validation`, `walrs_filter`, `walrs_fieldfilter`, `walrs_form`, `walrs_fieldset_derive`
**Related:**
- [`md/discussions/value_feature_gating_and_isolation.md`](../discussions/value_feature_gating_and_isolation.md) — architectural analysis
- [`md/discussions/value_type_scope_and_cms_tradeoffs.md`](../discussions/value_type_scope_and_cms_tradeoffs.md) — scope decisions
- [`md/discussions/value_enum_impedance_mismatch.md`](../discussions/value_enum_impedance_mismatch.md) — prior `Value` vs. typed analysis
- #88 (closed) — `Fieldset` trait design for typed multi-field validation

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Current State](#2-current-state)
3. [Phase 1 — Feature-Gate In Place](#3-phase-1--feature-gate-in-place)
   - [3.1 Scope](#31-scope)
   - [3.2 Step-by-Step Tasks](#32-step-by-step-tasks)
   - [3.3 Success Criteria](#33-success-criteria)
   - [3.4 Risks & Mitigations](#34-risks--mitigations)
4. [Phase 2 — Evaluate Extraction](#4-phase-2--evaluate-extraction)
   - [4.1 Triggers](#41-triggers-to-start-phase-2)
   - [4.2 Option B1: Narrow Extraction](#42-option-b1--narrow-extraction)
   - [4.3 Option B2: Wide Extraction](#43-option-b2--wide-extraction)
   - [4.4 Decision Criteria](#44-decision-criteria)
5. [Out of Scope](#5-out-of-scope)
6. [Open Questions](#6-open-questions)

---

## 1. Problem Statement

The `Value` enum and its `Rule<Value>` dispatch (~2,040 LOC in
`walrs_validation` alone) ship unconditionally. Consumers who only
want typed validation (`Rule<T>`) or typed multi-field validation
(`Fieldset` via derive) still pay the compile cost of the dynamic
path.

`Value` serves genuine use cases — runtime-schema CMS forms,
config-driven pipelines, WASM `web_sys::FormData` — so removing it
isn't acceptable. The goal is to make it **opt-out** for typed-only
consumers while preserving the dynamic story.

See the companion analysis document for the full options evaluation.
This plan executes the recommended **Option C (Hybrid, Phased)**.

---

## 2. Current State

### Feature graph today

```toml
# walrs_validation/Cargo.toml
default = ["serde_json_bridge"]
async = []
serde_json_bridge = ["dep:serde_json"]   # gates From<serde_json::Value> + attributes module
chrono = ["dep:chrono"]
jiff = ["dep:jiff"]
```

- `Value` itself is unconditional (`crates/validation/src/lib.rs:96`,
  `:105`).
- `Rule<Value>` impls unconditional
  (`crates/validation/src/rule_impls/mod.rs` declares `mod value;`
  without gating).
- Only `attributes.rs` and `From<serde_json::Value>` are gated
  (under `serde_json_bridge`).

### Downstream coupling

| Crate | `walrs_validation` dep | `Value` in public API |
|---|---|---|
| `walrs_filter` | optional (`validation` feature, default-on) | `FilterOp<Value>` / `TryFilterOp<Value>` impls |
| `walrs_fieldfilter` | **mandatory** | `Field<Value>`, `FieldFilter` |
| `walrs_form` | **mandatory** | `FormData = IndexMap<String, Value>` |
| `walrs_fieldset_derive` | none direct | Emits `walrs_validation::Value::...` in generated code |

### LOC footprint

| File | Lines | Role |
|---|---:|---|
| `crates/validation/src/value.rs` | 602 | Enum, `From`/`Display`/`PartialOrd`, `ValueExt`, bridge (partial gate) |
| `crates/validation/src/rule_impls/value.rs` | 1,439 | `Rule<Value>` dispatch (~526 impl + ~900 tests) |
| `crates/filter/src/filter_op.rs` | 1,202 (102 `Value` refs) | `FilterOp<Value>` impl block |
| `crates/filter/src/try_filter_op.rs` | 597 (35 `Value` refs) | `TryFilterOp<Value>` impl block |

---

## 3. Phase 1 — Feature-Gate In Place

### 3.1 Scope

Add a new `value` feature to `walrs_validation` that gates the `Value`
enum, its `Rule<Value>` / `Condition<Value>` dispatch, and the
`attributes` module. Propagate the feature through `walrs_filter`.
Leave `walrs_fieldfilter` and `walrs_form` *requiring* the feature — no
API change there yet.

Default feature set **stays Value-enabled** so no downstream user sees
breakage.

### 3.2 Step-by-Step Tasks

#### Task 1 — `walrs_validation` feature + gating

`crates/validation/Cargo.toml`:

```toml
[features]
default = ["value", "serde_json_bridge"]
async = []
value = []
serde_json_bridge = ["dep:serde_json", "value"]   # bridge implies Value exists
chrono = ["dep:chrono"]
jiff = ["dep:jiff"]
```

`crates/validation/src/lib.rs`:

```rust
#[cfg(feature = "value")]
#[cfg_attr(docsrs, doc(cfg(feature = "value")))]
pub mod value;

#[cfg(feature = "value")]
#[cfg_attr(docsrs, doc(cfg(feature = "value")))]
pub use value::*;
```

`crates/validation/src/rule_impls/mod.rs`:

```rust
#[cfg(feature = "value")]
pub(crate) mod value;

#[cfg(all(feature = "value", feature = "serde_json_bridge"))]
pub(crate) mod attributes;
```

Audit any stray `impl ... for Value` or `impl ... for Rule<Value>`
blocks in other files (`rule.rs`, `traits.rs`) and guard individually.

#### Task 2 — `walrs_filter` feature propagation

`crates/filter/Cargo.toml`:

```toml
[features]
default = ["validation", "value"]
fn_traits = []
nightly = ["fn_traits"]
validation = ["dep:walrs_validation"]
value = ["walrs_validation/value"]
```

Gate `FilterOp<Value>` and `TryFilterOp<Value>` blocks in
`crates/filter/src/filter_op.rs` and `try_filter_op.rs` with
`#[cfg(feature = "value")]`.

#### Task 3 — `walrs_fieldfilter` feature pass-through (required, no opt-out yet)

`crates/fieldfilter/Cargo.toml`:

```toml
[dependencies]
walrs_validation = { path = "../validation", features = ["value"] }
walrs_filter = { path = "../filter", features = ["validation", "value"] }
```

Explicitly request the `value` feature so `walrs_fieldfilter` continues
to build regardless of what other consumers select.

Document in `crates/fieldfilter/README.md` that `walrs_fieldfilter`
currently requires the dynamic `Value` path and that typed-only
consumers should use `Rule<T>` directly from `walrs_validation` (or
wait for Phase 2).

#### Task 4 — `walrs_form` feature pass-through (same treatment)

Mirror Task 3 for `crates/form/Cargo.toml`. Update
`crates/form/README.md` similarly.

#### Task 5 — Rustdoc configuration

`walrs_validation` and `walrs_filter` already have
`all-features = true` under `[package.metadata.docs.rs]` — verify it's
present on `walrs_fieldfilter` and `walrs_form` too; add if missing so
docs.rs renders the full API.

Use `#[cfg_attr(docsrs, doc(cfg(feature = "value")))]` on every gated
item so docs.rs shows the feature requirement badges.

#### Task 6 — CI matrix additions

Extend the workspace CI (or add if absent) to run:

```
cargo hack check --workspace --feature-powerset --exclude-features nightly
cargo hack test  --workspace --feature-powerset --exclude-features nightly
```

Or at minimum these three configurations per crate:

1. `--no-default-features`
2. Default features
3. `--all-features`

The `--no-default-features` build on `walrs_validation` must compile
and pass tests without `Value` present.

#### Task 7 — Examples & benches audit

- `crates/validation/examples/value_validation.rs` — already uses
  `#[cfg(feature = "serde_json_bridge")]`. Ensure the example is gated
  on `value` too (or implied, since `serde_json_bridge` → `value`).
- Benchmarks referencing `Value` — guard with `#[cfg(feature = "value")]`
  or mark as `required-features = ["value"]` in `Cargo.toml`.

#### Task 8 — Documentation

- Update `crates/validation/README.md` with a "Feature Flags" section
  (already partially there for `serde_json_bridge`), listing `value`
  and its implications.
- Update the workspace root `README.md` if it documents features.
- Add a one-line note in the root `CLAUDE.md`? (No — CLAUDE.md is for
  instructions, not status.)

#### Task 9 — Release notes

Add a `CHANGELOG.md` entry (or equivalent) for the next
`walrs_validation` release noting the new feature and that default
behavior is unchanged.

### 3.3 Success Criteria

- [ ] `cargo build -p walrs_validation --no-default-features` succeeds.
- [ ] `cargo build -p walrs_validation --no-default-features --features chrono` succeeds (a representative non-Value feature still works).
- [ ] `cargo test -p walrs_validation --no-default-features` passes.
- [ ] `cargo build -p walrs_filter --no-default-features --features validation` succeeds without `Value`-related code compiling.
- [ ] `cargo build --workspace --all-features` still succeeds.
- [ ] `cargo test --workspace --all-features` still passes.
- [ ] `cargo clippy --workspace --all-features -- -D warnings` passes.
- [ ] `cargo fmt --all --check` passes.
- [ ] Default-feature builds of every crate produce **byte-identical public API** to pre-change (verify via `cargo public-api` or manual inspection).
- [ ] docs.rs rendering shows the `value` feature badge on gated items (verified locally with `RUSTDOCFLAGS="--cfg docsrs" cargo doc`).
- [ ] Coverage stays above 80% per `llvm-cov-all.sh` on the default-feature build.

### 3.4 Risks & Mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| Stray `Value` reference not gated, breaking `--no-default-features` | Medium | `cargo hack check --each-feature` catches this; run in CI before merge |
| Feature-gate cfg attribute explosion hurts readability | Low | Group gated items into submodules; gate the `mod` declaration once |
| Docs.rs rendering loses the `Value` API | Low | `all-features = true` already configured; verify locally |
| Consumer disables `value` on `walrs_validation` but depends on `walrs_fieldfilter`, causing a resolver conflict | Low | `walrs_fieldfilter` explicitly requests `features = ["value"]`; cargo unifies features across the graph |
| Coverage drops because `rule_impls/value.rs` tests only run with `value` feature | Medium | CI must run `cargo test --all-features` for coverage; document the dependency |
| Semver question — does adding a feature count as breaking? | Very Low | No — adding a *new* feature is non-breaking; changing *defaults* would be |

---

## 4. Phase 2 — Evaluate Extraction

### 4.1 Triggers to start Phase 2

Phase 2 is **not scheduled** — it's gated on one or more of the
following conditions becoming true:

1. `Fieldset` derive (via `walrs_fieldset_derive`) reaches feature
   parity with `FieldFilter<Value>` — including cross-field rules,
   filter pipelines, and nested fieldsets.
2. A concrete CMS/dyn-forms product build-out emerges that would
   benefit from independent versioning of the dynamic path.
3. `walrs_fieldfilter`'s typed API (`Fieldset`-based) is the
   advertised primary entry point in docs/README, with
   `FieldFilter<Value>` treated as the legacy/dynamic alternative.
4. Maintenance burden on `rule_impls/value.rs` begins to diverge
   noticeably from `rule_impls/scalar.rs` / `rule_impls/string.rs` —
   e.g., dynamic-path-only features appearing.

### 4.2 Option B1 — Narrow Extraction

Move **only** the dynamic-typed bits into a new `walrs_dyn` (or
`walrs_value`) crate:

- `Value` enum + `ValueExt` + bridge impls
- `Rule<Value>` / `Condition<Value>` dispatch
- `FilterOp<Value>` / `TryFilterOp<Value>` impls

`walrs_fieldfilter` and `walrs_form` keep using these types but import
them from the new crate. The `value` feature in `walrs_validation` is
removed; consumers opt in by adding the new crate.

**Effort:** ~3 days. Mostly re-exports and path updates. Low risk.

**When it's worth it:** if `Value` maintenance diverges from the typed
path, or if publishing the dynamic path on a separate cadence becomes
valuable.

### 4.3 Option B2 — Wide Extraction

Move the entire dynamic multi-field runtime to `walrs_dyn`:

- Everything from B1, plus
- `FieldFilter` (currently in `walrs_fieldfilter`)
- `FormData` (currently in `walrs_form`)

`walrs_fieldfilter` becomes the **typed** multi-field crate (built on
`Fieldset`). `walrs_dyn` is the **dynamic** multi-field runtime. Two
distinct products.

**Effort:** ~1–2 weeks. Touches examples, benches, docs, derive macro
output paths.

**When it's worth it:** when a product-shaped separation between
"typed forms" and "dynamic/CMS forms" is valuable for discoverability
and independent releases, and when `Fieldset` has clearly superseded
`FieldFilter` for typed consumers.

### 4.4 Decision Criteria

| Question | If Yes → | If No → |
|---|---|---|
| Has `Fieldset` reached full parity with `FieldFilter`? | B1/B2 feasible | Stay on Phase 1 |
| Is there external pressure (CMS product, user request) to isolate dynamic path? | B2 justified | B1 at most |
| Is maintenance diverging between typed and dynamic paths? | B1 justified | Stay on Phase 1 |
| Are we ready to publish an additional crate to crates.io? | B1/B2 feasible | Stay on Phase 1 |

Revisit every 6 months or on significant product direction change.

---

## 5. Out of Scope

- **Removing `Value`.** Explicitly rejected — see
  [`value_type_scope_and_cms_tradeoffs.md`](../discussions/value_type_scope_and_cms_tradeoffs.md).
- **Flipping the default.** `value` remains default-on to preserve
  non-breaking semantics.
- **Changing `walrs_fieldfilter` or `walrs_form` to be `Value`-optional
  in Phase 1.** Those require the typed `Fieldset` path to mature
  (Phase 2 scope).
- **Replacing `Value` with `serde_json::Value`.** Loses numeric type
  discrimination (see prior discussion).

---

## 6. Open Questions

1. **Should the `value` feature be default-on or default-off in 1.0?**
   - Default-on in Phase 1 (zero breakage).
   - Revisit at the 1.0 release — if `Fieldset` is the clearly
     advertised path by then, default-off becomes defensible.
2. **Should `serde_json_bridge` imply `value`, or be independent?**
   - Plan assumes `serde_json_bridge → value` (bridge doesn't make
     sense without `Value`). Confirm during implementation.
3. **Should `walrs_fieldset_derive` emit a feature-gate on the
   `Value::...` variants it generates?**
   - Probably not — consumers using the derive macro implicitly want
     the `Value` path. But revisit in Phase 2 if derive starts
     emitting both typed and dynamic code.
4. **Is `walrs_dyn` the right name for Phase 2, or `walrs_value`?**
   - `walrs_value` is narrower (just the enum + impls) — matches B1.
   - `walrs_dyn` is broader (dynamic runtime) — matches B2.
   - Decide when Phase 2 is triggered.
