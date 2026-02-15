#![cfg_attr(feature = "fn_traits", feature(fn_traits))]
#![cfg_attr(feature = "fn_traits", feature(unboxed_closures))]
#![cfg_attr(feature = "debug_closure_helpers", feature(debug_closure_helpers))]

#[macro_use]
extern crate derive_builder;

pub mod filters;
pub mod input;
pub(crate) mod input_common;
pub mod ref_input;
pub mod rule;
pub mod traits;
pub mod validators;

// Re-export violation types from walrs_validator for backwards compatibility
pub use walrs_validator::{Violation, Violations, ViolationType, ViolationMessage};

pub use filters::*;
pub use input::*;
pub use ref_input::*;
pub use rule::{Condition, Rule, RuleResult};
pub use traits::*;
pub use validators::*;


