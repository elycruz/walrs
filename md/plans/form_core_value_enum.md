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
                Value::Str(s) => Rule::<String>::Pattern(p.clone()).validate_str(s.as_str()),
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::Email => match value {
                Value::Str(s) => Rule::<String>::Email.validate_str(s.as_str()),
                _ => Err(Violation::new(ViolationType::TypeMismatch, "Expected a string")),
            },
            Rule::Url => match value {
                Value::Str(s) => Rule::<String>::Url.validate_str(s.as_str()),
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

---

## Addendum: Concrete Type Group Impls — `Rule<Scalar>` and `Rule<str>`

**Question:** If we want a more concrete implementation (impls for concrete type groups), can
we achieve this? Say, if I wanted to support something like `Rule<Scalar>` or `Rule<str>`, is
there a design that will allow us to do this?

**Short answer:** Yes — both are achievable without breaking existing code. The details differ
per approach. See the analysis below.

---

### Approach A — `Scalar` newtype enum

Define a `Scalar` enum in `form_core` (parallel to the `Value` enum, but limited to scalar
primitives):

```rust
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Scalar {
    Bool(bool),
    Char(char),
    I8(i8),   I16(i16),   I32(i32),   I64(i64),   I128(i128),   ISize(isize),
    U8(u8),   U16(u16),   U32(u32),   U64(u64),   U128(u128),   USize(usize),
    F32(f32), F64(f64),
}
```

Because every inner type is `Copy`, `Scalar` itself derives `Copy`. Add a dedicated impl in
`walrs_inputfilter`:

```rust
impl Validate<Scalar> for Rule<Scalar> { … }
impl ValidateRef<Scalar> for Rule<Scalar> { … }
```

**Pros**
- A single impl block covers every scalar primitive with no 14-way monomorphisation.
- Completely additive — `Rule<i32>`, `Rule<f64>`, etc. keep using the existing blanket
  `Validate<T: SteppableValue>` path unchanged.
- Follows exactly the same design pattern already planned for `form_core::Value`.
- `Scalar` is `Copy`, so `Validate<Scalar>` (pass-by-copy) works cleanly.
- `Equals`, `OneOf`, `Min`, `Max`, `Range` all work naturally via a `partial_cmp_scalar`
  helper (same design as `partial_cmp_value` from the `Value` plan).
- Serializable via `serde`.

**Cons**
- `Scalar` can never satisfy `SteppableValue` (which is a `macro_rules!`-implemented marker
  on concrete primitives). A dedicated impl is required — not the blanket one.
- Cross-variant numeric comparisons need an explicit `partial_cmp_scalar` helper; type-mixing
  (e.g., `Min(Scalar::I64(n))` vs. value `Scalar::F64(v)`) must return `TypeMismatch`.
- `Step` must dispatch on the variant pair; no single `rem_check()` call is available.

**When to use:** When you genuinely need a single runtime-typed "any scalar" rule — e.g., a
schema-driven form where a field's scalar type is not statically known at compile time.
*Not* needed for statically-typed `Rule<i32>` / `Rule<f64>` fields.

---

### Approach B — `Rule<str>` and its variants

`str` is `?Sized`, making `Rule<str>` **impossible** — `Rule<T>` contains `Vec<Rule<T>>`,
`Box<Rule<T>>`, and `Arc<dyn Fn(&T)>`, all of which require `T: Sized`. This is a hard Rust
type-system boundary that cannot be lifted.

**B1 — Status quo: `impl ValidateRef<str> for Rule<String>` (already exists)**

- Works perfectly and is already in production. `Rule<String>` validates `&str` via
  `validate_ref(&str)`.
- Zero implementation cost.
- **Recommendation: keep this; it is the correct approach.**

**B2 — Type alias `type StrRule = Rule<String>`**

- A purely cosmetic alias: `StrRule::MinLength(3)` instead of `Rule::<String>::MinLength(3)`.
- No breaking change, zero runtime cost.
- **Recommendation: acceptable as a readability aid, not a necessity.**

**B3 — Dedicated `StrValidator` struct**

- Warranted only if string-specific variants grow beyond what `Rule<T>` can carry
  (e.g., `StrRule::Ascii`, `StrRule::Utf8MaxBytes(n)`, `StrRule::NotBlank`).
- Would implement `ValidateRef<str>` directly.
- Fragments the rule API — callers must know two types.
- **Recommendation: only if the string-specific variant set grows significantly.**

---

### Approach C — `Rule<Box<dyn ScalarValue>>`

**Not feasible.** `ScalarValue` extends `InputValue` which requires `Copy`. `dyn Trait` is
never `Copy` because it is a fat pointer. The compiler rejects `Box<dyn ScalarValue>` at the
trait-object site. Relaxing to `Box<dyn Any>` loses all type-safe dispatch and requires
`downcast` back to concrete types, recovering none of the benefit.

**Recommendation: do not pursue.**

---

### Approach D — Phantom-type-tagged `Rule<T, Kind>`

Add `PhantomData<Kind>` to `Rule<T>` and write blanket impls gated on the kind marker:

```rust
pub enum Rule<T, Kind = ()> { … }
pub struct ScalarKind;
impl<T: ScalarValue> Validate<T> for Rule<T, ScalarKind> { … }
```

**Pros**
- Theoretically allows per-group specialisation without separate types.

**Cons**
- **Massive breaking change.** Every existing `Rule<String>`, `Rule<i32>`, `Rule<f64>` call
  site must change. Even with `Kind = ()` as the default, every `impl` block that names
  `Rule<T>` explicitly must be rewritten.
- The phantom carries no runtime value — all dispatch still happens in `match self` arms.
- Serde derives become more complex (`#[serde(bound = "…")]`).
- No practical gain over Approach A or B1.

**Recommendation: do not pursue unless a phantom kind is needed for another architectural
reason.**

---

### Approach E — Separate `ScalarRule`, `StringRule`, `NumberRule` enums

Define non-generic enums per category:

```rust
pub enum ScalarRule { Required, Equals(Scalar), OneOf(Vec<Scalar>), … }
pub enum StringRule { Required, MinLength(usize), Email, Pattern(String), … }
pub enum NumberRule { Required, Min(f64), Max(f64), Step(f64), … }
```

**Pros**
- No generics — simpler ergonomics for callers who only use one category.
- Each enum is lean; no unused variants (e.g., `NumberRule` has no `Email`).

**Cons**
- Completely abandons the unified `Rule<T>` API.
- Cannot compose across categories in `All` / `Any` / `When` without boxing to a common
  trait object.
- Three separate codepaths for combinators.
- The biggest advantage of `Rule<T>` — one tree structure for all types — disappears.

**Recommendation: only for a stripped-down public API (e.g., WASM bindings). Not a
replacement for the core system.**

---

### Summary Table

| Approach | Verdict |
|---|---|
| **A — `Scalar` enum** | ✅ Viable and pragmatic — additive, same pattern as `Value` |
| **B1 — `Rule<String>` + `ValidateRef<str>`** | ✅ Keep as-is — already correct |
| **B2 — `type StrRule` alias** | ⚠️ Optional readability aid only |
| **B3 — `StrValidator` struct** | ⚠️ Only if string variants proliferate significantly |
| **C — `Rule<Box<dyn ScalarValue>>`** | ❌ Impossible (`Copy` bound on `ScalarValue`) |
| **D — `Rule<T, Kind>` phantom** | ❌ High breaking-change cost, no practical gain |
| **E — Separate rule enums** | ❌ Fragments the unified `Rule<T>` API |

### Overall Recommendation

1. **Strings** — Do nothing new. `Rule<String>` with `ValidateRef<str>` already achieves
   `Rule<str>` semantics. Optionally add a `type StrRule = Rule<String>` alias for ergonomics.
2. **`Rule<Scalar>`** — Introduce `Scalar` (Approach A) **only** when you need
   heterogeneous scalar validation at runtime (schema-driven / WASM boundary). Follow the
   exact same pattern planned for `form_core::Value`.
3. **Typed fields** — The existing blanket `impl<T: SteppableValue + IsEmpty + Clone>
   Validate<T> for Rule<T>` already handles every concrete numeric type. `Rule<i32>`,
   `Rule<f64>`, etc. are unchanged.

### Can we achieve `Rule<Scalar>` and `Rule<str>` semantics without breaking existing code?

| Goal | Achievable? | How |
|---|---|---|
| `Rule<Scalar>` (dynamic scalar) | ✅ Yes — additive new type | New `Scalar` enum + dedicated impl in `inputfilter`; no existing impls touched |
| `Rule<str>` (validate `&str`) | ✅ Already exists | `Rule<String>` implements `ValidateRef<str>`; use B2 alias if ergonomics matter |
| Keep `Rule<i32>`, `Rule<f64>` etc. | ✅ Completely untouched | Blanket `SteppableValue` impl remains |

### Implementation Notes for `Rule<Scalar>`

- `Scalar` derives `Copy + Clone` (all inner types are `Copy`), so `Validate<Scalar>` passes
  values by copy.
- `Condition<Scalar>` requires a `partial_cmp_scalar` helper for `GreaterThan`/`LessThan`;
  mixed-variant comparisons return `None` and become `TypeMismatch` violations.
- `Rule<Scalar>` will not accidentally satisfy the blanket `SteppableValue` impl because
  `SteppableValue` is implemented only on concrete primitive types via `macro_rules!`.
  There is no coherence conflict.
- Place the `Scalar` enum in `form_core/src/scalar.rs`; place the
  `impl Validate<Scalar> for Rule<Scalar>` in `inputfilter/src/scalar_impls.rs` (same
  structure as `value_impls.rs`).

---

## Addendum: Validation Coverage for All Target Types

**Question:** What is the most optimal design to support validation of all of the following?

```
&str / String
number primitives  (i8–i128, u8–u128, isize, usize, f32, f64)
primitive scalars  (bool, char)
Vec<T>, &[T]
HashMap, HashSet, BTreeMap, BTreeSet, VecDeque
IndexMap, IndexSet
user-defined structs
```

---

### 1. Type-Group Coverage Matrix

| Type group | Already works | Gap |
|---|---|---|
| `&str`, `String` | ✅ `ValidateRef<str> for Rule<String>` + `LengthValidator<str>` | None |
| numeric primitives (`i8`–`f64`) | ✅ `Validate<T: SteppableValue>` blanket + `RangeValidator`/`StepValidator` | None |
| `bool`, `char` | ❌ `ScalarValue` but **not** `SteppableValue` — no `Validate` impl exists | Gap 2 |
| `Vec<T>`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `VecDeque` | ⚠️ `WithLength` impls + `LengthValidator` work, but `validate_len_ref` is NOT wired to `ValidateRef<T>` | Gap 1 |
| `&[T]` | ⚠️ `WithLength for [T]` + `LengthValidator<[T]>` work; `Rule<[T]>` is **impossible** (`T: Sized` required) | Gap 3 — use `LengthValidator` directly |
| `IndexMap`, `IndexSet` | ❌ `indexmap` crate not added; no `WithLength` impl | Gap 4 |
| user-defined structs | ⚠️ `Rule::Custom(Arc<dyn Fn(&T)->RuleResult>)` works as an escape hatch | Gap 5 — no field-level composition |

---

### 2. Concrete Minimal Design per Gap

#### Gap 1 — Wire `validate_len_ref` into `ValidateRef<C>` for sized collections

`Rule<T>` requires `T: Sized` (it stores `Vec<Rule<T>>`, `Min(T)`, etc.), so
`impl<C: WithLength + ?Sized> ValidateRef<C> for Rule<C>` is **impossible**. But
all real collection types — `Vec<T>`, `HashMap<K,V>`, etc. — are `Sized`.

**Fix**: add a blanket impl in `validator/src/rule.rs`:

```rust
impl<C: WithLength> ValidateRef<C> for Rule<C> {
    fn validate_ref(&self, value: &C) -> ValidatorResult {
        self.validate_len_ref(value)
    }
}
```

This covers every concrete collection that already satisfies `WithLength`. The
existing `impl ValidateRef<str> for Rule<String>` is unaffected — `String ≠ str`,
so there is no coherence conflict. **Additive, non-breaking.**

> Fix the known limitation in `validate_len_ref`'s `When` arm at the same time:
> it currently ignores `condition` and always applies `then_rule` when the collection
> is non-empty. After this landing, evaluate `Condition::IsEmpty` /
> `Condition::IsNotEmpty` properly against `value.length() == 0`.

#### Gap 2 — `bool` and `char` validation

`bool` and `char` satisfy `ScalarValue` but **not** `SteppableValue`, so the blanket
numeric impl is inaccessible. Widening the blanket requires specialisation (unstable).

**Fix**: two dedicated impls in `validator/src/rule.rs` (same file, same pattern as
the `Rule<String>` impl). Also add `IsEmpty for bool` and `IsEmpty for char`.

```rust
impl IsEmpty for bool { fn is_empty(&self) -> bool { false } }
impl IsEmpty for char { fn is_empty(&self) -> bool { false } }

impl Validate<bool> for Rule<bool> {
    fn validate(&self, value: bool) -> ValidatorResult {
        match self {
            Rule::Required  => Ok(()),   // bool is always "present"
            Rule::Equals(e) => if value == *e { Ok(()) } else { Err(not_equal_violation(e)) },
            Rule::OneOf(v)  => if v.contains(&value) { Ok(()) } else { Err(not_one_of_violation()) },
            Rule::All(rules)  => { for r in rules { r.validate(value)?; } Ok(()) },
            Rule::Any(rules)  => { /* first-pass short-circuit */ … },
            Rule::Not(inner)  => inner.validate(value).map(|_| Err(negation_failed_violation())).unwrap_or(Ok(())),
            Rule::When { condition, then_rule, else_rule } => { … }
            Rule::Custom(f)   => f(&value),
            Rule::WithMessage { rule, message } => { … }
            Rule::Ref(name)   => Err(unresolved_ref_violation(name)),
            // Nonsensical for bool — type mismatch
            Rule::Min(_) | Rule::Max(_) | Rule::Range { .. } | Rule::Step(_) |
            Rule::MinLength(_) | Rule::MaxLength(_) | Rule::ExactLength(_) |
            Rule::Pattern(_) | Rule::Email | Rule::Url => {
                Err(Violation::new(ViolationType::TypeMismatch, "Rule not applicable to bool"))
            }
        }
    }
}

impl Validate<char> for Rule<char> {
    // char is PartialOrd, so Min/Max/Range are meaningful.
    // Step is not, since char has no rem operation.
    fn validate(&self, value: char) -> ValidatorResult { … }
}
```

**Additive, non-breaking.**

#### Gap 3 — `&[T]` with the `Rule` system

`[T]` is `?Sized`; `Rule<[T]>` is a compile-time impossibility. This is a hard
Rust type-system boundary.

**Decision**: Do not add a `Rule<[T]>` type. Document the convention:

> For slice validation (`&[T]`), use `LengthValidator<[T]>` directly or convert to
> `Vec<T>` at the boundary and use `Rule<Vec<T>>`.

For element-iteration rules (validate each element), introduce a separate
`CollectionValidator<C, ElemV>` struct in `validator/src/length.rs`:

```rust
pub struct CollectionValidator<C: WithLength, ElemV> {
    pub length_rule:    Option<Rule<C>>,          // optional length constraints
    pub element_rule:   Option<ElemV>,             // applied to each element
}
// impl<C: WithLength + IntoIterator, ElemV: ValidateRef<C::Item>>
//     ValidateRef<C> for CollectionValidator<C, ElemV>
```

This is **new additive** — no breaking changes.

#### Gap 4 — `IndexMap` / `IndexSet`

Add a feature-gated `indexmap` dependency:

```toml
# validator/Cargo.toml
[features]
indexmap = ["dep:indexmap"]

[dependencies]
indexmap = { version = "2", optional = true }
```

In `validator/src/length.rs`:

```rust
#[cfg(feature = "indexmap")]
use indexmap::{IndexMap, IndexSet};

#[cfg(feature = "indexmap")]
validate_type_with_len!(IndexMap<K, V, S>, K, V, S);

#[cfg(feature = "indexmap")]
validate_type_with_len!(IndexSet<T, S>, T, S);
```

Once `WithLength` is implemented, Gap 1's blanket `impl<C: WithLength> ValidateRef<C>
for Rule<C>` covers them automatically. **Additive, feature-gated, non-breaking.**

#### Gap 5 — User-defined structs

`Rule::Custom(Arc<dyn Fn(&T) -> RuleResult + Send + Sync>)` already validates any
`T` today — no changes needed for the immediate case.

For composable field-level validation (future work), add a `SchemaValidator<T>` in a
new `validator/src/schema.rs`:

```rust
pub struct SchemaValidator<T> {
    /// Each entry is a type-erased (extractor + rule) pair.
    fields: Vec<Box<dyn Fn(&T) -> ValidatorResult + Send + Sync>>,
}

impl<T> SchemaValidator<T> {
    pub fn field<F, R>(mut self, extractor: impl Fn(&T) -> &F + 'static + Send + Sync,
                        rule: R) -> Self
    where
        F: ?Sized,
        R: ValidateRef<F> + 'static + Send + Sync,
    {
        self.fields.push(Box::new(move |v| rule.validate_ref(extractor(v))));
        self
    }
}

impl<T> ValidateRef<T> for SchemaValidator<T> {
    fn validate_ref(&self, value: &T) -> ValidatorResult {
        for f in &self.fields { f(value)?; }
        Ok(())
    }
}
```

A `#[derive(Validate)]` proc-macro (`walrs_validator_derive`) is a further-future
crate that generates `SchemaValidator` construction automatically from struct field
annotations.

---

### 3. Recommended Implementation Sequence

| Priority | Task | File(s) | Risk |
|---|---|---|---|
| **P0** | `impl<C: WithLength> ValidateRef<C> for Rule<C>` + fix `When` condition in `validate_len_ref` | `validator/src/rule.rs` | Zero — additive |
| **P1** | `impl IsEmpty for bool/char` + `impl Validate<bool/char> for Rule<bool/char>` | `validator/src/rule.rs` | Zero — additive |
| **P2** | `indexmap` feature gate + `WithLength` impls | `validator/Cargo.toml`, `validator/src/length.rs` | Zero — feature-gated |
| **P3** | Document `&[T]` convention; optionally add `CollectionValidator<C, ElemV>` | `validator/src/length.rs`, `README.md` | Zero |
| **P4** *(optional)* | `SchemaValidator<T>` for struct field rules | `validator/src/schema.rs` | Zero — new module |
| **P5** *(future)* | `walrs_validator_derive` proc-macro crate for `#[derive(Validate)]` | new crate | Separate crate |

---

### 4. Single-Trait vs Per-Type Architecture

A natural question: could a single `Validatable` marker trait unify everything
into one `impl<T: Validatable> ValidateRef<T> for Rule<T>`?

**Analysis**:
- `Validatable` would need to be `?Sized` to cover `str` and `[T]`, but `Rule<T>`
  requires `T: Sized`. This is the same conflict as Gap 3 and cannot be resolved
  without language-level support.
- The real dispatch logic differs fundamentally per type: string rules (regex, email),
  numeric rules (min/max/step), collection rules (length). There is no uniform
  implementation body — putting all three in one `match self` arm requires runtime
  type dispatch (`TypeId`), which is strictly worse than compile-time dispatch.
- `bool` and `char` cannot satisfy `SteppableValue` without lying about arithmetic;
  `HashMap` cannot satisfy `InputValue` at all.

**Verdict**: A single `Validatable` trait is not the right model. The multi-lane
design is correct and should be formalised as:

| Lane | Entry point | Covers |
|---|---|---|
| **Scalar lane** | `rule.validate(value)` (`Validate<T: Copy>`) | numerics, bool, char |
| **Ref lane** | `rule.validate_ref(&value)` (`ValidateRef<T>`) | `str`, sized collections (after P0), structs via `Custom` |
| **Slice lane** | `LengthValidator<[T]>` directly | `&[T]` — documented exception |

After P0 lands, `validate_ref` becomes the single public entry point for all
non-`Copy` types. The slice lane is a documented boundary, not a gap.

---

### 5. Verdict

**The current multi-path design is the correct long-term architecture.**
Converging on a single `ValidateRef<T: ?Sized>` is blocked by the `T: Sized`
constraint baked into `Rule<T>`. No stable Rust feature removes this constraint.

The four additions above (Gaps 1–4, Priority P0–P2) are all **purely additive**,
touch at most three files, and require no breaking changes. After they land:

- Every `Sized` type with a `WithLength` impl is covered by `ValidateRef<T> for Rule<T>`.
- `bool` and `char` get proper `Validate<T>` impls.
- `IndexMap`/`IndexSet` are covered behind a feature flag.
- `&[T]` is covered by `LengthValidator<[T]>` with clear documentation.
- Structs are covered by `Rule::Custom` today and `SchemaValidator` tomorrow.

