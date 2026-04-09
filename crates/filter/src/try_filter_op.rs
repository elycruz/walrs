//! Composable, fallible filter operations for value transformation.
//!
//! This module provides a [`TryFilterOp<T>`] enum that represents composable
//! fallible filter operations. It is the fallible counterpart to
//! [`FilterOp<T>`](crate::FilterOp), allowing filters that can fail to
//! participate in the same processing pipeline.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::{FilterError, FilterOp};

#[cfg(feature = "validation")]
use walrs_validation::Value;

/// A composable, fallible value transformer.
///
/// `TryFilterOp` provides a way to define fallible filter operations that can
/// be composed with infallible [`FilterOp`] filters. Errors are represented as
/// [`FilterError`], which can be converted to [`Violation`](walrs_validation::Violation)
/// for integration with the validation error pipeline.
///
/// # Variants
///
/// - [`Infallible`](Self::Infallible) — wraps an infallible `FilterOp`, lifting it into the fallible pipeline
/// - [`Chain`](Self::Chain) — applies filters sequentially, short-circuiting on the first error
/// - [`TryCustom`](Self::TryCustom) — custom fallible filter function
///
/// # Example
///
/// ```rust
/// use walrs_filter::{TryFilterOp, FilterOp, FilterError};
/// use std::sync::Arc;
///
/// // Lift an infallible filter into the fallible pipeline
/// let trim: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
/// assert_eq!(trim.try_apply("  hello  ".to_string()).unwrap(), "hello");
///
/// // Custom fallible filter
/// let parse_hex: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s: String| {
///     if s.chars().all(|c| c.is_ascii_hexdigit()) {
///         Ok(s.to_uppercase())
///     } else {
///         Err(FilterError::new("invalid hex string").with_name("HexNormalize"))
///     }
/// }));
/// assert_eq!(parse_hex.try_apply("abcdef".to_string()).unwrap(), "ABCDEF");
/// assert!(parse_hex.try_apply("xyz".to_string()).is_err());
///
/// // Chain: trim then apply custom fallible filter
/// let chain: TryFilterOp<String> = TryFilterOp::Chain(vec![
///     TryFilterOp::Infallible(FilterOp::Trim),
///     parse_hex.clone(),
/// ]);
/// assert_eq!(chain.try_apply("  abcdef  ".to_string()).unwrap(), "ABCDEF");
/// assert!(chain.try_apply("  xyz  ".to_string()).is_err());
/// ```
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum TryFilterOp<T> {
  /// Wraps an infallible [`FilterOp`], always succeeding.
  Infallible(FilterOp<T>),

  /// Applies fallible filters sequentially, short-circuiting on the first error.
  Chain(Vec<TryFilterOp<T>>),

  /// Custom fallible filter function (not serializable).
  #[serde(skip)]
  TryCustom(Arc<dyn Fn(T) -> Result<T, FilterError> + Send + Sync>),
}

impl<T: Debug> Debug for TryFilterOp<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Infallible(op) => f.debug_tuple("Infallible").field(op).finish(),
      Self::Chain(ops) => f.debug_tuple("Chain").field(ops).finish(),
      Self::TryCustom(_) => write!(f, "TryCustom(<fn>)"),
    }
  }
}

impl<T: PartialEq> PartialEq for TryFilterOp<T> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Infallible(a), Self::Infallible(b)) => a == b,
      (Self::Chain(a), Self::Chain(b)) => a == b,
      // TryCustom filters are never equal
      (Self::TryCustom(_), Self::TryCustom(_)) => false,
      _ => false,
    }
  }
}

// ============================================================================
// String TryFilterOp Implementation
// ============================================================================

impl TryFilterOp<String> {
  /// Apply the fallible filter to a `&str` reference, returning a `Result<Cow<'_, str>, FilterError>`.
  ///
  /// Returns `Ok(Cow::Borrowed)` when the filter is a no-op, avoiding allocation.
  /// Returns `Ok(Cow::Owned)` when the value is transformed.
  /// Returns `Err(FilterError)` when the filter fails.
  pub fn try_apply_ref<'a>(&self, value: &'a str) -> Result<Cow<'a, str>, FilterError> {
    match self {
      TryFilterOp::Infallible(op) => Ok(op.apply_ref(value)),
      TryFilterOp::Chain(ops) => {
        if ops.is_empty() {
          return Ok(Cow::Borrowed(value));
        }
        let first_result = ops[0].try_apply_ref(value)?;
        if ops.len() == 1 {
          return Ok(first_result);
        }
        let mut result = first_result.into_owned();
        for op in &ops[1..] {
          result = op.try_apply(result)?;
        }
        Ok(Cow::Owned(result))
      }
      TryFilterOp::TryCustom(f) => f(value.to_string()).map(Cow::Owned),
    }
  }

  /// Apply the fallible filter to an owned `String` value.
  ///
  /// Convenience wrapper that delegates to [`try_apply_ref`](Self::try_apply_ref).
  pub fn try_apply(&self, value: String) -> Result<String, FilterError> {
    self.try_apply_ref(&value).map(Cow::into_owned)
  }
}

// ============================================================================
// Value TryFilterOp Implementation (requires "validation" feature)
// ============================================================================

#[cfg(feature = "validation")]
impl TryFilterOp<Value> {
  /// Apply the fallible filter to a `&Value` reference, returning a `Result<Value, FilterError>`.
  pub fn try_apply_ref(&self, value: &Value) -> Result<Value, FilterError> {
    match self {
      TryFilterOp::Infallible(op) => Ok(op.apply_ref(value)),
      TryFilterOp::Chain(ops) => {
        let mut result = value.clone();
        for op in ops {
          result = op.try_apply_ref(&result)?;
        }
        Ok(result)
      }
      TryFilterOp::TryCustom(f) => f(value.clone()),
    }
  }

  /// Apply the fallible filter to an owned `Value`.
  pub fn try_apply(&self, value: Value) -> Result<Value, FilterError> {
    self.try_apply_ref(&value)
  }
}

// ============================================================================
// Numeric TryFilterOp Implementations
// ============================================================================

macro_rules! impl_numeric_try_filter_op {
    ($($t:ty),*) => {
        $(
            impl TryFilterOp<$t> {
                /// Apply the fallible filter to a numeric value.
                pub fn try_apply(&self, value: $t) -> Result<$t, FilterError> {
                    match self {
                        TryFilterOp::Infallible(op) => Ok(op.apply(value)),
                        TryFilterOp::Chain(ops) => {
                            ops.iter().try_fold(value, |v, op| op.try_apply(v))
                        }
                        TryFilterOp::TryCustom(f) => f(value),
                    }
                }
            }
        )*
    };
}

impl_numeric_try_filter_op!(i32, i64, f32, f64);

// ============================================================================
// From<FilterOp<T>> for TryFilterOp<T> — lift infallible to fallible
// ============================================================================

impl<T> From<FilterOp<T>> for TryFilterOp<T> {
  fn from(op: FilterOp<T>) -> Self {
    TryFilterOp::Infallible(op)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_infallible_string_trim() {
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    assert_eq!(op.try_apply("  hello  ".to_string()).unwrap(), "hello");
  }

  #[test]
  fn test_infallible_string_apply_ref() {
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    let result = op.try_apply_ref("  hello  ").unwrap();
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_try_custom_success() {
    let op: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s: String| {
      Ok(s.to_uppercase())
    }));
    assert_eq!(op.try_apply("hello".to_string()).unwrap(), "HELLO");
  }

  #[test]
  fn test_try_custom_failure() {
    let op: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s: String| {
      if s.is_empty() {
        Err(FilterError::new("input must not be empty"))
      } else {
        Ok(s)
      }
    }));
    assert!(op.try_apply("".to_string()).is_err());
    assert!(op.try_apply("hello".to_string()).is_ok());
  }

  #[test]
  fn test_chain_all_succeed() {
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::Infallible(FilterOp::Lowercase),
    ]);
    assert_eq!(op.try_apply("  HELLO  ".to_string()).unwrap(), "hello");
  }

  #[test]
  fn test_chain_short_circuits_on_error() {
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::TryCustom(Arc::new(|_| {
        Err(FilterError::new("always fails"))
      })),
      TryFilterOp::Infallible(FilterOp::Lowercase), // should not execute
    ]);
    let err = op.try_apply("  HELLO  ".to_string()).unwrap_err();
    assert_eq!(err.message(), "always fails");
  }

  #[test]
  fn test_chain_empty() {
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![]);
    assert_eq!(op.try_apply("hello".to_string()).unwrap(), "hello");
  }

  #[test]
  fn test_chain_single() {
    let op: TryFilterOp<String> =
      TryFilterOp::Chain(vec![TryFilterOp::Infallible(FilterOp::Trim)]);
    assert_eq!(op.try_apply("  hello  ".to_string()).unwrap(), "hello");
  }

  #[test]
  fn test_from_filter_op() {
    let infallible = FilterOp::<String>::Trim;
    let fallible: TryFilterOp<String> = infallible.into();
    assert_eq!(
      fallible.try_apply("  hello  ".to_string()).unwrap(),
      "hello"
    );
  }

  #[test]
  fn test_debug_format() {
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    let debug = format!("{:?}", op);
    assert!(debug.contains("Infallible"));
    assert!(debug.contains("Trim"));

    let custom: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s| Ok(s)));
    let debug = format!("{:?}", custom);
    assert!(debug.contains("TryCustom"));
  }

  #[test]
  fn test_partial_eq() {
    let a: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    let b: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    assert_eq!(a, b);

    let c: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Lowercase);
    assert_ne!(a, c);

    // TryCustom is never equal
    let d: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s| Ok(s)));
    let e: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s| Ok(s)));
    assert_ne!(d, e);
  }

  // ---- Numeric tests ----

  #[test]
  fn test_infallible_numeric_clamp() {
    let op: TryFilterOp<i32> = TryFilterOp::Infallible(FilterOp::Clamp { min: 0, max: 100 });
    assert_eq!(op.try_apply(150).unwrap(), 100);
    assert_eq!(op.try_apply(-10).unwrap(), 0);
    assert_eq!(op.try_apply(50).unwrap(), 50);
  }

  #[test]
  fn test_try_custom_numeric() {
    let op: TryFilterOp<i64> = TryFilterOp::TryCustom(Arc::new(|v| {
      if v < 0 {
        Err(FilterError::new("negative values not allowed"))
      } else {
        Ok(v * 2)
      }
    }));
    assert_eq!(op.try_apply(5).unwrap(), 10);
    assert!(op.try_apply(-1).is_err());
  }

  #[test]
  fn test_chain_numeric() {
    let op: TryFilterOp<i32> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Clamp { min: 0, max: 100 }),
      TryFilterOp::TryCustom(Arc::new(|v| Ok(v * 2))),
    ]);
    assert_eq!(op.try_apply(50).unwrap(), 100);
    assert_eq!(op.try_apply(200).unwrap(), 200); // clamped to 100, then * 2
  }

  // ---- Value tests ----

  #[cfg(feature = "validation")]
  #[test]
  fn test_infallible_value_trim() {
    let op: TryFilterOp<Value> = TryFilterOp::Infallible(FilterOp::Trim);
    let result = op
      .try_apply(Value::Str("  hello  ".to_string()))
      .unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_try_custom_value() {
    let op: TryFilterOp<Value> = TryFilterOp::TryCustom(Arc::new(|v: Value| {
      if let Value::Str(s) = &v {
        if s.is_empty() {
          Err(FilterError::new("empty string"))
        } else {
          Ok(Value::Str(s.to_uppercase()))
        }
      } else {
        Ok(v)
      }
    }));
    assert_eq!(
      op.try_apply(Value::Str("hello".to_string())).unwrap(),
      Value::Str("HELLO".to_string())
    );
    assert!(op.try_apply(Value::Str("".to_string())).is_err());
    // Non-string values pass through
    assert_eq!(op.try_apply(Value::I64(42)).unwrap(), Value::I64(42));
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_chain_value() {
    let op: TryFilterOp<Value> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::Infallible(FilterOp::Lowercase),
    ]);
    let result = op
      .try_apply(Value::Str("  HELLO  ".to_string()))
      .unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  // ---- apply_ref tests ----

  #[test]
  fn test_try_apply_ref_borrowed_noop() {
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    let result = op.try_apply_ref("hello").unwrap();
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_try_apply_ref_owned_when_modified() {
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    let result = op.try_apply_ref("  hello  ").unwrap();
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "hello");
  }

  #[cfg(feature = "validation")]
  #[test]
  fn test_value_try_apply_ref() {
    let op: TryFilterOp<Value> = TryFilterOp::Infallible(FilterOp::Trim);
    let value = Value::Str("  hello  ".to_string());
    let result = op.try_apply_ref(&value).unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  // ---- Serialization tests ----

  #[test]
  fn test_serialization_infallible() {
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    let json = serde_json::to_string(&op).unwrap();
    assert!(json.contains("Infallible"));
    assert!(json.contains("Trim"));
  }

  #[test]
  fn test_serialization_chain() {
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::Infallible(FilterOp::Lowercase),
    ]);
    let json = serde_json::to_string(&op).unwrap();
    assert!(json.contains("Chain"));
    assert!(json.contains("Trim"));
    assert!(json.contains("Lowercase"));
  }
}
