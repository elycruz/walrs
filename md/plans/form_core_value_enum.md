# Plan: Introduce a Custom `form_core::Value` Enum

**Date:** February 19, 2026  
**Status:** Planned  
**Crate:** `walrs_form_core`

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

### Can you implement `Rule<serde_json::Value>`?

**Partially yes, but with serious gaps.**

- `serde_json::Value` **does** satisfy `Clone + PartialEq + Debug + Serialize + Deserialize`,
  so you can construct and store `Rule<serde_json::Value>` variants like `Required`, `All`,
  `Custom`, `Equals(value)`, etc.
- `serde_json::Value` does **not** implement `SteppableValue`, `NumberValue`, `ScalarValue`,
  `InputValue`, or `IsEmpty` (the validator's own traits). This means the entire
  `impl<T: SteppableValue + IsEmpty> Rule<T>` block — i.e., `Rule::validate()` for numeric
  rules like `Min`, `Max`, `Range`, `Step` — **is inaccessible** for `Rule<serde_json::Value>`.
- The existing `Field<Value>::validate()` in `inputfilter/src/field.rs` already shows this
  pain: it only handles `Rule::Required` by hand and leaves a `TODO` comment for everything
  else.
- You could add `impl Rule<serde_json::Value>` manually with a big `match` that inspects
  `value.as_f64()`, `value.as_i64()`, etc. — but you would be reimplementing the whole
  validation dispatch outside the existing trait machinery.

### Will you know when you're validating a float?

**No — not cleanly.** `serde_json::Value::Number` wraps a single opaque `serde_json::Number`
type. There is no `Value::Float(f64)` or `Value::Integer(i64)` variant — just
`Value::Number(n)`, where you call `.as_f64()`, `.as_i64()`, or `.as_u64()` at runtime.
Concretely:

- `Rule::<serde_json::Value>::Min(Value::Number(...))` gives you a `serde_json::Number` as
  the bound, and to compare it against an incoming `serde_json::Value` you must call
  `.as_f64()` on both sides — with silent `None` failures if the number doesn't fit.
- There is no compile-time type tag telling a rule "this field holds floats." You have to
  encode that in your own dispatch logic or annotations.
- Float-specific rules (e.g., step precision with `f64::EPSILON`) silently degrade to a
  best-effort `as_f64()` cast with no type safety.

### Flexibility comparison

| Concern | `serde_json::Value` | Custom `form_core::Value` |
|---|---|---|
| `Rule<T>` numeric validation (`SteppableValue` bound) | ❌ Blocked — must write manual dispatch | ✅ Can impl all required traits |
| `IsEmpty` for `Condition::IsEmpty` | ❌ Not implemented on it | ✅ Trivial to add |
| Know at compile time if value is float/int | ❌ All numbers are `Number(n)` | ✅ `Value::F64(f64)` vs `Value::I64(i64)` |
| Bridge to/from JSON | ✅ Is JSON | ✅ `From`/`Into` impls, trivial |
| Orphan rule — can't add external trait impls | ❌ Can't impl `SteppableValue for serde_json::Value` in a third crate | ✅ You own `form_core::Value` |
| WASM / feature-gating | ✅ Already in tree | ✅ Keep `serde_json` as an optional bridge feature |

### The decisive constraint: the orphan rule

`SteppableValue` is defined in `walrs_validator`, and `serde_json::Value` is from an external
crate. You **cannot** implement `walrs_validator::SteppableValue for serde_json::Value` in
`form_core` or `inputfilter` — Rust's coherence rules forbid it. This permanently walls off
the entire numeric validation `impl` block for `Rule<serde_json::Value>`.

---

## Decision

Introduce a **custom `form_core::Value` enum** to replace the `serde_json::Value` re-export.
This resolves the orphan rule, gives distinct numeric variants (so you know at compile time
whether you're validating a float), and bridges to/from `serde_json::Value` via `From` impls.

---

## Enum Definition (Proposed)

```rust
/// Native form value type.
///
/// Distinct numeric variants (`I64`, `U64`, `F64`) allow compile-time type
/// discrimination for validation rules, unlike `serde_json::Value::Number`.
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

Replace `pub use serde_json::Value` with the native enum definition above.
Add ergonomic `From` impls:

```rust
impl From<&str>  for Value { ... }  // → Value::Str
impl From<String> for Value { ... } // → Value::Str
impl From<bool>  for Value { ... }  // → Value::Bool
impl From<i64>   for Value { ... }  // → Value::I64
impl From<u64>   for Value { ... }  // → Value::U64
impl From<f64>   for Value { ... }  // → Value::F64
// ... other numeric primitive widening conversions (i32 → I64, u32 → U64, f32 → F64, etc.)
```

### Step 2 — Add `serde_json` bridge impls (feature-gated)

In `form_core/Cargo.toml`, make `serde_json` an optional dependency:

```toml
[features]
default = ["serde_json_bridge"]
serde_json_bridge = ["serde_json"]

[dependencies]
serde_json = { version = "1.0", optional = true }
```

In `form_core/src/value.rs`, behind `#[cfg(feature = "serde_json_bridge")]`:

```rust
impl From<serde_json::Value> for Value { ... }
impl From<Value> for serde_json::Value { ... }
```

> **Note:** `TryFrom` may also be appropriate for lossy conversions (e.g., a `serde_json`
> integer that doesn't fit in `i64`).

### Step 3 — Implement `walrs_validator` traits for `form_core::Value` in `walrs_inputfilter`

Because `walrs_inputfilter` depends on both crates, it is the correct place to add these
impls without violating the orphan rule:

```rust
// inputfilter/src/value_impls.rs  (new file)

use walrs_form_core::Value;
use walrs_validator::{IsEmpty, InputValue, ScalarValue, NumberValue, SteppableValue};

impl IsEmpty for Value { ... }       // Required for Condition::IsEmpty / Rule::Required
impl InputValue for Value {}         // Marker trait
impl ScalarValue for Value {}        // Marker trait
impl SteppableValue for Value { ... } // rem_check dispatches on I64/U64/F64 variants
```

> The numeric trait impls will dispatch internally on the enum variant.  Operations that
> don't make sense for non-numeric variants (e.g., `Step` on `Value::Str`) should return
> `false` / an appropriate violation rather than panic.

### Step 4 — Update `Field<Value>::validate()` in `inputfilter/src/field.rs`

Remove the manual `Rule::Required` stub and the `TODO` comment.  Delegate directly to the
now-accessible `rule.validate(value.clone(), locale)` from the `SteppableValue` impl:

```rust
impl Field<Value> {
    pub fn validate(&self, value: &Value) -> Result<(), Violations> {
        match &self.rule {
            Some(rule) => rule.validate(value.clone(), self.locale.as_deref())
                .map_err(|v| { let mut vs = Violations::empty(); vs.push(v); vs }),
            None => Ok(()),
        }
    }
}
```

### Step 5 — Update exports and run tests

- Update `form_core/src/lib.rs` to export the new `Value` enum (already exporting `value::Value`; no change needed unless the re-export path changes).
- Fix any downstream call sites in `walrs_form`, `walrs_inputfilter`, etc. that relied on `serde_json::Value`-specific APIs (e.g., `Value::Number(...)` construction, `.as_i64()` on the JSON type).
- Run `cargo test --workspace` and fix failures.

---

## Further Considerations

1. **`SteppableValue` for `form_core::Value`** — Since `Value` is not `Copy`, the numeric
   trait impls need special care.  `InputValue` requires `Copy`; consider either:
   - Making `Value` `Copy` (impractical — `String` and `Vec` are not `Copy`), or
   - Introducing a separate `NumericValue` newtype / narrower enum (`I64(i64)`, `U64(u64)`,
     `F64(f64)`) for use with numeric `Rule<T>` variants, while keeping `Value` for
     `Field<Value>` storage, or
   - Adjusting `InputValue` to remove the `Copy` bound (wider change, but consistent with
     the direction the codebase is heading).

2. **Integer vs. float split** — `I64` + `U64` + `F64` variants give lossless round-tripping
   with JSON and preserve type intent.  A single `Number(f64)` would be simpler but loses
   large `u64` precision.  **Recommended:** keep `I64` / `U64` / `F64` separate.

3. **Orphan rule placement** — Impls of `walrs_validator` traits for `form_core::Value` must
   live in `walrs_inputfilter` (the crate that depends on both sides). This is already the
   natural home of `Field<Value>`.

4. **`ValueExt::is_empty_value()`** — Update the existing `ValueExt` impl to match on the
   new variants (`Null`, `Str(s) if s.is_empty()`, `Array(a) if a.is_empty()`, etc.).

5. **`serde_json` as a mandatory vs. optional dep** — Keeping it optional (`serde_json_bridge`
   feature) reduces the WASM binary footprint for consumers that only need native value
   handling without JSON I/O.

