# Forms Ecosystem Review — Issue #208

**Date:** 2026-04-16
**Scope:** walrs_validation, walrs_filter, walrs_fieldfilter, walrs_form, walrs_fieldset_derive
**Files Reviewed:** ~80 across 5 crates
**All tests pass across all crates.**

---

## Aggregate Severity Summary

| Severity | validation | filter | fieldfilter | form | fieldset_derive | **Total** |
|---|---|---|---|---|---|---|
| 🔴 Critical | 0 | 0 | 1 | 0 | 0 | **1** |
| 🟠 High | 2 | 1 | 1 | 2 | 2 | **8** |
| 🟡 Medium | 3 | 3 | 2 | 5 | 4 | **17** |
| 🔵 Low | 3 | 0 | 3 | 5 | 3 | **14** |

**Total findings: 40** (1 critical, 8 high, 17 medium, 14 low)

---

## 🔴 Critical Findings (1)

### C1. `fieldfilter/src/rule.rs` — `RequiredIf`/`RequiredUnless` tautological logic
**Crate:** walrs_fieldfilter
`RequiredIf`/`RequiredUnless` check the condition and value on the **same field**, making them tautological — they can never trigger correctly for cross-field dependencies.

---

## 🟠 High Findings (8)

### H1. `validation/src/violation.rs:72` — Grammar bug in `too_long()` message
Says `"must at most"` instead of `"must be at most"`. User-facing.

### H2. `validation/README.md:177-182` — `with_message_provider` example won't compile
Closure signature doesn't match actual API (`&MessageContext<T>` not `(&T, _locale)`).

### H3. `filter/README.md` — License mismatch
Says MIT/Apache-2.0, should be Elastic-2.0.

### H4. `fieldfilter/src/rule.rs` — `CustomAsync` rules silently pass in sync `evaluate()`
Async-only rules return `Ok(())` when evaluated synchronously — silent validation bypass.

### H5. `form/src/form.rs` — `bind_data`/`validate` don't recurse into `FieldsetElement`
Nested fieldset children are silently ignored during data binding and validation.

### H6. `form` — Uses deprecated `FormViolations` type
Should migrate to `FieldsetViolations`.

### H7. `fieldset_derive/src/parse.rs` — `expect()` panics in proc-macro parsing
Missing required sub-attributes (e.g., `range(min=1)` without max) cause `proc macro panicked` instead of helpful compile errors.

### H8. `fieldset_derive/src/gen_validate.rs:324` — Invalid regex panics at runtime
`#[validate(pattern = "[invalid")]` compiles but `.unwrap()` panics at runtime.

---

## 🟡 Medium Findings (17)

| # | Crate | Finding |
|---|---|---|
| M1 | validation | README `Message::from_fn` doesn't exist → `Message::provider` |
| M2 | validation | Double-space before "Received" in violation messages |
| M3 | validation | `FieldsetViolations` missing `PartialEq` derive |
| M4 | filter | `get_slug_filter_regex`/`get_dash_filter_regex` should be `pub(crate)` |
| M5 | filter | Missing `Clone`/`Debug` on `XmlEntitiesFilter`, `Eq` on `FilterError` |
| M6 | filter | `Chain::apply_ref` doesn't preserve `Cow::Borrowed` for no-op chains |
| M7 | fieldfilter | `apply_locale()` drops locale when rule is None |
| M8 | fieldfilter | 2 doc warnings (missing docs) |
| M9 | form | `set_nested` OOM via large index in untrusted paths |
| M10 | form | `parse_path` accepts unclosed brackets |
| M11 | form | Unused `regex` dependency |
| M12 | form | Missing `PartialEq` on `FormData`/`Form`/`Element` |
| M13 | form | Missing `PartialEq` on `FormData`/`Form`/`Element` |
| M14 | fieldset_derive | Char extraction uses byte length, not char count (multi-byte UTF-8 fails) |
| M15 | fieldset_derive | `Other`/nested types silently `Default::default()` in `TryFrom` |
| M16 | fieldset_derive | `Other`/nested types silently skipped in `From` impl |
| M17 | fieldset_derive | Unknown `#[fieldset(...)]` attributes silently ignored (typos undetected) |

---

## 🔵 Low Findings (14)

| # | Crate | Finding |
|---|---|---|
| L1 | validation | `Value` missing `Default` impl |
| L2 | validation | `ViolationType` missing `Display` impl |
| L3 | validation | `too_short`/`too_long` inconsistent wording (intentional) |
| L4 | fieldfilter | Redundant `fields` vec |
| L5 | fieldfilter | `filter_ref`/`filter` asymmetry |
| L6 | fieldfilter | Async clone overhead |
| L7 | form | README license mismatch |
| L8 | form | Broken validation example in README |
| L9 | form | Duplicate architecture list in README |
| L10 | form | Minor `json!()` inconsistency |
| L11 | form | Missing test for path edge cases |
| L12 | fieldset_derive | Parse errors silently swallowed with `let _ =` |
| L13 | fieldset_derive | Unused variable `_field_name_str` in gen_filter.rs |
| L14 | fieldset_derive | README naming confusion (`DeriveFieldset` vs `Fieldset`) |

---

## Cross-Cutting Themes

1. **Silent data loss** — Multiple places where errors/data are silently dropped (fieldset_derive nested types, form fieldset recursion, fieldfilter async rules in sync mode).
2. **README drift** — All 5 READMEs have at least minor issues; none are doc-tested.
3. **Missing trait impls** — `PartialEq` is missing on several key aggregation types, blocking ergonomic test assertions.
4. **License headers** — At least 2 crates' READMEs state wrong license.

## Recommended Priority Order

1. Fix **C1** (RequiredIf/RequiredUnless logic) — actual logic bug
2. Fix **H4** (CustomAsync silent pass) — validation bypass
3. Fix **H5** (form fieldset recursion) — silent data loss
4. Fix **H7, H8** (proc-macro panics) — terrible DX
5. Fix **H1** (grammar) + **H2, H3** (README) — user-facing
6. Address **M9** (OOM via path index) — potential DoS
7. Batch remaining medium/low findings
