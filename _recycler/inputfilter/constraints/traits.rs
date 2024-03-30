use std::fmt::{Debug, Display};
use crate::{InputValue, ViolationMessage, ViolationTuple, ValidationResult};

pub type Filter<T> = dyn Fn(T) -> T + Send + Sync;

pub type Validator<T> = dyn Fn(T) -> ValidationResult + Send + Sync;

/// Violation message getter for `ValueMissing` Violation Enum type.
pub type ValueMissingCallback = dyn Fn() -> ViolationMessage + Send + Sync;

pub trait InputConstraints<'a, 'b, T: 'b, FT: 'b>: Display + Debug
    where T: InputValue {

    fn validate(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>>;

    fn validate_detailed(&self, value: Option<T>) -> Result<(), Vec<ViolationTuple>>;

    /// @todo Consider changing `value` type to just `FT` here - Makes method easier to use.
    fn filter(&self, value: Option<FT>) -> Option<FT>;

    fn validate_and_filter(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>>;

    fn validate_and_filter_detailed(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationTuple>>;
}

pub trait InputConstraints2<'a, 'b, T: 'b, FT: 'b>: Display + Debug
    where T: InputValue {

    fn validate(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>>;

    fn validate_detailed(&self, value: Option<T>) -> Result<(), Vec<ViolationTuple>>;

    fn filter(&self, value: FT) -> FT;

    fn validate_and_filter(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>>;

    fn validate_and_filter_detailed(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationTuple>>;
}
