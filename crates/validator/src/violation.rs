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
  // Signals invalid format for email, URL, IP address, and/or other formatted strings.
  TypeMismatch,
  ValueMissing,
}

#[must_use]
#[derive(Clone, PartialEq, Debug)]
pub struct Violation(pub ViolationType, pub ViolationMessage);

impl Violation {
  /// Creates a new `Violation` with the given type and message.
  pub fn new(violation_type: ViolationType, message: impl Into<String>) -> Self {
    Self(violation_type, message.into())
  }

  /// Returns the violation type.
  pub fn violation_type(&self) -> ViolationType {
    self.0
  }

  /// Returns a reference to the violation message.
  pub fn message(&self) -> &str {
    &self.1
  }

  /// Consumes the violation and returns the message.
  pub fn into_message(self) -> String {
    self.1
  }

  // ============================================================================
  // Violation Constructors
  // ============================================================================

  /// Value is required but was missing or empty.
  pub fn value_missing() -> Self {
    Self::new(ViolationType::ValueMissing, "Value is required.")
  }

  /// Value length is below the allowed minimum.
  pub fn too_short(min: usize, actual: usize) -> Self {
    Self::new(
      ViolationType::TooShort,
      format!("Value length must be at least {};  Received {}.", min, actual),
    )
  }

  /// Value length exceeds the allowed maximum.
  pub fn too_long(max: usize, actual: usize) -> Self {
    Self::new(
      ViolationType::TooLong,
      format!("Value length must at most {};  Received {}.", max, actual),
    )
  }

  /// Value length does not match the required exact length.
  pub fn exact_length(expected: usize, actual: usize) -> Self {
    Self::new(
      ViolationType::TooShort,
      format!("Value length must be exactly {} (got {}).", expected, actual),
    )
  }

  /// Value does not match the required regex pattern.
  pub fn pattern_mismatch(pattern: &str) -> Self {
    Self::new(
      ViolationType::PatternMismatch,
      format!("Value does not match pattern: {}", pattern),
    )
  }

  /// Value is not a valid email address.
  pub fn invalid_email() -> Self {
    Self::new(ViolationType::TypeMismatch, "Invalid email address.")
  }

  /// Value is not a valid URL.
  pub fn invalid_url() -> Self {
    Self::new(ViolationType::TypeMismatch, "Invalid URL.")
  }

  /// Value is below the allowed minimum.
  pub fn range_underflow<T: Display>(min: &T) -> Self {
    Self::new(
      ViolationType::RangeUnderflow,
      format!("Value must be at least {}.", min),
    )
  }

  /// Value exceeds the allowed maximum.
  pub fn range_overflow<T: Display>(max: &T) -> Self {
    Self::new(
      ViolationType::RangeOverflow,
      format!("Value must be at most {}.", max),
    )
  }

  /// Value is not a valid multiple of the required step.
  pub fn step_mismatch<T: Display>(step: &T) -> Self {
    Self::new(
      ViolationType::StepMismatch,
      format!("Value must be a multiple of {}.", step),
    )
  }

  /// Value does not equal the expected value.
  pub fn not_equal<T: Display>(expected: &T) -> Self {
    Self::new(
      ViolationType::NotEqual,
      format!("Value must equal {}.", expected),
    )
  }

  /// Value is not one of the allowed values.
  pub fn not_one_of() -> Self {
    Self::new(ViolationType::NotEqual, "Value must be one of the allowed values.")
  }

  /// A named rule reference could not be resolved.
  pub fn unresolved_ref(name: &str) -> Self {
    Self::new(
      ViolationType::CustomError,
      format!("Unresolved rule reference: {}.", name),
    )
  }

  /// The negated rule unexpectedly passed.
  pub fn negation_failed() -> Self {
    Self::new(ViolationType::CustomError, "Value must not satisfy the negated rule.")
  }
}

/// `Display` impl (and `ToString` (which we get for free)) for `Violation` type.
///
/// ```rust
/// use walrs_validator::{ViolationType::ValueMissing, Violation};
///
/// let violation = Violation(ValueMissing, "Value missing.".to_string());
/// let displayed = format!("{}", violation);
///
/// assert_eq!(&displayed, "Value missing.");
///
/// // `Display` impl, gives us `to_string()` for free:
/// assert_eq!(&violation.to_string(), "Value missing.");
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

#[must_use]
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

impl From<Violation> for Violations {
  fn from(val: Violation) -> Self {
    Violations(vec![val])
  }
}

impl From<Violations> for Vec<String> {
  fn from(val: Violations) -> Self {
    val.0.into_iter().map(|violation| violation.1).collect()
  }
}

impl Violations {
  /// Creates a new `Violations` instance from a vector of violations.
  pub fn new(violations: Vec<Violation>) -> Self {
    Self(violations)
  }

  /// Creates an empty `Violations` instance.
  pub fn empty() -> Self {
    Self(Vec::new())
  }

  /// Returns `true` if there are no violations.
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// Returns the number of violations.
  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// Adds a violation to the collection.
  pub fn push(&mut self, violation: Violation) -> &mut Self {
    self.0.push(violation);
    self
  }

  /// Converts the violations into a vector of violation messages.
  pub fn to_string_vec(self) -> Vec<String> {
    self.into()
  }

  /// Returns an iterator over the violations.
  pub fn iter(&self) -> impl Iterator<Item = &Violation> {
    self.0.iter()
  }

  /// Returns a mutable iterator over the violations.
  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Violation> {
    self.0.iter_mut()
  }
}

impl Default for Violations {
  fn default() -> Self {
    Self::empty()
  }
}

impl IntoIterator for Violations {
  type Item = Violation;
  type IntoIter = std::vec::IntoIter<Violation>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl<'a> IntoIterator for &'a Violations {
  type Item = &'a Violation;
  type IntoIter = std::slice::Iter<'a, Violation>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

impl<'a> IntoIterator for &'a mut Violations {
  type Item = &'a mut Violation;
  type IntoIter = std::slice::IterMut<'a, Violation>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.iter_mut()
  }
}

impl FromIterator<Violation> for Violations {
  fn from_iter<I: IntoIterator<Item = Violation>>(iter: I) -> Self {
    Violations(iter.into_iter().collect())
  }
}

impl Extend<Violation> for Violations {
  fn extend<I: IntoIterator<Item = Violation>>(&mut self, iter: I) {
    self.0.extend(iter);
  }
}

impl Display for Violations {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let messages: Vec<&str> = self.0.iter().map(|v| v.1.as_str()).collect();
    write!(f, "{}", messages.join("; "))
  }
}

impl Error for Violations {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    // Return the first violation as the source if available
    self.0.first().map(|v| v as &(dyn Error + 'static))
  }
}

#[cfg(test)]
mod test {
  use super::{ViolationType::ValueMissing, *};
  use crate::ViolationType::TypeMismatch;

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
  fn test_violations_display() {
    let vs = Violations(vec![
      Violation(ValueMissing, "value is missing.".to_string()),
      Violation(TypeMismatch, "type mismatch.".to_string()),
    ]);
    assert_eq!(format!("{}", vs), "value is missing.; type mismatch.");
  }

  #[test]
  fn test_violations_error() {
    let vs = Violations(vec![Violation(
      ValueMissing,
      "value is missing.".to_string(),
    )]);

    // Test that Violations implements Error
    let err: &dyn Error = &vs;
    assert!(err.source().is_some());

    // Test empty violations
    let empty_vs = Violations(vec![]);
    let empty_err: &dyn Error = &empty_vs;
    assert!(empty_err.source().is_none());
  }
}
