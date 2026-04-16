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
//! - [`FormViolations`] - (**Deprecated**: use [`FieldsetViolations`] instead)
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

#[macro_use]
extern crate derive_builder;

pub mod field;
#[allow(deprecated)]
pub mod field_filter;
pub mod fieldset;
#[allow(deprecated)]
pub mod form_violations;

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

#[allow(deprecated)]
pub use form_violations::FormViolations;
#[cfg(feature = "derive")]
pub use walrs_fieldset_derive::Fieldset as DeriveFieldset;

#[cfg(feature = "async")]
pub use fieldset::FieldsetAsync;

pub use rule::{Condition, Rule, RuleResult};
