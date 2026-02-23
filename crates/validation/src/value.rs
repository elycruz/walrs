//! Custom Value enum for dynamic form data.
//!
//! This module provides a native `Value` enum with distinct numeric variants
//! (`I64`, `U64`, `F64`), enabling proper type discrimination in validation
//! rules and avoiding the precision/orphan-rule issues of `serde_json::Value`.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Native form value type.
///
/// Distinct numeric variants (`I64`, `U64`, `F64`) allow explicit type
/// discrimination in validation rules and avoid silent precision loss,
/// unlike `serde_json::Value::Number`.
///
/// # Example
///
/// ```rust
/// use walrs_validation::Value;
///
/// let s = Value::from("hello");
/// assert_eq!(s.as_str(), Some("hello"));
///
/// let n = Value::from(42i64);
/// assert_eq!(n.as_i64(), Some(42));
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
  /// JSON null.
  Null,
  /// A boolean.
  Bool(bool),
  /// A signed 64-bit integer.
  I64(i64),
  /// An unsigned 64-bit integer.
  U64(u64),
  /// A 64-bit float.
  F64(f64),
  /// A UTF-8 string.
  Str(String),
  /// An ordered list of values.
  Array(Vec<Value>),
  /// An ordered map of string keys to values.
  Object(IndexMap<String, Value>),
}

// ============================================================================
// Display
// ============================================================================

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Null => write!(f, "null"),
      Value::Bool(b) => write!(f, "{}", b),
      Value::I64(n) => write!(f, "{}", n),
      Value::U64(n) => write!(f, "{}", n),
      Value::F64(n) => write!(f, "{}", n),
      Value::Str(s) => write!(f, "{}", s),
      Value::Array(arr) => {
        write!(f, "[")?;
        for (i, v) in arr.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{}", v)?;
        }
        write!(f, "]")
      }
      Value::Object(map) => {
        write!(f, "{{")?;
        for (i, (k, v)) in map.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "\"{}\": {}", k, v)?;
        }
        write!(f, "}}")
      }
    }
  }
}

// ============================================================================
// PartialOrd — same-variant only
// ============================================================================

impl PartialOrd for Value {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self, other) {
      (Value::I64(a), Value::I64(b)) => a.partial_cmp(b),
      (Value::U64(a), Value::U64(b)) => a.partial_cmp(b),
      (Value::F64(a), Value::F64(b)) => a.partial_cmp(b),
      (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
      (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
      _ => None,
    }
  }
}

// ============================================================================
// Accessors
// ============================================================================

impl Value {
  /// Returns `true` if this value is `Null`.
  pub fn is_null(&self) -> bool {
    matches!(self, Value::Null)
  }

  /// If the value is a `Bool`, returns the inner value.
  pub fn as_bool(&self) -> Option<bool> {
    match self {
      Value::Bool(b) => Some(*b),
      _ => None,
    }
  }

  /// If the value is an `I64`, returns the inner value.
  pub fn as_i64(&self) -> Option<i64> {
    match self {
      Value::I64(n) => Some(*n),
      _ => None,
    }
  }

  /// If the value is a `U64`, returns the inner value.
  pub fn as_u64(&self) -> Option<u64> {
    match self {
      Value::U64(n) => Some(*n),
      _ => None,
    }
  }

  /// If the value is an `F64`, returns the inner value.
  pub fn as_f64(&self) -> Option<f64> {
    match self {
      Value::F64(n) => Some(*n),
      _ => None,
    }
  }

  /// If the value is a `Str`, returns a string slice.
  pub fn as_str(&self) -> Option<&str> {
    match self {
      Value::Str(s) => Some(s.as_str()),
      _ => None,
    }
  }

  /// If the value is an `Array`, returns a slice of the elements.
  pub fn as_array(&self) -> Option<&[Value]> {
    match self {
      Value::Array(arr) => Some(arr.as_slice()),
      _ => None,
    }
  }

  /// If the value is an `Object`, returns a reference to the map.
  pub fn as_object(&self) -> Option<&IndexMap<String, Value>> {
    match self {
      Value::Object(map) => Some(map),
      _ => None,
    }
  }
}

// ============================================================================
// From impls
// ============================================================================

impl From<bool> for Value {
  fn from(v: bool) -> Self {
    Value::Bool(v)
  }
}

impl From<i32> for Value {
  fn from(v: i32) -> Self {
    Value::I64(v as i64)
  }
}

impl From<i64> for Value {
  fn from(v: i64) -> Self {
    Value::I64(v)
  }
}

impl From<u32> for Value {
  fn from(v: u32) -> Self {
    Value::U64(v as u64)
  }
}

impl From<u64> for Value {
  fn from(v: u64) -> Self {
    Value::U64(v)
  }
}

impl From<f32> for Value {
  fn from(v: f32) -> Self {
    Value::F64(v as f64)
  }
}

impl From<f64> for Value {
  fn from(v: f64) -> Self {
    Value::F64(v)
  }
}

impl From<&str> for Value {
  fn from(v: &str) -> Self {
    Value::Str(v.to_string())
  }
}

impl From<String> for Value {
  fn from(v: String) -> Self {
    Value::Str(v)
  }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
  fn from(v: Vec<T>) -> Self {
    Value::Array(v.into_iter().map(Into::into).collect())
  }
}

impl<V: Into<Value>> From<IndexMap<String, V>> for Value {
  fn from(m: IndexMap<String, V>) -> Self {
    Value::Object(m.into_iter().map(|(k, v)| (k, v.into())).collect())
  }
}

impl From<()> for Value {
  fn from(_: ()) -> Self {
    Value::Null
  }
}

// ============================================================================
// serde_json bridge (feature-gated)
// ============================================================================

#[cfg(feature = "serde_json_bridge")]
impl From<serde_json::Value> for Value {
  fn from(v: serde_json::Value) -> Self {
    match v {
      serde_json::Value::Null => Value::Null,
      serde_json::Value::Bool(b) => Value::Bool(b),
      serde_json::Value::Number(n) => {
        if let Some(i) = n.as_i64() {
          Value::I64(i)
        } else if let Some(u) = n.as_u64() {
          Value::U64(u)
        } else if let Some(f) = n.as_f64() {
          Value::F64(f)
        } else {
          Value::Null
        }
      }
      serde_json::Value::String(s) => Value::Str(s),
      serde_json::Value::Array(arr) => {
        Value::Array(arr.into_iter().map(Value::from).collect())
      }
      serde_json::Value::Object(map) => {
        Value::Object(map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
      }
    }
  }
}

#[cfg(feature = "serde_json_bridge")]
impl From<Value> for serde_json::Value {
  fn from(v: Value) -> Self {
    match v {
      Value::Null => serde_json::Value::Null,
      Value::Bool(b) => serde_json::Value::Bool(b),
      Value::I64(n) => serde_json::Value::Number(n.into()),
      Value::U64(n) => serde_json::Value::Number(n.into()),
      Value::F64(n) => serde_json::Number::from_f64(n)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null),
      Value::Str(s) => serde_json::Value::String(s),
      Value::Array(arr) => {
        serde_json::Value::Array(arr.into_iter().map(serde_json::Value::from).collect())
      }
      Value::Object(map) => {
        let m: serde_json::Map<String, serde_json::Value> = map
          .into_iter()
          .map(|(k, v)| (k, serde_json::Value::from(v)))
          .collect();
        serde_json::Value::Object(m)
      }
    }
  }
}

// ============================================================================
// ValueExt
// ============================================================================

/// Extension trait for Value to add form-specific helper methods.
pub trait ValueExt {
  /// Checks if the value is "empty" (null, empty string, empty array, or empty object).
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_validation::{Value, ValueExt};
  ///
  /// assert!(Value::Null.is_empty_value());
  /// assert!(Value::Str("".to_string()).is_empty_value());
  /// assert!(!Value::Str("hello".to_string()).is_empty_value());
  /// assert!(Value::Array(vec![]).is_empty_value());
  /// assert!(!Value::Bool(false).is_empty_value());
  /// ```
  fn is_empty_value(&self) -> bool;
}

impl ValueExt for Value {
  fn is_empty_value(&self) -> bool {
    match self {
      Value::Null => true,
      Value::Str(s) => s.is_empty(),
      Value::Array(arr) => arr.is_empty(),
      Value::Object(obj) => obj.is_empty(),
      _ => false,
    }
  }
}

// ============================================================================
// Convenience macro
// ============================================================================

/// Convenience macro for constructing `Value` literals, similar to `serde_json::json!`.
///
/// # Examples
///
/// ```rust
/// use walrs_validation::{Value, value};
///
/// let v = value!(null);
/// assert!(v.is_null());
///
/// let v = value!(true);
/// assert_eq!(v.as_bool(), Some(true));
///
/// let v = value!("hello");
/// assert_eq!(v.as_str(), Some("hello"));
///
/// let v = value!(42);
/// assert_eq!(v.as_i64(), Some(42));
///
/// let v = value!(3.14);
/// assert_eq!(v, Value::F64(3.14));
///
/// let v = value!([1, 2, 3]);
/// assert_eq!(v.as_array().unwrap().len(), 3);
/// ```
#[macro_export]
macro_rules! value {
  (null) => {
    $crate::Value::Null
  };
  (true) => {
    $crate::Value::Bool(true)
  };
  (false) => {
    $crate::Value::Bool(false)
  };
  ([ $($elem:tt),* $(,)? ]) => {
    $crate::Value::Array(vec![ $( $crate::value!($elem) ),* ])
  };
  ({ $($key:tt : $val:tt),* $(,)? }) => {
    $crate::Value::Object({
      let mut map = indexmap::IndexMap::new();
      $( map.insert($key.to_string(), $crate::value!($val)); )*
      map
    })
  };
  ($e:expr) => {
    $crate::Value::from($e)
  };
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_null_is_empty() {
    assert!(Value::Null.is_empty_value());
  }

  #[test]
  fn test_empty_string_is_empty() {
    assert!(Value::Str("".to_string()).is_empty_value());
  }

  #[test]
  fn test_non_empty_string_is_not_empty() {
    assert!(!Value::Str("hello".to_string()).is_empty_value());
  }

  #[test]
  fn test_empty_array_is_empty() {
    assert!(Value::Array(vec![]).is_empty_value());
  }

  #[test]
  fn test_non_empty_array_is_not_empty() {
    assert!(!Value::Array(vec![Value::I64(1), Value::I64(2)]).is_empty_value());
  }

  #[test]
  fn test_empty_object_is_empty() {
    assert!(Value::Object(IndexMap::new()).is_empty_value());
  }

  #[test]
  fn test_non_empty_object_is_not_empty() {
    let mut map = IndexMap::new();
    map.insert("key".to_string(), Value::Str("value".to_string()));
    assert!(!Value::Object(map).is_empty_value());
  }

  #[test]
  fn test_bool_is_not_empty() {
    assert!(!Value::Bool(false).is_empty_value());
    assert!(!Value::Bool(true).is_empty_value());
  }

  #[test]
  fn test_number_is_not_empty() {
    assert!(!Value::I64(0).is_empty_value());
    assert!(!Value::I64(42).is_empty_value());
    assert!(!Value::F64(3.15).is_empty_value());
  }

  #[test]
  fn test_from_primitives() {
    assert_eq!(Value::from(true), Value::Bool(true));
    assert_eq!(Value::from(42i32), Value::I64(42));
    assert_eq!(Value::from(42i64), Value::I64(42));
    assert_eq!(Value::from(42u32), Value::U64(42));
    assert_eq!(Value::from(42u64), Value::U64(42));
    assert_eq!(Value::from(3.14f64), Value::F64(3.14));
    assert_eq!(Value::from("hello"), Value::Str("hello".to_string()));
    assert_eq!(
      Value::from("hello".to_string()),
      Value::Str("hello".to_string())
    );
  }

  #[test]
  fn test_accessors() {
    assert!(Value::Null.is_null());
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::I64(42).as_i64(), Some(42));
    assert_eq!(Value::U64(42).as_u64(), Some(42));
    assert_eq!(Value::F64(3.14).as_f64(), Some(3.14));
    assert_eq!(Value::Str("hi".to_string()).as_str(), Some("hi"));
    assert!(Value::Array(vec![]).as_array().is_some());
    assert!(Value::Object(IndexMap::new()).as_object().is_some());
  }

  #[test]
  fn test_partial_ord() {
    assert!(Value::I64(1) < Value::I64(2));
    assert!(Value::F64(1.0) < Value::F64(2.0));
    assert!(Value::Str("a".into()) < Value::Str("b".into()));
    // Cross-variant is None
    assert_eq!(Value::I64(1).partial_cmp(&Value::F64(1.0)), None);
  }

  #[test]
  fn test_display() {
    assert_eq!(format!("{}", Value::Null), "null");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::I64(42)), "42");
    assert_eq!(format!("{}", Value::Str("hi".into())), "hi");
  }

  #[test]
  #[cfg(feature = "serde_json_bridge")]
  fn test_serialization_roundtrip() {
    let original = Value::Str("hello".to_string());
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
  }

  #[test]
  fn test_value_macro() {
    assert_eq!(value!(null), Value::Null);
    assert_eq!(value!(true), Value::Bool(true));
    assert_eq!(value!(false), Value::Bool(false));
    assert_eq!(value!(42), Value::I64(42));
    assert_eq!(value!("hello"), Value::Str("hello".to_string()));
    let arr = value!([1, 2, 3]);
    assert_eq!(arr.as_array().unwrap().len(), 3);
  }

  #[cfg(feature = "serde_json_bridge")]
  mod bridge_tests {
    use super::*;

    #[test]
    fn test_from_serde_json() {
      let sj = serde_json::json!({"name": "test", "age": 42, "score": 3.14});
      let v = Value::from(sj);
      match &v {
        Value::Object(map) => {
          assert_eq!(map.get("name"), Some(&Value::Str("test".to_string())));
          assert_eq!(map.get("age"), Some(&Value::I64(42)));
          assert_eq!(map.get("score"), Some(&Value::F64(3.14)));
        }
        _ => panic!("Expected Object"),
      }
    }

    #[test]
    fn test_to_serde_json() {
      let v = Value::I64(42);
      let sj: serde_json::Value = v.into();
      assert_eq!(sj, serde_json::json!(42));
    }
  }
}
