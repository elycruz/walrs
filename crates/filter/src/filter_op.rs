//! Composable, serializable filter operations for value transformation.
//!
//! This module provides a `FilterOp<T>` enum that represents composable
//! filter operations. Most variants delegate to the filter struct
//! implementations in this crate (e.g., [`SlugFilter`], [`StripTagsFilter`]).

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::{Filter, SlugFilter, StripTagsFilter, XmlEntitiesFilter};

#[cfg(feature = "validation")]
use walrs_validation::Value;

/// A composable, serializable value transformer.
///
/// `FilterOp` provides a way to define filter operations that can be serialized
/// to JSON/YAML for config-driven form processing. Most variants delegate to
/// filter struct implementations in the `walrs_filter` crate.
///
/// # Calling convention
///
/// - **`apply(&self, value: T) -> T`** — for `Copy` (scalar/numeric) types.
/// - **`apply_ref(&self, value: &U) -> T`** — for `?Sized` reference types
///   (e.g., `&str`), returning an owned result.
///
/// For `FilterOp<String>`, prefer `apply_ref` when you already have a `&str`,
/// avoiding an allocation. `apply` is a convenience wrapper that delegates to
/// `apply_ref`.
///
/// # Example
///
/// ```rust
/// use walrs_filter::FilterOp;
///
/// let filter = FilterOp::<String>::Trim;
///
/// // By reference — no allocation needed at the call site
/// let result = filter.apply_ref("  hello  ");
/// assert_eq!(result, "hello");
///
/// // By value — delegates to apply_ref internally
/// let result = filter.apply("  hello  ".to_string());
/// assert_eq!(result, "hello");
///
/// // Chain multiple filters
/// let chain: FilterOp<String> = FilterOp::Chain(vec![
///     FilterOp::Trim,
///     FilterOp::Lowercase,
/// ]);
/// let result = chain.apply_ref("  HELLO  ");
/// assert_eq!(result, "hello");
/// ```
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum FilterOp<T> {
  // ---- String Filters ----
  /// Trim whitespace from start and end.
  Trim,

  /// Convert to lowercase.
  Lowercase,

  /// Convert to uppercase.
  Uppercase,

  /// Remove HTML tags using Ammonia sanitizer.
  StripTags,

  /// Encode special characters as XML/HTML entities.
  HtmlEntities,

  /// Convert to URL-friendly slug.
  Slug {
    /// Maximum length for the slug (None for unlimited).
    max_length: Option<usize>,
  },

  // ---- Numeric Filters ----
  /// Clamp value to a range (for numeric types).
  Clamp {
    /// Minimum value.
    min: T,
    /// Maximum value.
    max: T,
  },

  // ---- Composite ----
  /// Apply filters sequentially: f3(f2(f1(value))).
  Chain(Vec<FilterOp<T>>),

  // ---- Custom ----
  /// Custom filter function (not serializable).
  #[serde(skip)]
  Custom(Arc<dyn Fn(T) -> T + Send + Sync>),
}

impl<T: Debug> Debug for FilterOp<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Trim => write!(f, "Trim"),
      Self::Lowercase => write!(f, "Lowercase"),
      Self::Uppercase => write!(f, "Uppercase"),
      Self::StripTags => write!(f, "StripTags"),
      Self::HtmlEntities => write!(f, "HtmlEntities"),
      Self::Slug { max_length } => f
        .debug_struct("Slug")
        .field("max_length", max_length)
        .finish(),
      Self::Clamp { min, max } => f
        .debug_struct("Clamp")
        .field("min", min)
        .field("max", max)
        .finish(),
      Self::Chain(filters) => f.debug_tuple("Chain").field(filters).finish(),
      Self::Custom(_) => write!(f, "Custom(<fn>)"),
    }
  }
}

impl<T: PartialEq> PartialEq for FilterOp<T> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Trim, Self::Trim) => true,
      (Self::Lowercase, Self::Lowercase) => true,
      (Self::Uppercase, Self::Uppercase) => true,
      (Self::StripTags, Self::StripTags) => true,
      (Self::HtmlEntities, Self::HtmlEntities) => true,
      (Self::Slug { max_length: a }, Self::Slug { max_length: b }) => a == b,
      (Self::Clamp { min: a1, max: a2 }, Self::Clamp { min: b1, max: b2 }) => a1 == b1 && a2 == b2,
      (Self::Chain(a), Self::Chain(b)) => a == b,
      // Custom filters are never equal
      (Self::Custom(_), Self::Custom(_)) => false,
      _ => false,
    }
  }
}

// ============================================================================
// String FilterOp Implementation
// ============================================================================

impl FilterOp<String> {
  /// Apply the filter operation to a `&str` reference, returning an owned `String`.
  ///
  /// Prefer this method when you already have a `&str`, avoiding an
  /// unnecessary allocation at the call site.
  pub fn apply_ref(&self, value: &str) -> String {
    match self {
      FilterOp::Trim => value.trim().to_string(),
      FilterOp::Lowercase => value.to_lowercase(),
      FilterOp::Uppercase => value.to_uppercase(),
      FilterOp::StripTags => {
        let filter = StripTagsFilter::new();
        filter.filter(Cow::Borrowed(value)).into_owned()
      }
      FilterOp::HtmlEntities => {
        let filter = XmlEntitiesFilter::new();
        filter.filter(Cow::Borrowed(value)).into_owned()
      }
      FilterOp::Slug { max_length } => {
        let filter = SlugFilter::new(max_length.unwrap_or(200), false);
        filter.filter(Cow::Borrowed(value)).into_owned()
      }
      FilterOp::Clamp { .. } => value.to_string(), // Clamp doesn't apply to strings
      FilterOp::Chain(filters) => {
        let mut result = value.to_string();
        for f in filters {
          result = f.apply_ref(&result);
        }
        result
      }
      FilterOp::Custom(f) => f(value.to_string()),
    }
  }

  /// Apply the filter operation to an owned `String` value.
  ///
  /// Convenience wrapper that delegates to [`apply_ref`](Self::apply_ref).
  pub fn apply(&self, value: String) -> String {
    self.apply_ref(&value)
  }
}

// ============================================================================
// Value FilterOp Implementation (requires "validation" feature)
// ============================================================================

#[cfg(feature = "validation")]
impl FilterOp<Value> {
  /// Apply the filter operation to a `&Value` reference, returning an owned `Value`.
  ///
  /// String-based filters only apply to `Value::Str` variants.
  /// Numeric filters apply to `Value::I64` / `Value::U64` / `Value::F64` variants.
  /// Non-matching types are cloned unchanged.
  pub fn apply_ref(&self, value: &Value) -> Value {
    match self {
      FilterOp::Trim => {
        if let Value::Str(s) = value {
          Value::Str(s.trim().to_string())
        } else {
          value.clone()
        }
      }
      FilterOp::Lowercase => {
        if let Value::Str(s) = value {
          Value::Str(s.to_lowercase())
        } else {
          value.clone()
        }
      }
      FilterOp::Uppercase => {
        if let Value::Str(s) = value {
          Value::Str(s.to_uppercase())
        } else {
          value.clone()
        }
      }
      FilterOp::StripTags => {
        if let Value::Str(s) = value {
          let filter = StripTagsFilter::new();
          Value::Str(filter.filter(Cow::Borrowed(s.as_str())).into_owned())
        } else {
          value.clone()
        }
      }
      FilterOp::HtmlEntities => {
        if let Value::Str(s) = value {
          let filter = XmlEntitiesFilter::new();
          Value::Str(filter.filter(Cow::Borrowed(s.as_str())).into_owned())
        } else {
          value.clone()
        }
      }
      FilterOp::Slug { max_length } => {
        if let Value::Str(s) = value {
          let filter = SlugFilter::new(max_length.unwrap_or(200), false);
          Value::Str(filter.filter(Cow::Borrowed(s.as_str())).into_owned())
        } else {
          value.clone()
        }
      }
      FilterOp::Clamp { min, max } => {
        match (value, min, max) {
          (Value::I64(v), Value::I64(min_v), Value::I64(max_v)) => {
            Value::I64((*v).clamp(*min_v, *max_v))
          }
          (Value::U64(v), Value::U64(min_v), Value::U64(max_v)) => {
            Value::U64((*v).clamp(*min_v, *max_v))
          }
          (Value::F64(v), Value::F64(min_v), Value::F64(max_v)) => {
            Value::F64((*v).clamp(*min_v, *max_v))
          }
          _ => value.clone(),
        }
      }
      FilterOp::Chain(filters) => {
        let mut result = value.clone();
        for f in filters {
          result = f.apply_ref(&result);
        }
        result
      }
      FilterOp::Custom(f) => f(value.clone()),
    }
  }

  /// Apply the filter operation to an owned `Value`.
  ///
  /// Convenience wrapper that delegates to [`apply_ref`](Self::apply_ref).
  ///
  /// String-based filters only apply to `Value::Str` variants.
  /// Numeric filters apply to `Value::I64` / `Value::U64` / `Value::F64` variants.
  pub fn apply(&self, value: Value) -> Value {
    self.apply_ref(&value)
  }
}

// ============================================================================
// Numeric FilterOp Implementations
// ============================================================================

macro_rules! impl_numeric_filter_op {
    ($($t:ty),*) => {
        $(
            impl FilterOp<$t> {
                /// Apply the filter operation to a numeric value.
                pub fn apply(&self, value: $t) -> $t {
                    match self {
                        FilterOp::Clamp { min, max } => value.clamp(*min, *max),
                        FilterOp::Chain(filters) => {
                            filters.iter().fold(value, |v, f| f.apply(v))
                        }
                        FilterOp::Custom(f) => f(value),
                        // String filters don't apply to numeric types
                        _ => value,
                    }
                }
            }
        )*
    };
}

impl_numeric_filter_op!(i32, i64, f32, f64);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_trim_string() {
    let filter = FilterOp::<String>::Trim;
    assert_eq!(filter.apply("  hello  ".to_string()), "hello");
  }

  #[test]
  fn test_lowercase_string() {
    let filter = FilterOp::<String>::Lowercase;
    assert_eq!(filter.apply("HELLO".to_string()), "hello");
  }

  #[test]
  fn test_uppercase_string() {
    let filter = FilterOp::<String>::Uppercase;
    assert_eq!(filter.apply("hello".to_string()), "HELLO");
  }

  #[test]
  fn test_strip_tags_string() {
    let filter = FilterOp::<String>::StripTags;
    let result = filter.apply("<script>alert('xss')</script>Hello".to_string());
    assert!(!result.contains("<script>"));
    assert!(result.contains("Hello"));
  }

  #[test]
  fn test_slug_string() {
    let filter = FilterOp::<String>::Slug { max_length: None };
    assert_eq!(filter.apply("Hello World!".to_string()), "hello-world");
  }

  #[test]
  fn test_chain_string() {
    let filter: FilterOp<String> = FilterOp::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);
    assert_eq!(filter.apply("  HELLO  ".to_string()), "hello");
  }

  #[test]
  fn test_custom_string() {
    let filter: FilterOp<String> =
      FilterOp::Custom(Arc::new(|s: String| s.replace("world", "rust")));
    assert_eq!(filter.apply("hello world".to_string()), "hello rust");
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_trim_value() {
    let filter = FilterOp::<Value>::Trim;
    let result = filter.apply(Value::Str("  hello  ".to_string()));
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_clamp_value_f64() {
    let filter = FilterOp::<Value>::Clamp {
      min: Value::F64(0.0),
      max: Value::F64(100.0),
    };
    assert_eq!(filter.apply(Value::F64(150.0)), Value::F64(100.0));
    assert_eq!(filter.apply(Value::F64(-10.0)), Value::F64(0.0));
    assert_eq!(filter.apply(Value::F64(50.0)), Value::F64(50.0));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_clamp_value_i64() {
    use walrs_validation::value;
    let filter = FilterOp::<Value>::Clamp {
      min: value!(0),
      max: value!(100),
    };
    assert_eq!(filter.apply(value!(150)), value!(100));
    assert_eq!(filter.apply(value!(-10)), value!(0));
    assert_eq!(filter.apply(value!(50)), value!(50));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_filter_preserves_non_matching_types() {
    let filter = FilterOp::<Value>::Trim;
    // Trim shouldn't affect non-string values
    assert_eq!(filter.apply(Value::I64(42)), Value::I64(42));
    assert_eq!(filter.apply(Value::Bool(true)), Value::Bool(true));
  }

  #[test]
  fn test_clamp_i32() {
    let filter = FilterOp::<i32>::Clamp { min: 0, max: 100 };
    assert_eq!(filter.apply(150), 100);
    assert_eq!(filter.apply(-10), 0);
    assert_eq!(filter.apply(50), 50);
  }

  #[test]
  fn test_filter_serialization() {
    let filter = FilterOp::<String>::Slug {
      max_length: Some(50),
    };
    let json = serde_json::to_string(&filter).unwrap();
    assert!(json.contains("Slug"));
    assert!(json.contains("50"));
  }

  // ====================================================================
  // apply_ref tests — FilterOp<String>
  // ====================================================================

  #[test]
  fn test_trim_string_apply_ref() {
    let filter = FilterOp::<String>::Trim;
    assert_eq!(filter.apply_ref("  hello  "), "hello");
  }

  #[test]
  fn test_lowercase_string_apply_ref() {
    let filter = FilterOp::<String>::Lowercase;
    assert_eq!(filter.apply_ref("HELLO"), "hello");
  }

  #[test]
  fn test_uppercase_string_apply_ref() {
    let filter = FilterOp::<String>::Uppercase;
    assert_eq!(filter.apply_ref("hello"), "HELLO");
  }

  #[test]
  fn test_strip_tags_string_apply_ref() {
    let filter = FilterOp::<String>::StripTags;
    let result = filter.apply_ref("<script>alert('xss')</script>Hello");
    assert!(!result.contains("<script>"));
    assert!(result.contains("Hello"));
  }

  #[test]
  fn test_html_entities_string_apply_ref() {
    let filter = FilterOp::<String>::HtmlEntities;
    let result = filter.apply_ref("<b>Hello</b>");
    assert!(result.contains("&lt;"));
    assert!(result.contains("&gt;"));
  }

  #[test]
  fn test_slug_string_apply_ref() {
    let filter = FilterOp::<String>::Slug { max_length: None };
    assert_eq!(filter.apply_ref("Hello World!"), "hello-world");
  }

  #[test]
  fn test_chain_string_apply_ref() {
    let filter: FilterOp<String> = FilterOp::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);
    assert_eq!(filter.apply_ref("  HELLO  "), "hello");
  }

  #[test]
  fn test_custom_string_apply_ref() {
    let filter: FilterOp<String> =
      FilterOp::Custom(Arc::new(|s: String| s.replace("world", "rust")));
    assert_eq!(filter.apply_ref("hello world"), "hello rust");
  }

  #[test]
  fn test_clamp_noop_on_string_apply_ref() {
    let filter = FilterOp::<String>::Clamp {
      min: "a".to_string(),
      max: "z".to_string(),
    };
    assert_eq!(filter.apply_ref("hello"), "hello");
  }

  // ====================================================================
  // apply_ref tests — FilterOp<Value>
  // ====================================================================

  #[cfg(feature = "validation")]
  #[test]
  fn test_trim_value_apply_ref() {
    let filter = FilterOp::<Value>::Trim;
    let value = Value::Str("  hello  ".to_string());
    assert_eq!(filter.apply_ref(&value), Value::Str("hello".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_lowercase_value_apply_ref() {
    let filter = FilterOp::<Value>::Lowercase;
    let value = Value::Str("HELLO".to_string());
    assert_eq!(filter.apply_ref(&value), Value::Str("hello".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_clamp_value_f64_apply_ref() {
    let filter = FilterOp::<Value>::Clamp {
      min: Value::F64(0.0),
      max: Value::F64(100.0),
    };
    let high = Value::F64(150.0);
    let low = Value::F64(-10.0);
    let mid = Value::F64(50.0);
    assert_eq!(filter.apply_ref(&high), Value::F64(100.0));
    assert_eq!(filter.apply_ref(&low), Value::F64(0.0));
    assert_eq!(filter.apply_ref(&mid), Value::F64(50.0));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_chain_value_apply_ref() {
    let filter: FilterOp<Value> = FilterOp::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);
    let value = Value::Str("  HELLO  ".to_string());
    assert_eq!(
      filter.apply_ref(&value),
      Value::Str("hello".to_string())
    );
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_filter_preserves_non_matching_types_apply_ref() {
    let filter = FilterOp::<Value>::Trim;
    let int_val = Value::I64(42);
    let bool_val = Value::Bool(true);
    assert_eq!(filter.apply_ref(&int_val), Value::I64(42));
    assert_eq!(filter.apply_ref(&bool_val), Value::Bool(true));
  }
}
