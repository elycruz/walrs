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
//! ## FilterOp Enum
//!
//! The [`FilterOp`] enum provides a composable, serializable way to define
//! filter operations for config-driven form processing.
//!
//! ## Example
//!
//! ```rust
//! use walrs_filter::{Filter, SlugFilter, StripTagsFilter, FilterOp};
//! use std::borrow::Cow;
//!
//! // Use filter structs directly via the Filter trait
//! let slug_filter = SlugFilter::new(200, false);
//! let slug = slug_filter.filter(Cow::Borrowed("Hello World!"));
//! assert_eq!(slug, "hello-world");
//!
//! // Use FilterOp for composable, serializable filter pipelines
//! let op = FilterOp::<String>::Chain(vec![
//!     FilterOp::Trim,
//!     FilterOp::Lowercase,
//! ]);
//! // apply_ref accepts &str directly — no allocation needed
//! assert_eq!(op.apply_ref("  HELLO  "), "hello");
//! // apply accepts an owned String (delegates to apply_ref)
//! assert_eq!(op.apply("  HELLO  ".to_string()), "hello");
//! ```

#![cfg_attr(feature = "fn_traits", feature(fn_traits))]
#![cfg_attr(feature = "fn_traits", feature(unboxed_closures))]

#[macro_use]
extern crate derive_builder;

pub mod filter_op;
pub mod slug;
pub mod strip_tags;
pub mod traits;
pub mod xml_entities;

pub use filter_op::*;
pub use slug::*;
pub use strip_tags::*;
pub use traits::*;
pub use xml_entities::*;
