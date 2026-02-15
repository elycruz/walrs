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
/// use walrs_inputfilter::validators::{
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

pub trait InputValue: Copy + Default + PartialEq + PartialOrd + Display + Serialize {}

impl InputValue for i8 {}
impl InputValue for i16 {}
impl InputValue for i32 {}
impl InputValue for i64 {}
impl InputValue for i128 {}
impl InputValue for isize {}

impl InputValue for u8 {}
impl InputValue for u16 {}
impl InputValue for u32 {}
impl InputValue for u64 {}
impl InputValue for u128 {}
impl InputValue for usize {}

impl InputValue for f32 {}
impl InputValue for f64 {}

impl InputValue for bool {}
impl InputValue for char {}
impl InputValue for &str {}

pub trait ScalarValue: InputValue {}

impl ScalarValue for i8 {}
impl ScalarValue for i16 {}
impl ScalarValue for i32 {}
impl ScalarValue for i64 {}
impl ScalarValue for i128 {}
impl ScalarValue for isize {}

impl ScalarValue for u8 {}
impl ScalarValue for u16 {}
impl ScalarValue for u32 {}
impl ScalarValue for u64 {}
impl ScalarValue for u128 {}
impl ScalarValue for usize {}

impl ScalarValue for f32 {}
impl ScalarValue for f64 {}

impl ScalarValue for bool {}
impl ScalarValue for char {}

pub trait NumberValue: ScalarValue + Add + Sub + Mul + Div + Rem<Output = Self> {}

impl NumberValue for i8 {}
impl NumberValue for i16 {}
impl NumberValue for i32 {}
impl NumberValue for i64 {}
impl NumberValue for i128 {}
impl NumberValue for isize {}

impl NumberValue for u8 {}
impl NumberValue for u16 {}
impl NumberValue for u32 {}
impl NumberValue for u64 {}
impl NumberValue for u128 {}
impl NumberValue for usize {}

impl NumberValue for f32 {}
impl NumberValue for f64 {}
