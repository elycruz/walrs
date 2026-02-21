use std::fmt;

/// Error types for the navigation component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationError {
  /// Invalid page index
  InvalidIndex(usize),
  /// Page not found
  PageNotFound,
  /// Cycle detected in navigation tree
  CycleDetected,
  /// Invalid page configuration
  InvalidConfiguration(String),
  /// Deserialization error
  DeserializationError(String),
  /// Serialization error
  SerializationError(String),
}

impl fmt::Display for NavigationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      NavigationError::InvalidIndex(idx) => {
        write!(f, "Invalid page index: {}", idx)
      }
      NavigationError::PageNotFound => {
        write!(f, "Page not found")
      }
      NavigationError::CycleDetected => {
        write!(f, "Cycle detected in navigation tree")
      }
      NavigationError::InvalidConfiguration(msg) => {
        write!(f, "Invalid configuration: {}", msg)
      }
      NavigationError::DeserializationError(msg) => {
        write!(f, "Deserialization error: {}", msg)
      }
      NavigationError::SerializationError(msg) => {
        write!(f, "Serialization error: {}", msg)
      }
    }
  }
}

impl std::error::Error for NavigationError {}

pub type Result<T> = std::result::Result<T, NavigationError>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_invalid_index_display() {
    let error = NavigationError::InvalidIndex(42);
    assert_eq!(format!("{}", error), "Invalid page index: 42");
  }

  #[test]
  fn test_page_not_found_display() {
    let error = NavigationError::PageNotFound;
    assert_eq!(format!("{}", error), "Page not found");
  }

  #[test]
  fn test_cycle_detected_display() {
    let error = NavigationError::CycleDetected;
    assert_eq!(format!("{}", error), "Cycle detected in navigation tree");
  }

  #[test]
  fn test_invalid_configuration_display() {
    let error = NavigationError::InvalidConfiguration("missing label".to_string());
    assert_eq!(format!("{}", error), "Invalid configuration: missing label");
  }

  #[test]
  fn test_deserialization_error_display() {
    let error = NavigationError::DeserializationError("invalid JSON".to_string());
    assert_eq!(format!("{}", error), "Deserialization error: invalid JSON");
  }

  #[test]
  fn test_serialization_error_display() {
    let error = NavigationError::SerializationError("failed to serialize".to_string());
    assert_eq!(
      format!("{}", error),
      "Serialization error: failed to serialize"
    );
  }

  #[test]
  fn test_error_trait_implementation() {
    let error: Box<dyn std::error::Error> = Box::new(NavigationError::PageNotFound);
    assert_eq!(error.to_string(), "Page not found");
  }

  #[test]
  fn test_error_debug() {
    let error = NavigationError::InvalidIndex(5);
    assert_eq!(format!("{:?}", error), "InvalidIndex(5)");

    let error = NavigationError::PageNotFound;
    assert_eq!(format!("{:?}", error), "PageNotFound");

    let error = NavigationError::CycleDetected;
    assert_eq!(format!("{:?}", error), "CycleDetected");

    let error = NavigationError::InvalidConfiguration("test".to_string());
    assert_eq!(format!("{:?}", error), "InvalidConfiguration(\"test\")");

    let error = NavigationError::DeserializationError("test".to_string());
    assert_eq!(format!("{:?}", error), "DeserializationError(\"test\")");

    let error = NavigationError::SerializationError("test".to_string());
    assert_eq!(format!("{:?}", error), "SerializationError(\"test\")");
  }

  #[test]
  fn test_error_clone() {
    let error1 = NavigationError::InvalidConfiguration("test".to_string());
    let error2 = error1.clone();
    assert_eq!(error1, error2);
  }

  #[test]
  fn test_error_equality() {
    assert_eq!(NavigationError::PageNotFound, NavigationError::PageNotFound);
    assert_eq!(
      NavigationError::CycleDetected,
      NavigationError::CycleDetected
    );
    assert_eq!(
      NavigationError::InvalidIndex(1),
      NavigationError::InvalidIndex(1)
    );
    assert_ne!(
      NavigationError::InvalidIndex(1),
      NavigationError::InvalidIndex(2)
    );
    assert_ne!(
      NavigationError::PageNotFound,
      NavigationError::CycleDetected
    );
  }

  #[test]
  fn test_result_type_alias() {
    fn returns_ok() -> Result<i32> {
      Ok(42)
    }

    fn returns_err() -> Result<i32> {
      Err(NavigationError::PageNotFound)
    }

    assert_eq!(returns_ok().unwrap(), 42);
    assert_eq!(returns_err().unwrap_err(), NavigationError::PageNotFound);
  }
}
