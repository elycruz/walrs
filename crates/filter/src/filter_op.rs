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

/// Iteratively flatten nested `Chain` variants into a list of non-chain operation references.
///
/// Prevents stack overflow when deeply nested `FilterOp::Chain(vec![FilterOp::Chain(…)])`
/// values are applied.
fn flatten_chain<T>(filters: &[FilterOp<T>]) -> Vec<&FilterOp<T>> {
  let mut flat = Vec::new();
  let mut stack: Vec<&FilterOp<T>> = filters.iter().rev().collect();
  while let Some(op) = stack.pop() {
    if let FilterOp::Chain(inner) = op {
      stack.extend(inner.iter().rev());
    } else {
      flat.push(op);
    }
  }
  flat
}

/// A composable, serializable value transformer.
///
/// `FilterOp` provides a way to define filter operations that can be serialized
/// to JSON/YAML for config-driven form processing. Most variants delegate to
/// filter struct implementations in the `walrs_filter` crate.
///
/// # Calling convention
///
/// - **`apply(&self, value: T) -> T`** — for `Copy` (scalar/numeric) types.
/// - **`apply_ref(&self, value: &str) -> Cow<'_, str>`** — for `FilterOp<String>`,
///   returns `Cow::Borrowed` when input is unchanged (zero-copy) or `Cow::Owned`
///   when the value is transformed.
///
/// For `FilterOp<String>`, prefer `apply_ref` when you already have a `&str`,
/// avoiding an allocation when the filter is a no-op. `apply` is a convenience
/// wrapper that delegates to `apply_ref`.
///
/// # Example
///
/// ```rust
/// use walrs_filter::FilterOp;
/// use std::borrow::Cow;
///
/// let filter = FilterOp::<String>::Trim;
///
/// // No-op case — returns Cow::Borrowed, zero allocation
/// let result = filter.apply_ref("already_trimmed");
/// assert!(matches!(result, Cow::Borrowed(_)));
///
/// // Mutation case — returns Cow::Owned
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

  /// Truncate a string to at most `max_length` characters (Unicode scalar values).
  ///
  /// Unlike [`Slug`](Self::Slug), `Truncate` does not alter the content of the string —
  /// it simply clips it at a character boundary. Non-string types are passed through unchanged.
  Truncate {
    /// Maximum number of Unicode scalar values to keep.
    max_length: usize,
  },

  /// Replace all occurrences of `from` with `to` in a string.
  ///
  /// Non-string types are passed through unchanged.
  Replace {
    /// Substring to search for.
    from: String,
    /// Replacement string.
    to: String,
  },

  // ---- Numeric Filters ----
  /// Clamp value to a range (for numeric types).
  ///
  /// # Panics
  ///
  /// Panics if `min > max`.
  ///
  /// For floating-point types, this also panics if either bound is `NaN`,
  /// matching the behavior of `clamp()`.
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
  ///
  /// # Serde limitation
  ///
  /// `Custom` is annotated with `#[serde(skip)]`. This means:
  /// - A `Custom` variant **cannot be serialized** — attempting to do so returns an error.
  /// - A `Chain` that contains a `Custom` variant will also **fail to serialize** because
  ///   serde will encounter the un-serializable variant.
  /// - Deserialization will never produce a `Custom` variant.
  ///
  /// If your filter pipeline must survive a serialization round-trip, avoid `Custom`
  /// or add the custom logic as a post-deserialization step.
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
      Self::Truncate { max_length } => f
        .debug_struct("Truncate")
        .field("max_length", max_length)
        .finish(),
      Self::Replace { from, to } => f
        .debug_struct("Replace")
        .field("from", from)
        .field("to", to)
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
      (Self::Truncate { max_length: a }, Self::Truncate { max_length: b }) => a == b,
      (Self::Replace { from: fa, to: ta }, Self::Replace { from: fb, to: tb }) => {
        fa == fb && ta == tb
      }
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
  /// Apply the filter operation to a `&str` reference, returning a `Cow<'_, str>`.
  ///
  /// Returns `Cow::Borrowed` when the filter is a no-op (input unchanged),
  /// avoiding allocation. Returns `Cow::Owned` when the value is transformed.
  ///
  /// Prefer this method when you already have a `&str`, avoiding an
  /// unnecessary allocation at the call site.
  pub fn apply_ref<'a>(&self, value: &'a str) -> Cow<'a, str> {
    match self {
      FilterOp::Trim => {
        let trimmed = value.trim();
        if trimmed.len() == value.len() {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(trimmed.to_string())
        }
      }
      FilterOp::Lowercase => {
        if value
          .chars()
          .all(|c| c.is_lowercase() || !c.is_alphabetic())
        {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.to_lowercase())
        }
      }
      FilterOp::Uppercase => {
        if value
          .chars()
          .all(|c| c.is_uppercase() || !c.is_alphabetic())
        {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.to_uppercase())
        }
      }
      FilterOp::StripTags => {
        let filter = StripTagsFilter::new();
        filter.filter(Cow::Borrowed(value))
      }
      FilterOp::HtmlEntities => {
        let filter = XmlEntitiesFilter::new();
        filter.filter(Cow::Borrowed(value))
      }
      FilterOp::Slug { max_length } => {
        let filter = SlugFilter::new(max_length.unwrap_or(200), false);
        filter.filter(Cow::Borrowed(value))
      }
      FilterOp::Truncate { max_length } => {
        let mut char_count = 0;
        for (idx, _) in value.char_indices() {
          char_count += 1;
          if char_count > *max_length {
            return Cow::Owned(value[..idx].to_string());
          }
        }
        Cow::Borrowed(value)
      }
      FilterOp::Replace { from, to } => {
        if from.is_empty() || !value.contains(from.as_str()) {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.replace(from.as_str(), to.as_str()))
        }
      }
      FilterOp::Clamp { .. } => Cow::Borrowed(value), // Clamp doesn't apply to strings
      FilterOp::Chain(filters) => {
        let flat = flatten_chain(filters);
        if flat.is_empty() {
          return Cow::Borrowed(value);
        }
        let first_result = flat[0].apply_ref(value);
        if flat.len() == 1 {
          return first_result;
        }
        let mut result = first_result.into_owned();
        for f in &flat[1..] {
          match f.apply_ref(&result) {
            Cow::Borrowed(_) => {} // No change, keep result as-is
            Cow::Owned(s) => result = s,
          }
        }
        Cow::Owned(result)
      }
      FilterOp::Custom(f) => Cow::Owned(f(value.to_string())),
    }
  }

  /// Apply the filter operation to an owned `String` value.
  ///
  /// Convenience wrapper that delegates to [`apply_ref`](Self::apply_ref).
  pub fn apply(&self, value: String) -> String {
    self.apply_ref(&value).into_owned()
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
          let trimmed = s.trim();
          if trimmed.len() == s.len() {
            value.clone()
          } else {
            Value::Str(trimmed.to_string())
          }
        } else {
          value.clone()
        }
      }
      FilterOp::Lowercase => {
        if let Value::Str(s) = value {
          if s.chars().all(|c| c.is_lowercase() || !c.is_alphabetic()) {
            value.clone()
          } else {
            Value::Str(s.to_lowercase())
          }
        } else {
          value.clone()
        }
      }
      FilterOp::Uppercase => {
        if let Value::Str(s) = value {
          if s.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
            value.clone()
          } else {
            Value::Str(s.to_uppercase())
          }
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
      FilterOp::Truncate { max_length } => {
        if let Value::Str(s) = value {
          match s.char_indices().nth(*max_length) {
            None => value.clone(),
            Some((idx, _)) => Value::Str(s[..idx].to_string()),
          }
        } else {
          value.clone()
        }
      }
      FilterOp::Replace { from, to } => {
        if let Value::Str(s) = value {
          if from.is_empty() || !s.contains(from.as_str()) {
            value.clone()
          } else {
            Value::Str(s.replace(from.as_str(), to.as_str()))
          }
        } else {
          value.clone()
        }
      }
      FilterOp::Clamp { min, max } => match (value, min, max) {
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
      },
      FilterOp::Chain(filters) => {
        let flat = flatten_chain(filters);
        if flat.is_empty() {
          return value.clone();
        }
        let mut result = value.clone();
        for f in flat {
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
                        FilterOp::Clamp { min, max } => {
                            value.clamp(*min, *max)
                        }
                        FilterOp::Chain(filters) => {
                            let flat = flatten_chain(filters);
                            flat.iter().fold(value, |v, f| f.apply(v))
                        }
                        FilterOp::Custom(f) => f(value),
                        // String/other filters don't apply to numeric types
                        _ => value,
                    }
                }
            }

            impl crate::Filter<$t> for FilterOp<$t> {
                type Output = $t;
                fn filter(&self, value: $t) -> $t {
                    self.apply(value)
                }
            }
        )*
    };
}

impl_numeric_filter_op!(i32, i64, f32, f64, u32, u64, usize);

// ============================================================================
// Filter trait implementations for FilterOp<String> and FilterOp<Value>
// ============================================================================

impl crate::Filter<String> for FilterOp<String> {
  type Output = String;

  fn filter(&self, value: String) -> String {
    self.apply(value)
  }
}

#[cfg(feature = "validation")]
impl crate::Filter<Value> for FilterOp<Value> {
  type Output = Value;

  fn filter(&self, value: Value) -> Value {
    self.apply(value)
  }
}

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
    assert_eq!(filter.apply_ref(&value), Value::Str("hello".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_chain_value_empty_returns_original() {
    let filter: FilterOp<Value> = FilterOp::Chain(vec![]);
    let value = Value::Str("hello".to_string());
    assert_eq!(filter.apply_ref(&value), value);
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

  // ====================================================================
  // No-op (zero-copy) tests — FilterOp<String>::apply_ref
  // ====================================================================

  #[test]
  fn test_trim_noop_returns_borrowed() {
    let filter = FilterOp::<String>::Trim;
    let result = filter.apply_ref("already_trimmed");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "already_trimmed");
  }

  #[test]
  fn test_lowercase_noop_returns_borrowed() {
    let filter = FilterOp::<String>::Lowercase;
    let result = filter.apply_ref("already lowercase 123");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "already lowercase 123");
  }

  #[test]
  fn test_uppercase_noop_returns_borrowed() {
    let filter = FilterOp::<String>::Uppercase;
    let result = filter.apply_ref("ALREADY UPPERCASE 123");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "ALREADY UPPERCASE 123");
  }

  #[test]
  fn test_strip_tags_noop_returns_borrowed() {
    let filter = FilterOp::<String>::StripTags;
    // No HTML tags — filter should detect no-op
    let result = filter.apply_ref("Hello World");
    assert_eq!(result, "Hello World");
  }

  #[test]
  fn test_html_entities_noop_returns_borrowed() {
    let filter = FilterOp::<String>::HtmlEntities;
    // No special characters — filter should detect no-op
    let result = filter.apply_ref("Hello World");
    assert_eq!(result, "Hello World");
  }

  #[test]
  fn test_slug_noop_returns_borrowed() {
    let filter = FilterOp::<String>::Slug { max_length: None };
    // Already a valid slug
    let result = filter.apply_ref("hello-world");
    assert_eq!(result, "hello-world");
  }

  #[test]
  fn test_clamp_noop_returns_borrowed() {
    let filter = FilterOp::<String>::Clamp {
      min: "a".to_string(),
      max: "z".to_string(),
    };
    let result = filter.apply_ref("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_chain_empty_returns_borrowed() {
    let filter: FilterOp<String> = FilterOp::Chain(vec![]);
    let result = filter.apply_ref("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_trim_mutation_returns_owned() {
    let filter = FilterOp::<String>::Trim;
    let result = filter.apply_ref("  hello  ");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_lowercase_mutation_returns_owned() {
    let filter = FilterOp::<String>::Lowercase;
    let result = filter.apply_ref("HELLO");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_uppercase_mutation_returns_owned() {
    let filter = FilterOp::<String>::Uppercase;
    let result = filter.apply_ref("hello");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "hello".to_uppercase());
  }

  // ====================================================================
  // No-op tests — FilterOp<Value>::apply_ref
  // ====================================================================

  #[cfg(feature = "validation")]
  #[test]
  fn test_trim_value_noop() {
    let filter = FilterOp::<Value>::Trim;
    let value = Value::Str("already_trimmed".to_string());
    let result = filter.apply_ref(&value);
    assert_eq!(result, Value::Str("already_trimmed".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_lowercase_value_noop() {
    let filter = FilterOp::<Value>::Lowercase;
    let value = Value::Str("already lowercase 123".to_string());
    let result = filter.apply_ref(&value);
    assert_eq!(result, Value::Str("already lowercase 123".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_uppercase_value_noop() {
    let filter = FilterOp::<Value>::Uppercase;
    let value = Value::Str("ALREADY UPPERCASE 123".to_string());
    let result = filter.apply_ref(&value);
    assert_eq!(result, Value::Str("ALREADY UPPERCASE 123".to_string()));
  }

  // ====================================================================
  // New variant tests — Truncate and Replace
  // ====================================================================

  #[test]
  fn test_truncate_string_shorter_than_max() {
    let filter = FilterOp::<String>::Truncate { max_length: 10 };
    let result = filter.apply_ref("short");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "short");
  }

  #[test]
  fn test_truncate_string_exceeds_max() {
    let filter = FilterOp::<String>::Truncate { max_length: 5 };
    let result = filter.apply_ref("Hello World");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "Hello");
  }

  #[test]
  fn test_truncate_string_unicode() {
    // "café" has 4 Unicode scalar values
    let filter = FilterOp::<String>::Truncate { max_length: 3 };
    let result = filter.apply("café".to_string());
    assert_eq!(result, "caf");
  }

  #[test]
  fn test_replace_string_match() {
    let filter = FilterOp::<String>::Replace {
      from: "world".to_string(),
      to: "rust".to_string(),
    };
    let result = filter.apply_ref("hello world");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "hello rust");
  }

  #[test]
  fn test_replace_string_no_match() {
    let filter = FilterOp::<String>::Replace {
      from: "xyz".to_string(),
      to: "abc".to_string(),
    };
    let result = filter.apply_ref("hello world");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "hello world");
  }

  #[test]
  fn test_replace_all_occurrences() {
    let filter = FilterOp::<String>::Replace {
      from: "o".to_string(),
      to: "0".to_string(),
    };
    assert_eq!(filter.apply("foo bar boo".to_string()), "f00 bar b00");
  }

  #[test]
  fn test_replace_empty_from_is_noop() {
    // Empty `from` must be treated as a no-op — returning Cow::Borrowed.
    let filter = FilterOp::<String>::Replace {
      from: "".to_string(),
      to: "x".to_string(),
    };
    let result = filter.apply_ref("hello world");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "hello world");
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_replace_value_empty_from_is_noop() {
    // Empty `from` on Value::Str must also be a no-op (returns the original clone).
    let filter = FilterOp::<Value>::Replace {
      from: "".to_string(),
      to: "x".to_string(),
    };
    let value = Value::Str("hello world".to_string());
    assert_eq!(filter.apply_ref(&value), value);
  }

  // ====================================================================
  // f32 Clamp tests
  // ====================================================================

  #[test]
  fn test_clamp_f32() {
    let filter = FilterOp::<f32>::Clamp {
      min: 0.0_f32,
      max: 1.0_f32,
    };
    assert_eq!(filter.apply(1.5_f32), 1.0_f32);
    assert_eq!(filter.apply(-0.5_f32), 0.0_f32);
    assert_eq!(filter.apply(0.5_f32), 0.5_f32);
  }

  // ====================================================================
  // Expanded numeric type tests (u32, u64, usize)
  // ====================================================================

  #[test]
  fn test_clamp_u32() {
    let filter = FilterOp::<u32>::Clamp { min: 10, max: 100 };
    assert_eq!(filter.apply(200_u32), 100_u32);
    assert_eq!(filter.apply(5_u32), 10_u32);
    assert_eq!(filter.apply(50_u32), 50_u32);
  }

  #[test]
  fn test_clamp_u64() {
    let filter = FilterOp::<u64>::Clamp { min: 0, max: 1000 };
    assert_eq!(filter.apply(5000_u64), 1000_u64);
    assert_eq!(filter.apply(500_u64), 500_u64);
  }

  #[test]
  fn test_clamp_usize() {
    let filter = FilterOp::<usize>::Clamp { min: 1, max: 255 };
    assert_eq!(filter.apply(300_usize), 255_usize);
    assert_eq!(filter.apply(0_usize), 1_usize);
    assert_eq!(filter.apply(128_usize), 128_usize);
  }

  // ====================================================================
  // Filter trait impl tests
  // ====================================================================

  #[test]
  fn test_filter_trait_string() {
    use crate::Filter;
    let filter = FilterOp::<String>::Trim;
    assert_eq!(filter.filter("  hello  ".to_string()), "hello");
  }

  #[test]
  fn test_filter_trait_i32() {
    use crate::Filter;
    let filter = FilterOp::<i32>::Clamp { min: 0, max: 10 };
    assert_eq!(filter.filter(20), 10);
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_filter_trait_value() {
    use crate::Filter;
    let filter = FilterOp::<Value>::Trim;
    assert_eq!(
      filter.filter(Value::Str("  hello  ".to_string())),
      Value::Str("hello".to_string())
    );
  }

  // ====================================================================
  // Slug max_length truncation test
  // ====================================================================

  #[test]
  fn test_slug_max_length_truncation() {
    // "hello-world-this-is-a-long-title" is 32 chars
    let filter = FilterOp::<String>::Slug {
      max_length: Some(11),
    };
    let result = filter.apply("Hello World This Is A Long Title".to_string());
    assert!(
      result.len() <= 11,
      "slug length {} exceeds max 11",
      result.len()
    );
  }

  // ====================================================================
  // Serde round-trip tests
  // ====================================================================

  #[test]
  fn test_serde_roundtrip_trim() {
    let op = FilterOp::<String>::Trim;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  #[test]
  fn test_serde_roundtrip_slug() {
    let op = FilterOp::<String>::Slug {
      max_length: Some(50),
    };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  #[test]
  fn test_serde_roundtrip_chain() {
    let op: FilterOp<String> = FilterOp::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  #[test]
  fn test_serde_roundtrip_truncate() {
    let op = FilterOp::<String>::Truncate { max_length: 20 };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  #[test]
  fn test_serde_roundtrip_replace() {
    let op = FilterOp::<String>::Replace {
      from: "foo".to_string(),
      to: "bar".to_string(),
    };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  #[test]
  fn test_serde_roundtrip_clamp_i32() {
    let op = FilterOp::<i32>::Clamp { min: 0, max: 100 };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ====================================================================
  // Custom serde behavior test — Custom cannot be serialized
  // ====================================================================

  #[test]
  fn test_custom_serde_skip_direct() {
    // A standalone Custom variant cannot be serialized — returns an error.
    let op: FilterOp<String> = FilterOp::Custom(Arc::new(|s: String| s.to_uppercase()));
    let result = serde_json::to_string(&op);
    assert!(result.is_err(), "Custom variant should fail serialization");
  }

  #[test]
  fn test_custom_serde_skip_in_chain() {
    // A Chain containing Custom also fails to serialize.
    let op: FilterOp<String> = FilterOp::Chain(vec![
      FilterOp::Trim,
      FilterOp::Custom(Arc::new(|s: String| s.to_uppercase())),
      FilterOp::Lowercase,
    ]);
    let result = serde_json::to_string(&op);
    assert!(
      result.is_err(),
      "Chain containing Custom should fail serialization"
    );
  }

  // ====================================================================
  // Value — Truncate and Replace tests
  // ====================================================================

  #[cfg(feature = "validation")]
  #[test]
  fn test_truncate_value_str() {
    let filter = FilterOp::<Value>::Truncate { max_length: 5 };
    let value = Value::Str("Hello World".to_string());
    assert_eq!(filter.apply_ref(&value), Value::Str("Hello".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_replace_value_str() {
    let filter = FilterOp::<Value>::Replace {
      from: "World".to_string(),
      to: "Rust".to_string(),
    };
    let value = Value::Str("Hello World".to_string());
    assert_eq!(
      filter.apply_ref(&value),
      Value::Str("Hello Rust".to_string())
    );
  }

  // ====================================================================
  // Deep nesting tests — flatten_chain prevents stack overflow
  // ====================================================================

  #[test]
  fn test_deeply_nested_chain_string() {
    let mut chain = FilterOp::<String>::Trim;
    for _ in 0..10_000 {
      chain = FilterOp::Chain(vec![chain]);
    }
    assert_eq!(chain.apply("  hello  ".to_string()), "hello");
  }

  #[test]
  fn test_deeply_nested_chain_string_apply_ref() {
    let mut chain = FilterOp::<String>::Trim;
    for _ in 0..10_000 {
      chain = FilterOp::Chain(vec![chain]);
    }
    assert_eq!(chain.apply_ref("  hello  "), "hello");
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_deeply_nested_chain_value() {
    let mut chain = FilterOp::<Value>::Trim;
    for _ in 0..10_000 {
      chain = FilterOp::Chain(vec![chain]);
    }
    assert_eq!(
      chain.apply(Value::Str("  hello  ".to_string())),
      Value::Str("hello".to_string())
    );
  }

  #[test]
  fn test_deeply_nested_chain_numeric() {
    let mut chain = FilterOp::<i32>::Clamp { min: 0, max: 100 };
    for _ in 0..10_000 {
      chain = FilterOp::Chain(vec![chain]);
    }
    assert_eq!(chain.apply(150), 100);
  }

  #[test]
  fn test_nested_chain_with_multiple_ops() {
    // Build a chain of chains where the inner chains have multiple operations
    let inner = FilterOp::<String>::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);
    let mut chain = inner;
    for _ in 0..1_000 {
      chain = FilterOp::Chain(vec![chain]);
    }
    assert_eq!(chain.apply("  HELLO  ".to_string()), "hello");
  }
}
