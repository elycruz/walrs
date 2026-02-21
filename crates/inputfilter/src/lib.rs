#![cfg_attr(feature = "fn_traits", feature(fn_traits))]
#![cfg_attr(feature = "fn_traits", feature(unboxed_closures))]
#![cfg_attr(feature = "debug_closure_helpers", feature(debug_closure_helpers))]

//! # walrs_inputfilter
//!
//! Field-level validation and filtering for form processing.
//!
//! This crate provides:
//!
//! - [`Field`] - Unified validation configuration (replaces old `Input`/`RefInput`)
//! - [`FieldFilter`] - Multi-field validation with cross-field rules
//! - [`Filter`] - Serializable filter enum for value transformation
//! - [`FormViolations`] - Collection of form-level validation errors
//!
//! ## Example
//!
//! ```rust
//! use walrs_inputfilter::{Field, FieldBuilder, Filter, FieldFilter};
//! use walrs_inputfilter::filter_enum::Filter as FilterEnum;
//! use walrs_validation::Rule;
//! use walrs_form_core::Value;
//! use serde_json::json;
//!
//! // Create a field with filters and rule (use Rule::All for multiple rules)
//! let email_field = FieldBuilder::<String>::default()
//!     .name("email".to_string())
//!     .filters(vec![FilterEnum::Trim, FilterEnum::Lowercase])
//!     .rule(Rule::Required.and(Rule::Email))
//!     .build()
//!     .unwrap();
//!
//! // Process (filter + validate) a value
//! let result = email_field.process("  TEST@EXAMPLE.COM  ".to_string());
//! assert!(result.is_ok());
//! assert_eq!(result.unwrap(), "test@example.com");
//! ```

#[macro_use]
extern crate derive_builder;

// New unified API (Step 2-3)
pub mod field;
pub mod field_filter;
pub mod filter_enum;
pub mod form_violations;

// Legacy modules (to be deprecated)
pub mod filters;
pub mod input;
pub(crate) mod input_common;
pub mod ref_input;
pub mod rule;
pub mod traits;
pub mod validators;

// Re-export types from walrs_validation for backwards compatibility
pub use walrs_validation::{
  CompiledRule, IsEmpty, Message, MessageContext, MessageParams, Violation, ViolationMessage,
  ViolationType, Violations,
};

// Re-export types from walrs_form_core
pub use walrs_form_core::{Attributes, Value, ValueExt};

// New unified API exports
pub use field::{Field, FieldBuilder};
pub use field_filter::{CrossFieldRule, CrossFieldRuleType, FieldFilter};
pub use filter_enum::Filter;
pub use form_violations::FormViolations;

// Legacy exports
pub use filters::*;
pub use input::*;
pub use ref_input::*;
pub use rule::{Condition, Rule, RuleResult};
pub use traits::*;
pub use validators::*;
