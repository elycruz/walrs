//! # walrs_validation
//!
//! Validator structs for input validation.
//!
//! This crate provides reusable validator implementations that can validate
//! input values against various constraints. Validators are typically used
//! in form processing pipelines to ensure user input meets requirements.
//!
//! ## Available Validators
//!
//! - [`LengthValidator`] - Validates string/collection length constraints
//! - [`PatternValidator`] - Validates strings against regex patterns
//! - [`RangeValidator`] - Validates scalar values within a range (numbers, chars, etc.)
//! - [`StepValidator`] - Validates that numeric values are multiples of a step
//! - [`EqualityValidator`] - Validates equality against a specified value
//!
//! ## Combinators
//!
//! Validators can be combined using logical operations:
//! - [`ValidatorAnd`] - Both validators must pass (AND logic)
//! - [`ValidatorOr`] - At least one validator must pass (OR logic)
//! - [`ValidatorNot`] - Negates a validator
//! - [`ValidatorOptional`] - Skips validation for empty values
//! - [`ValidatorWhen`] - Conditional validation
//! - [`ValidatorAll`] - Collects all validation errors
//!
//! ## Example
//!
//! ```rust
//! use walrs_validation::{
//!     LengthValidatorBuilder, RangeValidatorBuilder,
//!     Validate, ValidateRef, ValidateExt,
//! };
//!
//! // Length validation
//! let length_validator = LengthValidatorBuilder::<str>::default()
//!     .min_length(3)
//!     .max_length(20)
//!     .build()
//!     .unwrap();
//!
//! assert!(length_validator.validate_ref("hello").is_ok());
//! assert!(length_validator.validate_ref("hi").is_err());
//!
//! // Range validation with combinators
//! let min_validator = RangeValidatorBuilder::<i32>::default()
//!     .min(0)
//!     .build()
//!     .unwrap();
//!
//! let max_validator = RangeValidatorBuilder::<i32>::default()
//!     .max(100)
//!     .build()
//!     .unwrap();
//!
//! let range_validator = min_validator.and(max_validator);
//! assert!(range_validator.validate(50).is_ok());
//! assert!(range_validator.validate(-1).is_err());
//! ```

#![cfg_attr(feature = "fn_traits", feature(fn_traits))]
#![cfg_attr(feature = "fn_traits", feature(unboxed_closures))]
#![cfg_attr(feature = "debug_closure_helpers", feature(debug_closure_helpers))]

#[macro_use]
extern crate derive_builder;

pub mod attributes;
pub mod combinators;
pub mod equal;
pub mod fn_validator;
pub mod impls;
pub mod length;
pub mod message;
pub mod pattern;
pub mod range;
pub mod rule;
pub mod step;
pub mod traits;
pub mod value;
pub mod violation;

pub use attributes::*;
pub use combinators::*;
pub use equal::*;
pub use fn_validator::*;
pub use length::*;
pub use message::*;
pub use pattern::*;
pub use range::*;
pub use rule::{CompiledRule, Condition, Rule, RuleResult};
pub use step::*;
pub use traits::*;
pub use value::*;
pub use violation::*;
