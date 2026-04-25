//! # walrs_fieldfilter
//!
//! Field-level validation and filtering for form processing.
//!
//! This crate provides:
//!
//! - [`Fieldset`] - Typed struct validation and filtering (recommended for new code)
//! - `FieldsetAsync` - Async version of `Fieldset` (behind `async` feature)
//! - [`Field`] - Unified validation configuration
//! - [`FieldFilter`] - Multi-field validation with cross-field rules
//! - [`FilterOp`] - Serializable filter enum for value transformation (re-exported from `walrs_filter`)
//! - [`TryFilterOp`] - Fallible filter enum for transformations that can fail (re-exported from `walrs_filter`)
//! - [`FilterError`] - Error type for fallible filters (re-exported from `walrs_filter`)
//!
//! ## Example
//!
//! ```rust
//! use walrs_fieldfilter::{Field, FieldBuilder, FieldFilter, TryFilterOp, FilterError};
//! use walrs_filter::FilterOp;
//! use walrs_validation::Rule;
//! use walrs_validation::Value;
//! use serde_json::json;
//! use std::sync::Arc;
//!
//! // Create a field with filters and rule (use Rule::All for multiple rules)
//! let email_field = FieldBuilder::<String>::default()
//!     .name("email".to_string())
//!     .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
//!     .rule(Rule::Required.and(Rule::Email(Default::default())))
//!     .build()
//!     .unwrap();
//!
//! // Clean (filter + validate) a value
//! let result = email_field.clean("  TEST@EXAMPLE.COM  ".to_string());
//! assert!(result.is_ok());
//! assert_eq!(result.unwrap(), "test@example.com");
//!
//! // Field with fallible filters
//! let encoded_field = FieldBuilder::<String>::default()
//!     .name("data".to_string())
//!     .try_filters(vec![
//!         TryFilterOp::TryCustom(Arc::new(|s: String| {
//!             if s.contains('\0') {
//!                 Err(FilterError::new("null bytes not allowed"))
//!             } else {
//!                 Ok(s)
//!             }
//!         })),
//!     ])
//!     .build()
//!     .unwrap();
//!
//! assert!(encoded_field.clean("hello".to_string()).is_ok());
//! assert!(encoded_field.clean("bad\0input".to_string()).is_err());
//! ```
//!
//! ## Typed ↔ Dynamic interop
//!
//! Two parallel processing paths exist; pick by where your data shape is known:
//!
//! - **Typed path — [`Fieldset`]** (recommended for new code). When fields
//!   are known at compile time, define a struct and either implement
//!   [`Fieldset`] manually or use `#[derive(Fieldset)]` (behind the `derive`
//!   feature, re-exported as [`DeriveFieldset`]). You get statically-checked
//!   fields, native Rust types, and per-field `clean()` semantics.
//!
//! - **Dynamic path — [`FieldFilter`]**. When fields are not known until
//!   runtime (e.g., user-defined forms, schema-driven payloads), build a
//!   `FieldFilter` from `Field<Value>` entries plus optional
//!   [`CrossFieldRule`]s. The form data travels as `walrs_form::FormData`
//!   (an ordered `IndexMap<String, Value>`).
//!
//! The two paths are bridged by the derive macro's optional attributes:
//!
//! ```ignore
//! #[derive(Fieldset)]
//! #[fieldset(into_form_data, try_from_form_data)]
//! struct Signup { /* ... */ }
//! ```
//!
//! - `#[fieldset(into_form_data)]` generates `impl From<&T> for walrs_form::FormData`.
//! - `#[fieldset(try_from_form_data)]` generates
//!   `impl TryFrom<walrs_form::FormData> for T` returning
//!   [`FieldsetViolations`] on shape errors.
//!
//! Together they let an HTTP layer that speaks dynamic `FormData` round-trip
//! through a typed struct: `FormData → T → clean() → FormData`. See the
//! runnable `derive_formdata_bridge` example in the `walrs_form` crate
//! (`cargo run --example derive_formdata_bridge -p walrs_form`).

#[macro_use]
extern crate derive_builder;

pub mod field;
pub mod field_filter;
pub mod fieldset;

pub mod rule;

// Re-export IndexMap for consumers
pub use indexmap::IndexMap;

// Re-export types from walrs_validation
pub use walrs_validation::{
  Attributes, FieldsetViolations, IsEmpty, Message, MessageContext, MessageParams, Value, ValueExt,
  Violation, ViolationMessage, ViolationType, Violations,
};

#[cfg(feature = "async")]
pub use walrs_validation::{ValidateAsync, ValidateRefAsync};

// Re-export FilterOp and TryFilterOp from walrs_filter
pub use walrs_filter::{FilterError, FilterOp, TryFilterOp};

pub use field::{Field, FieldBuilder};
pub use field_filter::{CrossFieldRule, CrossFieldRuleType, FieldFilter};
pub use fieldset::Fieldset;
#[cfg(feature = "derive")]
pub use walrs_fieldset_derive::Fieldset as DeriveFieldset;

#[cfg(feature = "async")]
pub use fieldset::FieldsetAsync;

pub use rule::{Condition, Rule, RuleResult};
