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

// ============================================================================
// Adapter for Trait Unification
// ============================================================================

/// Adapter that wraps a `Validate<T>` validator to implement `ValidateRef<T>`.
///
/// This allows validators designed for owned values to be used in contexts
/// that expect reference-based validation.
///
/// # Example
///
/// ```rust
/// use walrs_validation::{
///     RangeValidator, ValidateRefAdapter, Validate, ValidateRef
/// };
///
/// let range_validator = RangeValidator::<i32>::builder()
///     .min(0)
///     .max(100)
///     .build()
///     .unwrap();
///
/// // Wrap to use as ValidateRef
/// let ref_validator = ValidateRefAdapter::adapt(&range_validator);
///
/// assert!(ref_validator.validate(50).is_ok());
/// assert!(ref_validator.validate(150).is_err());
/// assert!(ref_validator.validate_ref(&50).is_ok());
/// assert!(ref_validator.validate_ref(&150).is_err());
/// ```
#[derive(Clone, Debug)]
pub struct ValidateRefAdapter<'a, V> {
  inner: &'a V,
}

impl<'a, V> ValidateRefAdapter<'a, V> {
  /// Creates a new adapter wrapping the given validator.
  pub fn adapt(inner: &'a V) -> Self {
    Self { inner }
  }

  /// Proxy for `ValidateRefAdapter::adapt`.  Here for completeness and consistency.
  pub fn new(inner: &'a V) -> Self {
    Self::adapt(inner)
  }

  /// Returns a reference to the inner validator.
  pub fn inner(&self) -> &V {
    self.inner
  }
}

impl<T: Copy, V: Validate<T>> Validate<T> for ValidateRefAdapter<'_, V> {
  fn validate(&self, value: T) -> ValidatorResult {
    self.inner.validate(value)
  }
}

impl<T: Copy, V: Validate<T>> ValidateRef<T> for ValidateRefAdapter<'_, V> {
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    self.inner.validate(*value)
  }
}

impl<'a, T> From<&'a T> for ValidateRefAdapter<'a, T>
  where T: Validate<T> + Sized {
  fn from(value: &'a T) -> Self {
    ValidateRefAdapter { inner : value }
  }
}

/// Extension trait for easily converting `Validate` validators to `ValidateRef`.
pub trait AsRefValidator<'a, T>: Validate<T> + Sized {
  /// Wraps this validator in an adapter that implements `ValidateRef`.
  fn as_ref_validator(&'a self) -> ValidateRefAdapter<'a, Self> {
    ValidateRefAdapter::adapt(self)
  }
}

impl<T, V: Validate<T>> AsRefValidator<'_, T> for V {}

use serde::Serialize;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Rem, Sub};

/// Macro to implement a marker trait for multiple types.
macro_rules! impl_marker_trait {
  ($trait:ident for $($ty:ty),+ $(,)?) => {
    $(impl $trait for $ty {})+
  };
}

pub trait InputValue: Copy + Default + PartialEq + PartialOrd + Display + Serialize {}

impl_marker_trait!(InputValue for
  i8, i16, i32, i64, i128, isize,
  u8, u16, u32, u64, u128, usize,
  f32, f64,
  bool, char, &str
);

pub trait ScalarValue: InputValue {}

impl_marker_trait!(ScalarValue for
  i8, i16, i32, i64, i128, isize,
  u8, u16, u32, u64, u128, usize,
  f32, f64,
  bool, char
);

pub trait NumberValue: ScalarValue + Add + Sub + Mul + Div {}

impl_marker_trait!(NumberValue for
  i8, i16, i32, i64, i128, isize,
  u8, u16, u32, u64, u128, usize,
  f32, f64
);

/// Trait for numeric types that support step/remainder validation.
///
/// This extends `NumberValue` with a `rem_check` method for validating
/// that a value is a multiple of a given step.
pub trait SteppableValue: NumberValue + Rem<Output = Self> {
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

/// Trait used by [`LengthValidator`] to get the length of a value.
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

