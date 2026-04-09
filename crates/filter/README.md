# walrs_filter

Filter/transformation structs for input filtering.

This crate provides reusable filter implementations that can transform input values. Filters are typically used in form processing pipelines to sanitize, normalize, or transform user input before, or after, validation.

## Available Filters

- **`SlugFilter`** - Converts strings to URL-friendly slugs.
- **`StripTagsFilter`** - Removes/sanitizes HTML tags using [Ammonia](https://docs.rs/ammonia).
- **`XmlEntitiesFilter`** - Encodes special characters as XML entities.

## FilterOp Enum

The `FilterOp<T>` enum provides a composable, serializable way to define filter operations for config-driven form processing. It delegates to the filter structs above.

Available operations:
- `Trim` - Remove leading/trailing whitespace
- `Uppercase` / `Lowercase` - Case transformation
- `StripTags` - Remove HTML tags
- `HtmlEntities` - Encode XML/HTML entities
- `Slug { max_length }` - URL-safe slug generation
- `Truncate { max_length }` - Clip string to at most `max_length` characters
- `Replace { from, to }` - Replace all occurrences of a substring
- `Clamp { min, max }` - Numeric clamping
- `Chain(ops)` - Sequential filter chain
- `Custom(fn)` - Runtime filter function (not serializable — see [Serde notes](#serde-notes))

```rust
use walrs_filter::FilterOp;
use std::borrow::Cow;

fn main() {
    let op = FilterOp::<String>::Chain(vec![
        FilterOp::Trim,
        FilterOp::Lowercase,
    ]);
    // apply_ref returns Cow — zero-copy when input is unchanged
    assert_eq!(op.apply_ref("  HELLO  "), "hello");

    // No-op case — returns Cow::Borrowed, no allocation
    let trim = FilterOp::<String>::Trim;
    let result = trim.apply_ref("already_trimmed");
    assert!(matches!(result, Cow::Borrowed(_)));

    // apply accepts an owned String (delegates to apply_ref)
    assert_eq!(op.apply("  HELLO  ".to_string()), "hello");

    // Truncate string to max 10 characters
    let truncate = FilterOp::<String>::Truncate { max_length: 10 };
    assert_eq!(truncate.apply("Hello World!".to_string()), "Hello Worl");

    // Replace substrings
    let replace = FilterOp::<String>::Replace {
        from: "foo".to_string(),
        to: "bar".to_string(),
    };
    assert_eq!(replace.apply("foo baz foo".to_string()), "bar baz bar");
}
```

When the `validation` feature is enabled (default), `FilterOp<Value>` is available
for dynamic value transformation using `walrs_validation::Value`.

### `apply_ref` vs `apply`

For `FilterOp<String>`:

- **`apply_ref(&self, value: &str) -> Cow<'_, str>`** — preferred when you already have a `&str`.
  Returns `Cow::Borrowed` for no-ops (zero allocation) and `Cow::Owned` when the value is transformed.
- **`apply(&self, value: String) -> String`** — convenience wrapper that delegates to `apply_ref`.

For `FilterOp<T>` where `T: Copy` (numeric types), only `apply(value: T) -> T` is available.

For `FilterOp<Value>`, both `apply_ref(&self, &Value) -> Value` and `apply(&self, Value) -> Value` are available.

### FilterOp vs concrete filter structs

| Use case | Recommendation |
|----------|----------------|
| Config-driven pipeline (load from JSON/YAML) | `FilterOp<String>` or `FilterOp<Value>` |
| Static pipeline in code | Either — `FilterOp::Chain(...)` is ergonomic |
| Polymorphic dispatch via trait objects (`Box<dyn Filter<T>>`) | Concrete structs (`SlugFilter`, etc.) or implement `Filter` |
| Numeric clamping | `FilterOp::Clamp { min, max }` |
| Custom runtime logic | `FilterOp::Custom(Arc::new(fn))` (not serializable) |

### Serde notes

`FilterOp` serializes with `#[serde(tag = "type", content = "config")]` (adjacent tagging):

```json
{"type":"Trim"}
{"type":"Slug","config":{"max_length":50}}
{"type":"Truncate","config":{"max_length":20}}
{"type":"Replace","config":{"from":"foo","to":"bar"}}
{"type":"Chain","config":[{"type":"Trim"},{"type":"Lowercase"}]}
```

**`Custom` cannot be serialized.** Attempting to serialize a `FilterOp::Custom` (or a `Chain`
that contains one) returns an error. If your pipeline must survive a round-trip, avoid `Custom`
or inject custom logic after deserialization.

## Serialization Guide

`FilterOp` is designed for config-driven pipelines — define your filter chain in JSON/YAML and
load it at runtime:

```rust
use walrs_filter::FilterOp;

fn main() {
    // Define a filter chain as JSON
    let json = r#"{"type":"Chain","config":[
        {"type":"Trim"},
        {"type":"Lowercase"},
        {"type":"Slug","config":{"max_length":50}}
    ]}"#;

    // Deserialize at runtime
    let filter: FilterOp<String> = serde_json::from_str(json).unwrap();

    // Apply to input
    let result = filter.apply("  Hello World!  ".to_string());
    assert_eq!(result, "hello-world");
}
```

Supported JSON variant types: `Trim`, `Lowercase`, `Uppercase`, `StripTags`, `HtmlEntities`,
`Slug` (with `max_length`), `Truncate` (with `max_length`), `Replace` (with `from`/`to`),
`Clamp` (with `min`/`max`), `Chain` (with array of ops).

## TryFilterOp Enum (Fallible Filters)

The `TryFilterOp<T>` enum is the fallible counterpart to `FilterOp<T>`. Use it for
filters that can legitimately fail (e.g., base64 decode, JSON parse, URL decode).
Errors are represented as [`FilterError`](#filtererror), which integrates with the
validation error pipeline.

Available variants:
- `Infallible(FilterOp<T>)` - Wraps an infallible filter, lifting it into the fallible pipeline
- `Chain(Vec<TryFilterOp<T>>)` - Sequential filter chain that short-circuits on the first error
- `TryCustom(Arc<dyn Fn(T) -> Result<T, FilterError>>)` - Custom fallible filter function (not serializable)

```rust
use walrs_filter::{TryFilterOp, FilterOp, FilterError};
use std::sync::Arc;

fn main() {
    // Lift an infallible filter into the fallible pipeline
    let trim: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    assert_eq!(trim.try_apply("  hello  ".to_string()).unwrap(), "hello");

    // From trait also works
    let lowercase: TryFilterOp<String> = FilterOp::Lowercase.into();
    assert_eq!(lowercase.try_apply("HELLO".to_string()).unwrap(), "hello");

    // Custom fallible filter
    let parse_hex: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s: String| {
        if s.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(s.to_uppercase())
        } else {
            Err(FilterError::new("invalid hex string").with_name("HexNormalize"))
        }
    }));
    assert_eq!(parse_hex.try_apply("abcdef".to_string()).unwrap(), "ABCDEF");
    assert!(parse_hex.try_apply("xyz".to_string()).is_err());

    // Chain: trim → validate → lowercase (short-circuits on error)
    let pipeline: TryFilterOp<String> = TryFilterOp::Chain(vec![
        TryFilterOp::Infallible(FilterOp::Trim),
        TryFilterOp::TryCustom(Arc::new(|s: String| {
            if s.is_empty() {
                Err(FilterError::new("value must not be empty after trimming"))
            } else {
                Ok(s)
            }
        })),
        TryFilterOp::Infallible(FilterOp::Lowercase),
    ]);
    assert_eq!(pipeline.try_apply("  HELLO  ".to_string()).unwrap(), "hello");
    assert!(pipeline.try_apply("   ".to_string()).is_err());
}
```

**`TryCustom` cannot be serialized.** Attempting to serialize a `TryFilterOp::TryCustom` (or a
`Chain` that contains one) returns an error. If your pipeline must survive a round-trip, avoid
`TryCustom` or inject custom logic after deserialization.

When the `validation` feature is enabled (default), `TryFilterOp<Value>` is available
for dynamic value transformation, and `FilterError` can be converted to `Violation`/`Violations`.

## FilterError

`FilterError` represents a failure during a fallible filter transformation.
It carries a human-readable message and an optional filter name for context.

```rust
use walrs_filter::FilterError;

fn main() {
    let err = FilterError::new("invalid base64 input")
        .with_name("Base64Decode");

    assert_eq!(err.message(), "invalid base64 input");
    assert_eq!(err.filter_name(), Some("Base64Decode"));
    assert_eq!(err.to_string(), "Filter 'Base64Decode' failed: invalid base64 input");
}
```

With the `validation` feature enabled, `FilterError` converts to `Violation`
(using `ViolationType::CustomError`) and `Violations` via `From` impls,
allowing seamless integration with the validation error pipeline.

## The Filter Trait

All filter structs implement the `Filter<T>` trait:

```rust
pub trait Filter<T> {
    type Output;
    fn filter(&self, value: T) -> Self::Output;
}
```

`FilterOp<T>` also implements `Filter<T>` — you can use it anywhere a `Filter<String>`,
`Filter<i32>`, `Filter<Value>`, etc. is expected:

```rust
use walrs_filter::{Filter, FilterOp};

let filter: Box<dyn Filter<String, Output = String>> = Box::new(FilterOp::Trim);
assert_eq!(filter.filter("  hello  ".to_string()), "hello");
```

## The TryFilter Trait

The `TryFilter<T>` trait is the fallible counterpart to [`Filter<T>`](#the-filter-trait):

```rust
pub trait TryFilter<T> {
    type Output;
    fn try_filter(&self, value: T) -> Result<Self::Output, FilterError>;
}
```

`TryFilterOp<T>` implements `TryFilter<T>` for `String` and `Value`:

```rust
use walrs_filter::{TryFilter, TryFilterOp, FilterOp, FilterError};

let filter: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
assert_eq!(filter.try_filter("  hello  ".to_string()).unwrap(), "hello");
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
walrs_filter = { path = "../filter" }  # or from crates.io when published
```

## Example

```rust
use walrs_filter::{Filter, SlugFilter, StripTagsFilter};
use std::borrow::Cow;

fn main () {
    // Create a slug from a title
    let slug_filter = SlugFilter::new(200, false);
    let slug = slug_filter.filter(Cow::Borrowed("Hello World!"));
    assert_eq!(slug, "hello-world");

    // Strip HTML tags
    let strip_filter = StripTagsFilter::new();
    let clean = strip_filter.filter(Cow::Borrowed("<script>alert('xss')</script>Hello"));
    assert_eq!(clean, "Hello");
}
```

## Features

- **`validation`** (default) — Enables `FilterOp<Value>`, `TryFilterOp<Value>`, and
  `FilterError` → `Violation`/`Violations` conversions via `walrs_validation`.
  Without this feature, only `FilterOp<String>` and scalar numeric types are available.
- **`fn_traits`** — Enables nightly Rust `Fn`/`FnMut`/`FnOnce` trait implementations on
  filter structs, allowing them to be called as closures. Requires a nightly compiler.
- **`nightly`** — Catch-all for nightly features. Currently enables `fn_traits`.

## Running Examples

The crate includes several examples demonstrating filter usage:

```bash
# Basic filter usage (SlugFilter, StripTagsFilter, XmlEntitiesFilter)
cargo run -p walrs_filter --example basic_filters

# Chaining multiple filters together
cargo run -p walrs_filter --example filter_chain

# Fallible filters with TryFilterOp
cargo run -p walrs_filter --example try_filters

# FilterOp enum: all variants, serialization, numeric clamping
cargo run -p walrs_filter --example filter_op_usage
```

## Running Benchmarks

Benchmarks are available to measure filter performance:

```bash
# Run all benchmarks
cargo bench -p walrs_filter

# Run specific benchmark group
cargo bench -p walrs_filter -- SlugFilter
cargo bench -p walrs_filter -- StripTagsFilter
cargo bench -p walrs_filter -- XmlEntitiesFilter
cargo bench -p walrs_filter -- FilterOp_Chain
cargo bench -p walrs_filter -- FilterOp_Clamp
cargo bench -p walrs_filter -- TryFilterOp
cargo bench -p walrs_filter -- FilterOp_Value
```

Benchmark groups include:
- **SlugFilter** - Tests slug generation with various input sizes
- **StripTagsFilter** - Tests HTML sanitization with different HTML complexity
- **XmlEntitiesFilter** - Tests XML entity encoding
- **FilterComparison** - Compares performance across all filters
- **FilterOp_noop_vs_mutation** - Zero-copy noop vs mutating apply_ref for all string variants
- **FilterOp_Chain** - Composition overhead for 1, 3, and 5-filter chains
- **FilterOp_Clamp** - Numeric clamping performance (i32, f64, in-range and out-of-range)
- **TryFilterOp** - Fallible pipeline overhead (Infallible wrapping, Chain, TryCustom)
- **FilterOp_Value** - Dynamic dispatch performance with `walrs_validation::Value`

## License

MIT & Apache-2.0

