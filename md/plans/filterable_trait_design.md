# Plan: Type-Safe `Filterable` Trait for Concrete Struct Validation

**Date:** February 23, 2026
**Status:** Design / Investigation
**Crates affected:** `walrs_inputfilter`, `walrs_inputfilter_derive` (new), `walrs_form`, `walrs_validation`

---

## Table of Contents

1. [Background](#background)
2. [Problem Analysis](#problem-analysis)
3. [Proposed Design](#proposed-design)
   - [3.1 `Filterable` Trait](#31-filterable-trait)
   - [3.2 Derive Macro (`#[derive(Filterable)]`)](#32-derive-macro-derivefilterable)
   - [3.3 Supported Annotations](#33-supported-annotations)
   - [3.4 Cross-Field Validation](#34-cross-field-validation)
   - [3.5 Per-Field Filtering Strategy](#35-per-field-filtering-strategy)
   - [3.6 `Form` Integration](#36-form-integration)
4. [Backward Compatibility](#backward-compatibility)
5. [Crate Layout](#crate-layout)
6. [Open Question Decisions](#open-question-decisions)
7. [Code Sketches](#code-sketches)
8. [Out of Scope](#out-of-scope)

---

## Background

`FieldFilter` and `Form` are currently hardwired to `HashMap<String, Value>` / `FormData`
for hydration and validation. Field names are runtime strings, values are the
dynamically-typed `Value` enum, and type correctness is deferred entirely to
runtime `TypeMismatch` violations.

### Current signatures

```rust
// FieldFilter (inputfilter/src/field_filter.rs)
pub fn validate(&self, data: &HashMap<String, Value>) -> Result<(), FormViolations> { ... }
pub fn filter(&self, data: HashMap<String, Value>) -> HashMap<String, Value> { ... }

// Form (form/src/form.rs)
pub fn validate(&self, data: &FormData) -> Result<(), FormViolations> { ... }
pub fn bind_data(&mut self, data: FormData) -> &mut Self { ... }
```

This design is correct for dynamic/config-driven forms, JSON deserialization, and
WASM `web_sys::FormData` scenarios. However, in a statically-typed language,
users validating known structs should get **compiler errors** when wiring
incompatible rules to fields — not runtime failures.

---

## Problem Analysis

| Problem | Example |
|---------|---------|
| Field names are runtime strings | A typo `"emal"` instead of `"email"` silently passes |
| Values are dynamically typed | `Rule::<Value>::MinLength(3)` applied to `Value::I64(42)` → runtime `TypeMismatch` |
| No compile-time rule/type check | `Rule::<i64>::Min(5)` on a `String` field is only caught at runtime |
| Ownership mismatch | `FieldFilter::filter` takes `HashMap<String, Value>` by value, loses struct context |

### What users expect

```rust
struct UserAddress {
    street: String,
    zip: String,
}
```

Attach validation rules per field with compile-time type checking, call a single
`validate(&user_address)` method, with **the compiler rejecting rule/type
mismatches**.

---

## Proposed Design

### 3.1 `Filterable` Trait

Defined in `walrs_inputfilter`:

```rust
use crate::FormViolations;

/// Trait for structs that support type-safe field validation and filtering.
///
/// Implementors can either derive this trait via `#[derive(Filterable)]` or
/// implement it manually for full control over validation and filtering logic.
pub trait Filterable: Sized {
    /// Validate all fields and cross-field rules.
    ///
    /// Returns `Ok(())` when all fields pass validation, or
    /// `Err(FormViolations)` with per-field and form-level violations.
    fn validate(&self) -> Result<(), FormViolations>;

    /// Apply per-field filters, consuming and returning `Self`.
    ///
    /// Takes ownership to avoid `mem::take` / `Default` requirements on
    /// non-`Default` fields.  Users who need the original struct can
    /// `.clone()` before calling `filter()`.
    fn filter(self) -> Self;

    /// Filter then validate (provided default).
    ///
    /// Applies `filter()` first, then `validate()` on the result.
    /// Returns `Ok(filtered)` if validation passes, or `Err(FormViolations)`.
    fn process(self) -> Result<Self, FormViolations> {
        let filtered = self.filter();
        filtered.validate()?;
        Ok(filtered)
    }
}
```

#### Design rationale

- **`validate(&self)`** — borrows `self`, runs `Rule<T>::validate_ref()` /
  `Rule<T>::validate_str()` etc. with the correct concrete `T` per field.
  Collects errors into `FormViolations` keyed by field name. Does not consume
  the struct.

- **`filter(self)`** — takes ownership. Each field is destructured, filtered via
  `Filter<T>::apply()`, and reassembled into `Self`. Ownership transfer avoids
  `mem::take` hacks on non-`Default` fields.

- **`process(self)`** — provided default: filter then validate. Mirrors
  `FieldFilter::process` and `Field<T>::process` semantics.

### 3.2 Derive Macro (`#[derive(Filterable)]`)

A new proc-macro crate `walrs_inputfilter_derive` provides `#[derive(Filterable)]`.

#### Generated code for `validate(&self)`

For each field annotated with `#[validate(...)]`:

1. Constructs the appropriate `Rule<T>` (e.g., `Rule::<String>::Required.and(Rule::MinLength(3))`).
2. Calls the type-appropriate validation method on the field's value.
3. Collects any `Violation` into a `FormViolations` keyed by the field name (as a string literal).

```rust
// Generated for:
// #[validate(required, min_length = 3)]
// street: String,

{
    let rule = Rule::<String>::Required.and(Rule::<String>::MinLength(3));
    if let Err(violation) = rule.validate_ref(self.street.as_str()) {
        let mut vs = walrs_validation::Violations::empty();
        vs.push(violation);
        violations.add_field_violations("street", vs);
    }
}
```

**Type safety**: The rule is constructed as `Rule::<String>`, and the field is
`String`. A mismatched annotation (e.g., `min = 5` on a `String` field) would
produce a compile error because `Rule::<String>::Min(5)` doesn't exist (only
`Rule::<i64>::Min`, `Rule::<f64>::Min`, etc.).

#### Generated code for `filter(self)`

For each field annotated with `#[filter(...)]`:

1. Constructs the appropriate `Filter<T>` chain.
2. Applies it to the field via ownership transfer.

```rust
// Generated for:
// #[filter(trim, lowercase)]
// street: String,

fn filter(self) -> Self {
    let street = {
        let filters: Vec<Filter<String>> = vec![Filter::Trim, Filter::Lowercase];
        filters.iter().fold(self.street, |v, f| f.apply(v))
    };
    Self { street, ..self }
}
```

### 3.3 Supported Annotations

#### `#[validate(...)]` annotations

| Annotation | Maps to | Applicable types |
|---|---|---|
| `required` | `Rule::Required` | All |
| `min_length = N` | `Rule::MinLength(N)` | `String`, `Vec<T>` |
| `max_length = N` | `Rule::MaxLength(N)` | `String`, `Vec<T>` |
| `exact_length = N` | `Rule::ExactLength(N)` | `String`, `Vec<T>` |
| `email` | `Rule::Email` | `String` |
| `url` | `Rule::Url` | `String` |
| `pattern = "regex"` | `Rule::Pattern("regex".into())` | `String` |
| `min = N` | `Rule::Min(N)` | `i64`, `u64`, `f64` |
| `max = N` | `Rule::Max(N)` | `i64`, `u64`, `f64` |
| `step = N` | `Rule::Step(N)` | Steppable types |
| `one_of = [a, b, c]` | `Rule::OneOf(vec![a, b, c])` | Types with `PartialEq` |
| `custom = "path::to::fn"` | `Rule::Custom(Arc::new(path::to::fn))` | All |
| `nested` | Calls `field.validate()` (requires `Filterable`) | Nested structs |

Multiple `#[validate(...)]` attributes on the same field combine via `Rule::and()`.

#### `#[filter(...)]` annotations

| Annotation | Maps to | Applicable types |
|---|---|---|
| `trim` | `Filter::Trim` | `String` |
| `lowercase` | `Filter::Lowercase` | `String` |
| `uppercase` | `Filter::Uppercase` | `String` |
| `strip_tags` | `Filter::StripTags` | `String` |
| `html_entities` | `Filter::HtmlEntities` | `String` |
| `slug` | `Filter::Slug { max_length: None }` | `String` |
| `slug(max_length = N)` | `Filter::Slug { max_length: Some(N) }` | `String` |
| `custom = "path::to::fn"` | `Filter::Custom(Arc::new(path::to::fn))` | All |

Multiple `#[filter(...)]` attributes on the same field produce a `Filter::Chain(...)`.

### 3.4 Cross-Field Validation

#### MVP: Manual override

Users implement `validate()` manually, calling a generated helper for per-field checks:

```rust
#[derive(Filterable)]
struct Registration {
    #[validate(required, min_length = 8)]
    #[filter(trim)]
    password: String,

    #[validate(required)]
    #[filter(trim)]
    confirm: String,
}

impl Registration {
    /// Custom cross-field validation that wraps the generated per-field checks.
    fn validate_with_cross_field(&self) -> Result<(), FormViolations> {
        // Run generated per-field validation
        let mut violations = match self.validate() {
            Ok(()) => FormViolations::new(),
            Err(v) => v,
        };

        // Cross-field: passwords must match
        if self.password != self.confirm {
            violations.add_form_violation(
                Violation::new(ViolationType::NotEqual, "Passwords must match"),
            );
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}
```

#### Stretch goal: `#[cross_validate(...)]` attribute

```rust
#[derive(Filterable)]
#[cross_validate(passwords_match)]
struct Registration {
    #[validate(required, min_length = 8)]
    password: String,

    #[validate(required)]
    confirm: String,
}

fn passwords_match(r: &Registration) -> RuleResult {
    if r.password == r.confirm {
        Ok(())
    } else {
        Err(Violation::new(ViolationType::NotEqual, "Passwords must match"))
    }
}
```

Generated `validate()` calls each `#[cross_validate]` function after per-field
checks, passing `&self` as the argument. This provides type-safe access to all
fields without `HashMap::get`.

### 3.5 Per-Field Filtering Strategy

`Filter<T>::apply()` takes values by-move (`fn apply(&self, value: T) -> T`).
The generated `filter(self)` impl applies per-field filters via ownership
transfer, then reconstructs `Self` using struct update syntax:

```rust
fn filter(self) -> Self {
    let street = Filter::<String>::Trim.apply(self.street);
    let zip = self.zip; // no filter
    Self { street, zip }
}
```

- **No `Default` requirement**: The struct is destructured via `self.field`, not
  via `mem::take`.
- **No `Clone` requirement**: The original is consumed.
- **Fields without `#[filter]`**: Passed through unchanged.

For fields that are `Filterable` themselves (nested structs), the generated code
calls `field.filter()` recursively:

```rust
let address = self.address.filter(); // Address: Filterable
```

### 3.6 `Form` Integration

The existing `Form::validate` and `Form::bind_data` accept `&FormData` /
`FormData`. Integration with concrete structs can happen through:

#### `Into<FormData>` bridge

The derive macro can optionally generate `impl From<&MyStruct> for FormData`:

```rust
// Generated (opt-in via #[filterable(into_form_data)])
impl From<&UserAddress> for FormData {
    fn from(s: &UserAddress) -> FormData {
        let mut data = FormData::new();
        data.insert("street", Value::from(s.street.clone()));
        data.insert("zip", Value::from(s.zip.clone()));
        data
    }
}
```

This enables:

```rust
let address = UserAddress { street: "123 Main".into(), zip: "90210".into() };
let form_data = FormData::from(&address);
form.bind_data(form_data);
```

#### `Form` with `Filterable` (future extension)

A generic validation path on `Form` could be added:

```rust
impl Form {
    /// Validates a struct that implements `Filterable`.
    pub fn validate_struct<T: Filterable>(&self, data: &T) -> Result<(), FormViolations> {
        data.validate()
    }
}
```

However, this adds minimal value over calling `data.validate()` directly. The
more useful integration is `Into<FormData>` for binding struct values to form
elements.

---

## Backward Compatibility

The `HashMap<String, Value>` path is **preserved unchanged** for:

- Config-driven / JSON-deserialized forms
- WASM boundaries (`web_sys::FormData`)
- Dynamic form generation
- `FieldFilter::validate(&self, data: &HashMap<String, Value>)`
- `Form::validate(&self, data: &FormData)`

The struct-based `Filterable` path **supplements** the existing dynamic path.
No existing public APIs are changed or removed.

A future `impl Filterable for FormData` could unify both paths but is out of
scope for the initial implementation.

---

## Crate Layout

### New crate: `walrs_inputfilter_derive`

```
crates/
├── inputfilter/
│   ├── Cargo.toml          # adds `walrs_inputfilter_derive` as optional dep
│   └── src/
│       ├── lib.rs           # re-exports `Filterable` trait; conditionally
│       │                    #   re-exports derive macro
│       ├── filterable.rs    # `Filterable` trait definition
│       └── ...              # existing modules unchanged
├── inputfilter_derive/
│   ├── Cargo.toml           # proc-macro = true; depends on syn, quote, proc-macro2
│   └── src/
│       └── lib.rs           # #[proc_macro_derive(Filterable, attributes(validate, filter, ...))]
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

Or use the derive crate directly:

```toml
walrs_inputfilter_derive = "..."
```

---

## Open Question Decisions

### 1. Nested structs

**Decision: Support nested structs.**

When a field's type also implements `Filterable`, the derive macro generates:

- **`validate()`**: calls `self.field.validate()` and prefixes violations with
  the field name (e.g., `"address.street"`).
- **`filter()`**: calls `self.field.filter()`.

This maps naturally to `FormData`'s existing dot-notation path resolution.
Triggered by the `#[validate(nested)]` annotation.

```rust
#[derive(Filterable)]
struct Registration {
    #[validate(required)]
    name: String,

    #[validate(nested)]
    address: UserAddress,  // UserAddress: Filterable
}
```

Generated validation for nested field:

```rust
if let Err(nested_violations) = self.address.validate() {
    for (field_name, field_violations) in nested_violations.fields {
        violations.add_field_violations(
            format!("address.{}", field_name),
            field_violations,
        );
    }
    violations.add_form_violations(nested_violations.form);
}
```

### 2. `FieldFilter` generic extension

**Decision: `Filterable` replaces `FieldFilter` for typed use cases.**

`FieldFilter` stays as the dynamic path for `HashMap<String, Value>`. Making
`FieldFilter` generic would require significant API churn with little benefit,
since `Filterable` already provides the typed validation and filtering directly
on the struct.

The two paths serve different use cases:

| Path | Use case |
|------|----------|
| `FieldFilter` (dynamic) | JSON/YAML config-driven forms, WASM, runtime schemas |
| `Filterable` (typed) | Known Rust structs, compile-time checked rules |

### 3. WASM boundary bridge

**Decision: Generate `Into<FormData>` / `TryFrom<FormData>` as opt-in.**

Controlled via a struct-level attribute:

```rust
#[derive(Filterable)]
#[filterable(into_form_data, try_from_form_data)]
struct UserAddress {
    street: String,
    zip: String,
}
```

- `into_form_data` generates `impl From<&UserAddress> for FormData`.
- `try_from_form_data` generates `impl TryFrom<FormData> for UserAddress`
  (returns `Result<Self, String>` for missing/mistyped fields).

### 4. Proc-macro crate naming

**Decision: `walrs_inputfilter_derive`.**

Scoped to `walrs_inputfilter` since the trait and types (`FormViolations`,
`Filter`, etc.) live there. If future derive macros are needed for other crates,
they get their own `_derive` crate.

---

## Code Sketches

### Full example: User registration

```rust
use walrs_inputfilter::{Filterable, FormViolations};
use walrs_inputfilter::filter_enum::Filter;
use walrs_validation::{Rule, Violation, ViolationType};

#[derive(Filterable)]
#[filterable(into_form_data)]
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

### Generated `Filterable` impl for `UserAddress`

```rust
impl Filterable for UserAddress {
    fn validate(&self) -> Result<(), FormViolations> {
        let mut violations = FormViolations::new();

        // street: required + min_length(3)
        {
            let rule = Rule::<String>::Required.and(Rule::<String>::MinLength(3));
            if let Err(violation) = rule.validate_ref(self.street.as_str()) {
                let mut vs = walrs_validation::Violations::empty();
                vs.push(violation);
                violations.add_field_violations("street", vs);
            }
        }

        // zip: required + pattern
        {
            let rule = Rule::<String>::Required
                .and(Rule::<String>::Pattern(r"^\d{5}$".into()));
            if let Err(violation) = rule.validate_ref(self.zip.as_str()) {
                let mut vs = walrs_validation::Violations::empty();
                vs.push(violation);
                violations.add_field_violations("zip", vs);
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    fn filter(self) -> Self {
        let street = {
            let filters: Vec<walrs_inputfilter::filter_enum::Filter<String>> =
                vec![walrs_inputfilter::filter_enum::Filter::Trim];
            filters.iter().fold(self.street, |v, f| f.apply(v))
        };
        let zip = {
            let filters: Vec<walrs_inputfilter::filter_enum::Filter<String>> =
                vec![walrs_inputfilter::filter_enum::Filter::Trim];
            filters.iter().fold(self.zip, |v, f| f.apply(v))
        };
        Self { street, zip }
    }
}
```

### Generated `Filterable` impl for `UserRegistration` (with nesting and cross-field)

```rust
impl Filterable for UserRegistration {
    fn validate(&self) -> Result<(), FormViolations> {
        let mut violations = FormViolations::new();

        // name: required + min_length(2) + max_length(50)
        {
            let rule = Rule::<String>::Required
                .and(Rule::<String>::MinLength(2))
                .and(Rule::<String>::MaxLength(50));
            if let Err(violation) = rule.validate_ref(self.name.as_str()) {
                let mut vs = walrs_validation::Violations::empty();
                vs.push(violation);
                violations.add_field_violations("name", vs);
            }
        }

        // email: required + email
        {
            let rule = Rule::<String>::Required.and(Rule::<String>::Email);
            if let Err(violation) = rule.validate_ref(self.email.as_str()) {
                let mut vs = walrs_validation::Violations::empty();
                vs.push(violation);
                violations.add_field_violations("email", vs);
            }
        }

        // password: required + min_length(8)
        {
            let rule = Rule::<String>::Required.and(Rule::<String>::MinLength(8));
            if let Err(violation) = rule.validate_ref(self.password.as_str()) {
                let mut vs = walrs_validation::Violations::empty();
                vs.push(violation);
                violations.add_field_violations("password", vs);
            }
        }

        // confirm_password: required
        {
            let rule = Rule::<String>::Required;
            if let Err(violation) = rule.validate_ref(self.confirm_password.as_str()) {
                let mut vs = walrs_validation::Violations::empty();
                vs.push(violation);
                violations.add_field_violations("confirm_password", vs);
            }
        }

        // age: min(0) + max(150)
        {
            let rule = Rule::<i64>::Min(0).and(Rule::<i64>::Max(150));
            if let Err(violation) = rule.validate_scalar(self.age) {
                let mut vs = walrs_validation::Violations::empty();
                vs.push(violation);
                violations.add_field_violations("age", vs);
            }
        }

        // address: nested Filterable
        if let Err(nested_violations) = self.address.validate() {
            for (field_name, field_violations) in nested_violations.fields {
                violations.add_field_violations(
                    format!("address.{}", field_name),
                    field_violations,
                );
            }
            violations.add_form_violations(nested_violations.form);
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    fn filter(self) -> Self {
        let name = walrs_inputfilter::filter_enum::Filter::<String>::Trim
            .apply(self.name);
        let email = {
            let filters = vec![
                walrs_inputfilter::filter_enum::Filter::<String>::Trim,
                walrs_inputfilter::filter_enum::Filter::<String>::Lowercase,
            ];
            filters.iter().fold(self.email, |v, f| f.apply(v))
        };
        let address = self.address.filter(); // nested Filterable
        Self {
            name,
            email,
            password: self.password,
            confirm_password: self.confirm_password,
            age: self.age,
            address,
        }
    }
}
```

### Manual `Filterable` implementation (no derive)

```rust
use walrs_inputfilter::{Filterable, FormViolations};
use walrs_validation::{Rule, ValidateRef, Violation, ViolationType};

struct LoginForm {
    username: String,
    password: String,
}

impl Filterable for LoginForm {
    fn validate(&self) -> Result<(), FormViolations> {
        let mut violations = FormViolations::new();

        let username_rule = Rule::<String>::Required.and(Rule::MinLength(3));
        if let Err(v) = username_rule.validate_ref(self.username.as_str()) {
            let mut vs = walrs_validation::Violations::empty();
            vs.push(v);
            violations.add_field_violations("username", vs);
        }

        let password_rule = Rule::<String>::Required.and(Rule::MinLength(8));
        if let Err(v) = password_rule.validate_ref(self.password.as_str()) {
            let mut vs = walrs_validation::Violations::empty();
            vs.push(v);
            violations.add_field_violations("password", vs);
        }

        if violations.is_empty() { Ok(()) } else { Err(violations) }
    }

    fn filter(self) -> Self {
        let username = walrs_inputfilter::filter_enum::Filter::<String>::Trim
            .apply(self.username);
        Self { username, password: self.password }
    }
}
```

### Type mapping for derive macro code generation

The derive macro needs to map field types to the correct validation method:

| Field type | Rule type | Validation method |
|---|---|---|
| `String` | `Rule<String>` | `rule.validate_ref(self.field.as_str())` |
| `i64` | `Rule<i64>` | `rule.validate_scalar(self.field)` |
| `u64` | `Rule<u64>` | `rule.validate_scalar(self.field)` |
| `f64` | `Rule<f64>` | `rule.validate_scalar(self.field)` |
| `T: Filterable` | N/A | `self.field.validate()` (nested) |
| `Option<T>` | Conditional | Skip if `None` unless `required` |

For `Option<T>` fields:
- If `required`, `None` is a `ValueMissing` violation.
- If not `required`, `None` skips all other validations.
- If `Some(value)`, validate `value` with the inner type's rules.

---

## Out of Scope

The following are explicitly out of scope for the initial implementation:

1. **`impl Filterable for FormData`** — Unifying the dynamic and typed paths
   under a single trait is a potential future improvement.
2. **Runtime schema validation** — The derive macro generates static code;
   schema-driven validation at runtime continues to use `FieldFilter`.
3. **Async validation** — Cross-field rules involving async operations (e.g.,
   database lookups) are not covered by this design.
4. **`break_on_failure` semantics** — The typed path validates all fields and
   collects all violations. Short-circuit behavior could be added later.
