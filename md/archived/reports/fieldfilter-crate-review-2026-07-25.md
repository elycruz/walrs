# walrs_fieldfilter — Review Findings

**Issue:** #208 — Review forms ecosystem for correctness, API completeness, and soundness
**Review Unit:** walrs_fieldfilter — Field-Level Filtering and Validation
**Files Reviewed:** 18
**Test Results:** 109 passed (71 unit + 32 integration + 6 doctests), 0 failed
**Clippy Warnings:** 0

## Severity Summary

| Severity | Count |
|---|---|
| 🔴 Critical | 1 |
| 🟠 High | 1 |
| 🟡 Medium | 3 |
| 🔵 Low | 3 |
| ✅ Clean | 13 files |

## Findings

### 1. field_filter.rs:349-405 — `RequiredIf`/`RequiredUnless` evaluate condition and requirement on the same field

**Severity:** 🔴 Critical

Both `RequiredIf` and `RequiredUnless` evaluate the `condition` against `field`, then check whether `field` has a value. The doc comment says "condition on **another** field" but the struct only has a single `field`. This makes both rules logically broken:

```rust
// RequiredIf evaluates condition on `field`, then checks if `field` has a value:
CrossFieldRuleType::RequiredIf { field, condition } => {
    let condition_met = data
        .get(field)                                    // ← checks condition on `field`
        .map(|v| evaluate_condition(condition, v))
        .unwrap_or(false);
    if condition_met {
        let has_value = data
            .get(field)                                // ← also checks value on `field`
            .map(|v| !v.is_empty_value())
            .unwrap_or(false);
        // ...
    }
}
```

Example of the tautology: `RequiredIf { field: "email", condition: IsNotEmpty }` means "if email is not empty, then email is required to have a value" — which is always true when the field is present and always vacuously passes when absent.

**Impact:** `RequiredIf` and `RequiredUnless` cannot express their intended semantics. Any user relying on these for conditional field requirements gets silently incorrect validation.

**Suggested fix:** Add a `condition_field` to the struct:

```rust
RequiredIf {
    field: String,           // the field that becomes required
    condition_field: String,  // the field whose value is checked
    condition: Condition<Value>,
},
RequiredUnless {
    field: String,
    condition_field: String,
    condition: Condition<Value>,
},
```

Then evaluate:
```rust
let condition_met = data
    .get(condition_field)
    .map(|v| evaluate_condition(condition, v))
    .unwrap_or(false);
if condition_met {
    let has_value = data
        .get(field)
        .map(|v| !v.is_empty_value())
        .unwrap_or(false);
    // ...
}
```

---

### 2. field_filter.rs:478-479 — `CustomAsync` silently passes in sync `evaluate()`

**Severity:** 🟠 High

When a `CrossFieldRuleType::CustomAsync` variant is evaluated via the sync `evaluate()` method, it unconditionally returns `Ok(())`:

```rust
CrossFieldRuleType::Custom(f) => f(data),

#[cfg(feature = "async")]
CrossFieldRuleType::CustomAsync(_) => Ok(()),  // ← silently passes
```

If a user adds a `CustomAsync` rule but calls `FieldFilter::validate()` (sync) instead of `validate_async()`, all async cross-field rules are silently skipped — no error, no warning, no panic.

**Impact:** Async cross-field validation rules are invisible to sync validation. This is a silent correctness hole — the user thinks their custom validation ran but it didn't.

**Suggested fix:** Either:
- Return an error: `Err(Violation::new(ViolationType::CustomError, "CustomAsync rules require validate_async()"))`
- Or `panic!("CustomAsync rules cannot be evaluated synchronously; use validate_async()")`
- Or add a compile-time mechanism (though hard with enum variants)

---

### 3. field.rs:139-145 — `apply_locale()` consumes locale even when rule is `None`

**Severity:** 🟡 Medium

Due to the `let` chain, `self.locale.take()` is called even when `self.rule` is `None`, causing the locale to be silently consumed:

```rust
pub fn apply_locale(&mut self) {
    if self.locale.is_some()
        && let (Some(locale), Some(rule)) = (self.locale.take(), self.rule.take())
    {
        self.rule = Some(rule.with_locale(locale.as_ref()));
    }
    // If rule was None: locale is now None too (consumed by take()), rule stays None
}
```

The test acknowledges this behavior (`test_apply_locale_no_op_without_rule`) but it's surprising and arguably incorrect — the locale should be preserved if there's nothing to apply it to.

**Impact:** Calling `apply_locale()` on a field with a locale but no rule silently drops the locale. If a rule is added later, it won't have the locale.

**Suggested fix:**
```rust
pub fn apply_locale(&mut self) {
    if let (Some(locale), Some(rule)) = (&self.locale, &self.rule) {
        self.rule = Some(rule.clone().with_locale(locale.as_ref()));
        self.locale = None;
    }
}
```

Or simply guard both:
```rust
pub fn apply_locale(&mut self) {
    if self.locale.is_some() && self.rule.is_some() {
        let locale = self.locale.take().unwrap();
        let rule = self.rule.take().unwrap();
        self.rule = Some(rule.with_locale(locale.as_ref()));
    }
}
```

---

### 4. field.rs:132 — Broken doc link `validate_ref`

**Severity:** 🟡 Medium

`cargo doc` reports:
```
warning: unresolved link to `validate_ref`
   --> crates/fieldfilter/src/field.rs:132:9
```

The doc comment references `` [`validate_ref`] `` without qualifying it.

**Impact:** Broken link in generated documentation.

**Suggested fix:** Change to `` [`Self::validate_ref`] `` or `` [`validate_ref`](Self::validate_ref) ``.

---

### 5. form_violations.rs:6 — Redundant explicit doc link

**Severity:** 🟡 Medium

`cargo doc` reports:
```
warning: redundant explicit link target
 --> crates/fieldfilter/src/form_violations.rs:6:51
```

**Suggested fix:**
```rust
//! **Deprecated:** Prefer [`FieldsetViolations`]
```

---

### 6. field_filter.rs:219-228 — `CrossFieldRule.fields` is redundant/unused

**Severity:** 🔵 Low

`CrossFieldRule` has a `fields: Vec<String>` member listing involved fields, but `evaluate()` never reads it — each `CrossFieldRuleType` variant already specifies its own fields. This is purely informational metadata.

```rust
pub struct CrossFieldRule {
    pub name: Option<Cow<'static, str>>,
    pub fields: Vec<String>,  // ← never used in evaluate()
    pub rule: CrossFieldRuleType,
}
```

**Impact:** Unnecessary allocation and potential for `fields` to be inconsistent with the fields in the rule type.

**Suggested fix:** Either remove `fields` or add a validation check that ensures consistency.

---

### 7. field_filter.rs:548 — `validate_async` clones values unnecessarily

**Severity:** 🔵 Low

The async `validate_async` clones each value:
```rust
let value = data.get(field_name).cloned().unwrap_or(Value::Null);
```

While the sync `validate` borrows:
```rust
let value = data.get(field_name).unwrap_or(&null);
```

This is likely forced by async lifetime constraints, but worth noting as a performance asymmetry.

**Impact:** Extra clones in async validation path. Minor for typical form data sizes.

---

### 8. field_filter.rs:160-168 — `filter_ref` silently drops unregistered fields

**Severity:** 🔵 Low

`filter_ref` returns only fields that exist in both `data` and `self.fields`. Unregistered fields are silently dropped, while `filter()` (owned) preserves them. This asymmetry could surprise users.

```rust
pub fn filter_ref(&self, data: &IndexMap<String, Value>) -> IndexMap<String, Value> {
    let mut result = IndexMap::with_capacity(self.fields.len());
    for (field_name, field) in &self.fields {
        if let Some(value) = data.get(field_name) {
            result.insert(field_name.clone(), field.filter_ref(value));
        }
    }
    result  // ← fields in `data` but not in `self.fields` are NOT included
}
```

The doc comment is accurate but the behavioral asymmetry with `filter()` is worth noting.

**Impact:** Users switching between `filter()` and `filter_ref()` may lose data unexpectedly.

**Suggested fix:** Either align the behavior (include unregistered fields as-is) or add a doc note explicitly calling out the difference from `filter()`.

## ✅ Clean Files

- `Cargo.toml` — dependencies are appropriate, no redundancies
- `src/lib.rs` — re-exports are well-organized
- `src/rule.rs` — clean re-export module
- `src/fieldset.rs` — trait design is solid, tests are comprehensive
- `src/form_violations.rs` — deprecated but correctly implemented (minus doc warning)
- `tests/async_fieldfilter.rs` — good async coverage
- `tests/derive_fieldset.rs` — thorough derive macro testing
- `examples/derive_simple.rs`
- `examples/derive_nested.rs`
- `examples/field_basics.rs`
- `examples/field_filter.rs`
- `examples/filters.rs`
- `examples/form_violations.rs`
- `examples/json_serialization.rs`
- `examples/localized_messages.rs`
- `examples/rule_composition.rs`
- `README.md` — mostly accurate (except RequiredIf/RequiredUnless description)

## Test Coverage Gaps

1. **`RequiredIf` — zero tests.** No unit or integration test exercises this rule type, which allowed the critical logic bug to go undetected.
2. **`RequiredUnless` — zero tests.** Same gap as above.
3. **`CustomAsync` in sync evaluate path — untested.** No test verifies what happens when a `CustomAsync` rule is evaluated synchronously.
4. **`apply_locale` with both locale and rule set then used** — the test only checks the field state after `apply_locale()`, not that validation actually produces localized messages.
5. **`FieldFilter::filter()` with unregistered fields** — no test verifies that `filter()` preserves fields not in `self.fields`.
6. **`Field<Value>::validate()` vs `validate_ref()`** — `validate()` delegates to `validate_value()` while `validate_ref()` uses `ValidateRef` trait. No test verifies these produce identical results for edge cases.

## Recommendations (Priority Order)

1. **Fix `RequiredIf`/`RequiredUnless` struct and logic** — add `condition_field` parameter and fix evaluation. Add comprehensive tests.
2. **Make `CustomAsync` fail explicitly in sync path** — return an error or panic instead of silently passing.
3. **Fix `apply_locale` to not consume locale when rule is absent** — guard both fields before taking.
4. **Fix the two `cargo doc` warnings** — broken link and redundant explicit link.
5. **Add tests for all untested cross-field rule types** — especially `RequiredIf`, `RequiredUnless`, and the `CustomAsync`-in-sync edge case.
6. **Document `filter_ref` vs `filter` behavioral difference** — or align the behavior.
7. **Consider removing or validating `CrossFieldRule.fields`** — either use it or lose it.
