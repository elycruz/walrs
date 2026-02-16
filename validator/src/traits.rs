use crate::Violation;

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
/// use walrs_validator::{
///     RangeValidatorBuilder, ValidateAdapter, Validate, ValidateRef
/// };
///
/// let range_validator = RangeValidatorBuilder::<i32>::default()
///     .min(0)
///     .max(100)
///     .build()
///     .unwrap();
///
/// // Wrap to use as ValidateRef
/// let ref_validator = ValidateAdapter::new(range_validator);
///
/// assert!(ref_validator.validate_ref(&50).is_ok());
/// assert!(ref_validator.validate_ref(&150).is_err());
/// ```
#[derive(Clone, Debug)]
pub struct ValidateAdapter<V> {
  inner: V,
}

impl<V> ValidateAdapter<V> {
  /// Creates a new adapter wrapping the given validator.
  pub fn new(inner: V) -> Self {
    Self { inner }
  }

  /// Returns a reference to the inner validator.
  pub fn inner(&self) -> &V {
    &self.inner
  }

  /// Consumes the adapter and returns the inner validator.
  pub fn into_inner(self) -> V {
    self.inner
  }
}

impl<T: Copy, V: Validate<T>> Validate<T> for ValidateAdapter<V> {
  fn validate(&self, value: T) -> ValidatorResult {
    self.inner.validate(value)
  }
}

impl<T: Copy, V: Validate<T>> ValidateRef<T> for ValidateAdapter<V> {
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    self.inner.validate(*value)
  }
}

/// Extension trait for easily converting `Validate` validators to `ValidateRef`.
pub trait ValidateToRef<T>: Validate<T> + Sized {
  /// Wraps this validator in an adapter that implements `ValidateRef`.
  fn as_ref_validator(self) -> ValidateAdapter<Self> {
    ValidateAdapter::new(self)
  }
}

impl<T, V: Validate<T>> ValidateToRef<T> for V {}

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

impl_steppable_integer!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

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
