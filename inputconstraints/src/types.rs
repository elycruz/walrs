use crate::number::NumberInputConstraints;
use crate::text::TextInputConstraints;
use std::fmt::{Debug, Display};

pub type ValidationMessage = String;
pub type ValidationResultError = (ValidationResultEnum, ValidationMessage);
pub type ValidationResult = Result<(), ValidationResultError>;

pub trait InputConstraints<T: Clone + Debug + Display + PartialEq>: Debug {
  fn validate(&self, x: Option<T>) -> Result<(), ValidationResultError>;
}

pub enum Constraints<'a, T> {
  TextInput(TextInputConstraints<'a>),
  NumberInput(NumberInputConstraints<'a, T>),
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
  TypeMismatch, // When value is in invalid format, and not validated against `pattern` (email, url, etc.) - currently unused.
  Valid,
  ValueMissing,
}
