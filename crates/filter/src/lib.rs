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
//! filter operations for config-driven form processing. In addition to
//! string/numeric transforms like `Trim`, `Lowercase`, and `Clamp`, it
//! exposes a suite of sanitize variants: `Digits`, `Alnum`, `Alpha`,
//! `StripNewlines`, `NormalizeWhitespace`, `AllowChars`, `DenyChars`, and
//! `UrlEncode`.
//!
//! ## TryFilterOp Enum
//!
//! The [`TryFilterOp`] enum provides a composable, serializable way to define
//! **fallible** filter operations. Use this for filters that can legitimately
//! fail (e.g., JSON parse, URL decode, type coercion). Errors are represented
//! as [`FilterError`], which can be converted to
//! [`Violation`](walrs_validation::Violation) for integration with the
//! validation error pipeline. Built-in fallible variants include `ToBool`,
//! `ToInt`, `ToFloat`, and `UrlDecode`.
//!
//! ## Example
//!
//! ```rust
//! use walrs_filter::{Filter, SlugFilter, StripTagsFilter, FilterOp, TryFilterOp, FilterError};
//! use std::borrow::Cow;
//! use std::sync::Arc;
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
//! // apply_ref returns Cow — zero-copy when input is unchanged
//! assert_eq!(op.apply_ref("  HELLO  "), "hello");
//! // apply accepts an owned String (delegates to apply_ref)
//! assert_eq!(op.apply("  HELLO  ".to_string()), "hello");
//!
//! // Use TryFilterOp for fallible filter pipelines
//! let try_op: TryFilterOp<String> = TryFilterOp::Chain(vec![
//!     TryFilterOp::Infallible(FilterOp::Trim),
//!     TryFilterOp::TryCustom(Arc::new(|s: String| {
//!         if s.is_empty() {
//!             Err(FilterError::new("value must not be empty after trimming"))
//!         } else {
//!             Ok(s)
//!         }
//!     })),
//! ]);
//! assert!(try_op.try_apply("  hello  ".to_string()).is_ok());
//! assert!(try_op.try_apply("     ".to_string()).is_err());
//! ```

#![cfg_attr(feature = "fn_traits", feature(fn_traits))]
#![cfg_attr(feature = "fn_traits", feature(unboxed_closures))]

#[macro_use]
extern crate derive_builder;

pub mod filter_error;
pub mod filter_op;
pub mod slug;
pub mod strip_tags;
pub mod traits;
pub mod try_filter_op;
pub mod xml_entities;

pub use filter_error::*;
pub use filter_op::*;
pub use slug::*;
pub use strip_tags::*;
pub use traits::*;
pub use try_filter_op::*;
pub use xml_entities::*;
