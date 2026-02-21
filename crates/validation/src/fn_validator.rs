//! Closure-based validators.
//!
//! This module provides [`FnValidator`] and [`FnRefValidator`], which wrap
//! owned closures/functions as first-class validators that implement the
//! [`Validate`] and [`ValidateRef`] traits respectively.
//!
//! Because both types implement the same traits as every other validator in
//! the crate, they compose freely with all combinators exposed by
//! [`ValidateExt`](crate::ValidateExt) and [`ValidateRefExt`](crate::ValidateRefExt):
//! `and`, `or`, `not`, `optional`, `when`, and [`ValidatorAll`](crate::ValidatorAll).
//!
//! # Examples
//!
//! ```rust
//! use walrs_validation::{
//!     FnValidator, FnRefValidator,
//!     Validate, ValidateRef,
//!     ValidateExt, ValidateRefExt,
//!     ValidatorAll,
//!     Violation, ViolationType,
//! };
//!
//! // --- FnValidator (Copy types) -------------------------------------------
//!
//! let positive = FnValidator::new(|v: i32| {
//!     if v > 0 { Ok(()) }
//!     else { Err(Violation::new(ViolationType::RangeUnderflow, "must be positive")) }
//! });
//!
//! let lte_100 = FnValidator::new(|v: i32| {
//!     if v <= 100 { Ok(()) }
//!     else { Err(Violation::new(ViolationType::RangeOverflow, "must be <= 100")) }
//! });
//!
//! // AND: both must pass
//! let range = positive.and(lte_100);
//! assert!(range.validate(50).is_ok());
//! assert!(range.validate(-1).is_err());
//! assert!(range.validate(101).is_err());
//!
//! // --- FnRefValidator (unsized / borrowed types) --------------------------
//!
//! let non_empty = FnRefValidator::new(|v: &str| {
//!     if !v.is_empty() { Ok(()) }
//!     else { Err(Violation::new(ViolationType::ValueMissing, "must not be empty")) }
//! });
//!
//! let short_enough = FnRefValidator::new(|v: &str| {
//!     if v.len() <= 10 { Ok(()) }
//!     else { Err(Violation::new(ViolationType::TooLong, "must be <= 10 chars")) }
//! });
//!
//! // OR: at least one must pass
//! let either = non_empty.or(short_enough);
//! assert!(either.validate_ref("hi").is_ok());
//! assert!(either.validate_ref("").is_ok()); // empty, but short_enough still passes
//!
//! // ValidatorAll: collect every violation instead of short-circuiting
//! let all: ValidatorAll<str> = ValidatorAll::new(vec![
//!     Box::new(FnRefValidator::new(|v: &str| {
//!         if !v.is_empty() { Ok(()) }
//!         else { Err(Violation::new(ViolationType::ValueMissing, "must not be empty")) }
//!     })),
//!     Box::new(FnRefValidator::new(|v: &str| {
//!         if v.len() >= 3 { Ok(()) }
//!         else { Err(Violation::new(ViolationType::TooShort, "min 3 chars")) }
//!     })),
//! ]);
//! assert!(all.validate_ref("hello").is_ok());
//! assert!(all.validate_ref("").is_err());
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
/// Because `FnValidator<T>` implements [`Validate<T>`], it automatically gains
/// the full combinator API from [`ValidateExt`](crate::ValidateExt):
/// [`and`](crate::ValidateExt::and), [`or`](crate::ValidateExt::or),
/// [`not`](crate::ValidateExt::not), [`optional`](crate::ValidateExt::optional),
/// and [`when`](crate::ValidateExt::when).
///
/// # Example
///
/// ```rust
/// use walrs_validation::{FnValidator, Validate, ValidateExt, Violation, ViolationType};
///
/// let positive = FnValidator::new(|v: i32| {
///     if v > 0 { Ok(()) }
///     else { Err(Violation::new(ViolationType::RangeUnderflow, "must be positive")) }
/// });
///
/// assert!(positive.validate(42).is_ok());
/// assert!(positive.validate(-1).is_err());
///
/// // Combine with another FnValidator using .and()
/// let lte_100 = FnValidator::new(|v: i32| {
///     if v <= 100 { Ok(()) }
///     else { Err(Violation::new(ViolationType::RangeOverflow, "must be <= 100")) }
/// });
/// let range = positive.and(lte_100);
/// assert!(range.validate(50).is_ok());
/// assert!(range.validate(0).is_err());
/// assert!(range.validate(101).is_err());
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
/// Because `FnRefValidator<T>` implements [`ValidateRef<T>`], it automatically
/// gains the full combinator API from [`ValidateRefExt`](crate::ValidateRefExt):
/// [`and`](crate::ValidateRefExt::and), [`or`](crate::ValidateRefExt::or),
/// [`not`](crate::ValidateRefExt::not), [`optional`](crate::ValidateRefExt::optional),
/// and [`when`](crate::ValidateRefExt::when). It can also be boxed into a
/// [`ValidatorAll`](crate::ValidatorAll) collection to collect all violations.
///
/// # Example
///
/// ```rust
/// use walrs_validation::{FnRefValidator, ValidateRef, ValidateRefExt, Violation, ViolationType};
///
/// let non_empty = FnRefValidator::new(|v: &str| {
///     if !v.is_empty() { Ok(()) }
///     else { Err(Violation::new(ViolationType::ValueMissing, "must not be empty")) }
/// });
///
/// assert!(non_empty.validate_ref("hello").is_ok());
/// assert!(non_empty.validate_ref("").is_err());
///
/// // Combine with .and()
/// let min_len_3 = FnRefValidator::new(|v: &str| {
///     if v.len() >= 3 { Ok(()) }
///     else { Err(Violation::new(ViolationType::TooShort, "min 3 chars")) }
/// });
/// let strict = non_empty.and(min_len_3);
/// assert!(strict.validate_ref("hello").is_ok());
/// assert!(strict.validate_ref("hi").is_err());
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{ValidateExt, ValidateRefExt, ValidatorAll, Violation, ViolationType};

  // -------------------------------------------------------------------------
  // Helpers — reusable validator factories
  // -------------------------------------------------------------------------

  fn positive() -> FnValidator<i32> {
    FnValidator::new(|v: i32| {
      if v > 0 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
      }
    })
  }

  fn lte_100() -> FnValidator<i32> {
    FnValidator::new(|v: i32| {
      if v <= 100 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::RangeOverflow, "must be <= 100"))
      }
    })
  }

  fn non_empty() -> FnRefValidator<str> {
    FnRefValidator::new(|v: &str| {
      if !v.is_empty() {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
      }
    })
  }

  fn min_len_3() -> FnRefValidator<str> {
    FnRefValidator::new(|v: &str| {
      if v.len() >= 3 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::TooShort, "min 3 chars"))
      }
    })
  }

  fn short_enough() -> FnRefValidator<str> {
    FnRefValidator::new(|v: &str| {
      if v.len() <= 10 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::TooLong, "must be <= 10 chars"))
      }
    })
  }

  // -------------------------------------------------------------------------
  // FnValidator — standalone
  // -------------------------------------------------------------------------

  #[test]
  fn fn_validator_new_passes_valid_value() {
    assert!(positive().validate(1).is_ok());
  }

  #[test]
  fn fn_validator_new_rejects_invalid_value() {
    assert!(positive().validate(0).is_err());
    assert!(positive().validate(-1).is_err());
  }

  #[test]
  fn fn_validator_error_carries_correct_violation_type_and_message() {
    let err = positive().validate(0).unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::RangeUnderflow);
    assert_eq!(err.message(), "must be positive");
  }

  #[test]
  fn fn_validator_from_named_fn() {
    fn rule(v: i32) -> crate::ValidatorResult {
      if v > 0 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
      }
    }
    let v = FnValidator::new(rule);
    assert!(v.validate(1).is_ok());
    assert!(v.validate(0).is_err());
  }

  #[test]
  fn fn_validator_from_closure() {
    let v = FnValidator::from(|v: i32| {
      if v > 0 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
      }
    });
    assert!(v.validate(5).is_ok());
    assert!(v.validate(-5).is_err());
  }

  #[test]
  fn fn_validator_clone_shares_inner_fn() {
    let original = positive();
    let cloned = original.clone();
    assert!(original.validate(1).is_ok());
    assert!(cloned.validate(1).is_ok());
    assert!(cloned.validate(-1).is_err());
  }

  #[test]
  fn fn_validator_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FnValidator<i32>>();
  }

  #[test]
  fn fn_validator_debug_shows_placeholder() {
    let s = format!("{:?}", positive());
    assert!(s.contains("FnValidator"));
    assert!(s.contains("<fn>"));
  }

  // -------------------------------------------------------------------------
  // FnValidator — combinators (ValidateExt)
  // -------------------------------------------------------------------------

  #[test]
  fn fn_validator_and_both_must_pass() {
    let range = positive().and(lte_100());
    assert!(range.validate(1).is_ok());
    assert!(range.validate(100).is_ok());
    assert!(range.validate(-1).is_err());  // positive fails
    assert!(range.validate(101).is_err()); // lte_100 fails
  }

  #[test]
  fn fn_validator_or_at_least_one_must_pass() {
    let even = FnValidator::new(|v: i32| {
      if v % 2 == 0 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::CustomError, "must be even"))
      }
    });
    let gt_50 = FnValidator::new(|v: i32| {
      if v > 50 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::RangeUnderflow, "must be > 50"))
      }
    });
    let combined = even.or(gt_50);
    assert!(combined.validate(4).is_ok());  // even passes
    assert!(combined.validate(99).is_ok()); // gt_50 passes
    assert!(combined.validate(3).is_err()); // odd AND <= 50 — both fail
  }

  #[test]
  fn fn_validator_not_inverts_result() {
    let not_positive = positive().not("must not be positive");
    assert!(not_positive.validate(-1).is_ok());
    assert!(not_positive.validate(0).is_ok());
    assert!(not_positive.validate(1).is_err());
  }

  #[test]
  fn fn_validator_optional_skips_when_empty() {
    // treat 0 as "absent/empty"
    let optional = positive().optional(|v: i32| v == 0);
    assert!(optional.validate(0).is_ok());   // absent — skipped
    assert!(optional.validate(5).is_ok());   // present and valid
    assert!(optional.validate(-1).is_err()); // present and invalid
  }

  #[test]
  fn fn_validator_when_skips_unless_condition_met() {
    // only validate lte_100 when value is positive
    let when_positive = lte_100().when(|&v: &i32| v > 0);
    assert!(when_positive.validate(50).is_ok());   // condition met, passes
    assert!(when_positive.validate(200).is_err()); // condition met, fails
    assert!(when_positive.validate(-5).is_ok());   // condition not met, skipped
    assert!(when_positive.validate(0).is_ok());    // condition not met, skipped
  }

  // -------------------------------------------------------------------------
  // FnRefValidator — standalone
  // -------------------------------------------------------------------------

  #[test]
  fn fn_ref_validator_new_passes_valid_value() {
    assert!(non_empty().validate_ref("hello").is_ok());
  }

  #[test]
  fn fn_ref_validator_new_rejects_invalid_value() {
    assert!(non_empty().validate_ref("").is_err());
  }

  #[test]
  fn fn_ref_validator_error_carries_correct_violation_type_and_message() {
    let err = non_empty().validate_ref("").unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::ValueMissing);
    assert_eq!(err.message(), "must not be empty");
  }

  #[test]
  fn fn_ref_validator_from_named_fn() {
    fn rule(v: &str) -> crate::ValidatorResult {
      if !v.is_empty() {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
      }
    }
    let v = FnRefValidator::new(rule);
    assert!(v.validate_ref("hi").is_ok());
    assert!(v.validate_ref("").is_err());
  }

  #[test]
  fn fn_ref_validator_from_closure() {
    let v = FnRefValidator::from(|v: &str| {
      if !v.is_empty() {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
      }
    });
    assert!(v.validate_ref("hi").is_ok());
    assert!(v.validate_ref("").is_err());
  }

  #[test]
  fn fn_ref_validator_clone_shares_inner_fn() {
    let original = non_empty();
    let cloned = original.clone();
    assert!(original.validate_ref("x").is_ok());
    assert!(cloned.validate_ref("x").is_ok());
    assert!(cloned.validate_ref("").is_err());
  }

  #[test]
  fn fn_ref_validator_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FnRefValidator<str>>();
  }

  #[test]
  fn fn_ref_validator_debug_shows_placeholder() {
    let s = format!("{:?}", non_empty());
    assert!(s.contains("FnRefValidator"));
    assert!(s.contains("<fn>"));
  }

  // -------------------------------------------------------------------------
  // FnRefValidator — combinators (ValidateRefExt)
  // -------------------------------------------------------------------------

  #[test]
  fn fn_ref_validator_and_both_must_pass() {
    let strict = non_empty().and(short_enough());
    assert!(strict.validate_ref("hello").is_ok());
    assert!(strict.validate_ref("").is_err());              // non_empty fails
    assert!(strict.validate_ref("hello world!!").is_err()); // short_enough fails
  }

  #[test]
  fn fn_ref_validator_or_at_least_one_must_pass() {
    let starts_a = FnRefValidator::new(|v: &str| {
      if v.starts_with('a') {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::PatternMismatch, "must start with 'a'"))
      }
    });
    let ends_z = FnRefValidator::new(|v: &str| {
      if v.ends_with('z') {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::PatternMismatch, "must end with 'z'"))
      }
    });
    let either = starts_a.or(ends_z);
    assert!(either.validate_ref("apple").is_ok());  // starts with 'a'
    assert!(either.validate_ref("fuzz").is_ok());   // ends with 'z'
    assert!(either.validate_ref("hello").is_err()); // neither
  }

  #[test]
  fn fn_ref_validator_not_inverts_result() {
    let not_empty = non_empty().not("must be empty");
    assert!(not_empty.validate_ref("").is_ok());
    assert!(not_empty.validate_ref("hi").is_err());
  }

  #[test]
  fn fn_ref_validator_optional_skips_when_empty() {
    let optional = min_len_3().optional(|v: &str| v.is_empty());
    assert!(optional.validate_ref("").is_ok());      // absent — skipped
    assert!(optional.validate_ref("hello").is_ok()); // present and valid
    assert!(optional.validate_ref("hi").is_err());   // present and invalid
  }

  #[test]
  fn fn_ref_validator_when_skips_unless_condition_met() {
    // only enforce short_enough when the string is non-empty
    let when_non_empty = short_enough().when(|v: &str| !v.is_empty());
    assert!(when_non_empty.validate_ref("hi").is_ok());             // condition met, passes
    assert!(when_non_empty.validate_ref("hello world!!").is_err()); // condition met, fails
    assert!(when_non_empty.validate_ref("").is_ok());               // condition not met, skipped
  }

  // -------------------------------------------------------------------------
  // ValidatorAll — collects all violations
  // -------------------------------------------------------------------------

  #[test]
  fn validator_all_passes_when_all_pass() {
    let all: ValidatorAll<str> = ValidatorAll::new(vec![
      Box::new(non_empty()),
      Box::new(min_len_3()),
      Box::new(short_enough()),
    ]);
    assert!(all.validate_ref("hello").is_ok());
  }

  #[test]
  fn validator_all_single_violation() {
    let all: ValidatorAll<str> = ValidatorAll::new(vec![
      Box::new(non_empty()),
      Box::new(min_len_3()),
    ]);
    // non_empty passes but min_len_3 fails for "hi"
    let result = all.validate_all("hi");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().len(), 1);
  }

  #[test]
  fn validator_all_collects_all_violations() {
    let all: ValidatorAll<str> = ValidatorAll::new(vec![
      Box::new(non_empty()),
      Box::new(min_len_3()),
    ]);
    // Both fail for empty string
    let result = all.validate_all("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().len(), 2);
  }

  #[test]
  fn validator_all_with_mixed_fn_ref_validators() {
    let starts_upper = FnRefValidator::new(|v: &str| {
      if v.chars().next().map_or(false, |c| c.is_uppercase()) {
        Ok(())
      } else {
        Err(Violation::new(
          ViolationType::PatternMismatch,
          "must start with uppercase",
        ))
      }
    });
    let ends_period = FnRefValidator::new(|v: &str| {
      if v.ends_with('.') {
        Ok(())
      } else {
        Err(Violation::new(
          ViolationType::PatternMismatch,
          "must end with '.'",
        ))
      }
    });
    let all: ValidatorAll<str> = ValidatorAll::new(vec![
      Box::new(starts_upper),
      Box::new(ends_period),
    ]);

    assert!(all.validate_ref("Hello.").is_ok());
    assert!(all.validate_ref("Hello").is_err());  // missing period
    assert!(all.validate_ref("hello.").is_err()); // lowercase start

    // Both fail — validate_all reports two violations
    let both_fail = all.validate_all("hello");
    assert_eq!(both_fail.unwrap_err().len(), 2);
  }
}

