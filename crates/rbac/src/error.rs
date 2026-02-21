use crate::prelude::String;
use core::fmt;

/// Errors that can occur in the RBAC system.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::RbacError;
///
/// let err = RbacError::RoleNotFound("admin".to_string());
/// assert_eq!(format!("{}", err), "Role not found: admin");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum RbacError {
  /// A referenced role was not found in the RBAC.
  RoleNotFound(String),
  /// A cycle was detected in the role hierarchy.
  CycleDetected(String),
  /// An invalid configuration was provided.
  InvalidConfiguration(String),
  /// An error occurred during deserialization.
  DeserializationError(String),
  /// An error occurred during serialization.
  SerializationError(String),
}

impl fmt::Display for RbacError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      RbacError::RoleNotFound(role) => write!(f, "Role not found: {}", role),
      RbacError::CycleDetected(msg) => write!(f, "Cycle detected: {}", msg),
      RbacError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
      RbacError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
      RbacError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for RbacError {}

/// A specialized `Result` type for RBAC operations.
pub type Result<T> = core::result::Result<T, RbacError>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_display_role_not_found() {
    let err = RbacError::RoleNotFound("admin".to_string());
    assert_eq!(format!("{}", err), "Role not found: admin");
  }

  #[test]
  fn test_display_cycle_detected() {
    let err = RbacError::CycleDetected("a -> b -> a".to_string());
    assert_eq!(format!("{}", err), "Cycle detected: a -> b -> a");
  }

  #[test]
  fn test_display_invalid_configuration() {
    let err = RbacError::InvalidConfiguration("missing roles".to_string());
    assert_eq!(format!("{}", err), "Invalid configuration: missing roles");
  }

  #[test]
  fn test_display_deserialization_error() {
    let err = RbacError::DeserializationError("invalid JSON".to_string());
    assert_eq!(format!("{}", err), "Deserialization error: invalid JSON");
  }

  #[test]
  fn test_display_serialization_error() {
    let err = RbacError::SerializationError("write failed".to_string());
    assert_eq!(format!("{}", err), "Serialization error: write failed");
  }

  #[test]
  fn test_error_equality() {
    let err1 = RbacError::RoleNotFound("admin".to_string());
    let err2 = RbacError::RoleNotFound("admin".to_string());
    let err3 = RbacError::RoleNotFound("user".to_string());
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
  }

  #[test]
  fn test_error_clone() {
    let err = RbacError::CycleDetected("cycle".to_string());
    let cloned = err.clone();
    assert_eq!(err, cloned);
  }

  #[test]
  fn test_error_debug() {
    let err = RbacError::RoleNotFound("test".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("RoleNotFound"));
    assert!(debug.contains("test"));
  }
}
