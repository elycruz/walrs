use crate::ViolationMessage;
use std::fmt;
use std::fmt::{Debug, Display};

pub type ValidatorForSized<T> = dyn Fn(T) -> Result<(), Violation> + Send + Sync;
pub type ValidatorForRef<T> = dyn Fn(&T) -> Result<(), Violation> + Send + Sync;

#[derive(Clone, PartialEq, Debug)]
pub enum ViolationType {
  CustomError,
  PatternMismatch,
  RangeOverflow,
  RangeUnderflow,
  StepMismatch,
  TooLong,
  TooShort,
  NotEqual,
  TypeMismatch,
  ValueMissing,
}

// @todo Implement `Error` for this struct.
#[derive(Clone, PartialEq, Debug)]
pub struct Violation(pub ViolationType, pub ViolationMessage);

/// `Display` (and `ToString` (which we get for free)) impl for `Violation`.
///
/// ```rust
/// use walrs_inputfilter::{ViolationType::ValueMissing, Violation};
///
/// let violation = Violation(ValueMissing, "Value missing".to_string());
/// let displayed = format!("{}", violation);
///
/// assert_eq!(&displayed, "Value missing");
///
/// // `Display` impl, gives us `to_string()` for free:
/// assert_eq!(&violation.to_string(), "Value missing");
/// ```
impl Display for Violation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.1)
  }
}

impl std::ops::Deref for Violation {
  type Target = ViolationMessage;

  fn deref(&self) -> &Self::Target {
    &self.1
  }
}

impl std::ops::DerefMut for Violation {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.1
  }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Violations(pub Vec<Violation>);

impl std::ops::Deref for Violations {
  type Target = Vec<Violation>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for Violations {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Into<Vec<Violation>> for Violations {
  fn into(self) -> Vec<Violation> {
    self.0
  }
}

impl Into<Vec<String>> for Violations {
  fn into(self) -> Vec<String> {
    self.0.into_iter().map(|violation| violation.1).collect()
  }
}

pub type ValidationResult2 = Result<(), Violations>;


/// A trait for performing validations, and filtering (transformations), all in one.
pub trait InputFilterForSized<T, FT = T>: Display + Debug
where
  T: Copy,
  FT: From<T>,
{
  fn validate(&self, x: T) -> ValidationResult2;
  
  fn validate_option(&self, x: Option<T>) -> ValidationResult2;
  
  /// Validates, and filters, incoming value.
  fn filter(&self, value: T) -> Result<FT, Violations>;

  /// Validates, and filters, incoming value Option value.
  fn filter_option(&self, value: Option<T>) -> Result<Option<FT>, Violations>;
}

/// A trait for performing validations, and filtering (transformations), all in one,
/// for unsized types.
pub trait InputFilterForUnsized<'a, T, FT>: Display + Debug
where
  T: ?Sized + 'a,
  FT: From<&'a T>,
{
  fn validate_ref(&self, x: &T) -> ValidationResult2;
  
  fn validate_ref_option(&self, x: Option<&T>) -> ValidationResult2;
  
  fn filter(&self, value: &'a T) -> Result<FT, Violations>;

  fn filter_option(&self, value: Option<&'a T>) -> Result<Option<FT>, Violations>;
}

#[cfg(test)]
mod test {
  use super::ViolationType::ValueMissing;
  use super::*;

  #[test]
  fn test_violation_to_string() {
    let v = Violation(ValueMissing, "value is missing.".to_string());
    assert_eq!(&v.to_string(), "value is missing.");
  }

  #[test]
  fn test_violation_debug() {
    let v = Violation(ValueMissing, "value is missing.".to_string());
    assert_eq!(
      format!("{:?}", v),
      "Violation(ValueMissing, \"value is missing.\")"
    );
  }

  #[test]
  fn test_violation_display() {
    let v = Violation(ValueMissing, "value is missing.".to_string());
    assert_eq!(format!("{:}", v), "value is missing.");
  }
}
