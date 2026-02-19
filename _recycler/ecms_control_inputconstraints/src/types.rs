use std::fmt::{Debug, Display};

pub type Message = String;
pub type ValidationResultTuple = (ValidationResultEnum, Message);
pub type ValidationResult = Result<(), ValidationResultTuple>;
pub type Validator<'a, T> =
  &'a (dyn Fn(T) -> Option<Vec<(ValidationResultEnum, Message)>> + Send + Sync);

pub trait InputConstraints<T: Clone + Debug + Display + PartialEq>: Debug {
  fn validate(&self, x: Option<T>) -> Result<(), ValidationResultTuple>;
}

#[derive(PartialEq, Debug)]
pub enum ValidationResultEnum {
  CustomError,
  PatternMismatch,
  RangeOverflow,
  RangeUnderflow,
  StepMismatch,
  TooLong,
  TooShort,

  /// When value is in invalid format, and not validated against `pattern` (email, url, etc.)
  /// - currently unused.
  TypeMismatch,
  Valid,
  ValueMissing,
}
