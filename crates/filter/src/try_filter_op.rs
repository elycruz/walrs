//! Composable, fallible filter operations for value transformation.
//!
//! This module provides a [`TryFilterOp<T>`] enum that represents composable
//! fallible filter operations. It is the fallible counterpart to
//! [`FilterOp<T>`](crate::FilterOp), allowing filters that can fail to
//! participate in the same processing pipeline.

use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::{FilterError, FilterOp};

#[cfg(feature = "value")]
use walrs_validation::Value;

/// Parse a permissive boolean literal (case-insensitive).
///
/// Accepts `1`, `0`, `true`, `false`, `yes`, `no`, `on`, `off` — surrounding whitespace is ignored.
///
/// Uses [`str::eq_ignore_ascii_case`] to avoid allocating a lowercased copy of the input;
/// the common already-canonical paths (`"true"` / `"false"`) stay allocation-free.
fn parse_bool_literal(s: &str) -> Result<bool, FilterError> {
  let t = s.trim();
  for truthy in ["true", "1", "yes", "on"] {
    if t.eq_ignore_ascii_case(truthy) {
      return Ok(true);
    }
  }
  for falsy in ["false", "0", "no", "off"] {
    if t.eq_ignore_ascii_case(falsy) {
      return Ok(false);
    }
  }
  Err(FilterError::new(format!("cannot parse {s:?} as bool")).with_name("ToBool"))
}

/// Iteratively flatten nested `Chain` variants into a list of non-chain operation references.
///
/// Prevents stack overflow when deeply nested `TryFilterOp::Chain(vec![TryFilterOp::Chain(…)])`
/// values are applied.
fn flatten_try_chain<T>(ops: &[TryFilterOp<T>]) -> Vec<&TryFilterOp<T>> {
  let mut flat = Vec::new();
  let mut stack: Vec<&TryFilterOp<T>> = ops.iter().rev().collect();
  while let Some(op) = stack.pop() {
    if let TryFilterOp::Chain(inner) = op {
      stack.extend(inner.iter().rev());
    } else {
      flat.push(op);
    }
  }
  flat
}

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

  /// Parse the value as a boolean using a permissive set of accepted literals.
  ///
  /// Accepts (case-insensitive): `"1"`, `"0"`, `"true"`, `"false"`, `"yes"`, `"no"`,
  /// `"on"`, `"off"`. Errors on anything else.
  ///
  /// - On `TryFilterOp<String>`: normalises to the canonical `"true"` / `"false"` string.
  /// - On `TryFilterOp<Value>` (feature `validation`): converts `Value::Str` → `Value::Bool`;
  ///   `Value::Bool` passes through; other variants pass through unchanged.
  ToBool,

  /// Parse the value as a signed 64-bit integer (`i64`), accepting surrounding whitespace.
  ///
  /// - On `TryFilterOp<String>`: normalises to the canonical decimal representation.
  ///   **Note:** a leading `+` sign is accepted by the parser but is **not**
  ///   preserved in the canonical output (`"+42"` → `"42"`). Likewise, leading
  ///   zeros are stripped (`"042"` → `"42"`).
  /// - On `TryFilterOp<Value>` (feature `validation`): converts `Value::Str` → `Value::I64`;
  ///   numeric variants pass through; other variants pass through unchanged.
  ToInt,

  /// Parse the value as a 64-bit floating-point number (`f64`), accepting surrounding whitespace.
  ///
  /// - On `TryFilterOp<String>`: normalises to `Display`-canonical form.
  ///   **Note:** this canonical form is what Rust's default `f64` `Display` impl
  ///   produces — which drops the fractional part for whole-valued floats
  ///   (`"3.0"` → `"3"`), normalises exponents (`"-1e2"` → `"-100"`), and
  ///   preserves the sign of `-0.0` (`"-0.0"` → `"-0"`).
  /// - On `TryFilterOp<Value>` (feature `validation`): converts `Value::Str` → `Value::F64`;
  ///   numeric variants pass through; other variants pass through unchanged.
  ToFloat,

  /// Percent-decode the value, interpreting the result as UTF-8.
  ///
  /// Errors when the decoded bytes are not valid UTF-8.
  UrlDecode,

  /// Custom fallible filter function (not serializable).
  ///
  /// # Serde limitation
  ///
  /// `TryCustom` is annotated with `#[serde(skip)]`. This means:
  /// - A `TryCustom` variant **cannot be serialized** — attempting to do so returns an error.
  /// - A `Chain` that contains a `TryCustom` will also **fail to serialize**.
  /// - Deserialization will never produce a `TryCustom` variant.
  ///
  /// If your filter pipeline must survive a serialization round-trip, avoid `TryCustom`
  /// or add the custom logic as a post-deserialization step.
  #[serde(skip)]
  TryCustom(Arc<dyn Fn(T) -> Result<T, FilterError> + Send + Sync>),
}

impl<T: Debug> Debug for TryFilterOp<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Infallible(op) => f.debug_tuple("Infallible").field(op).finish(),
      Self::Chain(ops) => f.debug_tuple("Chain").field(ops).finish(),
      Self::ToBool => write!(f, "ToBool"),
      Self::ToInt => write!(f, "ToInt"),
      Self::ToFloat => write!(f, "ToFloat"),
      Self::UrlDecode => write!(f, "UrlDecode"),
      Self::TryCustom(_) => write!(f, "TryCustom(<fn>)"),
    }
  }
}

impl<T: PartialEq> PartialEq for TryFilterOp<T> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Infallible(a), Self::Infallible(b)) => a == b,
      (Self::Chain(a), Self::Chain(b)) => a == b,
      (Self::ToBool, Self::ToBool) => true,
      (Self::ToInt, Self::ToInt) => true,
      (Self::ToFloat, Self::ToFloat) => true,
      (Self::UrlDecode, Self::UrlDecode) => true,
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
        let flat = flatten_try_chain(ops);
        if flat.is_empty() {
          return Ok(Cow::Borrowed(value));
        }
        let first_result = flat[0].try_apply_ref(value)?;
        if flat.len() == 1 {
          return Ok(first_result);
        }
        let mut result = first_result.into_owned();
        for op in &flat[1..] {
          result = op.try_apply(result)?;
        }
        Ok(Cow::Owned(result))
      }
      TryFilterOp::ToBool => {
        let b = parse_bool_literal(value)?;
        let canonical = if b { "true" } else { "false" };
        if value == canonical {
          Ok(Cow::Borrowed(value))
        } else {
          Ok(Cow::Owned(canonical.to_string()))
        }
      }
      TryFilterOp::ToInt => {
        let parsed: i64 = value.trim().parse().map_err(|e: std::num::ParseIntError| {
          FilterError::new(format!("cannot parse {value:?} as i64: {e}")).with_name("ToInt")
        })?;
        let canonical = parsed.to_string();
        if value == canonical {
          Ok(Cow::Borrowed(value))
        } else {
          Ok(Cow::Owned(canonical))
        }
      }
      TryFilterOp::ToFloat => {
        let parsed: f64 = value
          .trim()
          .parse()
          .map_err(|e: std::num::ParseFloatError| {
            FilterError::new(format!("cannot parse {value:?} as f64: {e}")).with_name("ToFloat")
          })?;
        let canonical = format!("{}", parsed);
        if value == canonical {
          Ok(Cow::Borrowed(value))
        } else {
          Ok(Cow::Owned(canonical))
        }
      }
      TryFilterOp::UrlDecode => {
        // `percent_decode_str(value).decode_utf8()` returns `Cow<'_, str>` borrowing from
        // `value`, so the lifetimes line up and we can return it directly.
        let decoded = percent_decode_str(value).decode_utf8().map_err(|e| {
          FilterError::new(format!("invalid utf-8 after percent-decode: {e}"))
            .with_name("UrlDecode")
        })?;
        Ok(decoded)
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
// Value TryFilterOp Implementation (requires "value" feature)
// ============================================================================

#[cfg(feature = "value")]
impl TryFilterOp<Value> {
  /// Apply the fallible filter to a `&Value` reference, returning a `Result<Value, FilterError>`.
  pub fn try_apply_ref(&self, value: &Value) -> Result<Value, FilterError> {
    match self {
      TryFilterOp::Infallible(op) => Ok(op.apply_ref(value)),
      TryFilterOp::Chain(ops) => {
        let flat = flatten_try_chain(ops);
        if flat.is_empty() {
          return Ok(value.clone());
        }
        let mut result = value.clone();
        for op in flat {
          result = op.try_apply_ref(&result)?;
        }
        Ok(result)
      }
      TryFilterOp::ToBool => match value {
        Value::Str(s) => Ok(Value::Bool(parse_bool_literal(s)?)),
        Value::Bool(_) => Ok(value.clone()),
        other => Ok(other.clone()),
      },
      TryFilterOp::ToInt => match value {
        Value::Str(s) => {
          let parsed: i64 = s.trim().parse().map_err(|e: std::num::ParseIntError| {
            FilterError::new(format!("cannot parse {s:?} as i64: {e}")).with_name("ToInt")
          })?;
          Ok(Value::I64(parsed))
        }
        other => Ok(other.clone()),
      },
      TryFilterOp::ToFloat => match value {
        Value::Str(s) => {
          let parsed: f64 = s.trim().parse().map_err(|e: std::num::ParseFloatError| {
            FilterError::new(format!("cannot parse {s:?} as f64: {e}")).with_name("ToFloat")
          })?;
          Ok(Value::F64(parsed))
        }
        other => Ok(other.clone()),
      },
      TryFilterOp::UrlDecode => match value {
        Value::Str(s) => {
          let decoded = percent_decode_str(s).decode_utf8().map_err(|e| {
            FilterError::new(format!("invalid utf-8 after percent-decode: {e}"))
              .with_name("UrlDecode")
          })?;
          Ok(Value::Str(decoded.into_owned()))
        }
        other => Ok(other.clone()),
      },
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
                            let flat = flatten_try_chain(ops);
                            flat.iter().try_fold(value, |v, op| op.try_apply(v))
                        }
                        // String-oriented conversions (`ToBool`, `ToInt`, `ToFloat`,
                        // `UrlDecode`) are only meaningful for `TryFilterOp<String>`.
                        // Constructing one with a numeric `T` is a programming error —
                        // panic loudly rather than silently pass the value through.
                        TryFilterOp::ToBool
                        | TryFilterOp::ToInt
                        | TryFilterOp::ToFloat
                        | TryFilterOp::UrlDecode => unreachable!(
                            "string-oriented TryFilterOp variant applied to numeric TryFilterOp<{}>; these variants are only valid for TryFilterOp<String>",
                            stringify!($t)
                        ),
                        TryFilterOp::TryCustom(f) => f(value),
                    }
                }
            }
        )*
    };
}

impl_numeric_try_filter_op!(i32, i64, f32, f64, u32, u64, usize);

// ============================================================================
// From<FilterOp<T>> for TryFilterOp<T> — lift infallible to fallible
// ============================================================================

impl<T> From<FilterOp<T>> for TryFilterOp<T> {
  fn from(op: FilterOp<T>) -> Self {
    TryFilterOp::Infallible(op)
  }
}

// ============================================================================
// TryFilter trait implementations
// ============================================================================

impl crate::TryFilter<String> for TryFilterOp<String> {
  type Output = String;

  fn try_filter(&self, value: String) -> Result<String, crate::FilterError> {
    self.try_apply(value)
  }
}

#[cfg(feature = "value")]
impl crate::TryFilter<Value> for TryFilterOp<Value> {
  type Output = Value;

  fn try_filter(&self, value: Value) -> Result<Value, crate::FilterError> {
    self.try_apply(value)
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
    let op: TryFilterOp<String> =
      TryFilterOp::TryCustom(Arc::new(|s: String| Ok(s.to_uppercase())));
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
      TryFilterOp::TryCustom(Arc::new(|_| Err(FilterError::new("always fails")))),
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
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![TryFilterOp::Infallible(FilterOp::Trim)]);
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

    let custom: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(Ok));
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
    let d: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(Ok));
    let e: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(Ok));
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

  #[cfg(feature = "value")]
  #[test]
  fn test_infallible_value_trim() {
    let op: TryFilterOp<Value> = TryFilterOp::Infallible(FilterOp::Trim);
    let result = op.try_apply(Value::Str("  hello  ".to_string())).unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
  #[test]
  fn test_chain_value() {
    let op: TryFilterOp<Value> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::Infallible(FilterOp::Lowercase),
    ]);
    let result = op.try_apply(Value::Str("  HELLO  ".to_string())).unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_chain_value_empty() {
    let op: TryFilterOp<Value> = TryFilterOp::Chain(vec![]);
    let value = Value::Str("hello".to_string());
    let result = op.try_apply_ref(&value).unwrap();
    assert_eq!(result, value);
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

  #[cfg(feature = "value")]
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

  // ====================================================================
  // Serde round-trip tests
  // ====================================================================

  #[test]
  fn test_serde_roundtrip_infallible() {
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: TryFilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  #[test]
  fn test_serde_roundtrip_chain() {
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::Infallible(FilterOp::Lowercase),
    ]);
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: TryFilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  #[test]
  fn test_try_custom_serde_skip_in_chain() {
    // TryCustom is annotated with #[serde(skip)] — it causes serialization to fail.
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::TryCustom(Arc::new(|s| Ok(s.to_uppercase()))),
      TryFilterOp::Infallible(FilterOp::Lowercase),
    ]);
    let result = serde_json::to_string(&op);
    assert!(
      result.is_err(),
      "Chain containing TryCustom should fail serialization"
    );
  }

  // ====================================================================
  // TryFilter trait tests
  // ====================================================================

  #[test]
  fn test_try_filter_trait_string() {
    use crate::TryFilter;
    let op: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    assert_eq!(op.try_filter("  hello  ".to_string()).unwrap(), "hello");
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_try_filter_trait_value() {
    use crate::TryFilter;
    let op: TryFilterOp<Value> = TryFilterOp::Infallible(FilterOp::Trim);
    assert_eq!(
      op.try_filter(Value::Str("  hello  ".to_string())).unwrap(),
      Value::Str("hello".to_string())
    );
  }

  // ====================================================================
  // Deep nesting tests — flatten_try_chain prevents stack overflow
  // ====================================================================

  #[test]
  fn test_deeply_nested_try_chain_string() {
    let mut chain: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    for _ in 0..10_000 {
      chain = TryFilterOp::Chain(vec![chain]);
    }
    assert_eq!(chain.try_apply("  hello  ".to_string()).unwrap(), "hello");
  }

  #[test]
  fn test_deeply_nested_try_chain_string_apply_ref() {
    let mut chain: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
    for _ in 0..10_000 {
      chain = TryFilterOp::Chain(vec![chain]);
    }
    assert_eq!(chain.try_apply_ref("  hello  ").unwrap(), "hello");
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_deeply_nested_try_chain_value() {
    let mut chain: TryFilterOp<Value> = TryFilterOp::Infallible(FilterOp::Trim);
    for _ in 0..10_000 {
      chain = TryFilterOp::Chain(vec![chain]);
    }
    assert_eq!(
      chain
        .try_apply(Value::Str("  hello  ".to_string()))
        .unwrap(),
      Value::Str("hello".to_string())
    );
  }

  #[test]
  fn test_deeply_nested_try_chain_numeric() {
    let mut chain: TryFilterOp<i32> = TryFilterOp::Infallible(FilterOp::Clamp { min: 0, max: 100 });
    for _ in 0..10_000 {
      chain = TryFilterOp::Chain(vec![chain]);
    }
    assert_eq!(chain.try_apply(150).unwrap(), 100);
  }

  // ====================================================================
  // Conversion filter tests — #235
  // ====================================================================

  // ---- ToBool on String ----

  #[test]
  fn test_to_bool_string_truthy_variants() {
    let op = TryFilterOp::<String>::ToBool;
    for input in ["1", "true", "TRUE", "Yes", "on", "ON"] {
      assert_eq!(op.try_apply(input.to_string()).unwrap(), "true",);
    }
  }

  #[test]
  fn test_to_bool_string_falsy_variants() {
    let op = TryFilterOp::<String>::ToBool;
    for input in ["0", "false", "FALSE", "no", "off", " OFF "] {
      assert_eq!(op.try_apply(input.to_string()).unwrap(), "false",);
    }
  }

  #[test]
  fn test_to_bool_string_invalid_errors() {
    let op = TryFilterOp::<String>::ToBool;
    let err = op.try_apply("maybe".to_string()).unwrap_err();
    assert_eq!(err.filter_name(), Some("ToBool"));
  }

  #[test]
  fn test_to_bool_string_already_canonical_is_borrowed() {
    let op = TryFilterOp::<String>::ToBool;
    let result = op.try_apply_ref("true").unwrap();
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "true");

    let result = op.try_apply_ref("false").unwrap();
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "false");
  }

  #[test]
  fn test_serde_roundtrip_to_bool() {
    let op = TryFilterOp::<String>::ToBool;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: TryFilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- ToInt on String ----

  #[test]
  fn test_to_int_string_canonicalizes() {
    let op = TryFilterOp::<String>::ToInt;
    assert_eq!(op.try_apply("042".to_string()).unwrap(), "42");
    assert_eq!(op.try_apply("  -7 ".to_string()).unwrap(), "-7");
  }

  #[test]
  fn test_to_int_string_already_canonical_is_borrowed() {
    let op = TryFilterOp::<String>::ToInt;
    let result = op.try_apply_ref("42").unwrap();
    assert!(matches!(result, Cow::Borrowed(_)));
  }

  #[test]
  fn test_to_int_string_invalid_errors() {
    let op = TryFilterOp::<String>::ToInt;
    let err = op.try_apply("abc".to_string()).unwrap_err();
    assert_eq!(err.filter_name(), Some("ToInt"));
  }

  #[test]
  fn test_to_int_string_overflow_errors() {
    let op = TryFilterOp::<String>::ToInt;
    // `i64::MAX` is 9223372036854775807 — this exceeds it.
    assert!(op.try_apply("9223372036854775808".to_string()).is_err());
  }

  #[test]
  fn test_to_int_string_plus_sign_stripped() {
    // Rust's `i64::parse` accepts a leading `+`; the canonical form does not
    // preserve it. Documented behaviour.
    let op = TryFilterOp::<String>::ToInt;
    assert_eq!(op.try_apply("+42".to_string()).unwrap(), "42");
    assert_eq!(op.try_apply("  +42  ".to_string()).unwrap(), "42");
  }

  #[test]
  fn test_serde_roundtrip_to_int() {
    let op = TryFilterOp::<String>::ToInt;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: TryFilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- ToFloat on String ----

  #[test]
  fn test_to_float_string_canonicalizes() {
    let op = TryFilterOp::<String>::ToFloat;
    assert_eq!(op.try_apply(" 3.0 ".to_string()).unwrap(), "3");
    assert_eq!(op.try_apply("2.5".to_string()).unwrap(), "2.5");
    assert_eq!(op.try_apply("-1e2".to_string()).unwrap(), "-100");
  }

  #[test]
  fn test_to_float_string_invalid_errors() {
    let op = TryFilterOp::<String>::ToFloat;
    let err = op.try_apply("xyz".to_string()).unwrap_err();
    assert_eq!(err.filter_name(), Some("ToFloat"));
  }

  #[test]
  fn test_to_float_string_negative_zero_preserved() {
    // Rust's `Display` for `f64` renders `-0.0` as `"-0"` — the sign survives
    // String canonicalisation.
    let op = TryFilterOp::<String>::ToFloat;
    assert_eq!(op.try_apply("-0.0".to_string()).unwrap(), "-0");
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_float_value_preserves_negative_zero() {
    // On `Value`, `-0.0` survives as a `Value::F64` with its sign bit intact.
    let op = TryFilterOp::<Value>::ToFloat;
    let result = op.try_apply(Value::Str("-0.0".to_string())).unwrap();
    if let Value::F64(f) = result {
      assert!(
        f.is_sign_negative(),
        "-0.0 sign should survive Value::F64 path"
      );
      assert_eq!(f, 0.0);
    } else {
      panic!("expected Value::F64, got {result:?}");
    }
  }

  #[test]
  fn test_serde_roundtrip_to_float() {
    let op = TryFilterOp::<String>::ToFloat;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: TryFilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- UrlDecode on String ----

  #[test]
  fn test_url_decode_string_basic() {
    let op = TryFilterOp::<String>::UrlDecode;
    assert_eq!(
      op.try_apply("hello%20world".to_string()).unwrap(),
      "hello world"
    );
  }

  #[test]
  fn test_url_decode_string_unicode() {
    let op = TryFilterOp::<String>::UrlDecode;
    assert_eq!(op.try_apply("caf%C3%A9".to_string()).unwrap(), "café");
  }

  #[test]
  fn test_url_decode_string_no_escapes_is_borrowed() {
    let op = TryFilterOp::<String>::UrlDecode;
    let result = op.try_apply_ref("plaintext").unwrap();
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "plaintext");
  }

  #[test]
  fn test_url_decode_string_invalid_utf8_errors() {
    // `%FF` alone is not a valid UTF-8 byte sequence.
    let op = TryFilterOp::<String>::UrlDecode;
    let err = op.try_apply("%FF".to_string()).unwrap_err();
    assert_eq!(err.filter_name(), Some("UrlDecode"));
  }

  #[test]
  fn test_url_decode_lone_percent_passes_through() {
    // percent-encoding's decoder is lenient: a lone `%` with no hex pair is left as-is.
    let op = TryFilterOp::<String>::UrlDecode;
    assert_eq!(op.try_apply("100%".to_string()).unwrap(), "100%");
  }

  #[test]
  fn test_serde_roundtrip_url_decode() {
    let op = TryFilterOp::<String>::UrlDecode;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: TryFilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- Chain composition with conversions ----

  #[test]
  fn test_chain_trim_then_to_int() {
    let op: TryFilterOp<String> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::ToInt,
    ]);
    assert_eq!(op.try_apply("  042  ".to_string()).unwrap(), "42");
  }

  // ---- Debug format coverage ----

  #[test]
  fn test_debug_format_conversions() {
    assert_eq!(format!("{:?}", TryFilterOp::<String>::ToBool), "ToBool");
    assert_eq!(format!("{:?}", TryFilterOp::<String>::ToInt), "ToInt");
    assert_eq!(format!("{:?}", TryFilterOp::<String>::ToFloat), "ToFloat");
    assert_eq!(
      format!("{:?}", TryFilterOp::<String>::UrlDecode),
      "UrlDecode"
    );
  }

  #[test]
  fn test_partial_eq_conversions() {
    assert_eq!(TryFilterOp::<String>::ToBool, TryFilterOp::<String>::ToBool);
    assert_ne!(TryFilterOp::<String>::ToBool, TryFilterOp::<String>::ToInt);
  }

  // ---- TryFilterOp<Value> conversion tests ----

  #[cfg(feature = "value")]
  #[test]
  fn test_to_bool_value_str_to_bool() {
    let op = TryFilterOp::<Value>::ToBool;
    assert_eq!(
      op.try_apply(Value::Str("yes".to_string())).unwrap(),
      Value::Bool(true)
    );
    assert_eq!(
      op.try_apply(Value::Str("0".to_string())).unwrap(),
      Value::Bool(false)
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_bool_value_bool_pass_through() {
    let op = TryFilterOp::<Value>::ToBool;
    assert_eq!(op.try_apply(Value::Bool(true)).unwrap(), Value::Bool(true));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_bool_value_other_pass_through() {
    let op = TryFilterOp::<Value>::ToBool;
    assert_eq!(op.try_apply(Value::I64(42)).unwrap(), Value::I64(42));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_int_value_str_to_i64() {
    let op = TryFilterOp::<Value>::ToInt;
    assert_eq!(
      op.try_apply(Value::Str("  -7  ".to_string())).unwrap(),
      Value::I64(-7)
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_int_value_str_invalid_errors() {
    let op = TryFilterOp::<Value>::ToInt;
    assert!(op.try_apply(Value::Str("abc".to_string())).is_err());
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_int_value_non_str_pass_through() {
    let op = TryFilterOp::<Value>::ToInt;
    assert_eq!(op.try_apply(Value::I64(99)).unwrap(), Value::I64(99));
    assert_eq!(op.try_apply(Value::F64(1.5)).unwrap(), Value::F64(1.5));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_float_value_str_to_f64() {
    let op = TryFilterOp::<Value>::ToFloat;
    assert_eq!(
      op.try_apply(Value::Str("3.25".to_string())).unwrap(),
      Value::F64(3.25)
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_to_float_value_non_str_pass_through() {
    let op = TryFilterOp::<Value>::ToFloat;
    assert_eq!(op.try_apply(Value::F64(2.5)).unwrap(), Value::F64(2.5));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_url_decode_value_str() {
    let op = TryFilterOp::<Value>::UrlDecode;
    assert_eq!(
      op.try_apply(Value::Str("hello%20world".to_string()))
        .unwrap(),
      Value::Str("hello world".to_string())
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_url_decode_value_non_str_pass_through() {
    let op = TryFilterOp::<Value>::UrlDecode;
    assert_eq!(op.try_apply(Value::I64(7)).unwrap(), Value::I64(7));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_chain_value_trim_then_to_int() {
    let op: TryFilterOp<Value> = TryFilterOp::Chain(vec![
      TryFilterOp::Infallible(FilterOp::Trim),
      TryFilterOp::ToInt,
    ]);
    assert_eq!(
      op.try_apply(Value::Str("  42  ".to_string())).unwrap(),
      Value::I64(42)
    );
  }

  // ---- Numeric types reject string-oriented conversions ----

  #[test]
  #[should_panic(expected = "string-oriented TryFilterOp variant applied to numeric")]
  fn test_numeric_to_int_panics() {
    // `ToInt` on a numeric `TryFilterOp<T>` is a misconfiguration — the variant is
    // only meaningful for `TryFilterOp<String>` (and `TryFilterOp<Value>`).
    let op = TryFilterOp::<i32>::ToInt;
    let _ = op.try_apply(42);
  }

  #[test]
  #[should_panic(expected = "string-oriented TryFilterOp variant applied to numeric")]
  fn test_numeric_to_bool_panics() {
    let op = TryFilterOp::<f64>::ToBool;
    let _ = op.try_apply(1.0);
  }

  #[test]
  #[should_panic(expected = "string-oriented TryFilterOp variant applied to numeric")]
  fn test_numeric_to_float_panics() {
    let op = TryFilterOp::<i64>::ToFloat;
    let _ = op.try_apply(1);
  }

  #[test]
  #[should_panic(expected = "string-oriented TryFilterOp variant applied to numeric")]
  fn test_numeric_url_decode_panics() {
    let op = TryFilterOp::<u32>::UrlDecode;
    let _ = op.try_apply(1);
  }
}
