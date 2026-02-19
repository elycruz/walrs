use crate::prelude::{String, Vec};
use crate::error::{RbacError, Result};
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "std")]
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::BufReader;

/// Serializable representation of RBAC data.
///
/// Each role is a tuple of `(name, permissions, optional_children)`.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::RbacData;
///
/// let data = RbacData {
///   roles: vec![
///     ("guest".to_string(), vec!["read".to_string()], None),
///     ("admin".to_string(), vec!["manage".to_string()], Some(vec!["guest".to_string()])),
///   ],
/// };
///
/// assert_eq!(data.roles.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacData {
  pub roles: Vec<(String, Vec<String>, Option<Vec<String>>)>,
}

/// Reads `RbacData` from a JSON file.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::RbacData;
/// use std::convert::TryFrom;
/// use std::fs::File;
///
/// let file_path = "./test-fixtures/example-rbac.json";
/// let mut f = File::open(&file_path)?;
/// let data = RbacData::try_from(&mut f)?;
/// assert!(!data.roles.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[cfg(feature = "std")]
impl TryFrom<&mut File> for RbacData {
  type Error = RbacError;

  fn try_from(file: &mut File) -> Result<Self> {
    let buf = BufReader::new(file);
    serde_json::from_reader(buf)
      .map_err(|e| RbacError::DeserializationError(e.to_string()))
  }
}

impl RbacData {
  /// Deserializes `RbacData` from a JSON string.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacData;
  ///
  /// let json = r#"{"roles":[["admin",["manage"],null]]}"#;
  /// let data = RbacData::from_json(json).unwrap();
  /// assert_eq!(data.roles.len(), 1);
  ///
  /// let invalid = RbacData::from_json("not valid json");
  /// assert!(invalid.is_err());
  /// ```
  #[cfg(feature = "std")]
  pub fn from_json(json: &str) -> Result<Self> {
    serde_json::from_str(json)
      .map_err(|e| RbacError::DeserializationError(e.to_string()))
  }

  /// Serializes `RbacData` to a JSON string.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacData;
  ///
  /// let data = RbacData {
  ///   roles: vec![
  ///     ("admin".to_string(), vec!["manage".to_string()], None),
  ///   ],
  /// };
  ///
  /// let json = data.to_json().unwrap();
  /// assert!(json.contains("admin"));
  /// ```
  #[cfg(feature = "std")]
  pub fn to_json(&self) -> Result<String> {
    serde_json::to_string(self)
      .map_err(|e| RbacError::SerializationError(e.to_string()))
  }

  /// Serializes `RbacData` to a pretty-printed JSON string.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacData;
  ///
  /// let data = RbacData {
  ///   roles: vec![
  ///     ("admin".to_string(), vec!["manage".to_string()], None),
  ///   ],
  /// };
  ///
  /// let json = data.to_json_pretty().unwrap();
  /// assert!(json.contains("admin"));
  /// ```
  #[cfg(feature = "std")]
  pub fn to_json_pretty(&self) -> Result<String> {
    serde_json::to_string_pretty(self)
      .map_err(|e| RbacError::SerializationError(e.to_string()))
  }

  /// Deserializes `RbacData` from a YAML string.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacData;
  ///
  /// let yaml = "roles:\n  - - admin\n    - - manage\n    - null\n";
  /// let data = RbacData::from_yaml(yaml).unwrap();
  /// assert_eq!(data.roles.len(), 1);
  /// ```
  #[cfg(feature = "yaml")]
  pub fn from_yaml(yaml: &str) -> Result<Self> {
    serde_yaml::from_str(yaml)
      .map_err(|e| RbacError::DeserializationError(e.to_string()))
  }

  /// Serializes `RbacData` to a YAML string.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacData;
  ///
  /// let data = RbacData {
  ///   roles: vec![
  ///     ("admin".to_string(), vec!["manage".to_string()], None),
  ///   ],
  /// };
  ///
  /// let yaml = data.to_yaml().unwrap();
  /// assert!(yaml.contains("admin"));
  /// ```
  #[cfg(feature = "yaml")]
  pub fn to_yaml(&self) -> Result<String> {
    serde_yaml::to_string(self)
      .map_err(|e| RbacError::SerializationError(e.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rbac_data_creation() {
    let data = RbacData {
      roles: vec![
        ("admin".to_string(), vec!["manage".to_string()], None),
      ],
    };
    assert_eq!(data.roles.len(), 1);
    assert_eq!(data.roles[0].0, "admin");
  }

  #[test]
  fn test_rbac_data_clone() {
    let data = RbacData {
      roles: vec![
        ("user".to_string(), vec!["read".to_string()], None),
      ],
    };
    let cloned = data.clone();
    assert_eq!(data.roles.len(), cloned.roles.len());
    assert_eq!(data.roles[0].0, cloned.roles[0].0);
  }

  #[test]
  fn test_rbac_data_with_children() {
    let data = RbacData {
      roles: vec![
        ("guest".to_string(), vec!["read".to_string()], None),
        ("admin".to_string(), vec!["manage".to_string()], Some(vec!["guest".to_string()])),
      ],
    };
    assert_eq!(data.roles.len(), 2);
    assert!(data.roles[1].2.is_some());
    assert_eq!(data.roles[1].2.as_ref().unwrap()[0], "guest");
  }

  #[test]
  fn test_from_json() {
    let json = r#"{"roles":[["admin",["manage"],null],["user",["read"],["admin"]]]}"#;
    let data = RbacData::from_json(json).unwrap();
    assert_eq!(data.roles.len(), 2);
    assert_eq!(data.roles[0].0, "admin");
  }

  #[test]
  fn test_from_json_invalid() {
    let result = RbacData::from_json("not json");
    assert!(result.is_err());
  }

  #[test]
  fn test_to_json() {
    let data = RbacData {
      roles: vec![
        ("admin".to_string(), vec!["manage".to_string()], None),
      ],
    };
    let json = data.to_json().unwrap();
    assert!(json.contains("admin"));
    assert!(json.contains("manage"));
  }

  #[test]
  fn test_to_json_pretty() {
    let data = RbacData {
      roles: vec![
        ("admin".to_string(), vec!["manage".to_string()], None),
      ],
    };
    let json = data.to_json_pretty().unwrap();
    assert!(json.contains("admin"));
    assert!(json.contains('\n')); // pretty print has newlines
  }

  #[test]
  fn test_json_roundtrip() {
    let data = RbacData {
      roles: vec![
        ("guest".to_string(), vec!["read".to_string()], None),
        ("admin".to_string(), vec!["manage".to_string()], Some(vec!["guest".to_string()])),
      ],
    };
    let json = data.to_json().unwrap();
    let restored = RbacData::from_json(&json).unwrap();
    assert_eq!(data.roles.len(), restored.roles.len());
    assert_eq!(data.roles[0].0, restored.roles[0].0);
  }

  #[cfg(feature = "yaml")]
  #[test]
  fn test_from_yaml() {
    let yaml = "roles:\n  - - admin\n    - - manage\n    - null\n";
    let data = RbacData::from_yaml(yaml).unwrap();
    assert_eq!(data.roles.len(), 1);
    assert_eq!(data.roles[0].0, "admin");
  }

  #[cfg(feature = "yaml")]
  #[test]
  fn test_from_yaml_invalid() {
    let result = RbacData::from_yaml("not: [valid: yaml: data");
    assert!(result.is_err());
  }

  #[cfg(feature = "yaml")]
  #[test]
  fn test_to_yaml() {
    let data = RbacData {
      roles: vec![
        ("admin".to_string(), vec!["manage".to_string()], None),
      ],
    };
    let yaml = data.to_yaml().unwrap();
    assert!(yaml.contains("admin"));
  }

  #[cfg(feature = "yaml")]
  #[test]
  fn test_yaml_roundtrip() {
    let data = RbacData {
      roles: vec![
        ("guest".to_string(), vec!["read".to_string()], None),
        ("admin".to_string(), vec!["manage".to_string()], Some(vec!["guest".to_string()])),
      ],
    };
    let yaml = data.to_yaml().unwrap();
    let restored = RbacData::from_yaml(&yaml).unwrap();
    assert_eq!(data.roles.len(), restored.roles.len());
  }

  #[test]
  fn test_try_from_file() {
    let file_path = "./test-fixtures/example-rbac.json";
    let mut f = File::open(file_path).unwrap();
    let data = RbacData::try_from(&mut f).unwrap();
    assert!(!data.roles.is_empty());
  }
}
