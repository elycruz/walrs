# Validation Crate Review

**Date:** 2026-04-11
**Scope:** `crates/validation/` — ~12K lines across 17 source files
**Verdict:** Well-engineered overall. One bug found; several test coverage gaps identified.

---

## 🔴 Bug: NaN Silently Passes `Min`/`Max`/`Range` for `f32`/`f64`

**Files:** `rule_impls/scalar.rs:20-43`, `rule_impls/steppable.rs:18-39`

Due to IEEE 754 semantics, all NaN comparisons return `false`. This causes NaN values
to silently pass Min, Max, and Range validation:

```rust
Rule::Min(0.0).validate(f64::NAN)                        // → Ok(()) ❌ should fail
Rule::Max(100.0).validate(f64::NAN)                       // → Ok(()) ❌ should fail
Rule::Range { min: 0.0, max: 100.0 }.validate(f64::NAN)  // → Ok(()) ❌ should fail
```

**Root cause:** The check `if value < *min` evaluates to `false` for NaN, so the else
branch returns `Ok(())`.

**Suggested fix:** Add an explicit `value.is_nan()` guard before numeric comparisons
for float types. This could be done via a trait method on `ScalarValue`/`SteppableValue`
(returning `false` for integers, `self.is_nan()` for floats), or via separate float
impls.

> **Note:** The `Rule<Value>` path is **not affected** — it uses `partial_cmp()` which
> correctly returns `None` for NaN, mapped to a `TypeMismatch` error.

---

## ⚠️ Design Notes

These are not bugs but are worth awareness and potential documentation.

### `f64::EPSILON` in Step Validation

`SteppableValue::rem_check` for floats uses `(value % step).abs() < f64::EPSILON`.
This works for typical form-validation numbers but can produce false negatives for
very large floats (e.g., `1e15`) where the absolute remainder exceeds the fixed epsilon
even when the value is logically a valid multiple.

**File:** `traits.rs:85-92`

### `Equals(NaN)` / `OneOf([NaN])` Always Fails

Per IEEE 754, `NaN ≠ NaN`, so `Equals(f64::NAN).validate(f64::NAN)` returns `Err`.
Mathematically correct, but potentially surprising to users.

**File:** `rule_impls/scalar.rs:46-51`, `rule_impls/steppable.rs:48-53`

### Length Counting Uses Unicode Scalar Values

All string length rules use `.chars().count()`, which counts Unicode scalar values
rather than grapheme clusters. This is a reasonable and consistent choice but differs
from what users might expect for characters like emoji with modifiers (e.g., 👨‍👩‍👧‍👦
is 7 chars, not 1).

**File:** `rule_impls/string.rs` (all MinLength/MaxLength/ExactLength paths)

---

## ✅ Verified Correct

### Core Logic
- **`Rule::All`** — Correctly short-circuits on first failure (fail-fast)
- **`Rule::Any`** — Returns last error if all fail; empty `Any([])` returns `Ok`
- **`Rule::Not`** — Correctly inverts the inner result
- **`Rule::When`** — Condition evaluation and branch selection correct
- **Empty combinators** — `All([])` → `Ok`, `Any([])` → `Ok` (vacuous truth, correct)

### Safety
- **No `unsafe` code** found anywhere in the crate
- **All production `unwrap()` calls** are guarded by prior checks:
  - `labels.last().unwrap()` — protected by empty-hostname early return
  - `last_err.unwrap()` in `Any` — protected by `rules.is_empty()` check
- **No integer overflow risks** — comparisons only, no arithmetic on bounds
- **`Step(0)` / `Step(0.0)`** — correctly returns `false` (step mismatch)

### Serde
- **`CompiledPattern`** round-trips correctly (serializes as string, deserializes via `Regex::new`)
- **`#[serde(skip)]`** correctly applied to `Custom`, `CustomAsync`, `Ref`, `WithMessage`
- **`PartialEq`** for function-containing variants correctly returns `false`

### `Option<T>` Handling
- `None` with `Required` → `Err(value_missing)` — correct
- `None` without `Required` → `Ok(())` — correct
- `Some(v)` → delegates to inner validation — correct
- `None` vs `Some("")` correctly distinguished for strings

### String Validation
- **Email** — RFC-compliant; respects all `EmailOptions` flags; correctly uses `rfind('@')`
- **URL/URI** — Correct scheme filtering; URI allows relative, URL requires absolute
- **IP** — IPv4/IPv6/IPvFuture all correctly parsed via `std::net` + manual IPvFuture
- **Hostname** — RFC 952/1123 compliant; total length ≤253; TLD alphabetic check correct
- **Pattern** — `regex::Regex` is `Send + Sync`; compilation errors handled at construction
- **Required** — Empty string and whitespace-only both treated as missing (via `.trim().is_empty()`)

### Numeric Validation
- **Min/Max/Range** — All inclusive bounds; consistent and documented
- **Integer types** — All macro-generated impls correct for all target types
- **`f64::INFINITY`** / **`f64::NEG_INFINITY`** — Correctly fail appropriate bound checks

### Value Validation
- **Type dispatch** — Correctly matches `Value` variant and dispatches; `TypeMismatch` for incompatible types
- **No implicit coercion** — `Value::Str("123")` validated as string, not number
- **`Value::Null`** — Treated as empty for `Required`, consistent with `None`
- **NaN in Value** — `partial_cmp` returns `None` → `TypeMismatch` (correct)

### Date Validation (chrono & jiff)
- **Format handling** — All 5 `DateFormat` variants handled in all match arms
- **Feature gating** — `#[cfg(feature = "chrono")]` / `#[cfg(feature = "jiff")]` correct
- **Invalid dates** — Correctly rejected by underlying libraries (Feb 30, etc.)
- **DateRange** — Min/max bounds are inclusive

### Async Validation
- No shared mutable state; all futures properly awaited
- `CustomAsync` correctly integrated into combinators

---

## 📋 Test Coverage Gaps

Ordered by impact (most likely to hide real bugs first).

### 1. NaN Assertions Missing (High)

Existing NaN tests in edge-case tests only `println!` results — they never `assert!`.
This means the NaN bug described above is not caught by CI.

**Action:** Add assertions to NaN tests; add new tests for NaN + Min/Max/Range.

### 2. `Rule::Ref` Completely Untested (High)

`Rule::Ref` always returns `Err(Violation::unresolved_ref(name))` in all impls, but
zero tests verify this behavior or the error contents.

**Action:** Add basic `Rule::Ref` tests for string, numeric, and Value types.

### 3. `TypeMismatch` for `Value` Undertested (Medium-High)

Only 1 test (`test_type_mismatch_min`) explicitly checks `TypeMismatch` violations,
yet 18+ mismatch paths exist in `rule_impls/value.rs`.

**Untested scenarios include:**
- `MaxLength` / `ExactLength` on non-string Value
- `Pattern` / `Email` / `Url` / `Uri` / `Ip` / `Hostname` on non-string Value
- `Min` / `Max` / `Range` / `Step` on non-numeric Value
- `Date` / `DateRange` on non-string Value

**Action:** Add parametrized TypeMismatch tests for all Value rule variants.

### 4. Serde Round-Trip Testing Minimal (Medium)

Only 2 serde tests exist (`Range` and `All([Required, MinLength, MaxLength])`).

**Untested variants:**
- `Pattern` (has custom `Serialize`/`Deserialize` for `CompiledPattern`)
- `Email` / `Url` / `Uri` / `Ip` / `Hostname` with non-default options
- `Date` / `DateRange` with custom `DateFormat`
- `When` with conditions
- `OneOf` with various types
- `Any` combinator

**Action:** Add round-trip tests for variants with custom serde or complex options.

### 5. Deeply Nested Combinators Untested (Medium)

No tests exist with nesting patterns like:
- `All([All([...]), Any([...])])`
- `When { then: All([...]), else: Some(Any([...])) }`
- `Not(All([Any(...)]))`

**Action:** Add depth-2 and depth-3 nesting tests.

### 6. Empty `Any([])` Not Explicitly Tested (Medium)

`Any(vec![])` returns `Ok(())` by implementation, but no test asserts this.

**Action:** Add explicit empty-`Any` test alongside the existing empty-`All` coverage.

### 7. Date Edge Cases Missing (Medium)

No tests for: leap years (Feb 29 valid vs invalid years), impossible dates (Feb 30,
Apr 31), year boundaries, or negative/zero years.

**Action:** Add date edge-case tests for chrono and jiff paths.

### 8. Async `When` with `else_rule` Untested (Low-Medium)

`tests/async_validation.rs` tests `When` with `then_rule` only; no test covers the
`else_rule` branch in async context.

**Action:** Add async `When` test with `else_rule`.
