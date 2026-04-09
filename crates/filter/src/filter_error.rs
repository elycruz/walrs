//! Error type for fallible filter operations.
//!
//! [`FilterError`] represents a failure during a filter transformation.
//! It can be converted into a [`Violation`] for seamless integration with
//! the validation error pipeline.

use std::fmt;

/// An error produced by a fallible filter operation.
///
/// Contains a human-readable message and an optional filter name for
/// context in error reporting.
///
/// # Example
///
/// ```rust
/// use walrs_filter::FilterError;
///
/// let err = FilterError::new("invalid base64 input")
///     .with_name("Base64Decode");
///
/// assert_eq!(err.message(), "invalid base64 input");
/// assert_eq!(err.filter_name(), Some("Base64Decode"));
/// assert_eq!(err.to_string(), "Filter 'Base64Decode' failed: invalid base64 input");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FilterError {
  message: String,
  filter_name: Option<String>,
}

impl FilterError {
  /// Creates a new `FilterError` with the given message.
  pub fn new(message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
      filter_name: None,
    }
  }

  /// Attaches a filter name to this error for context.
  pub fn with_name(mut self, name: impl Into<String>) -> Self {
    self.filter_name = Some(name.into());
    self
  }

  /// Returns a reference to the error message.
  pub fn message(&self) -> &str {
    &self.message
  }

  /// Returns the optional filter name.
  pub fn filter_name(&self) -> Option<&str> {
    self.filter_name.as_deref()
  }
}

impl fmt::Display for FilterError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.filter_name {
      Some(name) => write!(f, "Filter '{}' failed: {}", name, self.message),
      None => write!(f, "Filter failed: {}", self.message),
    }
  }
}

impl std::error::Error for FilterError {}

// ============================================================================
// Conversion to Violation (requires "validation" feature)
// ============================================================================

#[cfg(feature = "validation")]
impl From<FilterError> for walrs_validation::Violation {
  fn from(err: FilterError) -> Self {
    walrs_validation::Violation::new(walrs_validation::ViolationType::CustomError, err.to_string())
  }
}

#[cfg(feature = "validation")]
impl From<FilterError> for walrs_validation::Violations {
  fn from(err: FilterError) -> Self {
    walrs_validation::Violations::new(vec![err.into()])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_filter_error_new() {
    let err = FilterError::new("bad input");
    assert_eq!(err.message(), "bad input");
    assert_eq!(err.filter_name(), None);
  }

  #[test]
  fn test_filter_error_with_name() {
    let err = FilterError::new("bad input").with_name("Base64Decode");
    assert_eq!(err.message(), "bad input");
    assert_eq!(err.filter_name(), Some("Base64Decode"));
  }

  #[test]
  fn test_filter_error_display_without_name() {
    let err = FilterError::new("bad input");
    assert_eq!(err.to_string(), "Filter failed: bad input");
  }

  #[test]
  fn test_filter_error_display_with_name() {
    let err = FilterError::new("bad input").with_name("UrlDecode");
    assert_eq!(err.to_string(), "Filter 'UrlDecode' failed: bad input");
  }

  #[test]
  fn test_filter_error_eq() {
    let a = FilterError::new("bad").with_name("X");
    let b = FilterError::new("bad").with_name("X");
    assert_eq!(a, b);
  }

  #[test]
  fn test_filter_error_clone() {
    let err = FilterError::new("bad").with_name("X");
    let cloned = err.clone();
    assert_eq!(err, cloned);
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_filter_error_to_violation() {
    let err = FilterError::new("bad input").with_name("Base64Decode");
    let violation: walrs_validation::Violation = err.into();
    assert_eq!(
      violation.violation_type(),
      walrs_validation::ViolationType::CustomError
    );
    assert!(violation.message().contains("Base64Decode"));
    assert!(violation.message().contains("bad input"));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_filter_error_to_violations() {
    let err = FilterError::new("bad input");
    let violations: walrs_validation::Violations = err.into();
    assert_eq!(violations.len(), 1);
  }
}
