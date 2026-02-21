//! # walrs_filter
//!
//! Filter/transformation structs for input filtering.
//!
//! This crate provides reusable filter implementations that can transform
//! input values. Filters are typically used in form processing pipelines
//! to sanitize, normalize, or transform user input before validation.
//!
//! ## Available Filters
//!
//! - [`SlugFilter`] - Converts strings to URL-friendly slugs
//! - [`StripTagsFilter`] - Removes/sanitizes HTML tags using Ammonia
//! - [`XmlEntitiesFilter`] - Encodes special characters as XML entities
//!
//! ## Example
//!
//! ```rust
//! use walrs_filter::{Filter, SlugFilter, StripTagsFilter};
//! use std::borrow::Cow;
//!
//! // Create a slug from a title
//! let slug_filter = SlugFilter::new(200, false);
//! let slug = slug_filter.filter(Cow::Borrowed("Hello World!"));
//! assert_eq!(slug, "hello-world");
//!
//! // Strip HTML tags
//! let strip_filter = StripTagsFilter::new();
//! let clean = strip_filter.filter(Cow::Borrowed("<script>alert('xss')</script>Hello"));
//! assert_eq!(clean, "Hello");
//! ```

#![cfg_attr(feature = "fn_traits", feature(fn_traits))]
#![cfg_attr(feature = "fn_traits", feature(unboxed_closures))]

#[macro_use]
extern crate derive_builder;

pub mod slug;
pub mod strip_tags;
pub mod traits;
pub mod xml_entities;

pub use slug::*;
pub use strip_tags::*;
pub use traits::*;
pub use xml_entities::*;
