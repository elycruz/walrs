use crate::{Violation};
use indexmap::{IndexMap, IndexSet};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

/// Result type for validation operations.
pub type ValidatorResult = Result<(), Violation>;

/// Trait for validating owned/copied values.
///
/// This trait is implemented by validators that work with `Copy` types.
pub trait Validate<T> {
  fn validate(&self, value: T) -> ValidatorResult;
}

/// Trait for validating referenced values.
///
/// This trait is implemented by validators that work with unsized types
/// like `str`, `[T]`, etc.
pub trait ValidateRef<T: ?Sized> {
  fn validate_ref(&self, value: &T) -> ValidatorResult;
}

use serde::Serialize;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Rem, Sub};

/// Macro to implement a marker trait for multiple types.
macro_rules! impl_marker_trait {
  ($trait:ident for $($ty:ty),+ $(,)?) => {
    $(impl $trait for $ty {})+
  };
}

pub trait ScalarValue: Copy + Default + PartialEq + PartialOrd + Display + Serialize {}

impl_marker_trait!(ScalarValue for
  i8, i16, i32, i64, i128, isize,
  u8, u16, u32, u64, u128, usize,
  f32, f64,
  bool, char
);

/// Trait for numeric types that support step/remainder validation.
///
/// This extends `NumberValue` with a `rem_check` method for validating
/// that a value is a multiple of a given step.
pub trait SteppableValue: ScalarValue + Add + Sub + Mul + Div + Rem<Output = Self> {
  /// Returns `true` if `self` is evenly divisible by `divisor`.
  ///
  /// For integer types, returns `false` if divisor is zero.
  /// For floating-point types, uses epsilon comparison.
  fn rem_check(self, divisor: Self) -> bool;
}

macro_rules! impl_steppable_integer {
  ($($t:ty),*) => {
    $(
      impl SteppableValue for $t {
        fn rem_check(self, divisor: Self) -> bool {
          if divisor == 0 {
            false
          } else {
            self % divisor == 0
          }
        }
      }
    )*
  };
}

impl_steppable_integer!(
  i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl SteppableValue for f32 {
  fn rem_check(self, divisor: Self) -> bool {
    if divisor == 0.0 {
      false
    } else {
      (self % divisor).abs() < f32::EPSILON
    }
  }
}

impl SteppableValue for f64 {
  fn rem_check(self, divisor: Self) -> bool {
    if divisor == 0.0 {
      false
    } else {
      (self % divisor).abs() < f64::EPSILON
    }
  }
}

/// Trait for types that can be converted to HTML form element attributes.
#[cfg(feature = "serde_json_bridge")]
pub trait ToAttributesList {
  /// Returns the validator's rules as key/value pairs suitable for HTML attributes.
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>>;
}

// ============================================================================
// IsEmpty
// ============================================================================

/// Trait for checking if a value is "empty", used by [`crate::Condition`] evaluation.
///
/// For strings "empty" means blank/whitespace-only.  For collections it means
/// zero elements.  Numeric scalars, `bool`, and `char` are never empty.
pub trait IsEmpty {
  /// Returns `true` if the value is considered empty.
  fn is_empty(&self) -> bool;
}

impl IsEmpty for String {
  fn is_empty(&self) -> bool { self.trim().is_empty() }
}

impl IsEmpty for str {
  fn is_empty(&self) -> bool { self.trim().is_empty() }
}

impl IsEmpty for &str {
  fn is_empty(&self) -> bool { self.trim().is_empty() }
}

impl<T> IsEmpty for Vec<T> {
  fn is_empty(&self) -> bool { self.is_empty() }
}

impl<T> IsEmpty for Option<T> {
  fn is_empty(&self) -> bool { self.is_none() }
}

impl IsEmpty for crate::Value {
  fn is_empty(&self) -> bool {
    crate::ValueExt::is_empty_value(self)
  }
}

/// Implements [`IsEmpty`] for types that are *never* considered empty
/// (numeric scalars, `bool`, `char`). Always returns `false`.
macro_rules! impl_is_empty_never {
  ($($t:ty),* $(,)?) => {
    $(impl IsEmpty for $t { fn is_empty(&self) -> bool { false } })*
  };
}

impl_is_empty_never!(
  i8, i16, i32, i64, i128, isize,
  u8, u16, u32, u64, u128, usize,
  f32, f64, bool, char
);

// ============================================================================
// WithLength
// ============================================================================

/// Trait for getting the length of a value.
///
/// Used by length validation rules (e.g., `Rule::MinLength`, `Rule::MaxLength`).
/// Inspired by the `validator_types` crate.
pub trait WithLength {
  fn length(&self) -> usize;
}

/// Implements [`WithLength`] for string-like types using Unicode char count.
macro_rules! impl_with_length_chars {
  ($type_:ty) => {
    impl WithLength for $type_ {
      fn length(&self) -> usize { self.chars().count() }
    }
  };
}

impl_with_length_chars!(str);
impl_with_length_chars!(&str);
impl_with_length_chars!(String);

/// Implements [`WithLength`] for collection types using `.len()`.
macro_rules! impl_with_length_len {
  ($type_:ty) => { impl_with_length_len!($type_,); };
  ($type_:ty, $($generic:ident),* $(,)?) => {
    impl<$($generic),*> WithLength for $type_ {
      fn length(&self) -> usize { self.len() }
    }
  };
}

impl_with_length_len!([T], T);
impl_with_length_len!(BTreeSet<T>, T);
impl_with_length_len!(BTreeMap<K, V>, K, V);
impl_with_length_len!(HashSet<T, S>, T, S);
impl_with_length_len!(HashMap<K, V, S>, K, V, S);
impl_with_length_len!(IndexMap<K, V, S>, K, V, S);
impl_with_length_len!(IndexSet<T, S>, T, S);
impl_with_length_len!(Vec<T>, T);
impl_with_length_len!(VecDeque<T>, T);

// ============================================================================
// Async Validation Traits
// ============================================================================

/// Trait for asynchronously validating owned/copied values.
///
/// This is the async counterpart of [`Validate`]. Async validators handle
/// both sync and async rules — sync rules execute inline while async
/// rules (e.g., `Rule::CustomAsync`) are awaited.
#[cfg(feature = "async")]
pub trait ValidateAsync<T: Send> {
  fn validate_async(&self, value: T) -> impl std::future::Future<Output = ValidatorResult> + Send;
}

/// Trait for asynchronously validating referenced values.
///
/// This is the async counterpart of [`ValidateRef`]. Async validators handle
/// both sync and async rules — sync rules execute inline while async
/// rules (e.g., `Rule::CustomAsync`) are awaited.
#[cfg(feature = "async")]
pub trait ValidateRefAsync<T: ?Sized + Sync> {
  fn validate_ref_async(&self, value: &T) -> impl std::future::Future<Output = ValidatorResult> + Send;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  // ==========================================================================
  // IsEmpty
  // ==========================================================================

  #[test]
  fn test_is_empty_string() {
    assert!(IsEmpty::is_empty(&String::new()));
    assert!(IsEmpty::is_empty(&"   ".to_string()));
    assert!(!IsEmpty::is_empty(&"hello".to_string()));
  }

  #[test]
  fn test_is_empty_str() {
    assert!(IsEmpty::is_empty(""));
    assert!(IsEmpty::is_empty("  \t\n "));
    assert!(!IsEmpty::is_empty("x"));
  }

  #[test]
  fn test_is_empty_str_ref() {
    let s: &str = "   ";
    assert!(IsEmpty::is_empty(s));
    let s2: &str = "abc";
    assert!(!IsEmpty::is_empty(s2));
  }

  #[test]
  fn test_is_empty_vec() {
    let empty: Vec<i32> = vec![];
    assert!(IsEmpty::is_empty(&empty));
    assert!(!IsEmpty::is_empty(&vec![1]));
  }

  #[test]
  fn test_is_empty_option() {
    let none: Option<i32> = None;
    assert!(IsEmpty::is_empty(&none));
    assert!(!IsEmpty::is_empty(&Some(0)));
  }

  #[test]
  fn test_is_empty_never_for_numerics() {
    assert!(!IsEmpty::is_empty(&0_i32));
    assert!(!IsEmpty::is_empty(&1_u8));
    assert!(!IsEmpty::is_empty(&0.0_f64));
    assert!(!IsEmpty::is_empty(&false));
    assert!(!IsEmpty::is_empty(&'a'));
  }

  // ==========================================================================
  // WithLength
  // ==========================================================================

  #[test]
  fn test_with_length_str_unicode() {
    // Unicode chars — length should count chars, not bytes
    let s = "héllo";
    assert_eq!(s.length(), 5);
    let s2 = "日本語";
    assert_eq!(s2.length(), 3);
  }

  #[test]
  fn test_with_length_string() {
    assert_eq!("hello".to_string().length(), 5);
    assert_eq!(String::new().length(), 0);
  }

  #[test]
  fn test_with_length_vec() {
    let v: Vec<i32> = vec![1, 2, 3];
    assert_eq!(v.length(), 3);
    let empty: Vec<i32> = vec![];
    assert_eq!(empty.length(), 0);
  }

  #[test]
  fn test_with_length_hashmap() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert("a", 1);
    map.insert("b", 2);
    assert_eq!(map.length(), 2);
  }

  #[test]
  fn test_with_length_indexmap() {
    let mut map = IndexMap::new();
    map.insert("a", 1);
    map.insert("b", 2);
    assert_eq!(map.length(), 2);
  }

  #[test]
  fn test_with_length_indexset() {
    let mut set = IndexSet::new();
    set.insert("a");
    set.insert("b");
    assert_eq!(set.length(), 2);
  }

  // ==========================================================================
  // SteppableValue::rem_check
  // ==========================================================================

  #[test]
  fn test_rem_check_integer() {
    assert!(10_i32.rem_check(5));
    assert!(!10_i32.rem_check(3));
    assert!(!10_i32.rem_check(0)); // zero divisor returns false
  }

  #[test]
  fn test_rem_check_float() {
    assert!(1.0_f64.rem_check(0.5));
    assert!(!1.0_f64.rem_check(0.3));
    assert!(!1.0_f64.rem_check(0.0)); // zero divisor returns false
  }

  #[test]
  fn test_rem_check_f32() {
    assert!(2.0_f32.rem_check(1.0));
    assert!(!2.0_f32.rem_check(0.0));
  }
}

