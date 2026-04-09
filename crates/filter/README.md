# walrs_filter

Filter/transformation structs for input filtering.

This crate provides reusable filter implementations that can transform input values. Filters are typically used in form processing pipelines to sanitize, normalize, or transform user input before, or after, validation.

## Available Filters

- **`SlugFilter`** - Converts strings to URL-friendly slugs.
- **`StripTagsFilter`** - Removes/sanitizes HTML tags using [Ammonia](https://docs.rs/ammonia).
- **`XmlEntitiesFilter`** - Encodes special characters as XML entities.

## FilterOp Enum

The `FilterOp<T>` enum provides a composable, serializable way to define filter
operations for config-driven form processing. It delegates to the filter structs above.

Available operations:
- `Trim` - Remove whitespace
- `Uppercase` / `Lowercase` - Case transformation
- `StripTags` - Remove HTML tags
- `HtmlEntities` - Encode XML/HTML entities
- `Slug` - URL-safe slug generation
- `Clamp(min, max)` - Numeric clamping
- `Chain(ops)` - Sequential filter chain
- `Custom(fn)` - Custom filter function

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
}
```

When the `validation` feature is enabled (default), `FilterOp<Value>` is available
for dynamic value transformation using `walrs_validation::Value`.

## TryFilterOp Enum (Fallible Filters)

The `TryFilterOp<T>` enum is the fallible counterpart to `FilterOp<T>`. Use it for
filters that can legitimately fail (e.g., base64 decode, JSON parse, URL decode).
Errors are represented as [`FilterError`](#filtererror), which integrates with the
validation error pipeline.

Available variants:
- `Infallible(FilterOp<T>)` - Wraps an infallible filter, lifting it into the fallible pipeline
- `Chain(Vec<TryFilterOp<T>>)` - Sequential filter chain that short-circuits on the first error
- `TryCustom(Arc<dyn Fn(T) -> Result<T, FilterError>>)` - Custom fallible filter function

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

## The TryFilter Trait

The `TryFilter<T>` trait is the fallible counterpart to [`Filter<T>`](#the-filter-trait):

```rust
pub trait TryFilter<T> {
    type Output;
    fn try_filter(&self, value: T) -> Result<Self::Output, FilterError>;
}
```

This allows implementing custom fallible filter structs that integrate with the
`TryFilterOp` pipeline.

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

## The Filter Trait

All filter structs implement the `Filter<T>` trait:

```rust
pub trait Filter<T> {
    type Output;
    fn filter(&self, value: T) -> Self::Output;
}
```

This allows filters to transform values, potentially to different types.

## Features

- **`validation`** (default) - Enables `FilterOp<Value>`, `TryFilterOp<Value>`, and `FilterError` → `Violation` conversions via `walrs_validation`.
- **`fn_traits`** - Enables nightly for `Fn` trait implementations when you want filters that can be called as functions.
- **`nightly`** - Catch all feature - enables any nightly features available in the crate, currently only 'fn_trait' one.

## Running Examples

The crate includes several examples demonstrating filter usage:

```bash
# Basic filter usage (SlugFilter, StripTagsFilter, XmlEntitiesFilter)
cargo run -p walrs_filter --example basic_filters

# Chaining multiple filters together
cargo run -p walrs_filter --example filter_chain

# Fallible filters with TryFilterOp
cargo run -p walrs_filter --example try_filters
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
```

Benchmark groups include:
- **SlugFilter** - Tests slug generation with various input sizes
- **StripTagsFilter** - Tests HTML sanitization with different HTML complexity
- **XmlEntitiesFilter** - Tests XML entity encoding
- **FilterComparison** - Compares performance across all filters

## License

MIT & Apache-2.0
