//! # walrs_validation
//!
//! Composable validation rules for input validation.
//!
//! This crate provides a serializable, composable validation rule system based on
//! the [`Rule`] enum, along with core validation traits.
//!
//! ## Validation Rules
//!
//! The [`Rule`] enum provides built-in validation for common constraints:
//! - `Rule::Required` - Value must not be empty
//! - `Rule::MinLength` / `Rule::MaxLength` - Length constraints
//! - `Rule::Min` / `Rule::Max` - Range constraints
//! - `Rule::Pattern` - Regex pattern matching
//! - `Rule::Email` - Email format validation
//! - `Rule::Step` - Step/multiple validation
//! - `Rule::Custom` - Custom closure-based validation
//!
//! ## Rule Composition
//!
//! Rules can be composed using methods on [`Rule`]:
//! - `.and()` - Both rules must pass (AND logic, produces `Rule::All`)
//! - `.or()` - At least one rule must pass (OR logic, produces `Rule::Any`)
//! - `.not()` - Negates a rule (produces `Rule::Not`)
//! - `.when()` / `.when_else()` - Conditional validation (produces `Rule::When`)
//!
//! ## Example
//!
//! ```rust
//! use walrs_validation::{Rule, Validate, ValidateRef};
//!
//! // Length validation using Rule
//! let length_rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(20));
//!
//! assert!(length_rule.validate_ref("hello").is_ok());
//! assert!(length_rule.validate_ref("hi").is_err());
//!
//! // Range validation with combinators
//! let range_rule = Rule::<i32>::Min(0).and(Rule::Max(100));
//!
//! assert!(range_rule.validate(50).is_ok());
//! assert!(range_rule.validate(-1).is_err());
//! ```

pub mod attributes;
pub(crate) mod rule_impls;
pub mod message;
pub mod rule;
pub mod traits;
pub mod value;
pub mod violation;

pub use attributes::*;
pub use message::*;
pub use rule::{CompiledRule, Condition, Rule, RuleResult};
pub use traits::*;
pub use value::*;
pub use violation::*;
