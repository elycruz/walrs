# walrs_fieldfilter

Field-level validation and filtering for form processing.

## Overview

`walrs_fieldfilter` provides the core validation and filtering infrastructure for form processing, built around `Field<T>` for single-field configuration and `Fieldset` for typed multi-field structs.

## Key Types

### Field<T>

Unified field configuration for validation and filtering:

```rust
use walrs_fieldfilter::{Field, FieldBuilder};
use walrs_filter::FilterOp;
use walrs_validation::Rule;

// Simple field with just a rule (filters are optional)
let required_field: Field<String> = FieldBuilder::default()
    .rule(Rule::Required)
    .build()
    .unwrap();

// Field with rule and filter operations
let sanitized_field: Field<String> = FieldBuilder::default()
    .rule(Rule::Required.and(Rule::MinLength(3)))
    .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
    .build()
    .unwrap();

// Validate
assert!(required_field.validate("".to_string()).is_err());
assert!(sanitized_field.validate("hello".to_string()).is_ok());
```

### FilterOp<T> Enum

Serializable filter operations for value transformation (defined in `walrs_filter`, re-exported here):

```rust
use walrs_filter::FilterOp;

let filters: Vec<FilterOp<String>> = vec![
    FilterOp::Trim,
    FilterOp::Lowercase,
    FilterOp::StripTags,
];

// apply_ref accepts &str — no allocation needed at the call site
let mut value = "  <b>HELLO</b>  ".to_string();
for filter in &filters {
    value = filter.apply_ref(&value).into_owned();
}
assert_eq!(value, "hello");
```

Available filter operations:

- `Trim` - Remove whitespace
- `Uppercase` / `Lowercase` - Case transformation
- `StripTags` - Remove HTML tags
- `Slug` - URL-safe slug generation
- `Clamp(min, max)` - Numeric clamping
- `Chain(ops)` - Sequential filter chain
- `Custom(fn)` - Custom filter function

For fallible transformations (e.g., parsing), use `TryFilterOp` together with `FilterError`.

### Fieldset

Typed struct validation and filtering with compile-time guarantees — the recommended approach for multi-field forms. Use the `derive` feature to auto-generate implementations:

```rust
use walrs_fieldfilter::{DeriveFieldset, Fieldset};

#[derive(Debug, DeriveFieldset)]
struct ContactForm {
    #[validate(required, email)]
    #[filter(trim, lowercase)]
    email: String,

    #[validate(required, min_length = 2)]
    #[filter(trim)]
    name: String,
}

fn main() {
    let form = ContactForm {
        email: "  USER@EXAMPLE.COM  ".into(),
        name: "  Alice  ".into(),
    };

    match form.sanitize() {
        Ok(sanitized) => println!("Sanitized: {:?}", sanitized),
        Err(violations) => eprintln!("Errors: {}", violations),
    }
}
```

**Key features:**

- `sanitize()` — filter then validate (convenience method)
- `validate()` — validate without filtering
- `filter()` — filter without validation
- Nested struct support with `#[validate(nested)]` and `#[filter(nested)]` (see `examples/derive_nested.rs`)
- Cross-field validation with `#[cross_validate(...)]` (see `examples/derive_cross_validate.rs`)
- `Option<T>` handling
- Custom validators and filters

See [`crates/fieldset_derive/README.md`](../fieldset_derive/README.md) for the full proc-macro reference (annotation tables, cross-field validation, async, etc.).

### Async (`FieldsetAsync`)

Behind the `async` feature, `walrs_fieldfilter` exposes `FieldsetAsync` — an async counterpart to `Fieldset` whose `validate`, `filter`, and `sanitize` methods return futures. The derive macro generates a `FieldsetAsync` impl when both `derive` and `async` features are enabled, allowing async validators and filters per field.

See `examples/derive_async.rs` for a runnable example, and `crates/fieldset_derive/README.md` for the async annotation reference.

## Public API surface

Top-level re-exports from `walrs_fieldfilter` (see `src/lib.rs`):

- **Core**: `Field<T>`, `FieldBuilder`, `Fieldset`, `Rule`, `RuleResult`, `Condition`
- **Filtering** (re-exported from `walrs_filter`): `FilterOp`, `TryFilterOp`, `FilterError`
- **Violations** (re-exported from `walrs_validation`): `Violation`, `Violations`, `FieldsetViolations`, `ViolationType`, `ViolationMessage`, `Message`, `MessageContext`, `MessageParams`, `Attributes`, `IsEmpty`
- **Convenience**: `IndexMap` (re-exported from `indexmap`, used for ordered field iteration)
- **Derive macro** (feature `derive`): `DeriveFieldset` — alias for `walrs_fieldset_derive::Fieldset`
- **Async** (feature `async`): `FieldsetAsync`, `ValidateAsync`, `ValidateRefAsync`

## Installation

### Basic (without derive macro)

```toml
[dependencies]
walrs_fieldfilter = { path = "../fieldfilter" }
```

### With derive macro

```toml
[dependencies]
walrs_fieldfilter = { path = "../fieldfilter", features = ["derive"] }
```

### With derive + async

```toml
[dependencies]
walrs_fieldfilter = { path = "../fieldfilter", features = ["derive", "async"] }
```

### Feature flags

| Feature | Enables |
|---|---|
| `derive` | `#[derive(Fieldset)]` via `walrs_fieldset_derive`, re-exported as `DeriveFieldset`. |
| `async` | `FieldsetAsync` trait and async re-exports (`ValidateAsync`, `ValidateRefAsync`). Combine with `derive` for an async-derived impl. |

## Examples

Runnable examples live in [`examples/`](./examples/). Run any of them with `cargo run -p walrs_fieldfilter --example <name> [--features ...]`.

| Example | Demonstrates | Required features |
|---|---|---|
| `field_basics` | Core `Field<T>` API: rules, filters, `sanitize`/`validate`/`filter` | _none_ |
| `filters` | `FilterOp` transformations and chaining | _none_ |
| `rule_composition` | Combining rules with `and` / `or` / `not` | _none_ |
| `json_serialization` | Serializing/deserializing `Field` and `FilterOp` via Serde | _none_ |
| `localized_messages` | Localized violation messages via `MessageContext` | _none_ |
| `derive_simple` | Basic `#[derive(Fieldset)]` on a flat struct | `derive` |
| `derive_nested` | Nested struct validation with `#[validate(nested)]` / `#[filter(nested)]` | `derive` |
| `derive_cross_validate` | Cross-field validation via `#[cross_validate(...)]` | `derive` |
| `derive_async` | `FieldsetAsync` with async validators and filters | `derive`, `async` |

Example invocations:

```sh
cargo run -p walrs_fieldfilter --example field_basics
cargo run -p walrs_fieldfilter --example derive_nested --features derive
cargo run -p walrs_fieldfilter --example derive_async --features "derive async"
```

## Architecture

This crate sits between `walrs_validation` / `walrs_filter` and `walrs_fieldset_derive`:

```
walrs_validation    → walrs_filter      → walrs_fieldfilter ← walrs_fieldset_derive
(Rule<T> enum,        (Filter trait,       (Field<T>,           (#[derive(Fieldset)])
 ValidateAsync)        FilterOp<T> enum)    Fieldset,
                                            FieldsetAsync)
```

## License

Elastic-2.0
