# walrs_filter

Filter/transformation structs for input filtering.

This crate provides reusable filter implementations that can transform input values. Filters are typically used in form processing pipelines to sanitize, normalize, or transform user input before, or after, validation.

## Available Filters

- **`SlugFilter`** - Converts strings to URL-friendly slugs.
- **`StripTagsFilter`** - Removes/sanitizes HTML tags using [Ammonia](https://docs.rs/ammonia).
- **`XmlEntitiesFilter`** - Encodes special characters as XML entities.

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

All filters implement the `Filter<T>` trait:

```rust
pub trait Filter<T> {
    type Output;
    fn filter(&self, value: T) -> Self::Output;
}
```

This allows filters to transform values, potentially to different types.

## Features

- **`fn_traits`** - Enables nightly for `Fn` trait implementations when you want filters that can be called as functions.
- **`nightly`** - Catch all feature - enables any nightly features available in the crate, currently only 'fn_trait' one.

## License

MIT OR Apache-2.0
