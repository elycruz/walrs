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

fn main() {
    let op = FilterOp::<String>::Chain(vec![
        FilterOp::Trim,
        FilterOp::Lowercase,
    ]);
    // apply_ref accepts &str directly — no allocation needed
    assert_eq!(op.apply_ref("  HELLO  "), "hello");
    // apply accepts an owned String (delegates to apply_ref)
    assert_eq!(op.apply("  HELLO  ".to_string()), "hello");
}
```

When the `validation` feature is enabled (default), `FilterOp<Value>` is available
for dynamic value transformation using `walrs_validation::Value`.

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

- **`validation`** (default) - Enables `FilterOp<Value>` support via `walrs_validation`.
- **`fn_traits`** - Enables nightly for `Fn` trait implementations when you want filters that can be called as functions.
- **`nightly`** - Catch all feature - enables any nightly features available in the crate, currently only 'fn_trait' one.

## Running Examples

The crate includes several examples demonstrating filter usage:

```bash
# Basic filter usage (SlugFilter, StripTagsFilter, XmlEntitiesFilter)
cargo run -p walrs_filter --example basic_filters

# Chaining multiple filters together
cargo run -p walrs_filter --example filter_chain
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
