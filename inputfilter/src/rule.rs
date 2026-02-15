//! # Rule Enum - Composable Validation Rules
//!
//! This module re-exports the composable validation rule system from `walrs_validator`.
//! Rules can be combined using tree composition (`All`, `Any`, `Not`, `When`)
//! and support both built-in validation types and custom closures.
//!
//! ## Example
//!
//! ```rust
//! use walrs_inputfilter::rule::{Rule, Condition};
//!
//! // Simple rules
//! let min_length = Rule::<String>::MinLength(3);
//! let max_length = Rule::<String>::MaxLength(50);
//!
//! // Composed rule using combinators
//! let length_rule = min_length.and(max_length);
//!
//! // Conditional rule
//! let conditional = Rule::<String>::When {
//!     condition: Condition::IsNotEmpty,
//!     then_rules: vec![Rule::MinLength(5)],
//!     else_rules: None,
//! };
//! ```

// Re-export everything from walrs_validator::rule
pub use walrs_validator::rule::{Condition, Rule, RuleResult};
pub use walrs_validator::{Message, MessageContext, MessageParams};

