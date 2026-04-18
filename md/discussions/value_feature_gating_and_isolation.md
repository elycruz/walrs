# Analysis: Feature-Gating and Isolating the `Value` Type

**Date:** 2026-04-18
**Related:** [`value_enum_impedance_mismatch.md`](./value_enum_impedance_mismatch.md),
[`value_type_scope_and_cms_tradeoffs.md`](./value_type_scope_and_cms_tradeoffs.md)

---

## Executive Summary

`Value` is *not* feature-gated today. The 602-line enum and its 1,439-line
`Rule<Value>` impl ship unconditionally with `walrs_validation`, and two
downstream crates (`walrs_fieldfilter`, `walrs_form`) bake `Value` into
their **public API shape** — `Field<Value>`, `FieldFilter`, `FormData`.

A cheap, additive step is possible right now: add a default-on `value`
feature to `walrs_validation` that gates the `Value` enum, its `Rule<Value>`
impls, and the related `attributes` module. This lets typed-only consumers
opt out at the leaf, costs no API breakage, and lays the groundwork for a
later extraction if wanted.

**Full isolation into a standalone crate (`walrs_dyn` or similar) is
architecturally cleaner but currently blocked** by `walrs_fieldfilter`'s
and `walrs_form`'s hardcoded `Value` dependency. Those crates need the
typed `Fieldset` path (from [#88](https://github.com/elycruz/walrs/issues/88))
to mature into a primary API before `Value` can be lifted out cleanly.

**Recommendation:** do the feature gate now (Phase 1). Revisit extraction
once `Fieldset` is the advertised typed path and `FieldFilter<Value>` can
be treated as the dynamic alternative rather than the only option (Phase 2).

---

## Table of Contents

1. [Current Gating State](#1-current-gating-state)
2. [What Depends on `Value` Today](#2-what-depends-on-value-today)
3. [Option A — Feature-Gate In Place](#3-option-a--feature-gate-in-place)
4. [Option B — Extract to a Standalone Crate](#4-option-b--extract-to-a-standalone-crate)
5. [Option C — Hybrid (Phased)](#5-option-c--hybrid-phased)
6. [Blockers and Preconditions](#6-blockers-and-preconditions)
7. [Recommendation](#7-recommendation)
8. [Appendix — LOC and Dependency Data](#8-appendix--loc-and-dependency-data)

---

## 1. Current Gating State

`walrs_validation/Cargo.toml`:

```toml
[features]
default = ["serde_json_bridge"]
async = []
serde_json_bridge = ["dep:serde_json"]
chrono = ["dep:chrono"]
jiff = ["dep:jiff"]
```

- `serde_json_bridge` gates **only** the `From<serde_json::Value>`
  conversions and the `attributes` module
  (`rule_impls/attributes.rs:1` uses `#![cfg(feature = "serde_json_bridge")]`).
- `async`, `chrono`, `jiff` gate unrelated surfaces.
- **No feature controls `Value` itself.** `pub mod value;` and
  `pub use value::*;` at `lib.rs:96` / `lib.rs:105` are unconditional.
- **No feature controls `Rule<Value>` impls.** `rule_impls/mod.rs` declares
  the `value` submodule unconditionally.

`walrs_filter/Cargo.toml`:

```toml
default = ["validation"]
validation = ["dep:walrs_validation"]
```

- `validation` is default-on and gates the **entire** `walrs_validation`
  dep, including `FilterOp<Value>` / `TryFilterOp<Value>` impls.
- A consumer who disables the default feature gets a `walrs_filter` with
  no `Value` at all — this is the only place in the workspace where
  `Value` is already opt-out.

`walrs_fieldfilter/Cargo.toml`:

```toml
[dependencies]
walrs_validation = { path = "../validation" }   # mandatory
```

- `walrs_validation` is mandatory — no `optional = true`, no feature flag.
- `Field<Value>`, `FieldFilter`, and every cross-field rule are built
  directly on `Value`. There is no typed alternative wired through the
  public API yet.

`walrs_form/Cargo.toml`:

- `walrs_validation` and `walrs_fieldfilter` both mandatory.
- `FormData = IndexMap<String, Value>` is the only storage shape.

`walrs_fieldset_derive`:

- Proc-macro crate. No direct dependency on `walrs_validation`, but the
  generated code references `walrs_validation::Value` by path
  (e.g., `gen_form_data.rs` emits `walrs_validation::Value::I64(...)`).
  Callers must have `walrs_validation` in scope for the generated code
  to compile.

---

## 2. What Depends on `Value` Today

| Crate / File | `Value` refs | Coupling |
|---|---:|---|
| `crates/validation/src/value.rs` | — | Definition (602 lines) |
| `crates/validation/src/rule_impls/value.rs` | — | `Rule<Value>` dispatch (1,439 lines) |
| `crates/validation/src/rule_impls/attributes.rs` | — | Attribute-input bridge (already `#![cfg(feature = "serde_json_bridge")]`) |
| `crates/filter/src/filter_op.rs` | 102 | `FilterOp<Value>` impl |
| `crates/filter/src/try_filter_op.rs` | 35 | `TryFilterOp<Value>` impl |
| `crates/fieldfilter/src/field.rs` | 63 | `Field<Value>` type |
| `crates/fieldfilter/src/field_filter.rs` | 118 | `FieldFilter` storage + cross-field rules |
| `crates/form/src/form_data.rs` | 38 | `FormData(IndexMap<String, Value>)` |
| `crates/form/src/form.rs` | 19 | `Form` → `FieldFilter` delegation |
| `crates/form/src/{input,select,textarea}_element.rs` | 35 | Value binding on elements |
| `crates/fieldset_derive/src/gen_form_data.rs` | — | Emits `Value::*` variants |

**Dependency axis:** `validation → filter → fieldfilter → form`. Every
crate downstream of `validation` uses `Value`; the two leaf crates
(`fieldfilter`, `form`) expose it in their **public types**.

**Orphan-rule check:** `Value`, `Rule`, `FilterOp`, `Field`, `FieldFilter`
all live in the workspace. Any future move of `Value` to a new crate
only needs to keep impls co-located with whichever side owns a local
type. No orphan blockers exist.

---

## 3. Option A — Feature-Gate In Place

Add a `value` feature to `walrs_validation`. Guard the `value` module,
its `Rule<Value>` / `Condition<Value>` impls, and the `attributes`
module (which already requires `serde_json_bridge`). Propagate the
feature through `walrs_filter`, `walrs_fieldfilter`, `walrs_form`.

### Sketch

`walrs_validation/Cargo.toml`:

```toml
[features]
default = ["value", "serde_json_bridge"]
value = []
serde_json_bridge = ["dep:serde_json", "value"]   # bridge implies Value exists
```

`walrs_validation/src/lib.rs`:

```rust
#[cfg(feature = "value")]
pub mod value;
#[cfg(feature = "value")]
pub use value::*;
```

`walrs_validation/src/rule_impls/mod.rs`:

```rust
#[cfg(feature = "value")]
pub mod value;
#[cfg(all(feature = "value", feature = "serde_json_bridge"))]
pub mod attributes;
```

`walrs_filter/Cargo.toml`:

```toml
[features]
default = ["validation", "value"]
value = ["walrs_validation/value"]
validation = ["dep:walrs_validation"]
```

### Pros

- **Zero breaking change.** Default feature set leaves the public API
  identical.
- **Fast to ship.** Additive feature; no API moves.
- **Meaningful compile-time savings for typed-only consumers** — two
  files (~2,040 lines) drop out of `walrs_validation` alone, plus
  `FilterOp<Value>` / `TryFilterOp<Value>` (~140 refs) from
  `walrs_filter`.
- **Establishes a clear boundary** for a future extraction.

### Cons

- `walrs_fieldfilter` and `walrs_form` **cannot be built without the
  `value` feature** as their APIs stand today. You'd need `value` to
  remain a required dependency there — meaning the opt-out only helps
  consumers who depend on `walrs_validation` (and maybe `walrs_filter`)
  directly, not consumers of `FieldFilter` / `FormData`.
- Two files are still *shipped* to crates.io — only the compiler skips
  them. Binary-size impact exists only at final link.
- Documentation cost: every `Rule<Value>` reference in docs needs a
  `#[cfg_attr(docsrs, doc(cfg(feature = "value")))]` equivalent and a
  `all-features = true` `docs.rs` build (already configured).

### Migration cost: ~1 day of mechanical work

- Add `value` feature to three `Cargo.toml` files.
- Wrap `pub mod value;` and `rule_impls::value` with `cfg` attributes.
- Update `walrs_filter`'s `FilterOp<Value>` / `TryFilterOp<Value>` impls
  with `#[cfg(feature = "value")]`.
- Add `cfg_attr(docsrs, doc(cfg(feature = "value")))` for docs.rs
  rendering.
- Sanity-check with `cargo hack --each-feature check` to confirm the
  `--no-default-features` build compiles.

---

## 4. Option B — Extract to a Standalone Crate

Move `Value`, `Rule<Value>` impls, `FilterOp<Value>` / `TryFilterOp<Value>`
impls, and (optionally) `FieldFilter` + `FormData` into a new crate —
say `walrs_dyn` (or `walrs_value`, if kept narrow to the enum).

### Two layouts

**B1 — Narrow: enum + trait impls only**

```
walrs_dyn/
├── value.rs           (from walrs_validation/src/value.rs)
├── rule_impls_value.rs (from walrs_validation/src/rule_impls/value.rs)
├── filter_impls.rs    (FilterOp<Value>, TryFilterOp<Value>)
└── lib.rs
```

`walrs_fieldfilter` and `walrs_form` keep using `Value` but import it
from `walrs_dyn` — their API stays Value-centric. This is really
"Option A with an extra crate boundary" and doesn't buy much unless
you anticipate publishing the dynamic path on a separate cadence.

**B2 — Wide: dynamic runtime as a product**

```
walrs_dyn/
├── value.rs
├── rule_impls_value.rs
├── filter_impls.rs
├── field_filter.rs    (currently walrs_fieldfilter/src/field_filter.rs)
├── form_data.rs       (currently walrs_form/src/form_data.rs)
└── lib.rs
```

`walrs_fieldfilter` becomes the **typed** multi-field crate built on
`Fieldset`. `walrs_dyn` is the **dynamic** multi-field runtime.
Consumers pick the path that matches their use case.

### Pros

- **Product-shaped story** — "typed forms" and "dynamic/CMS forms" are
  separately discoverable on crates.io.
- **Independent versioning** — the dynamic path can stabilize on a
  different cadence than the typed core.
- **Smaller default install** for typed-only consumers (no
  `Value` code on the disk, not just skipped by the compiler).
- **Cleaner doc landing page** per crate.

### Cons

- **Significant refactor.** `FieldFilter` and `FormData` currently live
  deep in `fieldfilter` / `form`. Lifting them out means moving types,
  re-exporting, and touching every call site.
- **`walrs_fieldset_derive` output paths need updating** — the derive
  currently emits `walrs_validation::Value::...` tokens. It would need
  to emit `walrs_dyn::Value::...` or accept a configurable path.
- **Blocked by the `Fieldset` typed path** ([#88](https://github.com/elycruz/walrs/issues/88))
  being production-ready. Until `Fieldset` can *replace* `FieldFilter`
  for typed consumers, extracting the dynamic path leaves typed consumers
  with no multi-field story in `walrs_fieldfilter`.
- **Crate count grows.** Seven workspace crates become eight, most of
  which publish together.

### Migration cost: ~1–2 weeks of focused work

Most of that is test migration, call-site updates in examples/benches,
and documentation. The mechanical moves are straightforward thanks to
the clean orphan-rule picture.

---

## 5. Option C — Hybrid (Phased)

Do Option A now. Revisit Option B once `Fieldset` is mature.

### Phase 1 (now, ~1 day)

- Add `value` feature to `walrs_validation` gating the enum and
  `Rule<Value>` impls.
- Add `value` feature to `walrs_filter` gating `FilterOp<Value>` /
  `TryFilterOp<Value>`.
- Keep `walrs_fieldfilter` and `walrs_form` requiring `value` (they
  have no alternative yet).
- Document both the feature and the fact that `fieldfilter`/`form`
  currently require it.

### Phase 2 (after `Fieldset` stabilizes, size TBD)

- Make `walrs_fieldfilter`'s public API typed-first via `Fieldset`.
  `FieldFilter<Value>` becomes an *optional* surface behind the
  `value` feature.
- At that point, evaluate whether `FieldFilter<Value>` + `FormData`
  belong in their own crate (`walrs_dyn`) or stay behind the feature.

### Trigger to move from Phase 1 to Phase 2

- `Fieldset` derive handles the full rule/filter/cross-field matrix
  (feature-parity with `FieldFilter`).
- Examples and benches use `Fieldset` as the default path.
- Either a concrete CMS/dyn use case appears that would benefit from
  independent versioning, or `Value`-path maintenance begins to diverge
  from the typed path in scope.

---

## 6. Blockers and Preconditions

| Blocker | Affects | Resolution |
|---|---|---|
| `walrs_fieldfilter` public API is `Value`-only | Options A, B2 | `Fieldset` trait must become the primary typed API (tracked in [#88](https://github.com/elycruz/walrs/issues/88)) before `FieldFilter<Value>` can be made optional |
| `walrs_form::FormData = IndexMap<String, Value>` | Options A, B2 | Either keep `FormData` Value-coupled and require the feature, or introduce a `FormData<T>` generic / typed `impl Form` path |
| Derive macro emits fully-qualified `walrs_validation::Value::...` | Option B | Either make the target crate path configurable via a macro attribute, or re-export `Value` from `walrs_validation` under the new location |
| `attributes.rs` already gated behind `serde_json_bridge` | None (already done) | Reuse the same pattern for the new `value` feature |

No orphan-rule or coherence blockers exist — every Value-related impl
involves a type already in the workspace.

---

## 7. Recommendation

**Proceed with Option C (hybrid) starting with Phase 1 now.**

Phase 1 rationale:

1. It's a **one-day mechanical change** with no API breakage.
2. It gives typed-only consumers (the `Fieldset`/derive audience, once
   that lands fully) a way to skip ~2k LOC of dynamic dispatch.
3. It makes the `Value` dependency **explicit in the feature graph**,
   which is valuable documentation even if no one disables it yet.
4. It aligns the `Value` feature with the existing `serde_json_bridge`
   feature — they're naturally paired (the bridge implies `value`).
5. It lays the cleanest possible groundwork for a future extraction.

Phase 2 (extraction) should wait on two conditions:

- `walrs_fieldfilter` can offer a typed primary API that doesn't require
  `Value`.
- A demonstrable reason to version the dynamic path independently (CMS
  product build-out, external pressure from a user, or maintenance
  divergence).

Until then, extraction is premature — the extra crate boundary costs
more than the gating feature alone does.

### What not to do

- **Don't remove `Value`.** The CMS/WASM/config-driven niche (documented
  in [`value_type_scope_and_cms_tradeoffs.md`](./value_type_scope_and_cms_tradeoffs.md))
  genuinely requires it, and `serde_json::Value` is a worse substitute
  because it blurs `i64`/`u64`/`f64`.
- **Don't default-off the `value` feature.** `walrs_fieldfilter` and
  `walrs_form` both need it today; flipping the default causes
  breakage without buying anything.
- **Don't extract `Value` narrowly (Option B1) without the typed
  fieldfilter path.** It just adds a crate boundary without separating
  responsibilities.

---

## 8. Appendix — LOC and Dependency Data

### LOC footprint of `Value`-coupled code

| File | Lines | Role |
|---|---:|---|
| `crates/validation/src/value.rs` | 602 | Enum definition, `From` impls, `Display`, `PartialOrd`, `ValueExt`, bridge (partially gated) |
| `crates/validation/src/rule_impls/value.rs` | 1,439 | `Rule<Value>` dispatch, `validate_value` (526 impl + ~900 tests) |
| `crates/validation/src/rule_impls/attributes.rs` | — | Already gated via `serde_json_bridge` |
| `crates/filter/src/filter_op.rs` | 1,202 (102 `Value` refs) | `FilterOp<T>` generic + `FilterOp<Value>` impl |
| `crates/filter/src/try_filter_op.rs` | 597 (35 `Value` refs) | `TryFilterOp<Value>` impl |
| **Total Value-specific surface** | **~2,040** | In `walrs_validation` alone |

### Feature graph (after Phase 1)

```
walrs_validation
├── default = [value, serde_json_bridge]
├── value (new)
├── serde_json_bridge → value
├── async
├── chrono
└── jiff

walrs_filter
├── default = [validation, value]
├── value → walrs_validation/value
└── validation

walrs_fieldfilter          ← requires value (Phase 1: no opt-out)
walrs_form                 ← requires value (Phase 1: no opt-out)
walrs_fieldset_derive      ← unchanged (generated code references Value)
```

### Consumer matrix

| Consumer profile | Feature set | `Value` code compiled? |
|---|---|---|
| Typed validation only (`Rule<T>`) | `walrs_validation` with `--no-default-features --features serde_json_bridge` (or just `chrono`, etc.) | **No** |
| Typed filter pipeline | `walrs_filter` with `--no-default-features --features validation` | **No** |
| Fieldset derive (typed multi-field) | `walrs_fieldfilter` + `walrs_fieldset_derive` | **Yes** (unchanged until Phase 2) |
| CMS / dynamic forms | Defaults | **Yes** |
| WASM form model | Defaults + `wasm` feature on `walrs_form` | **Yes** |

### Estimated effort

| Phase | Effort | Risk |
|---|---|---|
| Phase 1 (gate in place) | ~1 day | Low — additive, mechanical, tests cover the `--no-default-features` path via `cargo hack` |
| Phase 2 — narrow extraction (B1) | ~3 days | Low — follows proven move pattern; mostly re-exports |
| Phase 2 — wide extraction (B2) | ~1–2 weeks | Medium — requires `Fieldset` parity, affects many call sites, docs, examples, benches |
