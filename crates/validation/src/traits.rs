use crate::{Violation};
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

/// Trait for checking if a value is "empty", used by [`Condition`] evaluation.
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
impl_with_length_len!(Vec<T>, T);
impl_with_length_len!(VecDeque<T>, T);

