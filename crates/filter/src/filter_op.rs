//! Composable, serializable filter operations for value transformation.
//!
//! This module provides a `FilterOp<T>` enum that represents composable
//! filter operations. Most variants delegate to the filter struct
//! implementations in this crate (e.g., [`SlugFilter`], [`StripTagsFilter`]).

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::{Filter, SlugFilter, StripTagsFilter, XmlEntitiesFilter};

#[cfg(feature = "value")]
use walrs_validation::Value;

/// RFC 3986 §2.3 "unreserved" character set: `ALPHA / DIGIT / "-" / "." / "_" / "~"`.
///
/// Derived from [`NON_ALPHANUMERIC`] by re-admitting the four unreserved punctuation
/// characters. Used by [`FilterOp::UrlEncode`] when `encode_unreserved` is `false`.
const RFC_3986_UNRESERVED: AsciiSet = NON_ALPHANUMERIC
  .remove(b'-')
  .remove(b'.')
  .remove(b'_')
  .remove(b'~');

/// Collapse runs of whitespace to a single ASCII space and trim leading/trailing
/// whitespace. Returns `Cow::Borrowed` when the input is already normalized.
///
/// Performs a single early-exit scan to detect whether the input is already
/// normalized. Only rebuilds when a deviation is found — so the clean-input path
/// allocates nothing and exits as soon as the first anomaly (leading whitespace,
/// a non-space whitespace character, or a whitespace run) is spotted.
fn normalize_whitespace(value: &str) -> Cow<'_, str> {
  let mut prev_ws = false;
  let mut seen_any = false;
  let mut trailing_ws = false;
  let mut dirty = false;
  for c in value.chars() {
    let ws = c.is_whitespace();
    if !seen_any {
      if ws {
        dirty = true;
        break;
      }
      seen_any = true;
    } else if ws && (prev_ws || c != ' ') {
      dirty = true;
      break;
    }
    prev_ws = ws;
    trailing_ws = ws;
  }
  if !dirty && !trailing_ws {
    return Cow::Borrowed(value);
  }

  let mut out = String::with_capacity(value.len());
  let mut prev_ws = true; // seed `true` so leading whitespace is dropped
  for c in value.chars() {
    if c.is_whitespace() {
      if !prev_ws {
        out.push(' ');
      }
      prev_ws = true;
    } else {
      out.push(c);
      prev_ws = false;
    }
  }
  if out.ends_with(' ') {
    out.pop();
  }
  Cow::Owned(out)
}

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

  /// Encode special characters as XML/HTML entities. Existing named,
  /// decimal, and hex entity references in the input are preserved
  /// verbatim so repeated application does not double-encode.
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

  // ---- Sanitize Filters ----
  /// Keep only ASCII digit characters (`0`–`9`).
  ///
  /// Returns `Cow::Borrowed` when the input is already digits-only.
  Digits,

  /// Keep only Unicode alphanumeric characters (optionally whitespace).
  ///
  /// `char::is_alphanumeric` is used, so non-ASCII letters and digits
  /// (`é`, `Ⅳ`, `日`) are preserved.
  Alnum {
    /// When `true`, any Unicode whitespace character is also kept — detected via
    /// [`char::is_whitespace`]. This includes tabs (`\t`), newlines (`\n`, `\r`),
    /// non-breaking space (U+00A0), and other Unicode whitespace — not just ASCII space.
    allow_whitespace: bool,
  },

  /// Keep only Unicode alphabetic characters (optionally whitespace).
  ///
  /// `char::is_alphabetic` is used, so non-ASCII letters (`é`, `日`) are preserved.
  Alpha {
    /// When `true`, any Unicode whitespace character is also kept — detected via
    /// [`char::is_whitespace`]. This includes tabs (`\t`), newlines (`\n`, `\r`),
    /// non-breaking space (U+00A0), and other Unicode whitespace — not just ASCII space.
    allow_whitespace: bool,
  },

  /// Remove `\r` and `\n` characters.
  StripNewlines,

  /// Collapse runs of whitespace to a single space, and trim leading/trailing whitespace.
  ///
  /// Mirrors Laminas\Filter\PregReplace-style whitespace normalization. Whitespace is
  /// detected via `char::is_whitespace` (Unicode-aware).
  NormalizeWhitespace,

  /// Keep only characters that appear in `set`.
  AllowChars {
    /// Characters to keep; any character not in this set is dropped.
    set: String,
  },

  /// Remove characters that appear in `set`.
  DenyChars {
    /// Characters to drop; any character in this set is removed.
    set: String,
  },

  /// Percent-encode the string.
  ///
  /// By default (`encode_unreserved: false`), conforms to RFC 3986 §2.3: ASCII
  /// alphanumerics and the four "unreserved" punctuation characters (`-`, `.`,
  /// `_`, `~`) are left as-is; everything else is `%HH`-encoded.
  ///
  /// Set `encode_unreserved: true` for the stricter `percent-encoding`
  /// `NON_ALPHANUMERIC` behaviour, which also encodes `-._~` — useful when
  /// building opaque tokens where any non-alphanumeric byte must be escaped.
  ///
  /// Infallible.
  UrlEncode {
    /// When `true`, also percent-encode the RFC 3986 "unreserved" characters
    /// `-`, `.`, `_`, and `~`. When `false` (default-ish — RFC-compliant),
    /// these pass through.
    encode_unreserved: bool,
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
      Self::Digits => write!(f, "Digits"),
      Self::Alnum { allow_whitespace } => f
        .debug_struct("Alnum")
        .field("allow_whitespace", allow_whitespace)
        .finish(),
      Self::Alpha { allow_whitespace } => f
        .debug_struct("Alpha")
        .field("allow_whitespace", allow_whitespace)
        .finish(),
      Self::StripNewlines => write!(f, "StripNewlines"),
      Self::NormalizeWhitespace => write!(f, "NormalizeWhitespace"),
      Self::AllowChars { set } => f.debug_struct("AllowChars").field("set", set).finish(),
      Self::DenyChars { set } => f.debug_struct("DenyChars").field("set", set).finish(),
      Self::UrlEncode { encode_unreserved } => f
        .debug_struct("UrlEncode")
        .field("encode_unreserved", encode_unreserved)
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
      (Self::Digits, Self::Digits) => true,
      (
        Self::Alnum {
          allow_whitespace: a,
        },
        Self::Alnum {
          allow_whitespace: b,
        },
      ) => a == b,
      (
        Self::Alpha {
          allow_whitespace: a,
        },
        Self::Alpha {
          allow_whitespace: b,
        },
      ) => a == b,
      (Self::StripNewlines, Self::StripNewlines) => true,
      (Self::NormalizeWhitespace, Self::NormalizeWhitespace) => true,
      (Self::AllowChars { set: a }, Self::AllowChars { set: b }) => a == b,
      (Self::DenyChars { set: a }, Self::DenyChars { set: b }) => a == b,
      (
        Self::UrlEncode {
          encode_unreserved: a,
        },
        Self::UrlEncode {
          encode_unreserved: b,
        },
      ) => a == b,
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
      FilterOp::Digits => {
        if value.chars().all(|c| c.is_ascii_digit()) {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.chars().filter(|c| c.is_ascii_digit()).collect())
        }
      }
      FilterOp::Alnum { allow_whitespace } => {
        let keep = |c: char| c.is_alphanumeric() || (*allow_whitespace && c.is_whitespace());
        if value.chars().all(&keep) {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.chars().filter(|c| keep(*c)).collect())
        }
      }
      FilterOp::Alpha { allow_whitespace } => {
        let keep = |c: char| c.is_alphabetic() || (*allow_whitespace && c.is_whitespace());
        if value.chars().all(&keep) {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.chars().filter(|c| keep(*c)).collect())
        }
      }
      FilterOp::StripNewlines => {
        if !value.chars().any(|c| c == '\n' || c == '\r') {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.chars().filter(|c| *c != '\n' && *c != '\r').collect())
        }
      }
      FilterOp::NormalizeWhitespace => normalize_whitespace(value),
      FilterOp::AllowChars { set } => {
        if value.chars().all(|c| set.contains(c)) {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.chars().filter(|c| set.contains(*c)).collect())
        }
      }
      FilterOp::DenyChars { set } => {
        if set.is_empty() || !value.chars().any(|c| set.contains(c)) {
          Cow::Borrowed(value)
        } else {
          Cow::Owned(value.chars().filter(|c| !set.contains(*c)).collect())
        }
      }
      FilterOp::UrlEncode { encode_unreserved } => {
        let set: &AsciiSet = if *encode_unreserved {
          NON_ALPHANUMERIC
        } else {
          &RFC_3986_UNRESERVED
        };
        // `PercentEncode` materialises as `Cow<str>` — `Borrowed` when nothing needed encoding.
        utf8_percent_encode(value, set).into()
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
// Value FilterOp Implementation (requires "value" feature)
// ============================================================================

/// Apply a string-typed `FilterOp` to a `Value::Str` variant, preserving the
/// zero-copy `Borrowed → clone the original Value` shortcut that existing `Trim`
/// / `Lowercase` arms use. Non-string variants pass through unchanged.
#[cfg(feature = "value")]
fn apply_string_op_to_value(op: FilterOp<String>, value: &Value) -> Value {
  if let Value::Str(s) = value {
    match op.apply_ref(s.as_str()) {
      Cow::Borrowed(_) => value.clone(),
      Cow::Owned(new_s) => Value::Str(new_s),
    }
  } else {
    value.clone()
  }
}

#[cfg(feature = "value")]
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
      FilterOp::Digits => apply_string_op_to_value(FilterOp::Digits, value),
      FilterOp::Alnum { allow_whitespace } => apply_string_op_to_value(
        FilterOp::Alnum {
          allow_whitespace: *allow_whitespace,
        },
        value,
      ),
      FilterOp::Alpha { allow_whitespace } => apply_string_op_to_value(
        FilterOp::Alpha {
          allow_whitespace: *allow_whitespace,
        },
        value,
      ),
      FilterOp::StripNewlines => apply_string_op_to_value(FilterOp::StripNewlines, value),
      FilterOp::NormalizeWhitespace => {
        apply_string_op_to_value(FilterOp::NormalizeWhitespace, value)
      }
      FilterOp::AllowChars { set } => {
        apply_string_op_to_value(FilterOp::AllowChars { set: set.clone() }, value)
      }
      FilterOp::DenyChars { set } => {
        apply_string_op_to_value(FilterOp::DenyChars { set: set.clone() }, value)
      }
      FilterOp::UrlEncode { encode_unreserved } => apply_string_op_to_value(
        FilterOp::UrlEncode {
          encode_unreserved: *encode_unreserved,
        },
        value,
      ),
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

#[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
  #[test]
  fn test_trim_value() {
    let filter = FilterOp::<Value>::Trim;
    let result = filter.apply(Value::Str("  hello  ".to_string()));
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
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
  fn test_html_entities_preserves_existing_entities() {
    let filter = FilterOp::<String>::HtmlEntities;

    // Already-encoded input should pass through unchanged (no double-encoding).
    assert_eq!(filter.apply_ref("Tom &amp; Jerry"), "Tom &amp; Jerry");
    assert_eq!(filter.apply_ref("&#39;hi&#39;"), "&#39;hi&#39;");
    assert_eq!(filter.apply_ref("&#x2F;path"), "&#x2F;path");

    // Raw specials around an existing entity — only the raw ones get encoded.
    assert_eq!(
      filter.apply_ref("<b>Tom &amp; Jerry</b>"),
      "&lt;b&gt;Tom &amp; Jerry&lt;/b&gt;"
    );
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

  #[cfg(feature = "value")]
  #[test]
  fn test_trim_value_apply_ref() {
    let filter = FilterOp::<Value>::Trim;
    let value = Value::Str("  hello  ".to_string());
    assert_eq!(filter.apply_ref(&value), Value::Str("hello".to_string()));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_lowercase_value_apply_ref() {
    let filter = FilterOp::<Value>::Lowercase;
    let value = Value::Str("HELLO".to_string());
    assert_eq!(filter.apply_ref(&value), Value::Str("hello".to_string()));
  }

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
  #[test]
  fn test_chain_value_apply_ref() {
    let filter: FilterOp<Value> = FilterOp::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);
    let value = Value::Str("  HELLO  ".to_string());
    assert_eq!(filter.apply_ref(&value), Value::Str("hello".to_string()));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_chain_value_empty_returns_original() {
    let filter: FilterOp<Value> = FilterOp::Chain(vec![]);
    let value = Value::Str("hello".to_string());
    assert_eq!(filter.apply_ref(&value), value);
  }

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
  #[test]
  fn test_trim_value_noop() {
    let filter = FilterOp::<Value>::Trim;
    let value = Value::Str("already_trimmed".to_string());
    let result = filter.apply_ref(&value);
    assert_eq!(result, Value::Str("already_trimmed".to_string()));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_lowercase_value_noop() {
    let filter = FilterOp::<Value>::Lowercase;
    let value = Value::Str("already lowercase 123".to_string());
    let result = filter.apply_ref(&value);
    assert_eq!(result, Value::Str("already lowercase 123".to_string()));
  }

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
  #[test]
  fn test_truncate_value_str() {
    let filter = FilterOp::<Value>::Truncate { max_length: 5 };
    let value = Value::Str("Hello World".to_string());
    assert_eq!(filter.apply_ref(&value), Value::Str("Hello".to_string()));
  }

  #[cfg(feature = "value")]
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

  #[cfg(feature = "value")]
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

  // ====================================================================
  // Sanitize filter tests — #235
  // ====================================================================

  // ---- Digits ----

  #[test]
  fn test_digits_strips_non_digits() {
    let filter = FilterOp::<String>::Digits;
    assert_eq!(filter.apply("abc123-def!456".to_string()), "123456");
  }

  #[test]
  fn test_digits_empty_input() {
    let filter = FilterOp::<String>::Digits;
    let result = filter.apply_ref("");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "");
  }

  #[test]
  fn test_digits_all_digits_is_borrowed() {
    let filter = FilterOp::<String>::Digits;
    let result = filter.apply_ref("1234567890");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "1234567890");
  }

  #[test]
  fn test_digits_mutation_is_owned() {
    let filter = FilterOp::<String>::Digits;
    let result = filter.apply_ref("a1b2c3");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "123");
  }

  #[test]
  fn test_digits_unicode_digits_are_dropped() {
    // `char::is_ascii_digit` rejects Unicode digits like `Ⅳ` and `٣` — design choice
    // to match PHP's FILTER_SANITIZE_NUMBER_INT semantics.
    let filter = FilterOp::<String>::Digits;
    assert_eq!(filter.apply("Ⅳ and 3".to_string()), "3");
  }

  #[test]
  fn test_serde_roundtrip_digits() {
    let op = FilterOp::<String>::Digits;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- Alnum ----

  #[test]
  fn test_alnum_keeps_letters_and_digits() {
    let filter = FilterOp::<String>::Alnum {
      allow_whitespace: false,
    };
    assert_eq!(filter.apply("abc 123 !@#".to_string()), "abc123");
  }

  #[test]
  fn test_alnum_allow_whitespace() {
    let filter = FilterOp::<String>::Alnum {
      allow_whitespace: true,
    };
    assert_eq!(filter.apply("abc 123 !@#".to_string()), "abc 123 ");
  }

  #[test]
  fn test_alnum_unicode() {
    // `char::is_alphanumeric` is Unicode-aware: `é`, `日` stay in, `-` drops out.
    let filter = FilterOp::<String>::Alnum {
      allow_whitespace: false,
    };
    assert_eq!(filter.apply("café-日本語".to_string()), "café日本語");
  }

  #[test]
  fn test_alnum_clean_input_is_borrowed() {
    let filter = FilterOp::<String>::Alnum {
      allow_whitespace: false,
    };
    let result = filter.apply_ref("abc123");
    assert!(matches!(result, Cow::Borrowed(_)));
  }

  #[test]
  fn test_serde_roundtrip_alnum() {
    let op = FilterOp::<String>::Alnum {
      allow_whitespace: true,
    };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- Alpha ----

  #[test]
  fn test_alpha_drops_digits() {
    let filter = FilterOp::<String>::Alpha {
      allow_whitespace: false,
    };
    assert_eq!(filter.apply("abc 123!".to_string()), "abc");
  }

  #[test]
  fn test_alpha_allow_whitespace() {
    let filter = FilterOp::<String>::Alpha {
      allow_whitespace: true,
    };
    assert_eq!(filter.apply("abc 123 def".to_string()), "abc  def");
  }

  #[test]
  fn test_alpha_unicode() {
    let filter = FilterOp::<String>::Alpha {
      allow_whitespace: false,
    };
    assert_eq!(filter.apply("日本語1-2".to_string()), "日本語");
  }

  #[test]
  fn test_serde_roundtrip_alpha() {
    let op = FilterOp::<String>::Alpha {
      allow_whitespace: false,
    };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- StripNewlines ----

  #[test]
  fn test_strip_newlines_removes_lf_and_cr() {
    let filter = FilterOp::<String>::StripNewlines;
    assert_eq!(filter.apply("a\nb\r\nc\rd".to_string()), "abcd");
  }

  #[test]
  fn test_strip_newlines_noop_when_clean() {
    let filter = FilterOp::<String>::StripNewlines;
    let result = filter.apply_ref("no newlines here");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "no newlines here");
  }

  #[test]
  fn test_strip_newlines_preserves_tabs_and_spaces() {
    let filter = FilterOp::<String>::StripNewlines;
    assert_eq!(filter.apply("a\tb c".to_string()), "a\tb c");
  }

  #[test]
  fn test_serde_roundtrip_strip_newlines() {
    let op = FilterOp::<String>::StripNewlines;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- NormalizeWhitespace ----

  #[test]
  fn test_normalize_whitespace_collapses_runs() {
    let filter = FilterOp::<String>::NormalizeWhitespace;
    assert_eq!(
      filter.apply("  hello    world\n\tgo  ".to_string()),
      "hello world go"
    );
  }

  #[test]
  fn test_normalize_whitespace_noop_when_clean() {
    let filter = FilterOp::<String>::NormalizeWhitespace;
    let result = filter.apply_ref("hello world go");
    assert!(matches!(result, Cow::Borrowed(_)));
  }

  #[test]
  fn test_normalize_whitespace_empty_string() {
    let filter = FilterOp::<String>::NormalizeWhitespace;
    let result = filter.apply_ref("");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "");
  }

  #[test]
  fn test_normalize_whitespace_whitespace_only() {
    let filter = FilterOp::<String>::NormalizeWhitespace;
    assert_eq!(filter.apply("   \t\n  ".to_string()), "");
  }

  #[test]
  fn test_normalize_whitespace_tab_becomes_space() {
    // Tabs aren't the ASCII space — normalising should collapse them to spaces.
    let filter = FilterOp::<String>::NormalizeWhitespace;
    assert_eq!(filter.apply("a\tb".to_string()), "a b");
  }

  #[test]
  fn test_normalize_whitespace_non_ascii_whitespace() {
    // Non-breaking space (U+00A0) is `char::is_whitespace` — it should collapse
    // to an ASCII space, not pass through unchanged.
    let filter = FilterOp::<String>::NormalizeWhitespace;
    let result = filter.apply("a\u{00A0}b".to_string());
    assert_eq!(result, "a b");
  }

  #[test]
  fn test_serde_roundtrip_normalize_whitespace() {
    let op = FilterOp::<String>::NormalizeWhitespace;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- AllowChars ----

  #[test]
  fn test_allow_chars_keeps_listed() {
    let filter = FilterOp::<String>::AllowChars {
      set: "abc".to_string(),
    };
    assert_eq!(filter.apply("abracadabra 123".to_string()), "abacaaba");
  }

  #[test]
  fn test_allow_chars_clean_input_is_borrowed() {
    let filter = FilterOp::<String>::AllowChars {
      set: "abc".to_string(),
    };
    let result = filter.apply_ref("cab");
    assert!(matches!(result, Cow::Borrowed(_)));
  }

  #[test]
  fn test_allow_chars_empty_set_drops_everything() {
    let filter = FilterOp::<String>::AllowChars { set: String::new() };
    assert_eq!(filter.apply("anything".to_string()), "");
  }

  #[test]
  fn test_allow_chars_unicode_set() {
    // Unicode chars in both set and input — the set uses `str::contains(char)`
    // so non-ASCII members (`é`, `ñ`) match on code-point equality.
    let filter = FilterOp::<String>::AllowChars {
      set: "café".to_string(),
    };
    let result = filter.apply_ref("café");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "café");

    assert_eq!(filter.apply("café-!".to_string()), "café");
  }

  #[test]
  fn test_serde_roundtrip_allow_chars() {
    let op = FilterOp::<String>::AllowChars {
      set: "0123456789".to_string(),
    };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- DenyChars ----

  #[test]
  fn test_deny_chars_removes_listed() {
    let filter = FilterOp::<String>::DenyChars {
      set: "aeiou".to_string(),
    };
    assert_eq!(filter.apply("Hello World".to_string()), "Hll Wrld");
  }

  #[test]
  fn test_deny_chars_empty_set_is_borrowed() {
    let filter = FilterOp::<String>::DenyChars { set: String::new() };
    let result = filter.apply_ref("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
  }

  #[test]
  fn test_deny_chars_no_overlap_is_borrowed() {
    let filter = FilterOp::<String>::DenyChars {
      set: "xyz".to_string(),
    };
    let result = filter.apply_ref("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
  }

  #[test]
  fn test_serde_roundtrip_deny_chars() {
    let op = FilterOp::<String>::DenyChars {
      set: "<>&".to_string(),
    };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);
  }

  // ---- UrlEncode ----

  #[test]
  fn test_url_encode_basic_rfc3986_default() {
    let filter = FilterOp::<String>::UrlEncode {
      encode_unreserved: false,
    };
    assert_eq!(filter.apply("hello world".to_string()), "hello%20world");
  }

  #[test]
  fn test_url_encode_rfc3986_preserves_unreserved_punctuation() {
    // `-._~` are RFC 3986 unreserved and must pass through when encode_unreserved = false.
    let filter = FilterOp::<String>::UrlEncode {
      encode_unreserved: false,
    };
    let result = filter.apply_ref("a-b_c.d~e");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "a-b_c.d~e");
  }

  #[test]
  fn test_url_encode_aggressive_encodes_unreserved_punctuation() {
    // With encode_unreserved = true, the stricter NON_ALPHANUMERIC set is used.
    let filter = FilterOp::<String>::UrlEncode {
      encode_unreserved: true,
    };
    assert_eq!(filter.apply("a-b_c.d~e".to_string()), "a%2Db%5Fc%2Ed%7Ee");
  }

  #[test]
  fn test_url_encode_unicode() {
    let filter = FilterOp::<String>::UrlEncode {
      encode_unreserved: false,
    };
    assert_eq!(filter.apply("café".to_string()), "caf%C3%A9");
  }

  #[test]
  fn test_url_encode_alphanumeric_only_is_borrowed() {
    let filter = FilterOp::<String>::UrlEncode {
      encode_unreserved: false,
    };
    let result = filter.apply_ref("HelloWorld123");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "HelloWorld123");
  }

  #[test]
  fn test_url_encode_special_chars() {
    let filter = FilterOp::<String>::UrlEncode {
      encode_unreserved: false,
    };
    assert_eq!(filter.apply("a&b=c".to_string()), "a%26b%3Dc");
  }

  #[test]
  fn test_serde_roundtrip_url_encode() {
    let op = FilterOp::<String>::UrlEncode {
      encode_unreserved: false,
    };
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op, deserialized);

    let op_strict = FilterOp::<String>::UrlEncode {
      encode_unreserved: true,
    };
    let json = serde_json::to_string(&op_strict).unwrap();
    let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(op_strict, deserialized);
    // The two modes are distinguishable via PartialEq.
    assert_ne!(op, op_strict);
  }

  // ---- Chain composition with new variants ----

  #[test]
  fn test_chain_trim_then_digits() {
    let filter: FilterOp<String> = FilterOp::Chain(vec![FilterOp::Trim, FilterOp::Digits]);
    assert_eq!(filter.apply("  abc123def  ".to_string()), "123");
  }

  #[test]
  fn test_chain_normalize_then_allow_chars() {
    let filter: FilterOp<String> = FilterOp::Chain(vec![
      FilterOp::NormalizeWhitespace,
      FilterOp::AllowChars {
        set: "abcdefghijklmnopqrstuvwxyz ".to_string(),
      },
    ]);
    assert_eq!(filter.apply("  hello  WORLD  ".to_string()), "hello ");
  }

  // ---- Debug format coverage ----

  #[test]
  fn test_debug_format_sanitize_variants() {
    assert_eq!(format!("{:?}", FilterOp::<String>::Digits), "Digits");
    let alnum = format!(
      "{:?}",
      FilterOp::<String>::Alnum {
        allow_whitespace: true
      }
    );
    assert!(alnum.contains("Alnum"));
    assert!(alnum.contains("allow_whitespace"));
    assert_eq!(
      format!("{:?}", FilterOp::<String>::StripNewlines),
      "StripNewlines"
    );
    assert_eq!(
      format!("{:?}", FilterOp::<String>::NormalizeWhitespace),
      "NormalizeWhitespace"
    );
    let url = format!(
      "{:?}",
      FilterOp::<String>::UrlEncode {
        encode_unreserved: false
      }
    );
    assert!(url.contains("UrlEncode"));
    assert!(url.contains("encode_unreserved"));
  }

  // ---- FilterOp<Value> sanitize filter tests ----

  #[cfg(feature = "value")]
  #[test]
  fn test_digits_value_str() {
    let filter = FilterOp::<Value>::Digits;
    assert_eq!(
      filter.apply(Value::Str("abc123".to_string())),
      Value::Str("123".to_string())
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_digits_value_non_str_pass_through() {
    let filter = FilterOp::<Value>::Digits;
    assert_eq!(filter.apply(Value::I64(42)), Value::I64(42));
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_normalize_whitespace_value_str() {
    let filter = FilterOp::<Value>::NormalizeWhitespace;
    assert_eq!(
      filter.apply(Value::Str("  hi   there  ".to_string())),
      Value::Str("hi there".to_string())
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_url_encode_value_str() {
    let filter = FilterOp::<Value>::UrlEncode {
      encode_unreserved: false,
    };
    assert_eq!(
      filter.apply(Value::Str("hello world".to_string())),
      Value::Str("hello%20world".to_string())
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_allow_chars_value_str() {
    let filter = FilterOp::<Value>::AllowChars {
      set: "abc".to_string(),
    };
    assert_eq!(
      filter.apply(Value::Str("abracadabra".to_string())),
      Value::Str("abacaaba".to_string())
    );
  }

  #[cfg(feature = "value")]
  #[test]
  fn test_deny_chars_value_str() {
    let filter = FilterOp::<Value>::DenyChars {
      set: "aeiou".to_string(),
    };
    assert_eq!(
      filter.apply(Value::Str("hello".to_string())),
      Value::Str("hll".to_string())
    );
  }
}
