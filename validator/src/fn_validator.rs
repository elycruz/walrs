//! Closure-based validators.
//!
//! This module provides [`FnValidator`] and [`FnRefValidator`], which wrap
//! owned closures/functions as first-class validators that implement the
//! [`Validate`] and [`ValidateRef`] traits respectively.
//!
//! # Examples
//!
//! ```rust
//! use walrs_validator::{
//!     FnValidator, FnRefValidator,
//!     Validate, ValidateRef,
//!     Violation, ViolationType,
//! };
//!
//! // Validate a copied value (e.g. i32)
//! let positive = FnValidator::new(|v: i32| {
//!     if v > 0 {
//!         Ok(())
//!     } else {
//!         Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
//!     }
//! });
//!
//! assert!(positive.validate(1).is_ok());
//! assert!(positive.validate(-1).is_err());
//!
//! // Validate a referenced/unsized value (e.g. &str)
//! let non_empty = FnRefValidator::new(|v: &str| {
//!     if !v.is_empty() {
//!         Ok(())
//!     } else {
//!         Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
//!     }
//! });
//!
//! assert!(non_empty.validate_ref("hello").is_ok());
//! assert!(non_empty.validate_ref("").is_err());
//! ```

use std::fmt;
use std::sync::Arc;

use crate::traits::{Validate, ValidateRef, ValidatorResult};

// ============================================================================
// FnValidator
// ============================================================================

/// A validator that wraps an owned closure or function for validating `Copy` values.
///
/// The closure receives the value by copy and returns a [`ValidatorResult`].
/// The inner function is reference-counted with [`Arc`], making `FnValidator`
/// cheaply cloneable and safe to share across threads.
///
/// # Example
///
/// ```rust
/// use walrs_validator::{FnValidator, Validate, Violation, ViolationType};
///
/// let positive = FnValidator::new(|v: i32| {
///     if v > 0 { Ok(()) }
///     else { Err(Violation::new(ViolationType::RangeUnderflow, "must be positive")) }
/// });
///
/// assert!(positive.validate(42).is_ok());
/// assert!(positive.validate(-1).is_err());
///
/// // Also constructable via `From`:
/// let also_positive = FnValidator::from(|v: i32| {
///     if v > 0 { Ok(()) }
///     else { Err(Violation::new(ViolationType::RangeUnderflow, "must be positive")) }
/// });
/// assert!(also_positive.validate(42).is_ok());
/// ```
pub struct FnValidator<T> {
  f: Arc<dyn Fn(T) -> ValidatorResult + Send + Sync>,
}

impl<T> FnValidator<T> {
  /// Creates a new `FnValidator` wrapping the given closure or function.
  pub fn new<F>(f: F) -> Self
  where
    F: Fn(T) -> ValidatorResult + Send + Sync + 'static,
  {
    Self { f: Arc::new(f) }
  }
}

impl<T: Copy> Validate<T> for FnValidator<T> {
  fn validate(&self, value: T) -> ValidatorResult {
    (self.f)(value)
  }
}

impl<T> Clone for FnValidator<T> {
  fn clone(&self) -> Self {
    Self { f: Arc::clone(&self.f) }
  }
}

impl<T> fmt::Debug for FnValidator<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("FnValidator")
      .field("f", &"<fn>")
      .finish()
  }
}

impl<T, F> From<F> for FnValidator<T>
where
  F: Fn(T) -> ValidatorResult + Send + Sync + 'static,
{
  fn from(f: F) -> Self {
    Self::new(f)
  }
}

// ============================================================================
// FnRefValidator
// ============================================================================

/// A validator that wraps an owned closure or function for validating referenced
/// (potentially unsized) values such as `str` or `[T]`.
///
/// The closure receives a `&T` reference and returns a [`ValidatorResult`].
/// The inner function is reference-counted with [`Arc`], making `FnRefValidator`
/// cheaply cloneable and safe to share across threads.
///
/// # Example
///
/// ```rust
/// use walrs_validator::{FnRefValidator, ValidateRef, Violation, ViolationType};
///
/// let non_empty = FnRefValidator::new(|v: &str| {
///     if !v.is_empty() { Ok(()) }
///     else { Err(Violation::new(ViolationType::ValueMissing, "must not be empty")) }
/// });
///
/// assert!(non_empty.validate_ref("hello").is_ok());
/// assert!(non_empty.validate_ref("").is_err());
///
/// // Also constructable via `From`:
/// let also_non_empty = FnRefValidator::from(|v: &str| {
///     if !v.is_empty() { Ok(()) }
///     else { Err(Violation::new(ViolationType::ValueMissing, "must not be empty")) }
/// });
/// assert!(also_non_empty.validate_ref("hello").is_ok());
/// ```
pub struct FnRefValidator<T: ?Sized> {
  f: Arc<dyn Fn(&T) -> ValidatorResult + Send + Sync>,
}

impl<T: ?Sized> FnRefValidator<T> {
  /// Creates a new `FnRefValidator` wrapping the given closure or function.
  pub fn new<F>(f: F) -> Self
  where
    F: Fn(&T) -> ValidatorResult + Send + Sync + 'static,
  {
    Self { f: Arc::new(f) }
  }
}

impl<T: ?Sized> ValidateRef<T> for FnRefValidator<T> {
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    (self.f)(value)
  }
}

impl<T: ?Sized> Clone for FnRefValidator<T> {
  fn clone(&self) -> Self {
    Self { f: Arc::clone(&self.f) }
  }
}

impl<T: ?Sized> fmt::Debug for FnRefValidator<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("FnRefValidator")
      .field("f", &"<fn>")
      .finish()
  }
}

impl<T: ?Sized, F> From<F> for FnRefValidator<T>
where
  F: Fn(&T) -> ValidatorResult + Send + Sync + 'static,
{
  fn from(f: F) -> Self {
    Self::new(f)
  }
}
