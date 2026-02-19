# Plan: Introduce a Custom `form_core::Value` Enum

**Date:** February 19, 2026  
**Status:** Planned  
**Crates affected:** `walrs_form_core`, `walrs_inputfilter`

---

## Background

`form_core::Value` is currently a thin re-export of `serde_json::Value`:

```rust
// form_core/src/value.rs
pub use serde_json::Value;
```

This was a pragmatic starting point, but it creates hard blockers as the validation
system matures. See [Analysis](#analysis) below.

---

## Analysis

### Why not keep `serde_json::Value`?

- `serde_json::Value` does **not** implement any of `walrs_validator`'s traits
  (`SteppableValue`, `NumberValue`, `ScalarValue`, `InputValue`, `IsEmpty`).
- The orphan rule forbids implementing those traits on `serde_json::Value` in any
  third-party crate. This permanently walls off the entire numeric validation
  `impl<T: SteppableValue + IsEmpty> Rule<T>` block.
- `serde_json::Value::Number` is a single opaque type — there is no `Float(f64)` vs
  `Integer(i64)` distinction, so you cannot know at compile time (or easily at runtime)
  whether you are validating a float or an integer.
- The existing `Field<Value>::validate()` in `inputfilter/src/field.rs` already shows
  the result: it handles only `Rule::Required` by hand and leaves a `TODO` for everything
  else.

### Why not implement `SteppableValue` / `InputValue` for `form_core::Value`?

Even with a custom enum, satisfying the `SteppableValue` bound is a dead end:

- `SteppableValue` extends `NumberValue` → `ScalarValue` → `InputValue`.
- `InputValue` requires `Copy` (see `validator/src/traits.rs`):
  ```rust
  pub trait InputValue: Copy + Default + PartialEq + PartialOrd + Display + Serialize {}
  ```
- `form_core::Value` contains `String` and `Vec<Value>`, so it can **never** be `Copy`.
- Therefore `Value` can never satisfy `SteppableValue`, and the blanket
  `impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for Rule<T>` is permanently
  inaccessible for `Rule<Value>`.

### The correct approach: dedicated `impl Validate<Value>` and `impl ValidateRef<Value>` for `Rule<form_core::Value>`

The right model is the one already used for `Rule<String>` in `rule.rs`: a **standalone,
dedicated impl block** with a full `match self` dispatch — not a blanket trait bound.
There are currently two such paths in `rule.rs`:

| Impl | Constraint | Handles |
|------|------------|---------|
| `impl ValidateRef<str> for Rule<String>` | hardcoded `String` | String rules: `Required`, `MinLength`, `Email`, `Pattern`, … |
| `impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for Rule<T>` | `SteppableValue + IsEmpty + Copy` | Numeric rules: `Min`, `Max`, `Range`, `Step`, … |

A **third dedicated path** is added in `walrs_inputfilter`:

```
impl ValidateRef<Value> for Rule<Value>   →  inputfilter/src/value_impls.rs
impl Validate<Value>    for Rule<Value>   →  inputfilter/src/value_impls.rs
```

`walrs_inputfilter` already depends on both `walrs_form_core` and `walrs_validator`,
satisfying the orphan rule.

### Dispatch inside `impl ValidateRef<Value> for Rule<Value>`

Every numeric rule variant stores a `Value` as its bound (`Rule::Min(Value)`,
`Rule::Max(Value)`, etc.). At validation time the incoming value is also a `Value`.
The dispatch matches on **both** the rule variant and the incoming value variant:

| Rule variant + incoming `Value` variant | Action |
|---|---|
| `Rule::Required` + any | `is_empty_value()` check |
| `Rule::Min(Value::F64(min))` + `Value::F64(v)` | `v >= min` |
| `Rule::Min(Value::I64(min))` + `Value::I64(v)` | `v >= min` |
| `Rule::Min(Value::U64(min))` + `Value::U64(v)` | `v >= min` |
| `Rule::Step(Value::F64(s))` + `Value::F64(v)` | epsilon-aware `(v % s).abs() < f64::EPSILON` |
| `Rule::MinLength(n)` + `Value::Str(s)` | `s.chars().count() >= n` |
| `Rule::Email` + `Value::Str(s)` | delegate to `Rule::<String>::Email` path |
| `Rule::Equals(bound)` + any | `value == bound` via `PartialEq` |
| Type-mismatched rule + variant | Return `TypeMismatch` violation |
| `Rule::All / Any / Not / When` | recurse via `ValidateRef<Value>` |
| `Rule::Custom(f)` | `f(value)` |

A private `partial_cmp_value` helper handles ordering for numeric comparisons and
`Condition::GreaterThan` / `Condition::LessThan` in `Rule::When`, returning `None`
for mixed-type comparisons (treated as false).

### Can you still validate all form value types?

**Yes.** The two-tier mental model is:

- **`Rule<Value>`** — dynamic/heterogeneous rule for `Field<Value>`, used with HTTP
  form data, JSON payloads, or WASM boundaries where the scalar type is not statically
  known. All rule variants are reachable via the variant-dispatch described above.
- **`Rule<f64>`, `Rule<i64>`, `Rule<String>`** etc. — typed/precise rules for
  strongly-typed `Field<T>`. Their existing impls are completely untouched.

### Complexity assessment

| Task | Complexity | Notes |
|---|---|---|
| `impl IsEmpty for Value` | **Low** — trivial `match` | Lives in `inputfilter/src/value_impls.rs` |
| `impl ValidateRef<Value> for Rule<Value>` | **Medium** — `match self` + inner `match value` for numeric variants | Main implementation surface |
| `impl Validate<Value> for Rule<Value>` | **Low** — delegates to `ValidateRef<Value>` | Owned → ref |
| `Condition<Value>` in `Rule::When` | **Medium** — needs `partial_cmp_value` helper for `GreaterThan`/`LessThan` | Most nuanced part |
| `Rule::Step(Value)` epsilon logic | **Low** — copy `f64::rem_check` pattern from `SteppableValue` | |
| Cross-variant type mismatch | **Low risk** — distinct `I64`/`U64`/`F64` variants; no silent `as_f64()` downcasts | |
| Fix `Field<Value>::validate()` | **Low** — remove `TODO` stub, delegate to `validate_ref` | |

---

## Decision

Introduce a **custom `form_core::Value` enum** with distinct numeric variants
(`I64`, `U64`, `F64`), then implement `Validate<Value>` and `ValidateRef<Value>`
for `Rule<Value>` as a dedicated impl block in `walrs_inputfilter`.

---

## Enum Definition (Proposed)

```rust
// form_core/src/value.rs

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Native form value type.
///
/// Distinct numeric variants (`I64`, `U64`, `F64`) allow explicit type
/// discrimination in validation rules and avoid silent precision loss,
/// unlike `serde_json::Value::Number`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Str(String),
    Array(Vec<Value>),
    Object(IndexMap<String, Value>),
}
```

---

## Implementation Steps

### Step 1 — Replace the re-export in `form_core/src/value.rs`

Replace `pub use serde_json::Value` with the native enum above.
Add ergonomic `From` impls for primitive construction:

```rust
impl From<&str>   for Value { ... }  // → Value::Str
impl From<String> for Value { ... }  // → Value::Str
impl From<bool>   for Value { ... }  // → Value::Bool
impl From<i64>    for Value { ... }  // → Value::I64
impl From<i32>    for Value { ... }  // → Value::I64  (widening)
impl From<u64>    for Value { ... }  // → Value::U64
impl From<u32>    for Value { ... }  // → Value::U64  (widening)
impl From<f64>    for Value { ... }  // → Value::F64
impl From<f32>    for Value { ... }  // → Value::F64  (widening)
```

Update `ValueExt::is_empty_value()` to match on the new variants:

```rust
impl ValueExt for Value {
    fn is_empty_value(&self) -> bool {
        match self {
            Value::Null      => true,
            Value::Str(s)    => s.is_empty(),
            Value::Array(a)  => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            _                => false,
        }
    }
}
```

### Step 2 — Add `serde_json` bridge impls (feature-gated)

Make `serde_json` an optional dependency in `form_core/Cargo.toml`:

```toml
[features]
default = ["serde_json_bridge"]
serde_json_bridge = ["dep:serde_json"]

[dependencies]
serde_json = { version = "1.0", optional = true }
```

Add bridge impls in `form_core/src/value.rs` behind the feature flag:

```rust
#[cfg(feature = "serde_json_bridge")]
impl From<serde_json::Value> for Value {
    // serde_json::Number: try i64 first, then u64, then f64
}

#[cfg(feature = "serde_json_bridge")]
impl From<Value> for serde_json::Value {
    // I64/U64/F64 → serde_json::Number::from / serde_json::Number::from_f64
}
```

> `TryFrom` should also be provided for conversions that can fail (e.g., a
> `serde_json` integer outside `i64` range going to `Value::I64`).

### Step 3 — Add `value_impls.rs` to `walrs_inputfilter`

Create `inputfilter/src/value_impls.rs`. All impls live here because `walrs_inputfilter`
depends on both `walrs_form_core` and `walrs_validator`, satisfying the orphan rule.

#### 3a — `IsEmpty for Value`

```rust
use walrs_form_core::{Value, ValueExt};
use walrs_validator::IsEmpty;

impl IsEmpty for Value {
    fn is_empty(&self) -> bool {
        self.is_empty_value()
    }
}
```

#### 3b — `impl ValidateRef<Value> for Rule<Value>`

A dedicated `match self` covering all `Rule` variants. Numeric comparisons use
`partial_cmp_value`; string rules pattern-match on `Value::Str`:

```rust
use std::cmp::Ordering;
use walrs_form_core::Value;
use walrs_validator::{
    rule::{Condition, Rule, RuleResult},
    ValidateRef, Violation, ViolationType,
};
use walrs_validator::rule::{
    value_missing_violation, too_short_violation, too_long_violation,
    exact_length_violation, pattern_mismatch_violation, invalid_email_violation,
    invalid_url_violation, range_underflow_violation, range_overflow_violation,
    step_mismatch_violation, not_equal_violation, not_one_of_violation,
    unresolved_ref_violation, negation_failed_violation,
};

impl ValidateRef<Value> for Rule<Value> {
    fn validate_ref(&self, value: &Value) -> RuleResult {
        match self {
            Rule::Required => {
                use walrs_form_core::ValueExt;
                if value.is_empty_value() { Err(value_missing_violation()) } else { Ok(()) }
            }
            Rule::MinLength(n) => match value {
                Value::Str(s) => {
                    let len = s.chars().count();
                    if len < *n { Err(too_short_violation(*n, len)) } else { Ok(()) }
                }
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::MaxLength(n) => match value {
                Value::Str(s) => {
                    let len = s.chars().count();
                    if len > *n { Err(too_long_violation(*n, len)) } else { Ok(()) }
                }
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::ExactLength(n) => match value {
                Value::Str(s) => {
                    let len = s.chars().count();
                    if len != *n { Err(exact_length_violation(*n, len)) } else { Ok(()) }
                }
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::Pattern(p) => match value {
                Value::Str(s) => Rule::<String>::Pattern(p.clone()).validate_ref(s.as_str(), None),
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::Email => match value {
                Value::Str(s) => Rule::<String>::Email.validate_ref(s.as_str(), None),
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::Url => match value {
                Value::Str(s) => Rule::<String>::Url.validate_ref(s.as_str(), None),
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::Min(bound) => match partial_cmp_value(value, bound) {
                Some(Ordering::Less) => Err(range_underflow_violation(bound)),
                Some(_)              => Ok(()),
                None                 => Err(Violation::new(ViolationType::TypeMismatch, "Incompatible types for Min")),
            },
            Rule::Max(bound) => match partial_cmp_value(value, bound) {
                Some(Ordering::Greater) => Err(range_overflow_violation(bound)),
                Some(_)                 => Ok(()),
                None                    => Err(Violation::new(ViolationType::TypeMismatch, "Incompatible types for Max")),
            },
            Rule::Range { min, max } => {
                match partial_cmp_value(value, min) {
                    Some(Ordering::Less) => return Err(range_underflow_violation(min)),
                    None => return Err(Violation::new(ViolationType::TypeMismatch, "Incompatible types for Range")),
                    _ => {}
                }
                match partial_cmp_value(value, max) {
                    Some(Ordering::Greater) => Err(range_overflow_violation(max)),
                    None => Err(Violation::new(ViolationType::TypeMismatch, "Incompatible types for Range")),
                    _ => Ok(()),
                }
            },
            Rule::Step(step) => {
                let ok = match (value, step) {
                    (Value::F64(v), Value::F64(s)) => (*s != 0.0) && ((*v % *s).abs() < f64::EPSILON),
                    (Value::I64(v), Value::I64(s)) => (*s != 0) && (*v % *s == 0),
                    (Value::U64(v), Value::U64(s)) => (*s != 0) && (*v % *s == 0),
                    _ => return Err(Violation::new(ViolationType::TypeMismatch, "Incompatible types for Step")),
                };
                if ok { Ok(()) } else { Err(step_mismatch_violation(step)) }
            },
            Rule::Equals(expected) => {
                if value == expected { Ok(()) } else { Err(not_equal_violation(expected)) }
            },
            Rule::OneOf(allowed) => {
                if allowed.iter().any(|v| v == value) { Ok(()) } else { Err(not_one_of_violation()) }
            },
            Rule::All(rules) => {
                for rule in rules { rule.validate_ref(value)?; }
                Ok(())
            },
            Rule::Any(rules) => {
                if rules.is_empty() { return Ok(()); }
                let mut last_err = None;
                for rule in rules {
                    match rule.validate_ref(value) {
                        Ok(()) => return Ok(()),
                        Err(e) => last_err = Some(e),
                    }
                }
                Err(last_err.unwrap())
            },
            Rule::Not(inner) => match inner.validate_ref(value) {
                Ok(()) => Err(negation_failed_violation()),
                Err(_) => Ok(()),
            },
            Rule::When { condition, then_rule, else_rule } => {
                if evaluate_value_condition(condition, value) {
                    then_rule.validate_ref(value)
                } else {
                    else_rule.as_ref().map_or(Ok(()), |r| r.validate_ref(value))
                }
            },
            Rule::Custom(f)  => f(value),
            Rule::Ref(name)  => Err(unresolved_ref_violation(name)),
            Rule::WithMessage { rule, message } => rule.validate_ref(value).map_err(|v| {
                let msg = message.resolve(value, None);
                Violation::new(v.violation_type(), msg)
            }),
        }
    }
}
```

Private helpers:

```rust
/// Compares two `Value`s of the same numeric or string variant.
/// Returns `None` for mixed-type or non-orderable comparisons.
fn partial_cmp_value(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::I64(x), Value::I64(y)) => x.partial_cmp(y),
        (Value::U64(x), Value::U64(y)) => x.partial_cmp(y),
        (Value::F64(x), Value::F64(y)) => x.partial_cmp(y),
        (Value::Str(x), Value::Str(y)) => x.partial_cmp(y),
        _ => None,
    }
}

/// Evaluates a `Condition<Value>` against an incoming value.
fn evaluate_value_condition(condition: &Condition<Value>, value: &Value) -> bool {
    match condition {
        Condition::IsEmpty        => { use walrs_form_core::ValueExt; value.is_empty_value() },
        Condition::IsNotEmpty     => { use walrs_form_core::ValueExt; !value.is_empty_value() },
        Condition::Equals(e)      => value == e,
        Condition::GreaterThan(t) => partial_cmp_value(value, t) == Some(Ordering::Greater),
        Condition::LessThan(t)    => partial_cmp_value(value, t) == Some(Ordering::Less),
        Condition::Matches(p)     => match value {
            Value::Str(s) => regex::Regex::new(p).map(|re| re.is_match(s)).unwrap_or(false),
            _ => false,
        },
        Condition::Custom(f) => f(value),
    }
}
```

#### 3c — `impl Validate<Value> for Rule<Value>`

Delegates entirely to `ValidateRef<Value>`:

```rust
impl Validate<Value> for Rule<Value> {
    fn validate(&self, value: Value) -> RuleResult {
        self.validate_ref(&value)
    }
}
```

### Step 4 — Fix `Field<Value>::validate()` in `inputfilter/src/field.rs`

Remove the manual `Rule::Required` stub and the `TODO` comment.
Delegate to the now-implemented `ValidateRef<Value>`:

```rust
impl Field<Value> {
    pub fn validate(&self, value: &Value) -> Result<(), Violations> {
        match &self.rule {
            Some(rule) => rule.validate_ref(value).map_err(|v| {
                let mut vs = Violations::empty();
                vs.push(v);
                vs
            }),
            None => Ok(()),
        }
    }
}
```

### Step 5 — Wire up and test

- Add `pub mod value_impls;` to `inputfilter/src/lib.rs`.
- Add `indexmap = "2"` to `form_core/Cargo.toml` (for the `Object` variant).
- Fix any downstream call sites in `walrs_form`, `walrs_inputfilter`, etc. that used
  `serde_json::Value`-specific APIs (e.g., `Value::Number(...)`, `.as_i64()`).
- Run `cargo test --workspace` and fix failures.

---

## Further Considerations

1. **`Condition<Value>` ordering** — `Value` may also implement `PartialOrd` using
   `partial_cmp_value` so that the generic `Condition::evaluate` in `rule.rs` works
   naturally. Alternatively, keep ordering entirely inside `evaluate_value_condition`
   and never expose `PartialOrd` on `Value` publicly.

2. **Type-mismatched rules** — When a numeric rule (e.g., `Rule::Min`) is applied to
   a non-numeric `Value` (e.g., `Value::Bool`), returning a `TypeMismatch` violation
   is safer than silently passing through — it makes misconfigured forms visible
   during development.

3. **Integer vs. float split** — `I64` + `U64` + `F64` variants give lossless
   round-tripping and make type intent explicit in rules. A single `Number(f64)` would
   be simpler but loses large `u64` precision and blurs the float/integer distinction
   that motivated this change.

4. **`serde_json` as optional dep** — The `serde_json_bridge` feature reduces WASM
   binary size for consumers that only need native value handling without JSON I/O.

5. **Typed fields remain unchanged** — `Rule<f64>`, `Rule<i64>`, `Rule<String>` and
   their existing `SteppableValue` / `ValidateRef<str>` impls are completely untouched.
   `Field<Value>` is purely additive.

