# Design: `Filterable` Trait & `walrs_inputfilter_derive`

**Date:** April 12, 2026
**Updated:** April 12, 2026
**Status:** Design
**Crates affected:** `walrs_validation` (new `FieldViolations` type), `walrs_inputfilter` (trait + migration), `walrs_inputfilter_derive` (new proc-macro crate)

---

## Table of Contents

1. [Goal](#goal)
2. [Background](#background)
3. [Unified Violations Type: `FieldViolations`](#unified-violations-type-fieldviolations)
   - [3.1 Definition](#31-definition)
   - [3.2 Cross-Field / Form-Level Violations](#32-cross-field--form-level-violations)
   - [3.3 Migration from `FormViolations`](#33-migration-from-formviolations)
4. [The `Filterable` Trait](#the-filterable-trait)
5. [Derive Macro: `#[derive(Filterable)]`](#derive-macro-derivefilterable)
   - [5.1 Validation Annotations](#51-validation-annotations)
   - [5.2 Filter Annotations (Infallible + Fallible)](#52-filter-annotations-infallible--fallible)
   - [5.3 Cross-Field Validation](#53-cross-field-validation)
   - [5.4 Nested Structs](#54-nested-structs)
   - [5.5 `Option<T>` Fields](#55-optiont-fields)
   - [5.6 `Into<FormData>` Bridge (opt-in)](#56-intoformdata-bridge-opt-in)
6. [Generated Code Examples](#generated-code-examples)
   - [6.1 Simple Struct](#61-simple-struct)
   - [6.2 Nested Struct with Cross-Field Validation](#62-nested-struct-with-cross-field-validation)
   - [6.3 Manual Implementation](#63-manual-implementation)
7. [Type Mapping](#type-mapping)
8. [Crate Layout](#crate-layout)
9. [Backward Compatibility](#backward-compatibility)
10. [Open Questions](#open-questions)
11. [Out of Scope](#out-of-scope)

---

## Goal

Provide a `Filterable` trait and companion `#[derive(Filterable)]` proc-macro so
that users can attach **compile-time type-checked** validation rules and filters
directly to struct fields, replacing runtime string-keyed `FieldFilter` +
`HashMap<String, Value>` for statically known structs.

Additionally, introduce a **unified violations type** (`FieldViolations`) that
can be used consistently across the entire forms/validations/filters ecosystem.

---

## Background

The current codebase provides two validation paths:

| Component | Path | Types |
|---|---|---|
| `Field<T>` | Single-field validation | `Rule<T>`, `FilterOp<T>`, `TryFilterOp<T>` |
| `FieldFilter` | Multi-field dynamic validation | `Field<Value>`, `IndexMap<String, Field<Value>>` |

Both are fully implemented and work with dynamic `Value` types. However, when
validating a known Rust struct, field names are runtime strings and type
mismatches (e.g., `Rule::<Value>::MinLength(3)` on a numeric Value) are only
caught at runtime.

The `Filterable` trait provides the **typed alternative**: field names are
compile-time string literals, rules are constructed with the correct `Rule<T>`
for each field's concrete type, and mismatches cause **compiler errors**.

---

## Unified Violations Type: `FieldViolations`

### 3.1 Definition

A new type defined in `walrs_validation` that replaces `FormViolations` as the
**single** key-value violations container across the ecosystem:

```rust
use indexmap::IndexMap;
use crate::Violations;

/// A key-value map of field names to their validation violations.
///
/// Used consistently across `walrs_validation`, `walrs_inputfilter`, and
/// `walrs_form` to represent multi-field validation errors.
///
/// - Keys are field names (e.g., `"email"`, `"address.street"`).
/// - Values are `Violations` (a vec of `Violation` instances).
/// - Cross-field / form-level violations use the key `""` (empty string).
///
/// # Example
///
/// ```rust
/// use walrs_validation::{FieldViolations, Violation, ViolationType};
///
/// let mut fv = FieldViolations::new();
/// fv.add("email", Violation::invalid_email());
/// fv.add("", Violation::new(ViolationType::NotEqual, "Passwords must match"));
///
/// assert!(!fv.is_empty());
/// assert!(fv.get("email").is_some());
/// assert!(fv.form_violations().is_some());
/// ```
#[derive(Clone, Debug, Default)]
pub struct FieldViolations(pub IndexMap<String, Violations>);

impl FieldViolations {
    /// Creates a new empty `FieldViolations`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if there are no violations for any key.
    pub fn is_empty(&self) -> bool {
        self.0.values().all(|v| v.is_empty())
    }

    /// Returns the total number of violations across all keys.
    pub fn len(&self) -> usize {
        self.0.values().map(|v| v.len()).sum()
    }

    /// Gets violations for a specific field.
    pub fn get(&self, field: &str) -> Option<&Violations> {
        self.0.get(field)
    }

    /// Gets mutable violations for a specific field.
    pub fn get_mut(&mut self, field: &str) -> Option<&mut Violations> {
        self.0.get_mut(field)
    }

    /// Adds a single violation for a field.
    pub fn add(&mut self, field: impl Into<String>, violation: crate::Violation) -> &mut Self {
        self.0
            .entry(field.into())
            .or_insert_with(Violations::empty)
            .push(violation);
        self
    }

    /// Adds multiple violations for a field.
    pub fn add_many(
        &mut self,
        field: impl Into<String>,
        violations: Violations,
    ) -> &mut Self {
        self.0
            .entry(field.into())
            .or_insert_with(Violations::empty)
            .extend(violations);
        self
    }

    /// Gets cross-field / form-level violations (key = "").
    pub fn form_violations(&self) -> Option<&Violations> {
        self.get("")
    }

    /// Adds a cross-field / form-level violation (key = "").
    pub fn add_form_violation(&mut self, violation: crate::Violation) -> &mut Self {
        self.add("", violation)
    }

    /// Returns an iterator over field names (keys) that have violations.
    pub fn fields(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// Returns an iterator over (field_name, violations) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Violations)> {
        self.0.iter()
    }

    /// Merges another `FieldViolations` into this one.
    pub fn merge(&mut self, other: FieldViolations) -> &mut Self {
        for (field, violations) in other.0 {
            self.add_many(field, violations);
        }
        self
    }

    /// Merges another `FieldViolations` with a key prefix.
    ///
    /// Useful for nested struct violations: `merge_prefixed("address", nested)`
    /// turns key `"street"` into `"address.street"`.
    pub fn merge_prefixed(
        &mut self,
        prefix: &str,
        other: FieldViolations,
    ) -> &mut Self {
        for (field, violations) in other.0 {
            let prefixed = if field.is_empty() {
                prefix.to_string()
            } else {
                format!("{}.{}", prefix, field)
            };
            self.add_many(prefixed, violations);
        }
        self
    }

    /// Clears all violations.
    pub fn clear(&mut self) -> &mut Self {
        self.0.clear();
        self
    }
}

impl std::fmt::Display for FieldViolations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (field, violations) in &self.0 {
            if !violations.is_empty() {
                let key = if field.is_empty() { "(form)" } else { field.as_str() };
                write!(f, "{}: {}\n", key, violations)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for FieldViolations {}

impl From<FieldViolations> for Result<(), FieldViolations> {
    fn from(fv: FieldViolations) -> Self {
        if fv.is_empty() { Ok(()) } else { Err(fv) }
    }
}
```

### 3.2 Cross-Field / Form-Level Violations

Cross-field violations (e.g., "passwords must match") use the **empty string
key** (`""`). The `form_violations()` and `add_form_violation()` methods
provide convenient access to this key.

This keeps the type as a simple flat key-value map while still supporting
both per-field and form-level violations through a single structure.

### 3.3 Migration from `FormViolations`

The existing `FormViolations` in `walrs_inputfilter` will be deprecated in
favor of `FieldViolations` from `walrs_validation`. Migration path:

1. Add `FieldViolations` to `walrs_validation`.
2. Add a `type FormViolations = FieldViolations;` alias in `walrs_inputfilter`
   for backward compatibility (marked `#[deprecated]`).
3. Update `FieldFilter` internals to use `FieldViolations`.
4. New code (`Filterable`, derive macros) uses `FieldViolations` exclusively.

The `FieldViolations::merge_prefixed()` method replaces the nested violation
key-prefixing that `FormViolations` relied on ad-hoc logic for.

---

## The `Filterable` Trait

Defined in `walrs_inputfilter::filterable`:

```rust
use walrs_validation::FieldViolations;

/// Trait for structs that support type-safe field validation and filtering.
///
/// Implementors can either derive this trait via `#[derive(Filterable)]` or
/// implement it manually for full control over validation and filtering logic.
pub trait Filterable: Sized {
    /// Validate all fields and cross-field rules.
    ///
    /// Returns `Ok(())` when all fields pass validation, or
    /// `Err(FieldViolations)` with per-field and form-level violations.
    fn validate(&self) -> Result<(), FieldViolations>;

    /// Apply all per-field filters (infallible and fallible).
    ///
    /// Returns `Ok(filtered)` on success, or `Err(FieldViolations)` if any
    /// fallible filter fails. Takes ownership to avoid `mem::take` / `Default`
    /// requirements on non-`Default` fields.
    fn filter(self) -> Result<Self, FieldViolations>;

    /// Filter then validate (provided default).
    ///
    /// Applies `filter()` first, then `validate()` on the result.
    /// Returns `Ok(filtered)` if both succeed, or `Err(FieldViolations)`.
    fn process(self) -> Result<Self, FieldViolations> {
        let filtered = self.filter()?;
        filtered.validate()?;
        Ok(filtered)
    }
}
```

### Design rationale

- **`validate(&self)`** — borrows `self`, runs `Rule<T>::validate_ref()` for
  strings/refs, `Rule<T>::validate()` for `Copy` types. Collects errors into
  `FieldViolations` keyed by field name string literals.

- **`filter(self) -> Result<Self, FieldViolations>`** — takes ownership. Runs
  **all** filters (both infallible `FilterOp` and fallible `TryFilterOp`) in a
  single pass. Returns `Result` so that fallible filter failures propagate
  naturally without requiring a separate `try_filter()` method or special
  handling in `process()`. Infallible filters (e.g., `Trim`) are simply wrapped
  in `Ok(...)` internally.

- **`process(self)`** — provided default: `filter()? → validate()?`. Mirrors
  the existing `Field<T>::process` semantics but simplified since `filter()`
  already handles fallible operations.

---

## Derive Macro: `#[derive(Filterable)]`

A new proc-macro crate `walrs_inputfilter_derive` provides `#[derive(Filterable)]`.

### 5.1 Validation Annotations

Placed on struct fields via `#[validate(...)]`:

| Annotation | Maps to | Applicable types |
|---|---|---|
| `required` | `Rule::Required` | All |
| `min_length = N` | `Rule::MinLength(N)` | `String`, `Vec<T>`, collections |
| `max_length = N` | `Rule::MaxLength(N)` | `String`, `Vec<T>`, collections |
| `exact_length = N` | `Rule::ExactLength(N)` | `String`, `Vec<T>`, collections |
| `email` | `Rule::Email(Default::default())` | `String` |
| `url` | `Rule::Url(Default::default())` | `String` |
| `uri` | `Rule::Uri(Default::default())` | `String` |
| `ip` | `Rule::Ip(Default::default())` | `String` |
| `hostname` | `Rule::Hostname(Default::default())` | `String` |
| `pattern = "regex"` | `Rule::Pattern(CompiledPattern::try_from("regex").unwrap())` | `String` |
| `date` | `Rule::Date(Default::default())` | `String` |
| `min = N` | `Rule::Min(N)` | Numeric scalars |
| `max = N` | `Rule::Max(N)` | Numeric scalars |
| `range(min = A, max = B)` | `Rule::Range { min: A, max: B }` | Numeric scalars |
| `step = N` | `Rule::Step(N)` | `SteppableValue` types |
| `one_of = [a, b, c]` | `Rule::OneOf(vec![a, b, c])` | Types with `PartialEq` |
| `custom = "path::to::fn"` | `Rule::Custom(Arc::new(path::to::fn))` | All |
| `nested` | Calls `field.validate()` (requires `Filterable`) | Nested structs |

Multiple `#[validate(...)]` attributes on the same field are combined via
`Rule::All(vec![...])`.

### 5.2 Filter Annotations (Infallible + Fallible)

All filters — infallible and fallible — are placed on struct fields via
`#[filter(...)]`:

#### Infallible filters

| Annotation | Maps to | Applicable types |
|---|---|---|
| `trim` | `FilterOp::Trim` | `String` |
| `lowercase` | `FilterOp::Lowercase` | `String` |
| `uppercase` | `FilterOp::Uppercase` | `String` |
| `strip_tags` | `FilterOp::StripTags` | `String` |
| `html_entities` | `FilterOp::HtmlEntities` | `String` |
| `slug` | `FilterOp::Slug { max_length: None }` | `String` |
| `slug(max_length = N)` | `FilterOp::Slug { max_length: Some(N) }` | `String` |
| `truncate(max_length = N)` | `FilterOp::Truncate { max_length: N }` | `String` |
| `replace(from = "x", to = "y")` | `FilterOp::Replace { from, to }` | `String` |
| `clamp(min = A, max = B)` | `FilterOp::Clamp { min: A, max: B }` | Numeric scalars |
| `custom = "path::to::fn"` | `FilterOp::Custom(Arc::new(path::to::fn))` | All |
| `nested` | Calls `field.filter()` (requires `Filterable`) | Nested structs |

#### Fallible filters

| Annotation | Maps to | Applicable types |
|---|---|---|
| `try_custom = "path::to::fn"` | `TryFilterOp::TryCustom(Arc::new(path::to::fn))` | All |

Since `filter()` returns `Result<Self, FieldViolations>`, both infallible and
fallible filters run in sequence within the same method. Infallible filters
always succeed; fallible filters may fail, in which case their error is
converted to a `Violation` and added to `FieldViolations` under the field's
name.

Multiple `#[filter(...)]` annotations on the same field produce a chained
sequence: infallible filters run first, then fallible filters.

### 5.3 Cross-Field Validation

#### MVP: `#[cross_validate(...)]` struct attribute

```rust
#[derive(Filterable)]
#[cross_validate(passwords_match)]
struct Registration {
    #[validate(required, min_length = 8)]
    password: String,

    #[validate(required)]
    confirm: String,
}

fn passwords_match(r: &Registration) -> walrs_validation::ValidatorResult {
    if r.password == r.confirm {
        Ok(())
    } else {
        Err(Violation::new(ViolationType::NotEqual, "Passwords must match"))
    }
}
```

Generated `validate()` calls each `#[cross_validate]` function **after**
per-field checks, passing `&self` as the argument. Any returned `Violation` is
added to `FieldViolations` under the `""` (empty string) key (form-level violations).

Multiple `#[cross_validate(...)]` attributes are supported — each function is
called in declaration order.

### 5.4 Nested Structs

When a field's type also implements `Filterable`, use `#[validate(nested)]`
and/or `#[filter(nested)]`:

```rust
#[derive(Filterable)]
struct UserRegistration {
    #[validate(required, min_length = 2)]
    #[filter(trim)]
    name: String,

    #[validate(nested)]
    #[filter(nested)]
    address: UserAddress,
}
```

Generated code:

- **`validate()`**: calls `self.address.validate()` and prefixes nested
  violations with the field name via `FieldViolations::merge_prefixed("address", ...)`.
- **`filter()`**: calls `self.address.filter()?` and propagates errors with
  prefix.

This maps naturally to `FormData`'s existing dot-notation path resolution.

### 5.5 `Option<T>` Fields

`Option<T>` fields receive special handling:

- If `required` is present: `None` produces a `ValueMissing` violation.
- If `required` is **not** present: `None` skips all other validations and
  filters.
- If `Some(value)`: validates/filters the inner value with the declared rules.

The derive macro detects `Option<T>` via syn type parsing and wraps the
generated code in `if let Some(ref inner) = self.field { ... }` guards.

For filtering, `Option<T>` fields map through the filter:

```rust
let field = self.field.map(|v| FilterOp::<String>::Trim.apply(v));
// For fallible filters on Option<T>:
let field = match self.field {
    Some(v) => Some(try_filter_op.try_apply(v).map_err(|e| ...)?),
    None => None,
};
```

### 5.6 `Into<FormData>` Bridge (opt-in)

Controlled via struct-level attributes:

```rust
#[derive(Filterable)]
#[filterable(into_form_data)]
struct UserAddress {
    street: String,
    zip: String,
}
```

- `into_form_data` generates `impl From<&UserAddress> for FormData`
- `try_from_form_data` generates `impl TryFrom<FormData> for UserAddress`

This enables bridging between the typed `Filterable` path and the dynamic
`FieldFilter` / `Form` path.

---

## Generated Code Examples

### 6.1 Simple Struct

```rust
use walrs_inputfilter::Filterable;
use walrs_filter::FilterOp;
use walrs_validation::{Rule, CompiledPattern};

#[derive(Filterable)]
struct UserAddress {
    #[validate(required, min_length = 3)]
    #[filter(trim)]
    street: String,

    #[validate(required, pattern = r"^\d{5}$")]
    #[filter(trim)]
    zip: String,
}
```

**Generated `Filterable` impl:**

```rust
impl Filterable for UserAddress {
    fn validate(&self) -> Result<(), walrs_validation::FieldViolations> {
        let mut violations = walrs_validation::FieldViolations::new();

        // street: required + min_length(3)
        {
            let rule = Rule::<String>::All(vec![
                Rule::Required,
                Rule::MinLength(3),
            ]);
            if let Err(violation) = rule.validate_ref(self.street.as_str()) {
                violations.add("street", violation);
            }
        }

        // zip: required + pattern
        {
            let rule = Rule::<String>::All(vec![
                Rule::Required,
                Rule::Pattern(
                    walrs_validation::CompiledPattern::try_from(r"^\d{5}$").unwrap()
                ),
            ]);
            if let Err(violation) = rule.validate_ref(self.zip.as_str()) {
                violations.add("zip", violation);
            }
        }

        if violations.is_empty() { Ok(()) } else { Err(violations) }
    }

    fn filter(self) -> Result<Self, walrs_validation::FieldViolations> {
        let street = FilterOp::<String>::Trim.apply(self.street);
        let zip = FilterOp::<String>::Trim.apply(self.zip);
        Ok(Self { street, zip })
    }
}
```

### 6.2 Nested Struct with Cross-Field Validation

```rust
use walrs_inputfilter::Filterable;
use walrs_validation::{Violation, ViolationType};

#[derive(Filterable)]
#[cross_validate(passwords_match)]
struct UserRegistration {
    #[validate(required, min_length = 2, max_length = 50)]
    #[filter(trim)]
    name: String,

    #[validate(required, email)]
    #[filter(trim, lowercase)]
    email: String,

    #[validate(required, min_length = 8)]
    password: String,

    #[validate(required)]
    confirm_password: String,

    #[validate(min = 0, max = 150)]
    age: i64,

    #[validate(nested)]
    #[filter(nested)]
    address: UserAddress,
}

fn passwords_match(r: &UserRegistration) -> walrs_validation::ValidatorResult {
    if r.password == r.confirm_password {
        Ok(())
    } else {
        Err(Violation::new(ViolationType::NotEqual, "Passwords must match"))
    }
}
```

**Generated `validate()` highlights:**

- String fields: `rule.validate_ref(self.field.as_str())`
- Numeric fields: `rule.validate(self.field)`
- Nested fields: `self.address.validate()` with `merge_prefixed("address", ...)`
- Cross-field: `passwords_match(self)` called after per-field checks, violations added under `""` key

**Generated `filter()` highlights:**

- String fields with `Chain`: fold over `FilterOp::Chain(vec![Trim, Lowercase])`
- Fields without `#[filter]`: passed through unchanged (`password: self.password`)
- Nested fields: `self.address.filter()?` with error prefix propagation
- Returns `Ok(Self { ... })` wrapping all filtered fields

### 6.3 Manual Implementation

Users can implement `Filterable` without the derive macro:

```rust
use walrs_inputfilter::Filterable;
use walrs_filter::FilterOp;
use walrs_validation::{FieldViolations, Rule, ValidateRef, Violation, ViolationType};

struct LoginForm {
    username: String,
    password: String,
}

impl Filterable for LoginForm {
    fn validate(&self) -> Result<(), FieldViolations> {
        let mut violations = FieldViolations::new();

        let username_rule = Rule::<String>::Required.and(Rule::MinLength(3));
        if let Err(v) = username_rule.validate_ref(self.username.as_str()) {
            violations.add("username", v);
        }

        let password_rule = Rule::<String>::Required.and(Rule::MinLength(8));
        if let Err(v) = password_rule.validate_ref(self.password.as_str()) {
            violations.add("password", v);
        }

        if violations.is_empty() { Ok(()) } else { Err(violations) }
    }

    fn filter(self) -> Result<Self, FieldViolations> {
        let username = FilterOp::<String>::Trim.apply(self.username);
        Ok(Self { username, password: self.password })
    }
}
```

---

## Type Mapping

The derive macro maps field types to the correct rule type and validation
method:

| Field type | Rule type | Validation call |
|---|---|---|
| `String` | `Rule<String>` | `rule.validate_ref(self.field.as_str())` |
| `i8`..`i128`, `isize` | `Rule<{int}>` | `rule.validate(self.field)` |
| `u8`..`u128`, `usize` | `Rule<{uint}>` | `rule.validate(self.field)` |
| `f32`, `f64` | `Rule<{float}>` | `rule.validate(self.field)` |
| `T: Filterable` | N/A (nested) | `self.field.validate()` |
| `Option<T>` | Conditional | Skip if `None` unless `required` |
| `Vec<T>` | `Rule<Vec<T>>` | `rule.validate_ref(&self.field)` |

For filtering (all return `Result`):

| Field type | Filter call |
|---|---|
| `String` | `Ok(FilterOp::<String>::apply(self.field))` |
| Numeric (`Copy`) | `Ok(FilterOp::<T>::apply(self.field))` |
| `T: Filterable` (nested) | `self.field.filter()?` |
| `Option<T>` | `self.field.map(\|v\| filter.apply(v))` wrapped in `Ok(...)` |
| Fallible (`TryFilterOp`) | `try_filter.try_apply(self.field).map_err(\|e\| ...)` |

---

## Crate Layout

```
crates/
├── inputfilter/
│   ├── Cargo.toml           # adds `walrs_inputfilter_derive` as optional dep
│   └── src/
│       ├── lib.rs            # re-exports `Filterable` trait; conditionally
│       │                     #   re-exports derive macro
│       ├── filterable.rs     # `Filterable` trait definition (NEW)
│       └── ...               # existing modules unchanged
├── inputfilter_derive/       # (NEW)
│   ├── Cargo.toml            # proc-macro = true
│   └── src/
│       ├── lib.rs            # #[proc_macro_derive(Filterable, attributes(validate, filter, cross_validate, filterable))]
│       ├── parse.rs          # Attribute parsing (syn-based)
│       ├── gen_validate.rs   # Code generation for validate()
│       ├── gen_filter.rs     # Code generation for filter() (infallible + fallible)
│       └── gen_form_data.rs  # Code generation for Into<FormData> / TryFrom<FormData>
```

### Cargo feature gate

```toml
# crates/inputfilter/Cargo.toml
[features]
default = []
derive = ["walrs_inputfilter_derive"]

[dependencies]
walrs_inputfilter_derive = { path = "../inputfilter_derive", optional = true }
```

Users opt in with:

```toml
walrs_inputfilter = { version = "...", features = ["derive"] }
```

### `walrs_inputfilter_derive/Cargo.toml`

```toml
[package]
name = "walrs_inputfilter_derive"
version = "0.1.0"
edition = "2024"
authors = ["Ely De La Cruz <elycruz@elycruz.com>"]
description = "Derive macro for walrs_inputfilter Filterable trait"
license = "Elastic-2.0"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full", "extra-traits"] }
quote = "1"
proc-macro2 = "1"
```

---

## Backward Compatibility

The `HashMap<String, Value>` / `FieldFilter` path is **preserved unchanged**:

- Config-driven / JSON-deserialized forms → `FieldFilter`
- WASM boundaries (`web_sys::FormData`) → `FormData` + `FieldFilter`
- Dynamic form generation → `FieldFilter`

The struct-based `Filterable` path **supplements** the existing dynamic path.
No existing public APIs are changed or removed.

The existing `FormViolations` type in `walrs_inputfilter` is deprecated and
replaced by a type alias to `FieldViolations` from `walrs_validation`. Existing
code using `FormViolations` continues to compile with a deprecation warning.

---

## Open Questions

1. **`break_on_failure` semantics** — The typed path currently validates all
   fields and collects all violations. Should we add a
   `#[filterable(break_on_failure)]` struct-level attribute for short-circuit
   validation?

2. **Async validation** — Should `Filterable` have an async counterpart
   (`FilterableAsync`) mirroring the `ValidateAsync` / `ValidateRefAsync` traits?
   This would enable `Rule::CustomAsync` in derive macros.

3. **Error message customization** — Should `#[validate]` annotations support
   inline custom messages (e.g., `#[validate(required, message = "Name is required")]`)
   mapping to `Rule::WithMessage { ... }`?

---

## Out of Scope

1. **`impl Filterable for FormData`** — Unifying the dynamic and typed paths
   under a single trait is a potential future improvement.
2. **Runtime schema validation** — The derive macro generates static code;
   schema-driven validation at runtime continues to use `FieldFilter`.
3. **Serde derive integration** — We do not generate `Serialize`/`Deserialize`
   impls; users derive those separately if needed.
