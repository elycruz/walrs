use crate::ViolationMessage;
use std::fmt;
use std::fmt::{Debug, Display};

pub type ValidatorForSized<T> = dyn Fn(T) -> ValidationResult2 + Send + Sync;
pub type ValidatorForRef<T> = dyn Fn(&T) -> ValidationResult2 + Send + Sync;

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

/// Type for representing validation errors.
///
/// ```rust
/// use walrs_inputfilter::{
///   ValidationErrType,
///   ViolationType::{ValueMissing},
///   Violation
/// };
///
/// fn returns_validation_err() -> ValidationErrType {
///   ValidationErrType::Element(vec![Violation(ValueMissing, "Value missing".to_string())])
/// }
///
/// match returns_validation_err() {
///  ValidationErrType::Element(violations) => {
///     println!("The following violations occurred:");
///     for v in violations {
///       println!("- {}", v);
///     }
///   }
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum ValidationErrType {
  // Struct(HashMap<Box<str>, ValidationErrType>),
  // Collection(HashMap<Box<str>, ValidationErrType>),
  Element(Vec<Violation>),
  // Other(Box<dyn Error>),
}

impl ValidationErrType {
  pub fn extend(&mut self, other: ValidationErrType) {
    match (self, other) {
      // (ValidationErrType::Struct(ref mut self_errs), ValidationErrType::Struct(other_errs)) => {
      //     for (k, v) in other_errs {
      //         self_errs.insert(k, v);
      //     }
      // },
      // (ValidationErrType::Collection(ref mut self_errs), ValidationErrType::Collection(other_errs)) => {
      //     for (k, v) in other_errs {
      //         self_errs.insert(k, v);
      //     }
      // },
      (ValidationErrType::Element(self_errs), ValidationErrType::Element(other_errs)) => {
        self_errs.extend(other_errs);
      }
      // (ValidationErrType::Other(_), ValidationErrType::Other(err)) => {
      //     *self = ValidationErrType::Other(err);
      // }
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      // ValidationErrType::Struct(errs) => errs.is_empty(),
      // ValidationErrType::Collection(errs) => errs.is_empty(),
      ValidationErrType::Element(errs) => errs.is_empty(),
      // ValidationErrType::Other(_) => false,
    }
  }
}

pub type ValidationResult2 = Result<(), ValidationErrType>;

pub enum ValidationValue<T: Copy> {
  Struct(T),
  Collection(T),
  Element(T),
}

pub enum ValidationRefValue<'b, T: ?Sized> {
  // Struct(&'b T),
  // Collection(&'b T),
  Element(&'b T),
}

/// Deref for Validation Ref Value.
impl<'b, T: ?Sized> std::ops::Deref for ValidationRefValue<'b, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    match self {
      // ValidationRefValue::Struct(v) => v,
      // ValidationRefValue::Collection(v) => v,
      ValidationRefValue::Element(v) => v,
    }
  }
}

pub trait Validate<T: Copy> {
  fn validate(x: T) -> ValidationResult2;
}

pub trait ValidateOption<T: Copy> {
  fn validate_option(x: Option<T>) -> ValidationResult2;
}

pub trait ValidateRef<T: ?Sized> {
  fn validate_ref(&self, x: &T) -> ValidationResult2;
}

pub trait ValidateRefOption<T: ?Sized> {
  fn validate_ref_option(&self, x: Option<&T>) -> ValidationResult2;
}

pub trait Filter<T> {
  fn filter(&self, x: T) -> T;
}

// pub type InputFilterResult<T, FT> = Result<FT, ValidationErrType>;
// pub type InputFilterOptionResult<T, FT> = Result<Option<FT>, ValidationErrType>;

/// A trait for performing validations, and filtering (transformations), all in one.
pub trait InputFilterForSized<T, FT = T>: Display + Debug
where
  T: Copy,
  FT: From<T>,
{
  /// Validates, and filters, incoming value.
  fn filter(&self, value: T) -> Result<FT, ValidationErrType>;

  /// Validates, and filters, incoming value Option value.
  fn filter_option(&self, value: Option<T>) -> Result<Option<FT>, ValidationErrType>;
}

/// A trait for performing validations, and filtering (transformations), all in one,
/// for unsized types.
pub trait InputFilterForUnsized<'a, T, FT>: Display + Debug
where
  T: ?Sized + 'a,
  FT: From<&'a T>,
{
  fn filter(&self, value: &'a T) -> Result<FT, ValidationErrType>;

  fn filter_option(&self, value: Option<&'a T>) -> Result<Option<FT>, ValidationErrType>;
}

#[cfg(test)]
mod test {
  use super::ViolationType::ValueMissing;
  use super::*;
  use std::collections::HashMap;

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

  #[test]
  fn test_validation_err_type() {
    let mut struct_errs = HashMap::<Box<str>, ValidationErrType>::new();
    struct_errs.insert("hello".into(), ValidationErrType::Element(vec![]));

    // let _ = ValidationErrType::Struct(struct_errs);
    // let _ = ValidationErrType::Collection(HashMap::new());
    let _ = ValidationErrType::Element(vec![Violation(ValueMissing, "Value missing".to_string())]);
    // let _ = ValidationErrType::Other(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Some error occurred")));
  }
}
