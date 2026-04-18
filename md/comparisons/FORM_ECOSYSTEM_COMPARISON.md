# Comparison: walrs Form Ecosystem vs Popular Rust Crates

This document compares the walrs form-related crates
(`walrs_validation`, `walrs_filter`, `walrs_fieldfilter`, `walrs_form`,
`walrs_fieldset_derive`) against the most commonly used Rust crates that
address the same concerns (`validator`, `garde`, `serde_with`, `ammonia`,
framework-native form extractors, etc.).

---

## 1. Validation: `walrs_validation` vs `validator` / `garde`

| Dimension | `walrs_validation` | `validator` (keats) | `garde` |
|---|---|---|---|
| Primary API | Composable `Rule<T>` enum + `Validate` / `ValidateRef` traits | `#[derive(Validate)]` + attributes | `#[derive(Validate)]` + attributes |
| Composition | Fluent combinators: `.and()`, `.or()`, `.not()`, `.when()`, `.when_else()` | None — attributes only | Limited (`#[garde(dive)]`, custom fns) |
| Serializable rules | **Yes** — rules are serde-able, config-driven pipelines | No | No |
| Zero-copy / ref validation | **Yes** (`ValidateRef<T: ?Sized>`) | No (owned only) | Partial |
| Async validation | **Yes** (`ValidateAsync` feature) | Partial (custom async fn) | No |
| Typed violation kind | **Yes** — `ViolationType` enum (`TooShort`, `StepMismatch`, …) | String + code | String + path |
| Message system | Static / templated / dynamic provider w/ locale context | String + external i18n crate | Static + i18n |
| Date validators | Built-in (chrono **or** jiff via feature flags) | Via chrono feature | chrono |
| Ecosystem / maturity | Early (0.1.x) | De-facto standard, huge uptake | Modern successor, growing fast |

**Verdict:** `walrs_validation` is more *functional / combinator-flavored*
and serializable, closer in spirit to JSON-Schema validators than to
`validator`. `garde` is the modern idiomatic pick in the ecosystem;
`validator` is the pragmatic default. walrs's typed `ViolationType` and
serializable rules are genuinely distinctive — neither competitor offers
that.

---

## 2. Filtering / Sanitization: `walrs_filter` vs ecosystem

There is **no dominant Rust filter crate** equivalent to PHP's
HTMLPurifier-style filter pipelines. The usual pattern:

- **`ammonia`** — HTML sanitization (walrs uses it internally).
- **`voca_rs`** / **`deunicode`** / **`slug`** — string transforms.
- **`serde_with`** — field-level transforms during deserialization.
- **`validator`'s** `#[validate(custom)]` — manual transform in a fn.

`walrs_filter`'s `FilterOp<T>` / `TryFilterOp<T>` with `Cow`-based
zero-copy application and a serializable pipeline is **largely unique in
the ecosystem**. The closest analogue is `serde_with`, but that is
deserialization-time only and doesn't give a runtime-composable,
config-driven pipeline.

---

## 3. Forms: `walrs_form` vs framework extractors

Rust has no cross-framework "form library." Competition:

- **axum**'s `Form`, **actix-web**'s `web::Form`, **rocket**'s `Form` —
  all frame deserialization extractors, not form *definition* libraries.
- **`leptos`** / **`dioxus`** / **`yew`** — component-side form
  primitives, tied to the framework.
- **`loco`** / **`askama`** — templating for HTML forms.

`walrs_form` occupies an empty niche: a **serializable HTML form model**
usable server-side (Rust) and client-side (WASM/JS via `web-sys`
FormData). Nothing else in the ecosystem does this across both runtimes.

---

## 4. Derive: `walrs_fieldset_derive` vs `validator` / `garde` derive

~90 attributes is broader than `validator` (~25) and comparable to
`garde` — but `walrs_fieldset_derive` is the only one that unifies
**validation + filtering + cross-field rules + FormData ↔ struct
conversion** in a single derive. The others cover validation only.

---

## 5. Field-level integration: `walrs_fieldfilter`

No direct analogue in the Rust ecosystem. The closest you get is:

- Hand-rolled: `serde` deserialization → `validator` validation →
  manual sanitization.
- Framework-specific middleware chaining these steps.

`walrs_fieldfilter`'s `Field<T>` + `Fieldset` trait + built-in
`CrossFieldRule` library (`FieldsEqual`, `RequiredIf`,
`MutuallyExclusive`, `DependentRequired`, …) is a cohesive abstraction
that the ecosystem currently requires users to stitch together by hand.

---

## 6. Honest Gaps

1. **No framework integrations** yet — no `axum` / `actix` extractors,
   no `tower` layer. `validator` / `garde` have community-maintained
   ones.
2. **Maturity** — 0.1.x vs. 5+ year tenure of `validator`; no downstream
   crates depend on walrs.
3. **Discoverability** — competitors ship with
   `ValidationErrors: IntoResponse` impls, `utoipa` integration, etc.
4. **i18n** — message providers exist but no ready integration with
   `fluent` / `rust-i18n`.

---

## 7. TL;DR

walrs is architecturally **more ambitious** than `validator` and
overlaps with `garde` on derive, but adds two capabilities the ecosystem
genuinely lacks:

1. **Serializable validation rules** (config-driven pipelines).
2. **A composable filter pipeline** (first-class, not deserialization-bound).

Plus a **WASM-friendly serializable form model** (`walrs_form`) with no
direct competitor. The weak points are maturity and framework glue, not
design.

---

## 8. Quick Reference Matrix

| Concern | walrs crate | Ecosystem default |
|---|---|---|
| Struct validation via derive | `walrs_fieldset_derive` | `validator`, `garde` |
| Rule composition & serialization | `walrs_validation` | (none) |
| Input filtering / sanitization pipeline | `walrs_filter` | `ammonia` + `serde_with` (ad-hoc) |
| Cross-field validation | `walrs_fieldfilter` | manual fn in `validator` / `garde` |
| HTML form definition (server + WASM) | `walrs_form` | framework-specific only |
| Async validation | `walrs_validation` (feature) | `validator` (partial) |
| Zero-copy (`&str`) validation | `walrs_validation` (`ValidateRef`) | (none) |
