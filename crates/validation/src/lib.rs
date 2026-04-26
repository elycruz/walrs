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
//! - `Rule::ExactLength` - Exact length constraint
//! - `Rule::Min` / `Rule::Max` - Numeric range constraints
//! - `Rule::Range` - Inclusive range constraint (min and max together)
//! - `Rule::Equals` - Exact value match
//! - `Rule::OneOf` - Value must be one of an allowed set
//! - `Rule::Pattern` - Regex pattern matching
//! - `Rule::Email` - Configurable email validation (DNS/IP/local domains, local part length)
//! - `Rule::Url` - Configurable URL validation (scheme filtering)
//! - `Rule::Uri` - Configurable URI validation (scheme, relative/absolute)
//! - `Rule::Ip` - Configurable IP address validation (IPv4/IPv6/IPvFuture)
//! - `Rule::Hostname` - Configurable hostname validation (DNS/IP/local/public IPv4)
//! - `Rule::Date` - Configurable date format validation (ISO 8601, US, EU, custom)
//! - `Rule::DateRange` - Date range validation with min/max bounds
//! - `Rule::Step` - Step/multiple validation
//! - `Rule::Custom` - Custom closure-based validation
//! - `Rule::CustomAsync` - Async custom closure-based validation (requires `async` feature)
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
//!
//! ## `Option<T>` Validation
//!
//! `Rule<T>` implements `Validate<Option<T>>` and `ValidateRef<Option<T>>`,
//! allowing direct validation of optional values:
//!
//! - `None` with `Rule::Required` → `Err(Violation::value_missing())`
//! - `None` without `Required` → `Ok(())`
//! - `Some(v)` → delegates to the inner `Validate<T>` / `ValidateRef<T>` impl
//!
//! ```rust
//! use walrs_validation::{Rule, Validate, ValidateRef};
//!
//! let rule = Rule::<String>::Required.and(Rule::MinLength(3));
//!
//! // None fails because the rule includes Required
//! assert!(rule.validate(None::<String>).is_err());
//!
//! // Some delegates to inner validation
//! assert!(rule.validate(Some("hello".to_string())).is_ok());
//! assert!(rule.validate(Some("hi".to_string())).is_err());
//!
//! // ValidateRef works with references
//! assert!(rule.validate_ref(&None::<String>).is_err());
//! assert!(rule.validate_ref(&Some("hello".to_string())).is_ok());
//!
//! // Without Required, None is accepted
//! let optional_rule = Rule::<i32>::Min(0).and(Rule::Max(100));
//! assert!(optional_rule.validate(None::<i32>).is_ok());
//! assert!(optional_rule.validate(Some(50)).is_ok());
//! ```
//!
//! ## Deprecated: dynamic `Value` path
//!
//! The dynamic `Value` enum and its `Rule<Value>` / `Condition<Value>` impls
//! are deprecated as of 0.2.0 and will be removed in the next major release.
//! Use `#[derive(Fieldset)]` (from `walrs_fieldset_derive`) on a typed struct
//! instead. See [issue #267](https://github.com/elycruz/walrs/issues/267).

pub use indexmap;

pub mod attributes;
pub mod fieldset_violations;
pub mod message;
pub mod options;
pub mod rule;
pub(crate) mod rule_impls;
pub mod traits;
#[cfg(feature = "value")]
#[cfg_attr(docsrs, doc(cfg(feature = "value")))]
pub mod value;
pub mod violation;

pub use attributes::*;
pub use fieldset_violations::*;
pub use message::*;
pub use options::*;
pub use rule::{CompiledPattern, Condition, Rule, RuleResult};
pub use traits::*;
#[cfg(feature = "value")]
#[cfg_attr(docsrs, doc(cfg(feature = "value")))]
#[allow(deprecated)]
pub use value::*;
pub use violation::*;
