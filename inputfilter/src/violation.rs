use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

/// A validation violation message.
pub type ViolationMessage = String;

#[derive(Clone, PartialEq, Debug, Copy)]
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

impl Error for Violation {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
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

impl From<Violations> for Vec<Violation> {
  fn from(val: Violations) -> Self {
    val.0
  }
}

impl From<Violations> for Vec<String> {
  fn from(val: Violations) -> Self {
    val.0.into_iter().map(|violation| violation.1).collect()
  }
}

impl Violations {
  pub fn to_string_vec(self) -> Vec<String> {
    self.into()
  }
}

// @todo `Violations` should implement `Error`.

#[cfg(test)]
mod test {
  use super::{*, ViolationType::ValueMissing};

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
