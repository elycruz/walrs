# Plan: Remove the Dynamic `Value` / `FieldFilter` / `FormData` Path

**Date:** 2026-04-25
**Status:** Proposed — awaiting first-pass review
**Tracking issue:** TBD (this plan opens it)
**Supersedes:** #258 (Phase 2 extraction tracker — we are deleting, not extracting)
**Crates affected:** `walrs_validation`, `walrs_filter`, `walrs_fieldfilter`, `walrs_form`, `walrs_fieldset_derive`

---

## 1. Decision

The walrs form ecosystem standardises on the **typed** path:

- `walrs_validation::Rule<T>` for per-field rules.
- `walrs_filter::FilterOp<T>` / `TryFilterOp<T>` for filters.
- `walrs_fieldfilter::Fieldset` (trait) + `walrs_fieldset_derive` (derive) for multi-field validation and filtering.

The **dynamic** path — `Value`, `Rule<Value>` dispatch, `FieldFilter<Value>`,
`FormData`, and the `#[fieldset(into_form_data, try_from_form_data)]` bridge
— is removed.

Rationale:

- Rust gives us compile-time field/type checks. Letting users define structs
  and deriving everything off them is a strictly better DX than a runtime
  schema map.
- The dynamic path costs ~4,300 LOC across five files plus a long tail of
  element types, tests, fuzz targets, examples, and docs. That maintenance
  burden has no offsetting product use case currently in flight.
- Phase 1 (#240, merged in PR #256) already feature-gated `Value` so typed
  consumers can opt out. The next logical step is to delete it rather than
  extract it (the option #258 was tracking).
- #259 + child PRs #263 / #264 / #265 just closed the typed-path parity
  gaps. As of those merges, every cross-field pattern, async validation,
  and (formerly) the typed↔dynamic bridge are expressible from a struct.
  Trigger 1 of #258 fired — and in firing it, made #258 itself moot.

## 2. What stays

| Crate | Item | Notes |
|---|---|---|
| `walrs_validation` | `Rule<T>`, `Condition<T>`, `rule_impls::{scalar, string, length, steppable}` | unchanged |
| `walrs_validation` | `Violation`, `FieldsetViolations`, `message`, `traits` | unchanged |
| `walrs_filter` | `FilterOp<T>`, `TryFilterOp<T>` (typed blanket impls only) | drop the `Value` impls |
| `walrs_fieldfilter` | `Fieldset` trait, `FieldsetAsync` trait, `Field<T>` | unchanged |
| `walrs_fieldset_derive` | `#[derive(Fieldset)]` and all `#[validate(...)]`, `#[filter(...)]`, `#[cross_validate(...)]`, `#[fieldset(async, break_on_failure)]` attributes | unchanged |

## 3. What goes

| File | LOC | Reason |
|---|---:|---|
| `crates/validation/src/value.rs` | 602 | `Value` enum |
| `crates/validation/src/rule_impls/value.rs` | 1,439 | `Rule<Value>` dispatch |
| `crates/fieldfilter/src/field_filter.rs` | 1,344 | `FieldFilter<Value>` |
| `crates/form/src/form_data.rs` | 199 | `FormData` |
| `crates/fieldset_derive/src/gen_form_data.rs` | 751 | `into_form_data` / `try_from_form_data` codegen |
| | **~4,335** | core deletion |

Plus the `Value`-impl blocks in `walrs_filter::filter_op`, the
`#[fieldset(into_form_data, try_from_form_data)]` attribute parsing in
`walrs_fieldset_derive`, all `Value` / `FormData` examples and tests, the
`value` feature flag in `walrs_validation`, and the fuzz target
`crates/fieldfilter/fuzz/fuzz_targets/fuzz_fieldfilter_validate.rs`.

The recently-merged interop docs (PR #263 / #262) are also reverted —
the bridge they document goes away.

## 4. Open question — `walrs_form`

`walrs_form` exists today as a thin shim around `FormData` plus typed
HTML element representations (`input_element`, `select_element`,
`button_element`, `textarea_element`, `fieldset_element`, `form`, …).

Once `FormData` is gone, two options:

**Option A — Delete `walrs_form` entirely.** Cleanest. Picks up if no
consumer needs typed HTML element types as a separate library.

**Option B — Reduce `walrs_form` to typed HTML rendering aids built atop
`Fieldset`.** Element types stay, but `FormData` and `path` go.
Rendering glue (e.g. SSR, htmx) is the only remaining justification.

Default to **Option A** unless a concrete consumer surfaces during
Phase 1. Decision deadline: end of Phase 2.

## 5. Phased execution

### Phase 0 — Deprecation (1 release cycle)

- Add `#[deprecated(since = "x.y.z", note = "removed in next major; use #[derive(Fieldset)] instead")]` on:
  - `walrs_validation::Value` and all variants.
  - `walrs_validation::rule_impls::value::*` public items.
  - `walrs_filter::FilterOp::*` `Value`-targeted impls.
  - `walrs_fieldfilter::FieldFilter` (the type).
  - `walrs_form::FormData` (and any re-exports).
  - `walrs_fieldset_derive` `into_form_data` / `try_from_form_data` attribute handling — emit a deprecation `note` from the macro when those keys are seen.
- Update `README.md` and each crate's `lib.rs` `//!` to mark the dynamic path as deprecated and point at the typed replacement.
- Cut a minor release containing only the deprecation annotations.

### Phase 1 — Remove the bridge

- Delete `crates/fieldset_derive/src/gen_form_data.rs`.
- Drop `into_form_data` / `try_from_form_data` from `FieldsetStructAttrs` parsing.
- Revert PR #263 doc additions (crate-level //! interop section, root README "Choosing a path" paragraph, `crates/form/examples/derive_formdata_bridge.rs`, `crates/fieldfilter/DESIGN.md` "Typed vs Dynamic" section).
- Remove the `walrs_form` dev-dep added to `crates/form/Cargo.toml` for the bridge example if nothing else uses it.

### Phase 2 — Remove `FieldFilter` and `FormData`

- Delete `crates/fieldfilter/src/field_filter.rs`. Drop its `pub use` in `lib.rs`.
- Delete `crates/form/src/form_data.rs`. Drop its `pub use` in `lib.rs`.
- Delete tests/examples that reference either: `crates/form/tests/derive_fieldset_formdata.rs`, `crates/form/examples/form_data_paths.rs`, the `field_filter`-named examples in `crates/fieldfilter/examples/`, the FormData-flavoured paths in `localized_form.rs` / `registration_form.rs` / `login_form.rs`.
- Delete `crates/fieldfilter/fuzz/fuzz_targets/fuzz_fieldfilter_validate.rs`.
- Decide `walrs_form` Option A vs B (§4); execute.

### Phase 3 — Remove `Value` and the `value` feature

- Delete `crates/validation/src/value.rs` and `crates/validation/src/rule_impls/value.rs`.
- Drop `value` from `walrs_validation`'s feature graph and from any downstream feature passthroughs in `walrs_filter`, `walrs_fieldfilter`, `walrs_form` (whatever remains).
- Drop `Value`-impl blocks in `crates/filter/src/filter_op.rs`.
- Update the workspace `Cargo.toml` if the `walrs_form` crate disappears entirely.

### Phase 4 — Major-version bump and re-publish

- Single coordinated `0.x → 0.(x+1)` (or `0.x → 1.0`) bump across all crates.
- Update root `README.md` sub-crates table, feature flags list, and umbrella `walrs` re-exports per the project's "after changing code" rule.
- CHANGELOG entries per crate calling out the removal explicitly.

## 6. Migration story (for our own examples and any external user)

Any `FieldFilter<Value>` usage migrates to `#[derive(Fieldset)]` on a
struct describing the same fields. The structured cross-field
variants added in #260 (PR #265) cover the patterns the dynamic path
was using; the async path added in #261 (PR #264) covers
`Rule::CustomAsync`. There is no like-for-like replacement for
runtime-schema use cases — that is the explicit non-goal of this plan.

## 7. Success criteria

- `cargo build --workspace` and `cargo test --workspace` pass on a
  workspace with zero references to `Value`, `FieldFilter`, or
  `FormData`.
- `walrs_validation` exposes no `value` feature.
- All examples and integration tests use `#[derive(Fieldset)]`.
- README + crate `//!` docs describe a single path.
- #258 is closed (superseded by this work).

## 8. Risks

- **External users.** If anyone outside this repo is on `FieldFilter` or
  `FormData`, their migration is non-trivial. Mitigation: the Phase 0
  deprecation cycle gives them a release to react and the issue tracker
  to push back.
- **Hidden coupling.** Some validation `Rule<T>` impls may indirectly
  depend on the `value` feature compiling (e.g. through `attributes.rs`).
  Mitigation: build with `--no-default-features` early in Phase 0 to
  surface these.
- **`walrs_form` element types.** If they prove valuable (Option B),
  retaining them adds scope to Phase 2.
- **Fuzz coverage gap.** The deleted fuzz target tests `FieldFilter`
  against `Value`; once removed, we lose that signal. Add a typed-path
  fuzz target if we're keeping fuzzing for this crate at all.

## 9. Out of scope

- Building a new dynamic-forms crate elsewhere. If a CMS-shaped need
  resurfaces later, that is a fresh project.
- Async filter support (`try_custom_async`). Already deferred in #261.
- Touching `walrs_acl`, `walrs_digraph`, or anything outside the form
  ecosystem.

## 10. References

- #258 — Phase 2 extraction tracker (superseded by this plan)
- #240 / PR #256 — Phase 1 `value` feature gating
- #259 — typed-path parity meta
  - PR #263 (#262) — interop docs (will be reverted in Phase 1)
  - PR #264 (#261) — `FieldsetAsync` derive emission
  - PR #265 (#260) — structured cross-field variants
- [`md/plans/value_feature_gating.md`](value_feature_gating.md) — Phase 1 / 2 plan that this supersedes from §4 onward
