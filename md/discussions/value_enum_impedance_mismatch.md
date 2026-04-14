# Analysis: Value Enum Impedance Mismatch with Typed Serde Structs

**Date:** April 8, 2026
**Issue:** [#90](https://github.com/elycruz/walrs/issues/90) — Assess `Value` enum impedance mismatch with typed serde structs
**Related:** [#88](https://github.com/elycruz/walrs/issues/88) (Fieldset trait), [#87](https://github.com/elycruz/walrs/issues/87) (Filter apply_ref), [#85](https://github.com/elycruz/walrs/issues/85) (Filter enum move)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Architecture](#current-architecture)
3. [Analysis Question 1: Current Usage Impact](#1-current-usage-impact)
4. [Analysis Question 2: Where Value Is Necessary](#2-where-value-enum-is-necessary)
5. [Analysis Question 3: Where Value Is Problematic](#3-where-value-enum-is-problematic)
6. [Analysis Question 4: Refactor Assessment](#4-refactor-assessment)
7. [Recommendation](#recommendation)

---

## Executive Summary

The `Value` enum serves a genuine and necessary role as walrs's dynamic typing
layer for config-driven forms, WASM boundaries, and JSON-schema validation.
However, it introduces measurable ergonomic friction, runtime overhead, and
correctness risks when used to validate data that is **already statically typed**
(e.g., `serde::Deserialize` structs in axum/actix handlers).

**Recommendation: Keep dual paths** — retain `Value`-based validation for dynamic
use cases and introduce the `Fieldset` trait (as designed in
[#88](https://github.com/elycruz/walrs/issues/88)) for compile-time-known structs.
This is the lowest-risk, highest-value approach. See [§ Recommendation](#recommendation)
for details.

---

## Current Architecture

### Value Enum Definition

```
File: crates/validation/src/value.rs (610 lines)
```

```rust
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
    Object(HashMap<String, Value>),
}
```

### Where Value is Used

| Component | File | Role of `Value` |
|-----------|------|-----------------|
| `Field<Value>` | `fieldfilter/src/field.rs:170` | Per-field validation/filtering config |
| `FieldFilter` | `fieldfilter/src/field_filter.rs:51` | Multi-field validation: `HashMap<String, Field<Value>>` |
| `Filter<Value>` | `fieldfilter/src/filter_enum.rs:155` | Value transformation (trim, clamp, etc.) |
| `Rule<Value>` | `validation/src/rule_impls/value.rs:46` | Dynamic rule dispatch (526 lines) |
| `CrossFieldRuleType` | `fieldfilter/src/field_filter.rs:197` | Cross-field conditions use `Condition<Value>` |
| `FormData` | `form/src/form_data.rs:8` | `HashMap<String, Value>` newtype |
| `Form` | `form/src/form.rs:53` | Delegates validation to `FieldFilter` |

### Three Validation Paths

The rule system currently has three distinct implementation paths:

| Path | Constraint | Location |
|------|-----------|----------|
| `impl ValidateRef<str> for Rule<String>` | String rules | `rule_impls/string.rs` |
| `impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for Rule<T>` | Numeric (`Copy`) types | `rule_impls/scalar.rs` |
| `impl Rule<Value> { fn validate_value() }` | Dynamic dispatch | `rule_impls/value.rs` |

---

## 1. Current Usage Impact

### 1.1 Allocation Cost of the Value Roundtrip

Consider a typical 7-field user registration form:

```rust
#[derive(Deserialize)]
struct CreateUser {
    username: String,        // String → Value::Str (0 alloc: moves ownership)
    email: String,           // String → Value::Str (0 alloc: moves ownership)
    password: String,        // String → Value::Str (0 alloc: moves ownership)
    confirm_password: String,// String → Value::Str (0 alloc: moves ownership)
    age: u32,                // u32 → Value::U64 (0 alloc: Copy → enum variant)
    bio: String,             // String → Value::Str (0 alloc: moves ownership)
    agree_tos: bool,         // bool → Value::Bool (0 alloc: Copy → enum variant)
}
```

**Struct → `HashMap<String, Value>` conversion:**

| Cost Category | Count | Notes |
|---------------|-------|-------|
| `HashMap` creation | 1 | Allocates bucket array (~7 entries → 1 allocation) |
| String key allocations | 7 | Each field name is `"username".to_string()`, etc. |
| `Value::Str` wrapping | 5 | Moves ownership (0 extra string alloc), but wraps in enum |
| `Value::U64`/`Value::Bool` wrapping | 2 | Zero-cost: primitives copied into enum variant |
| **Total allocations** | **8** | 1 HashMap + 7 key strings |

**After validation — extracting results back:**

| Cost Category | Count | Notes |
|---------------|-------|-------|
| `value.as_str().unwrap().to_string()` | 5 | Clones string out of `Value::Str` |
| `value.as_u64().unwrap() as u32` | 1 | Zero-cost extraction |
| `value.as_bool().unwrap()` | 1 | Zero-cost extraction |
| **Total extra allocs** | **5** | 5 string clones on extraction |

**Overall roundtrip: ~13 allocations** for a 7-field form, vs. **0 allocations** if
validating the struct directly. While these allocations are cheap in absolute terms
(sub-microsecond), they are **entirely avoidable** for typed structs.

### 1.2 Ergonomic Cost (Lines of Code)

**Current approach — validating a typed struct via FieldFilter:**

```rust
// Step 1: Define the struct (user already has this)
#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    age: u32,
}

// Step 2: Build a FieldFilter (14 lines)
let mut filter = FieldFilter::new();
filter
    .add_field("username", FieldBuilder::<Value>::default()
        .rule(Rule::Required.and(Rule::MinLength(3)))
        .filters(vec![Filter::Trim, Filter::Lowercase])
        .build().unwrap())
    .add_field("email", FieldBuilder::<Value>::default()
        .rule(Rule::Required.and(Rule::Email(Default::default())))
        .filters(vec![Filter::Trim, Filter::Lowercase])
        .build().unwrap())
    .add_field("age", FieldBuilder::<Value>::default()
        .rule(Rule::Required.and(Rule::Range {
            min: Value::U64(13),
            max: Value::U64(120),
        }))
        .build().unwrap());

// Step 3: Convert struct → HashMap<String, Value> (5 lines)
let mut data = HashMap::new();
data.insert("username".to_string(), Value::from(user.username));
data.insert("email".to_string(), Value::from(user.email));
data.insert("age".to_string(), Value::from(user.age as u64));

// Step 4: Validate (1 line)
let result = filter.validate(&data)?;

// Step 5: Extract filtered values back (optional, 5 lines)
let filtered = filter.clean(data)?;
let username = filtered.get("username").unwrap().as_str().unwrap().to_string();
let email = filtered.get("email").unwrap().as_str().unwrap().to_string();
let age = filtered.get("age").unwrap().as_u64().unwrap() as u32;
```

**Total: ~25 lines** of boilerplate conversion code (Steps 2–5).

**With proposed `Fieldset` trait (from #88 design):**

```rust
#[derive(Deserialize, Fieldset)]
struct CreateUser {
    #[validate(required, min_length = 3)]
    #[filter(trim, lowercase)]
    username: String,

    #[validate(required, email)]
    #[filter(trim, lowercase)]
    email: String,

    #[validate(required, range(min = 13, max = 120))]
    age: u32,
}

// Validate (1 line)
let validated = user.clean()?;
```

**Total: ~1 line** of validation code (annotations are co-located with the struct).

**Ergonomic savings: ~24 lines per form**, plus elimination of all string-key typo risks.

### 1.3 Correctness Risks from Type Erasure

The `Rule<Value>` implementation contains **18 `TypeMismatch` error paths** across
526 lines in `rule_impls/value.rs`. Each represents a runtime failure that could
have been caught at compile time with typed validation:

| Rule Variant | TypeMismatch Risk | Example |
|-------------|-------------------|---------|
| `MinLength(3)` | Applied to `Value::I64(42)` → runtime error | String rule on numeric field |
| `Min(Value::I64(10))` | Applied to `Value::Str("hello")` → `None` from `partial_cmp` | Numeric rule on string field |
| `Step(Value::I64(5))` | Applied to `Value::F64(10.0)` → mismatch (I64 vs F64) | Integer step on float value |
| `Pattern(regex)` | Applied to `Value::Bool(true)` → runtime error | Regex on non-string |
| `Email/Url/Ip/Date` | Applied to any non-`Value::Str` → runtime error | Format rules on wrong type |

**Cross-variant numeric mismatch is particularly subtle:**

```rust
// This SILENTLY fails — Rule stores I64, value is U64
let rule = Rule::<Value>::Min(Value::I64(10));
rule.validate_value(&Value::U64(5));
// → Err(TypeMismatch: "Incompatible types for Min.")
// Even though logically 5 < 10 should fail with RangeUnderflow, not TypeMismatch
```

The `PartialOrd` impl for `Value` returns `None` for cross-variant comparisons
(e.g., `I64` vs `U64`), making it easy to get unexpected `TypeMismatch` errors
instead of the intended validation error.

---

## 2. Where Value Enum Is Necessary

The `Value` enum is **essential** for these use cases:

### 2.1 Config-Driven / JSON-Schema Forms

When form definitions come from a database, YAML config, or JSON schema, field
types are not known at compile time:

```json
{
  "fields": {
    "email": { "rule": { "type": "all", "rules": [{"type": "required"}, {"type": "email"}] } },
    "age": { "rule": { "type": "range", "min": 13, "max": 120 } }
  }
}
```

This deserializes into `FieldFilter` with `Field<Value>` fields. The `Value` enum
is the only way to represent heterogeneous field data in this context.

### 2.2 WASM `web_sys::FormData` Bridge

Browser form data arrives as strings via `web_sys::FormData::get()`. The `Value`
enum serves as an intermediate representation before type coercion:

```rust
let form_data: web_sys::FormData = /* from browser */;
let mut data = HashMap::new();
for key in form_data.keys() {
    let val = form_data.get(&key).as_string().unwrap_or_default();
    data.insert(key, Value::Str(val));
}
field_filter.validate(&data)?;
```

### 2.3 Dynamic Form Generation

The `Form` struct in `walrs_form` generates HTML form elements dynamically.
`FormData` (`HashMap<String, Value>`) is the natural data model for binding values
to dynamically-created form elements:

```rust
let mut form = Form::new("registration");
form.bind_data(data);  // FormData → element values
```

### 2.4 Serialization of Validation Configs

`FieldFilter`, `Rule<Value>`, and `Filter<Value>` are all `Serialize + Deserialize`.
This enables storing validation configurations in databases, sending them over the
wire, or loading them from config files. The `Value` enum is the serialization-
compatible type parameter that makes this possible.

---

## 3. Where Value Enum Is Problematic

### 3.1 Typed API Endpoints

In typical Rust web frameworks, request data is already deserialized into typed structs:

```rust
// axum handler — data arrives pre-typed
async fn create_user(Json(user): Json<CreateUser>) -> impl IntoResponse {
    // 'user' is already a CreateUser with typed fields
    // Converting to HashMap<String, Value> LOSES type information
}
```

The `Value` conversion adds friction without adding value — the compiler already
knows the types.

### 3.2 Service-to-Service Validation

Internal services communicate with typed protobuf/gRPC or JSON structs. Validation
of these structs should operate directly on the concrete types, not require
conversion to `Value`.

### 3.3 Compile-Time Safety Loss

The core issue is **type erasure**: converting `u32` → `Value::U64` → `Rule<Value>`
means the compiler can no longer verify that a `MinLength` rule isn't accidentally
applied to a numeric field. The 18 `TypeMismatch` branches in `rule_impls/value.rs`
exist solely to handle errors that wouldn't be possible with typed `Rule<T>`.

### 3.4 Maintenance Burden

The `Rule<Value>::validate_value()` method (526 lines) duplicates logic from
`Rule<String>` and `Rule<T: SteppableValue>` with additional `match value { ... }`
dispatch. Each new rule variant requires updating **three** impl blocks:

1. `Rule<String>` (string rules)
2. `Rule<T: SteppableValue>` (numeric rules)
3. `Rule<Value>` (dynamic dispatch — must handle all combinations)

The `Filter<Value>` impl (70 lines) similarly duplicates `Filter<String>` logic
with `if let Value::Str(s) = value` guards.

---

## 4. Refactor Assessment

### 4.1 Can the Fieldset Trait Fully Replace Value-Based Validation for Typed Structs?

**Yes.** The `Fieldset` trait design from [#88](https://github.com/elycruz/walrs/issues/88)
can fully replace `Value`-based validation for compile-time-known structs:

- **Per-field validation:** Each field gets a `Rule<T>` where `T` matches the field's
  concrete type. `Rule::<String>::MinLength(3)` on a `String` field is checked at
  compile time. Attempting `Rule::<String>::Min(5)` would be a compile error.

- **Per-field filtering:** Each field gets a `Filter<T>` matched to its type.
  `Filter::<String>::Trim` on a `String` field is type-safe.

- **Cross-field validation:** Typed closures over `&Self` replace `HashMap::get`
  with string keys. Field access is checked by the compiler.

- **Process pipeline:** `filter(self) → validate(&self)` operates on the struct
  directly, with zero `Value` conversions.

### 4.2 Should Value Remain as a Parallel Dynamic Path?

**Yes.** The dynamic `Value` path serves use cases where types are genuinely unknown
at compile time (§2). Removing it would break:

- JSON-schema / config-driven form validation
- WASM `web_sys::FormData` integration
- Dynamic form generation via `Form` + `FormData`
- Serializable validation config storage

### 4.3 Migration Cost for Existing Code

| Component | Migration Effort | Notes |
|-----------|-----------------|-------|
| `Field<Value>` | None (kept) | Dynamic path preserved |
| `FieldFilter` | None (kept) | Dynamic path preserved |
| `Rule<Value>` impls | None (kept) | Dynamic path preserved |
| `Form` + `FormData` | None (kept) | Dynamic path preserved |
| New `Fieldset` trait | New code | Additive — does not change existing APIs |
| Derive macro crate | New code | `walrs_fieldset_derive` (new crate) |
| User migration | Opt-in | Users can adopt `Fieldset` incrementally |

**Migration cost is zero for existing code** — the `Fieldset` trait is purely
additive. Users choose between the dynamic path (`FieldFilter` + `Value`) and the
typed path (`Fieldset`) based on their use case.

### 4.4 Does the Value Enum Add Maintenance Burden?

**Yes, but it is manageable.**

Current duplication:
- `rule_impls/value.rs`: 526 lines (300 lines are tests, ~230 are impl)
- `filter_enum.rs` `Filter<Value>` impl: ~70 lines
- 18 `TypeMismatch` error branches in rule dispatch

The `Rule<Value>` impl delegates to `Rule<String>` methods where possible (e.g.,
`Rule::<String>::Email(opts).validate_str(s)` for string-type rules applied to
`Value::Str`), which reduces some duplication. However, adding a new rule variant
still requires updating the `Value` dispatch block.

With the `Fieldset` trait in place, the `Rule<Value>` path becomes a **stable
maintenance surface** — it only needs updates when new `Rule` variants are added,
and the typed path handles the common case.

---

## Recommendation

### **Keep dual paths** — `Value`-based for dynamic, `Fieldset` for typed

This is the recommended approach, aligned with the design in
[#88](https://github.com/elycruz/walrs/issues/88).

#### Rationale

1. **Zero migration cost:** Existing code using `Field<Value>` / `FieldFilter` /
   `FormData` continues to work unchanged.

2. **Compile-time safety for the common case:** The vast majority of Rust web
   applications use typed structs. The `Fieldset` trait eliminates all 18
   `TypeMismatch` runtime error paths for these users.

3. **Dynamic use cases preserved:** Config-driven forms, WASM bridges, and
   serializable validation configs continue to use the `Value` path.

4. **Incremental adoption:** Users can mix both paths — e.g., use `Fieldset`
   for API handlers and `FieldFilter` for admin panel forms loaded from config.

5. **Reduced maintenance over time:** As the typed path becomes the primary
   validation path, the `Value` impl stabilizes as a legacy/dynamic-only concern.

#### Implementation Priority

1. **Implement `Fieldset` trait** in `walrs_fieldfilter` (manual impl first).
2. **Create `walrs_fieldset_derive`** proc-macro crate for `#[derive(Fieldset)]`.
3. **Resolve #87** (Filter `apply_ref` pattern) — enables efficient `&str`-based
   filtering in the `Fieldset` impl.
4. **Resolve #85** (Filter enum move) — structural cleanup, not blocking.
5. **Document migration guide** showing both paths side-by-side.

#### Why Not "Phase Out Value"

Phasing out `Value` would break the dynamic form system (`Form`, `FormData`,
`FieldFilter`) which is a core part of walrs's functionality. The HTML form
generation and WASM bridge use cases genuinely require runtime-typed values.

#### Why Not "Refactor Value"

Adding `TryFrom<T>` impls or making `Value` generic would add complexity without
addressing the fundamental issue: `Value` erases type information that typed structs
already have. The `Fieldset` trait is a cleaner solution because it bypasses
`Value` entirely for typed use cases, rather than trying to make `Value` work better
with types it shouldn't need to interact with.

---

## Appendix: Quantitative Summary

| Metric | `Value` Path | `Fieldset` Path | Savings |
|--------|-------------|-------------------|---------|
| Allocations per 7-field form | ~13 | 0 | 13 allocations |
| Boilerplate lines per form | ~25 | ~1 | 24 lines |
| Runtime type-mismatch risk | 18 error paths | 0 (compile-time) | 18 error paths eliminated |
| `rule_impls/value.rs` maintenance | 526 lines | N/A (separate impl) | Stable maintenance surface |
| New rule variant update sites | 3 impls | 2 impls (no Value dispatch) | 1 fewer update site |
