//! # walrs_inputfilter
//!
//! Field-level validation and filtering for form processing.
//!
//! This crate provides:
//!
//! - [`Field`] - Unified validation configuration
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
//! use walrs_validation::Value;
//! use serde_json::json;
//!
//! // Create a field with filters and rule (use Rule::All for multiple rules)
//! let email_field = FieldBuilder::<String>::default()
//!     .name("email".to_string())
//!     .filters(vec![FilterEnum::Trim, FilterEnum::Lowercase])
//!     .rule(Rule::Required.and(Rule::Email(Default::default())))
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

pub mod field;
pub mod field_filter;
pub mod filter_enum;
pub mod form_violations;

pub mod filters;
pub mod rule;
pub mod traits;
pub mod validators;

// Re-export types from walrs_validation
pub use walrs_validation::{
  Attributes, CompiledRule, IsEmpty, Message, MessageContext, MessageParams, Value, ValueExt,
  Violation, ViolationMessage, ViolationType, Violations,
};

pub use field::{Field, FieldBuilder};
pub use field_filter::{CrossFieldRule, CrossFieldRuleType, FieldFilter};
pub use filter_enum::Filter;
pub use form_violations::FormViolations;

pub use filters::*;
pub use rule::{Condition, Rule, RuleResult};
pub use validators::*;
