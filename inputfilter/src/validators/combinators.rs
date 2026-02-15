//! Validator combinators for composing validators.
//!
//! This module provides combinators that allow you to compose validators
//! using logical operations like AND, OR, and NOT.
//!
//! # Examples
//!
//! ```rust
//! use walrs_inputfilter::validators::{
//!     RangeValidatorBuilder,
//!     Validate, ValidateExt,
//! };
//!
//! // Create individual validators
//! let min_5 = RangeValidatorBuilder::<i32>::default()
//!     .min(5)
//!     .build()
//!     .unwrap();
//!
//! let max_100 = RangeValidatorBuilder::<i32>::default()
//!     .max(100)
//!     .build()
//!     .unwrap();
//!
//! // Combine with AND - value must satisfy both
//! let range_5_to_100 = min_5.and(max_100);
//! assert!(range_5_to_100.validate(50).is_ok());
//! assert!(range_5_to_100.validate(3).is_err());
//! assert!(range_5_to_100.validate(150).is_err());
//! ```

use crate::{Violation, ViolationType, Violations};

use super::{Validate, ValidateRef, ValidatorResult};

// ============================================================================
// AND Combinator
// ============================================================================

/// A validator that combines two validators with AND logic.
///
/// Both validators must pass for the combined validator to pass.
/// If the first validator fails, it returns that error (short-circuit).
///
/// # Examples
///
/// ```rust
/// use walrs_inputfilter::validators::{RangeValidatorBuilder, Validate, ValidatorAnd};
///
/// let min_validator = RangeValidatorBuilder::<i32>::default()
///     .min(0)
///     .build()
///     .unwrap();
///
/// let max_validator = RangeValidatorBuilder::<i32>::default()
///     .max(100)
///     .build()
///     .unwrap();
///
/// let combined = ValidatorAnd::new(min_validator, max_validator);
///
/// assert!(combined.validate(50).is_ok());
/// assert!(combined.validate(-1).is_err());
/// assert!(combined.validate(101).is_err());
/// ```
#[derive(Clone, Debug)]
pub struct ValidatorAnd<V1, V2> {
  first: V1,
  second: V2,
}

impl<V1, V2> ValidatorAnd<V1, V2> {
  /// Creates a new AND combinator from two validators.
  pub fn new(first: V1, second: V2) -> Self {
    Self { first, second }
  }
}

impl<T: Copy, V1, V2> Validate<T> for ValidatorAnd<V1, V2>
where
  V1: Validate<T>,
  V2: Validate<T>,
{
  fn validate(&self, value: T) -> ValidatorResult {
    self.first.validate(value)?;
    self.second.validate(value)
  }
}

impl<T: ?Sized, V1, V2> ValidateRef<T> for ValidatorAnd<V1, V2>
where
  V1: ValidateRef<T>,
  V2: ValidateRef<T>,
{
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    self.first.validate_ref(value)?;
    self.second.validate_ref(value)
  }
}

// ============================================================================
// OR Combinator
// ============================================================================

/// A validator that combines two validators with OR logic.
///
/// At least one validator must pass for the combined validator to pass.
/// If both fail, returns a combined error message.
///
/// # Examples
///
/// ```rust
/// use walrs_inputfilter::validators::{RangeValidatorBuilder, Validate, ValidatorOr};
///
/// // Value must be either negative or greater than 100
/// let negative = RangeValidatorBuilder::<i32>::default()
///     .max(-1)
///     .build()
///     .unwrap();
///
/// let large = RangeValidatorBuilder::<i32>::default()
///     .min(101)
///     .build()
///     .unwrap();
///
/// let combined = ValidatorOr::new(negative, large);
///
/// assert!(combined.validate(-5).is_ok());
/// assert!(combined.validate(150).is_ok());
/// assert!(combined.validate(50).is_err()); // Neither negative nor > 100
/// ```
#[derive(Clone, Debug)]
pub struct ValidatorOr<V1, V2> {
  first: V1,
  second: V2,
}

impl<V1, V2> ValidatorOr<V1, V2> {
  /// Creates a new OR combinator from two validators.
  pub fn new(first: V1, second: V2) -> Self {
    Self { first, second }
  }
}

impl<T: Copy, V1, V2> Validate<T> for ValidatorOr<V1, V2>
where
  V1: Validate<T>,
  V2: Validate<T>,
{
  fn validate(&self, value: T) -> ValidatorResult {
    match self.first.validate(value) {
      Ok(()) => Ok(()),
      Err(first_err) => match self.second.validate(value) {
        Ok(()) => Ok(()),
        Err(second_err) => Err(Violation::new(
          ViolationType::CustomError,
          format!("{} OR {}", first_err.message(), second_err.message()),
        )),
      },
    }
  }
}

impl<T: ?Sized, V1, V2> ValidateRef<T> for ValidatorOr<V1, V2>
where
  V1: ValidateRef<T>,
  V2: ValidateRef<T>,
{
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    match self.first.validate_ref(value) {
      Ok(()) => Ok(()),
      Err(first_err) => match self.second.validate_ref(value) {
        Ok(()) => Ok(()),
        Err(second_err) => Err(Violation::new(
          ViolationType::CustomError,
          format!("{} OR {}", first_err.message(), second_err.message()),
        )),
      },
    }
  }
}

// ============================================================================
// NOT Combinator
// ============================================================================

/// A validator that negates another validator.
///
/// Passes when the inner validator fails, and fails when it passes.
///
/// # Examples
///
/// ```rust
/// use walrs_inputfilter::validators::{RangeValidatorBuilder, Validate, ValidatorNot};
///
/// // Value must NOT be in range 0-10
/// let in_range = RangeValidatorBuilder::<i32>::default()
///     .min(0)
///     .max(10)
///     .build()
///     .unwrap();
///
/// let not_in_range = ValidatorNot::new(in_range, "Value must not be between 0 and 10");
///
/// assert!(not_in_range.validate(15).is_ok());
/// assert!(not_in_range.validate(-5).is_ok());
/// assert!(not_in_range.validate(5).is_err());
/// ```
#[derive(Clone, Debug)]
pub struct ValidatorNot<V> {
  inner: V,
  message: String,
}

impl<V> ValidatorNot<V> {
  /// Creates a new NOT combinator with a custom error message.
  pub fn new(inner: V, message: impl Into<String>) -> Self {
    Self {
      inner,
      message: message.into(),
    }
  }
}

impl<T: Copy, V> Validate<T> for ValidatorNot<V>
where
  V: Validate<T>,
{
  fn validate(&self, value: T) -> ValidatorResult {
    match self.inner.validate(value) {
      Ok(()) => Err(Violation::new(ViolationType::CustomError, &self.message)),
      Err(_) => Ok(()),
    }
  }
}

impl<T: ?Sized, V> ValidateRef<T> for ValidatorNot<V>
where
  V: ValidateRef<T>,
{
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    match self.inner.validate_ref(value) {
      Ok(()) => Err(Violation::new(ViolationType::CustomError, &self.message)),
      Err(_) => Ok(()),
    }
  }
}

// ============================================================================
// Optional Combinator
// ============================================================================

/// A validator that only validates non-empty/non-default values.
///
/// Useful for optional fields where validation should only apply
/// when a value is present.
///
/// # Examples
///
/// ```rust
/// use walrs_inputfilter::validators::{LengthValidatorBuilder, ValidateRef, ValidatorOptional};
///
/// let length_validator = LengthValidatorBuilder::<str>::default()
///     .min_length(5)
///     .build()
///     .unwrap();
///
/// let optional_length = ValidatorOptional::new(length_validator, |s: &str| s.is_empty());
///
/// // Empty string passes (validation skipped)
/// assert!(optional_length.validate_ref("").is_ok());
///
/// // Non-empty strings are validated
/// assert!(optional_length.validate_ref("hello").is_ok());
/// assert!(optional_length.validate_ref("hi").is_err());
/// ```
#[derive(Clone)]
pub struct ValidatorOptional<V, F> {
  inner: V,
  is_empty: F,
}

impl<V, F> ValidatorOptional<V, F> {
  /// Creates a new optional validator with a custom emptiness check.
  pub fn new(inner: V, is_empty: F) -> Self {
    Self { inner, is_empty }
  }
}

impl<V: std::fmt::Debug, F> std::fmt::Debug for ValidatorOptional<V, F> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ValidatorOptional")
      .field("inner", &self.inner)
      .field("is_empty", &"<fn>")
      .finish()
  }
}

impl<T: Copy, V, F> Validate<T> for ValidatorOptional<V, F>
where
  V: Validate<T>,
  F: Fn(T) -> bool,
{
  fn validate(&self, value: T) -> ValidatorResult {
    if (self.is_empty)(value) {
      Ok(())
    } else {
      self.inner.validate(value)
    }
  }
}

impl<T: ?Sized, V, F> ValidateRef<T> for ValidatorOptional<V, F>
where
  V: ValidateRef<T>,
  F: Fn(&T) -> bool,
{
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    if (self.is_empty)(value) {
      Ok(())
    } else {
      self.inner.validate_ref(value)
    }
  }
}

// ============================================================================
// Conditional Combinator
// ============================================================================

/// A validator that only runs when a condition is met.
///
/// # Examples
///
/// ```rust
/// use walrs_inputfilter::validators::{RangeValidatorBuilder, Validate, ValidatorWhen};
///
/// let positive_validator = RangeValidatorBuilder::<i32>::default()
///     .min(1)
///     .build()
///     .unwrap();
///
/// // Only validate positive numbers when value > 0
/// let conditional = ValidatorWhen::new(
///     positive_validator,
///     |&v: &i32| v > 0
/// );
///
/// assert!(conditional.validate(5).is_ok());
/// assert!(conditional.validate(-5).is_ok()); // Condition not met, skipped
/// assert!(conditional.validate(0).is_ok());  // Condition not met, skipped
/// ```
#[derive(Clone)]
pub struct ValidatorWhen<V, F> {
  inner: V,
  condition: F,
}

impl<V, F> ValidatorWhen<V, F> {
  /// Creates a new conditional validator.
  pub fn new(inner: V, condition: F) -> Self {
    Self { inner, condition }
  }
}

impl<V: std::fmt::Debug, F> std::fmt::Debug for ValidatorWhen<V, F> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ValidatorWhen")
      .field("inner", &self.inner)
      .field("condition", &"<fn>")
      .finish()
  }
}

impl<T: Copy, V, F> Validate<T> for ValidatorWhen<V, F>
where
  V: Validate<T>,
  F: Fn(&T) -> bool,
{
  fn validate(&self, value: T) -> ValidatorResult {
    if (self.condition)(&value) {
      self.inner.validate(value)
    } else {
      Ok(())
    }
  }
}

impl<T: ?Sized, V, F> ValidateRef<T> for ValidatorWhen<V, F>
where
  V: ValidateRef<T>,
  F: Fn(&T) -> bool,
{
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    if (self.condition)(value) {
      self.inner.validate_ref(value)
    } else {
      Ok(())
    }
  }
}

// ============================================================================
// ValidatorAll - Runs all validators and collects all errors
// ============================================================================

/// A validator that runs all validators and collects all errors.
///
/// Unlike `ValidatorAnd` which short-circuits on first error,
/// this collects all violations from all validators.
///
/// # Examples
///
/// ```rust
/// use walrs_inputfilter::validators::{LengthValidatorBuilder, ValidatorAll, ValidateRef};
///
/// let min_length = LengthValidatorBuilder::<str>::default()
///     .min_length(5)
///     .build()
///     .unwrap();
///
/// let max_length = LengthValidatorBuilder::<str>::default()
///     .max_length(20)
///     .build()
///     .unwrap();
///
/// let all = ValidatorAll::new(vec![Box::new(min_length), Box::new(max_length)]);
/// assert!(all.validate_ref("hello world").is_ok());
/// assert!(all.validate_ref("hi").is_err()); // too short
/// ```
pub struct ValidatorAll<'a, T: ?Sized> {
  validators: Vec<Box<dyn ValidateRef<T> + Send + Sync + 'a>>,
}

impl<'a, T: ?Sized> ValidatorAll<'a, T> {
  /// Creates a new validator that runs all validators.
  pub fn new(validators: Vec<Box<dyn ValidateRef<T> + Send + Sync + 'a>>) -> Self {
    Self { validators }
  }

  /// Validates and returns all violations (not just the first).
  pub fn validate_all(&self, value: &T) -> Result<(), Violations> {
    let violations: Vec<Violation> = self
      .validators
      .iter()
      .filter_map(|v| v.validate_ref(value).err())
      .collect();

    if violations.is_empty() {
      Ok(())
    } else {
      Err(Violations::new(violations))
    }
  }
}

impl<T: ?Sized> ValidateRef<T> for ValidatorAll<'_, T> {
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    self
      .validate_all(value)
      .map_err(|vs| vs.into_iter().next().unwrap())
  }
}

impl<T: ?Sized> std::fmt::Debug for ValidatorAll<'_, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ValidatorAll")
      .field("validators", &format!("[{} validators]", self.validators.len()))
      .finish()
  }
}

// ============================================================================
// Extension Traits for fluent API
// ============================================================================

/// Extension trait for adding combinator methods to validators implementing `Validate`.
pub trait ValidateExt<T: Copy>: Validate<T> + Sized {
  /// Combines this validator with another using AND logic.
  fn and<V: Validate<T>>(self, other: V) -> ValidatorAnd<Self, V> {
    ValidatorAnd::new(self, other)
  }

  /// Combines this validator with another using OR logic.
  fn or<V: Validate<T>>(self, other: V) -> ValidatorOr<Self, V> {
    ValidatorOr::new(self, other)
  }

  /// Negates this validator.
  fn not(self, message: impl Into<String>) -> ValidatorNot<Self> {
    ValidatorNot::new(self, message)
  }

  /// Makes this validator optional (skips validation for empty values).
  fn optional<F: Fn(T) -> bool>(self, is_empty: F) -> ValidatorOptional<Self, F> {
    ValidatorOptional::new(self, is_empty)
  }

  /// Only runs this validator when a condition is met.
  fn when<F: Fn(&T) -> bool>(self, condition: F) -> ValidatorWhen<Self, F> {
    ValidatorWhen::new(self, condition)
  }
}

/// Extension trait for adding combinator methods to validators implementing `ValidateRef`.
pub trait ValidateRefExt<T: ?Sized>: ValidateRef<T> + Sized {
  /// Combines this validator with another using AND logic.
  fn and<V: ValidateRef<T>>(self, other: V) -> ValidatorAnd<Self, V> {
    ValidatorAnd::new(self, other)
  }

  /// Combines this validator with another using OR logic.
  fn or<V: ValidateRef<T>>(self, other: V) -> ValidatorOr<Self, V> {
    ValidatorOr::new(self, other)
  }

  /// Negates this validator.
  fn not(self, message: impl Into<String>) -> ValidatorNot<Self> {
    ValidatorNot::new(self, message)
  }

  /// Makes this validator optional (skips validation for empty values).
  fn optional<F: Fn(&T) -> bool>(self, is_empty: F) -> ValidatorOptional<Self, F> {
    ValidatorOptional::new(self, is_empty)
  }

  /// Only runs this validator when a condition is met.
  fn when<F: Fn(&T) -> bool>(self, condition: F) -> ValidatorWhen<Self, F> {
    ValidatorWhen::new(self, condition)
  }
}

// Blanket implementations
impl<T: Copy, V: Validate<T>> ValidateExt<T> for V {}
impl<T: ?Sized, V: ValidateRef<T>> ValidateRefExt<T> for V {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::validators::{LengthValidatorBuilder, RangeValidatorBuilder};

  #[test]
  fn test_validator_and() {
    let min = RangeValidatorBuilder::<i32>::default()
      .min(0)
      .build()
      .unwrap();

    let max = RangeValidatorBuilder::<i32>::default()
      .max(100)
      .build()
      .unwrap();

    let combined = min.and(max);

    assert!(combined.validate(50).is_ok());
    assert!(combined.validate(0).is_ok());
    assert!(combined.validate(100).is_ok());
    assert!(combined.validate(-1).is_err());
    assert!(combined.validate(101).is_err());
  }

  #[test]
  fn test_validator_or() {
    let negative = RangeValidatorBuilder::<i32>::default()
      .max(-1)
      .build()
      .unwrap();

    let large = RangeValidatorBuilder::<i32>::default()
      .min(100)
      .build()
      .unwrap();

    let combined = negative.or(large);

    assert!(combined.validate(-5).is_ok());
    assert!(combined.validate(150).is_ok());
    assert!(combined.validate(50).is_err());
  }

  #[test]
  fn test_validator_not() {
    let in_range = RangeValidatorBuilder::<i32>::default()
      .min(0)
      .max(10)
      .build()
      .unwrap();

    let not_in_range = in_range.not("Value must not be between 0 and 10");

    assert!(not_in_range.validate(15).is_ok());
    assert!(not_in_range.validate(-5).is_ok());
    assert!(not_in_range.validate(5).is_err());
  }

  #[test]
  fn test_validator_optional_ref() {
    let length = LengthValidatorBuilder::<str>::default()
      .min_length(5)
      .build()
      .unwrap();

    let optional = length.optional(|s: &str| s.is_empty());

    assert!(optional.validate_ref("").is_ok());
    assert!(optional.validate_ref("hello").is_ok());
    assert!(optional.validate_ref("hi").is_err());
  }

  #[test]
  fn test_validator_when() {
    let positive = RangeValidatorBuilder::<i32>::default()
      .min(1)
      .build()
      .unwrap();

    let when_positive = positive.when(|&v| v > 0);

    assert!(when_positive.validate(5).is_ok());
    assert!(when_positive.validate(-5).is_ok()); // Skipped
    assert!(when_positive.validate(0).is_ok()); // Skipped
  }

  #[test]
  fn test_chained_combinators() {
    // Complex example: value must be 0-100 AND (negative OR > 50)
    let range = RangeValidatorBuilder::<i32>::default()
      .min(0)
      .max(100)
      .build()
      .unwrap();

    let gt_50 = RangeValidatorBuilder::<i32>::default()
      .min(51)
      .build()
      .unwrap();

    let combined = range.and(gt_50);

    assert!(combined.validate(75).is_ok());
    assert!(combined.validate(25).is_err()); // In range but not > 50
    assert!(combined.validate(150).is_err()); // > 50 but not in range
  }
}

