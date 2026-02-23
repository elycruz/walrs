//! Filter enum for composable value transformation.
//!
//! This module provides a serializable `Filter<T>` enum that delegates to
//! existing filter implementations from `walrs_filter`.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::sync::Arc;
use walrs_filter::{Filter as FilterTrait, SlugFilter, StripTagsFilter, XmlEntitiesFilter};
use walrs_validation::Value;

/// A composable, serializable value transformer.
///
/// The `Filter` enum provides a way to define filters that can be serialized
/// to JSON/YAML for config-driven form processing. Most variants delegate to
/// implementations in the `walrs_filter` crate.
///
/// # Example
///
/// ```rust
/// use walrs_inputfilter::filter_enum::Filter;
///
/// let filter = Filter::<String>::Trim;
/// let result = filter.apply("  hello  ".to_string());
/// assert_eq!(result, "hello");
///
/// // Chain multiple filters
/// let chain: Filter<String> = Filter::Chain(vec![
///     Filter::Trim,
///     Filter::Lowercase,
/// ]);
/// let result = chain.apply("  HELLO  ".to_string());
/// assert_eq!(result, "hello");
/// ```
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Filter<T> {
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
  Chain(Vec<Filter<T>>),

  // ---- Custom ----
  /// Custom filter function (not serializable).
  #[serde(skip)]
  Custom(Arc<dyn Fn(T) -> T + Send + Sync>),
}

impl<T: Debug> Debug for Filter<T> {
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

impl<T: PartialEq> PartialEq for Filter<T> {
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
// String Filter Implementation
// ============================================================================

impl Filter<String> {
  /// Apply the filter to a String value.
  pub fn apply(&self, value: String) -> String {
    match self {
      Filter::Trim => value.trim().to_string(),
      Filter::Lowercase => value.to_lowercase(),
      Filter::Uppercase => value.to_uppercase(),
      Filter::StripTags => {
        let filter = StripTagsFilter::new();
        filter.filter(Cow::Owned(value)).into_owned()
      }
      Filter::HtmlEntities => {
        let filter = XmlEntitiesFilter::new();
        filter.filter(Cow::Owned(value)).into_owned()
      }
      Filter::Slug { max_length } => {
        let filter = SlugFilter::new(max_length.unwrap_or(200), false);
        filter.filter(Cow::Owned(value)).into_owned()
      }
      Filter::Clamp { .. } => value, // Clamp doesn't apply to strings
      Filter::Chain(filters) => filters.iter().fold(value, |v, f| f.apply(v)),
      Filter::Custom(f) => f(value),
    }
  }
}

// ============================================================================
// Value Filter Implementation
// ============================================================================

impl Filter<Value> {
  /// Apply the filter to a Value.
  ///
  /// String-based filters only apply to `Value::Str` variants.
  /// Numeric filters apply to `Value::I64` / `Value::U64` / `Value::F64` variants.
  pub fn apply(&self, value: Value) -> Value {
    match self {
      Filter::Trim => {
        if let Value::Str(s) = value {
          Value::Str(s.trim().to_string())
        } else {
          value
        }
      }
      Filter::Lowercase => {
        if let Value::Str(s) = value {
          Value::Str(s.to_lowercase())
        } else {
          value
        }
      }
      Filter::Uppercase => {
        if let Value::Str(s) = value {
          Value::Str(s.to_uppercase())
        } else {
          value
        }
      }
      Filter::StripTags => {
        if let Value::Str(s) = value {
          let filter = StripTagsFilter::new();
          Value::Str(filter.filter(Cow::Owned(s)).into_owned())
        } else {
          value
        }
      }
      Filter::HtmlEntities => {
        if let Value::Str(s) = value {
          let filter = XmlEntitiesFilter::new();
          Value::Str(filter.filter(Cow::Owned(s)).into_owned())
        } else {
          value
        }
      }
      Filter::Slug { max_length } => {
        if let Value::Str(s) = value {
          let filter = SlugFilter::new(max_length.unwrap_or(200), false);
          Value::Str(filter.filter(Cow::Owned(s)).into_owned())
        } else {
          value
        }
      }
      Filter::Clamp { min, max } => {
        match (&value, min, max) {
          (Value::I64(v), Value::I64(min_v), Value::I64(max_v)) => {
            Value::I64((*v).clamp(*min_v, *max_v))
          }
          (Value::U64(v), Value::U64(min_v), Value::U64(max_v)) => {
            Value::U64((*v).clamp(*min_v, *max_v))
          }
          (Value::F64(v), Value::F64(min_v), Value::F64(max_v)) => {
            Value::F64((*v).clamp(*min_v, *max_v))
          }
          _ => value,
        }
      }
      Filter::Chain(filters) => filters.iter().fold(value, |v, f| f.apply(v)),
      Filter::Custom(f) => f(value),
    }
  }
}

// ============================================================================
// Numeric Filter Implementations
// ============================================================================

macro_rules! impl_numeric_filter {
    ($($t:ty),*) => {
        $(
            impl Filter<$t> {
                /// Apply the filter to a numeric value.
                pub fn apply(&self, value: $t) -> $t {
                    match self {
                        Filter::Clamp { min, max } => value.clamp(*min, *max),
                        Filter::Chain(filters) => {
                            filters.iter().fold(value, |v, f| f.apply(v))
                        }
                        Filter::Custom(f) => f(value),
                        // String filters don't apply to numeric types
                        _ => value,
                    }
                }
            }
        )*
    };
}

impl_numeric_filter!(i32, i64, f32, f64);

#[cfg(test)]
mod tests {
  use super::*;
  use walrs_validation::value;

  #[test]
  fn test_trim_string() {
    let filter = Filter::<String>::Trim;
    assert_eq!(filter.apply("  hello  ".to_string()), "hello");
  }

  #[test]
  fn test_lowercase_string() {
    let filter = Filter::<String>::Lowercase;
    assert_eq!(filter.apply("HELLO".to_string()), "hello");
  }

  #[test]
  fn test_uppercase_string() {
    let filter = Filter::<String>::Uppercase;
    assert_eq!(filter.apply("hello".to_string()), "HELLO");
  }

  #[test]
  fn test_strip_tags_string() {
    let filter = Filter::<String>::StripTags;
    let result = filter.apply("<script>alert('xss')</script>Hello".to_string());
    assert!(!result.contains("<script>"));
    assert!(result.contains("Hello"));
  }

  #[test]
  fn test_slug_string() {
    let filter = Filter::<String>::Slug { max_length: None };
    assert_eq!(filter.apply("Hello World!".to_string()), "hello-world");
  }

  #[test]
  fn test_chain_string() {
    let filter: Filter<String> = Filter::Chain(vec![Filter::Trim, Filter::Lowercase]);
    assert_eq!(filter.apply("  HELLO  ".to_string()), "hello");
  }

  #[test]
  fn test_custom_string() {
    let filter: Filter<String> = Filter::Custom(Arc::new(|s: String| s.replace("world", "rust")));
    assert_eq!(filter.apply("hello world".to_string()), "hello rust");
  }

  #[test]
  fn test_trim_value() {
    let filter = Filter::<Value>::Trim;
    let result = filter.apply(Value::Str("  hello  ".to_string()));
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  #[test]
  fn test_clamp_value_f64() {
    let filter = Filter::<Value>::Clamp {
      min: Value::F64(0.0),
      max: Value::F64(100.0),
    };
    assert_eq!(filter.apply(Value::F64(150.0)), Value::F64(100.0));
    assert_eq!(filter.apply(Value::F64(-10.0)), Value::F64(0.0));
    assert_eq!(filter.apply(Value::F64(50.0)), Value::F64(50.0));
  }

  #[test]
  fn test_clamp_value_i64() {
    let filter = Filter::<Value>::Clamp {
      min: value!(0),
      max: value!(100),
    };
    assert_eq!(filter.apply(value!(150)), value!(100));
    assert_eq!(filter.apply(value!(-10)), value!(0));
    assert_eq!(filter.apply(value!(50)), value!(50));
  }

  #[test]
  fn test_filter_preserves_non_matching_types() {
    let filter = Filter::<Value>::Trim;
    // Trim shouldn't affect non-string values
    assert_eq!(filter.apply(Value::I64(42)), Value::I64(42));
    assert_eq!(filter.apply(Value::Bool(true)), Value::Bool(true));
  }

  #[test]
  fn test_clamp_i32() {
    let filter = Filter::<i32>::Clamp { min: 0, max: 100 };
    assert_eq!(filter.apply(150), 100);
    assert_eq!(filter.apply(-10), 0);
    assert_eq!(filter.apply(50), 50);
  }

  #[test]
  fn test_filter_serialization() {
    let filter = Filter::<String>::Slug {
      max_length: Some(50),
    };
    let json = serde_json::to_string(&filter).unwrap();
    assert!(json.contains("Slug"));
    assert!(json.contains("50"));
  }
}
