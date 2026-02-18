//! Value type re-exports and extensions for form data.
//!
//! This module re-exports `serde_json::Value` as the dynamic value type for form data
//! and provides the `ValueExt` trait for form-specific helper methods.

/// Re-export serde_json::Value as our dynamic value type for form data.
/// This provides all necessary features: serialization, helper methods
/// (as_str, as_i64, as_f64, as_bool, as_array, as_object), and From impls.
pub use serde_json::Value;

/// Extension trait for Value to add form-specific helper methods.
pub trait ValueExt {
    /// Checks if the value is "empty" (null, empty string, empty array, or empty object).
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_form_core::{Value, ValueExt};
    ///
    /// assert!(Value::Null.is_empty_value());
    /// assert!(Value::String("".to_string()).is_empty_value());
    /// assert!(!Value::String("hello".to_string()).is_empty_value());
    /// assert!(Value::Array(vec![]).is_empty_value());
    /// assert!(!Value::Bool(false).is_empty_value());
    /// ```
    fn is_empty_value(&self) -> bool;
}

impl ValueExt for Value {
    fn is_empty_value(&self) -> bool {
        match self {
            Value::Null => true,
            Value::String(s) => s.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Object(obj) => obj.is_empty(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_null_is_empty() {
        assert!(Value::Null.is_empty_value());
    }

    #[test]
    fn test_empty_string_is_empty() {
        assert!(Value::String("".to_string()).is_empty_value());
    }

    #[test]
    fn test_non_empty_string_is_not_empty() {
        assert!(!Value::String("hello".to_string()).is_empty_value());
    }

    #[test]
    fn test_empty_array_is_empty() {
        assert!(Value::Array(vec![]).is_empty_value());
    }

    #[test]
    fn test_non_empty_array_is_not_empty() {
        assert!(!json!([1, 2, 3]).is_empty_value());
    }

    #[test]
    fn test_empty_object_is_empty() {
        assert!(json!({}).is_empty_value());
    }

    #[test]
    fn test_non_empty_object_is_not_empty() {
        assert!(!json!({"key": "value"}).is_empty_value());
    }

    #[test]
    fn test_bool_is_not_empty() {
        assert!(!Value::Bool(false).is_empty_value());
        assert!(!Value::Bool(true).is_empty_value());
    }

    #[test]
    fn test_number_is_not_empty() {
        assert!(!json!(0).is_empty_value());
        assert!(!json!(42).is_empty_value());
        assert!(!json!(3.14).is_empty_value());
    }
}

