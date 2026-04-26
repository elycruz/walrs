# walrs_validation

[![Crates.io](https://img.shields.io/crates/v/walrs_validation.svg)](https://crates.io/crates/walrs_validation)
[![docs.rs](https://docs.rs/walrs_validation/badge.svg)](https://docs.rs/walrs_validation)
[![License](https://img.shields.io/crates/l/walrs_validation.svg)](https://github.com/elycruz/walrs/blob/main/LICENSE)
[![CI](https://github.com/elycruz/walrs/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/elycruz/walrs/actions/workflows/build-and-test.yml)

Composable validation rules for input validation.

This crate provides a serializable, composable validation rule system based on
the `Rule<T>` enum, along with core validation traits.

## Installation

```toml
[dependencies]
walrs_validation = "0.1"
```

To enable optional features:

```toml
[dependencies]
walrs_validation = { version = "0.1", features = ["async", "chrono"] }
```

## Validation Rules

The `Rule` enum provides built-in validation for common constraints:

- `Rule::Required` - Value must not be empty
- `Rule::MinLength` / `Rule::MaxLength` - Length constraints
- `Rule::ExactLength` - Exact length constraint
- `Rule::Min` / `Rule::Max` - Numeric range constraints
- `Rule::Range` - Inclusive range constraint (min and max together)
- `Rule::Equals` - Exact value match
- `Rule::OneOf` - Value must be one of an allowed set
- `Rule::Pattern` - Regex pattern matching
- `Rule::Email` - Configurable email validation (DNS/IP/local domains, local part length)
- `Rule::Url` - Configurable URL validation (scheme filtering)
- `Rule::Uri` - Configurable URI validation (scheme, relative/absolute)
- `Rule::Ip` - Configurable IP address validation (IPv4/IPv6/IPvFuture)
- `Rule::Step` - Step/multiple validation
- `Rule::Hostname` - Configurable hostname validation (DNS/IP/local/public IPv4)
- `Rule::Date` - Date format validation (ISO 8601, US, EU, RFC 2822, custom)
- `Rule::DateRange` - Date range validation with min/max bounds
- `Rule::Custom` - Custom closure-based validation
- `Rule::CustomAsync` - Async custom closure (requires `async` feature)

## Rule Composition

Rules can be composed using methods on `Rule`:

- `.and()` - Both rules must pass (AND logic, produces `Rule::All`)
- `.or()` - At least one rule must pass (OR logic, produces `Rule::Any`)
- `.not()` - Negates a rule (produces `Rule::Not`)
- `.when()` / `.when_else()` - Conditional validation (produces `Rule::When`)

## Example

```rust
use walrs_validation::{Rule, Validate, ValidateRef};

// Length validation
let length_rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(20));

assert!(length_rule.validate_ref("hello").is_ok());
assert!(length_rule.validate_ref("hi").is_err());

// Range validation
let range_rule = Rule::<i32>::Min(0).and(Rule::Max(100));

assert!(range_rule.validate(50).is_ok());
assert!(range_rule.validate(-1).is_err());
```

## `Option<T>` Validation

`Rule<T>` implements `Validate<Option<T>>` and `ValidateRef<Option<T>>`,
allowing direct validation of optional values:

- `None` with `Rule::Required` → `Err(Violation::value_missing())`
- `None` without `Required` → `Ok(())`
- `Some(v)` → delegates to the inner `Validate<T>` / `ValidateRef<T>` impl

```rust
use walrs_validation::{Rule, Validate, ValidateRef};

let rule = Rule::<String>::Required.and(Rule::MinLength(3));

// None fails because the rule includes Required
assert!(rule.validate(None::<String>).is_err());

// Some delegates to inner validation
assert!(rule.validate(Some("hello".to_string())).is_ok());
assert!(rule.validate(Some("hi".to_string())).is_err());

// ValidateRef works with references
assert!(rule.validate_ref(&None::<String>).is_err());
assert!(rule.validate_ref(&Some("hello".to_string())).is_ok());

// Without Required, None is accepted
let optional_rule = Rule::<i32>::Min(0).and(Rule::Max(100));
assert!(optional_rule.validate(None::<i32>).is_ok());
assert!(optional_rule.validate(Some(50)).is_ok());
```

Async variants (`ValidateAsync<Option<T>>`, `ValidateRefAsync<Option<T>>`) are
also available when the `async` feature is enabled.

## Validation Traits

The crate provides two main validation traits:

```rust
// For Copy types (numbers, etc.)
pub trait Validate<T> {
    fn validate(&self, value: T) -> ValidatorResult;
}

// For referenced types (str, slices, etc.)
pub trait ValidateRef<T: ?Sized> {
    fn validate_ref(&self, value: &T) -> ValidatorResult;
}
```

### Async variants (`async` feature)

Enable the `async` feature to use `ValidateAsync` and `ValidateRefAsync`:

```toml
walrs_validation = { version = "0.1", features = ["async"] }
```

```rust,ignore
use walrs_validation::{Rule, ValidateAsync, ValidateRefAsync};

// Rule::CustomAsync — async closure-based validation
let rule = Rule::<String>::custom_async(std::sync::Arc::new(|value: &String| {
    let value = value.clone();
    Box::pin(async move {
        // async I/O, DB lookup, etc.
        if value.is_empty() {
            Err(walrs_validation::Violation::value_missing())
        } else {
            Ok(())
        }
    })
}));

// Awaitable validation
let result = rule.validate_ref_async("hello").await;
assert!(result.is_ok());
```

## WithMessage — Custom Violation Messages

### `WithMessage` — custom violation messages

Attach a custom message to any rule (or composed rule) using `.with_message()`:

```rust
use walrs_validation::{Rule, ValidateRef};

let rule = Rule::<String>::MinLength(8)
    .with_message("Password must be at least 8 characters.");

let err = rule.validate_ref("short").unwrap_err();
assert_eq!(err.message(), "Password must be at least 8 characters.");
```

Use `.with_message_provider()` for dynamic messages based on the failing value:

```rust
use walrs_validation::{Rule, ValidateRef};

let rule = Rule::<String>::MinLength(5).with_message_provider(
    |ctx| {
        format!("\"{}\" is too short (minimum 5 characters).", ctx.value)
    },
    None,
);
```

### `Message` / `MessageParams` / `MessageContext`

The `Message` type is the building block of the `WithMessage` system. It can be
a static string, a parameterised template, or a closure-based provider:

```rust,ignore
use walrs_validation::{Message, Rule, ValidateRef};

// Static message
let msg = Message::from("Value is required.");

// Dynamic provider — receives the failing value and locale
let msg = Message::from_fn(|value: &String, _locale| {
    format!("\"{}\" failed validation.", value)
});
```

## Shared Types

This crate also provides shared foundation types used across form-related crates:

- **`Attributes`** - HTML attributes storage and rendering
- **`FieldsetViolations`** - Aggregate error container mapping field names to `Violations`

### Feature Flags

- **`serde_json_bridge`** (default) — Provides `Rule::to_attributes_list`
  HTML-attribute conversion via `serde_json::Value`. Disable with
  `default-features = false` to drop the `serde_json` dependency.
- **`async`** — Enables `ValidateAsync` / `ValidateRefAsync` traits and the
  `Rule::CustomAsync` variant.
- **`chrono`** — Enables `chrono::NaiveDate` date validation.
- **`jiff`** — Enables `jiff::civil::Date` date validation.

### `indexmap` Support

`indexmap` is a required dependency, used by `FieldsetViolations` for
deterministic ordering. The crate also re-exports `indexmap` for convenience.

### Date Validation (`chrono` / `jiff`)

Date validation requires enabling one of the date crate features:

Using `chrono` (most popular, widest ecosystem):

```toml
[dependencies]
walrs_validation = { version = "0.1", features = ["chrono"] }
```

Using `jiff` (modern API, best timezone handling):

```toml
[dependencies]
walrs_validation = { version = "0.1", features = ["jiff"] }
```

**String-based validation** — validate date strings with `Rule::Date` and `Rule::DateRange`:

```rust,ignore
use walrs_validation::{Rule, DateOptions, DateRangeOptions, DateFormat};

// Validate ISO 8601 date strings
let rule = Rule::<String>::Date(DateOptions::default());
assert!(rule.validate_ref("2026-02-23").is_ok());

// Validate date range
let rule = Rule::<String>::DateRange(DateRangeOptions {
    format: DateFormat::Iso8601,
    allow_time: false,
    min: Some("2020-01-01".into()),
    max: Some("2030-12-31".into()),
});
assert!(rule.validate_ref("2025-06-15").is_ok());
```

**Native type validation** — validate `chrono::NaiveDate` / `jiff::civil::Date` directly:

```rust,ignore
// With chrono feature
use chrono::NaiveDate;
use walrs_validation::Rule;

let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
let max = NaiveDate::from_ymd_opt(2030, 12, 31).unwrap();
let rule = Rule::<NaiveDate>::Range { min, max };

let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
assert!(rule.validate_date(&date).is_ok());
```

When both features are enabled, `chrono` takes precedence for string parsing.

## License

Elastic-2.0

