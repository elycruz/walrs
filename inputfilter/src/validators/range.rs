use std::fmt::{Debug, Display, Formatter};

use crate::{ScalarValue, Validate, ValidateRef, ValidatorResult, Violation, ViolationType};

/// A validator for checking that a scalar value falls within a specified range.
///
/// ```rust
///  use walrs_inputfilter::{RangeValidator, RangeValidatorBuilder, Validate};
///
///  let mut vldtr = RangeValidatorBuilder::default()
///  .min(1)
///  .max(10)
///  .build()
///  .unwrap();
///
///  assert_eq!(vldtr.min, Some(1));
///  assert_eq!(vldtr.max, Some(10));
///  assert_eq!(vldtr.validate(5), Ok(()));
///  assert_eq!(vldtr(5), Ok(()));
///
///  assert!(vldtr.validate(0).is_err());
///  assert!(vldtr(0).is_err());
///  assert!(vldtr.validate(11).is_err());
///  assert!(vldtr(11).is_err());
/// ```
///
#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct RangeValidator<'a, T: ScalarValue> {
  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "&range_underflow_msg_getter")]
  pub range_underflow_msg: &'a (dyn Fn(&RangeValidator<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&range_overflow_msg_getter")]
  pub range_overflow_msg: &'a (dyn Fn(&RangeValidator<'a, T>, T) -> String + Send + Sync),
}

impl<T: ScalarValue> RangeValidator<'_, T> {
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   RangeValidator, ViolationType,
  /// };
  ///
  /// let input = RangeValidator::<usize>::new();
  ///
  /// // Assert defaults
  /// // ----
  /// assert_eq!(input.min, None);
  /// assert_eq!(input.max, None);
  /// ```
  pub fn new() -> Self {
    RangeValidator {
      min: None,
      max: None,
      range_underflow_msg: &(range_underflow_msg_getter),
      range_overflow_msg: &(range_overflow_msg_getter),
    }
  }
}

impl<T: ScalarValue> Validate<T> for RangeValidator<'_, T> {
  /// Validates given value against contained constraints and returns a result of unit and/or a Vec of violation tuples
  /// if value doesn't pass validation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   RangeValidator, ViolationType,
  ///   RangeValidatorBuilder,
  ///   range_underflow_msg_getter, range_overflow_msg_getter,
  ///   Validate,
  ///   Violation,
  ///   ScalarValue
  /// };
  ///
  /// // Setup input constraints
  /// let usize_vldtr = RangeValidatorBuilder::<usize>::default()
  ///   .min(1)
  ///   .max(10)
  ///   .build()
  ///   .unwrap();
  ///
  /// let test_cases = [
  ///   ("With valid value (1)", &usize_vldtr, 1, Ok(())),
  ///   ("With valid value (2)", &usize_vldtr, 4, Ok(())),
  ///   ("With valid value (3)", &usize_vldtr, 10, Ok(())),
  ///   ("With \"out of lower bounds\" value", &usize_vldtr, 0, Err(
  ///     Violation(ViolationType::RangeUnderflow, range_underflow_msg_getter(&usize_vldtr, 0)),
  ///   )),
  ///   ("With \"out of upper bounds\" value", &usize_vldtr, 11, Err(
  ///     Violation(ViolationType::RangeOverflow, range_overflow_msg_getter(&usize_vldtr, 11)),
  ///   )),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: {}", i + 1, test_name);
  ///   assert_eq!(usize_vldtr.validate(value), expected_rslt);
  ///   assert_eq!(usize_vldtr(value), expected_rslt);
  /// }
  /// ```
  fn validate(&self, value: T) -> ValidatorResult {
    // Test lower bound
    if let Some(min) = self.min {
      if value < min {
        return Err(Violation(
          ViolationType::RangeUnderflow,
          (self.range_underflow_msg)(self, value),
        ));
      }
    }

    // Test upper bound
    if let Some(max) = self.max {
      if value > max {
        return Err(Violation(
          ViolationType::RangeOverflow,
          (self.range_overflow_msg)(self, value),
        ));
      }
    }

    Ok(())
  }
}

impl<T: ScalarValue> FnMut<(T,)> for RangeValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: ScalarValue> Fn<(T,)> for RangeValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: ScalarValue> FnOnce<(T,)> for RangeValidator<'_, T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

/// Returns generic range underflow message.
///
/// ```rust
/// use walrs_inputfilter::{RangeValidatorBuilder, range_underflow_msg_getter};
///
/// let input = RangeValidatorBuilder::<usize>::default()
///   .min(1)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_underflow_msg_getter(&input, 0), "`0` is less than minimum `1`.");
/// ```
pub fn range_underflow_msg_getter<T: ScalarValue>(rules: &RangeValidator<T>, x: T) -> String {
  format!("`{}` is less than minimum `{}`.", x, &rules.min.unwrap())
}

/// Returns generic range overflow message.
///
/// ```rust
/// use walrs_inputfilter::{RangeValidatorBuilder, range_overflow_msg_getter};
///
/// let input = RangeValidatorBuilder::<usize>::default()
///   .max(10)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_overflow_msg_getter(&input, 100), "`100` is greater than maximum `10`.");
/// ```
pub fn range_overflow_msg_getter<T: ScalarValue>(rules: &RangeValidator<T>, x: T) -> String {
  format!("`{}` is greater than maximum `{}`.", x, &rules.max.unwrap())
}

impl<T: ScalarValue> Default for RangeValidator<'_, T> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   RangeValidator
  /// };
  ///
  /// let input = RangeValidator::<usize>::default();
  ///
  /// // Assert defaults
  /// // ----
  /// assert_eq!(input.min, None);
  /// assert_eq!(input.max, None);
  /// ```
  fn default() -> Self {
    Self::new()
  }
}

impl<T: ScalarValue> Display for RangeValidator<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "RangeValidator {{ min: {}, max: {} }}",
      self.min.map_or("None".to_string(), |x| x.to_string()),
      self.max.map_or("None".to_string(), |x| x.to_string()),
    )
  }
}

impl<T: ScalarValue> Debug for RangeValidator<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::ViolationType::{RangeOverflow, RangeUnderflow};

  #[test]
  fn test_new_and_default() {
    let input = RangeValidator::<usize>::new();

    // Assert defaults
    // ----
    assert_eq!(input.min, None);
    assert_eq!(input.max, None);

    let input_def = RangeValidator::<usize>::default();

    // Assert defaults
    // ----
    assert_eq!(input_def.min, None);
    assert_eq!(input_def.max, None);
  }

  #[test]
  fn test_validate() {
    // Setup validators and test cases
    // ---
    let usize_required = RangeValidatorBuilder::<usize>::default()
        .min(1)
        .max(10)
        .build()
        .unwrap();

    let empty_vldtr = RangeValidator::<usize>::new();

    let test_cases = [
      ("With valid value", &usize_required, 1, Ok(())),
      ("With valid value", &usize_required, 4, Ok(())),
      ("With valid value", &usize_required, 10, Ok(())),
      ("With valid value", &empty_vldtr, 10, Ok(())),
      (
        "With \"out of lower bounds\" value",
        &usize_required,
        0,
        Err(Violation(
          RangeUnderflow,
          range_underflow_msg_getter(&usize_required, 0),
        )),
      ),
      (
        "With \"out of upper bounds\" value",
        &usize_required,
        11,
        Err(Violation(
          RangeOverflow,
          range_overflow_msg_getter(&usize_required, 11),
        )),
      ),
    ];

    // Run test cases
    for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
      println!("Case {}: {}", i + 1, test_name);
      assert_eq!(input.validate(value), expected_rslt);
      assert_eq!((&input)(value), expected_rslt);
    }
  }

  #[test]
  fn test_fn_traits() {
    let vldtr = RangeValidatorBuilder::<i32>::default()
        .build()
        .unwrap();

    fn call_fn_once<T: ScalarValue>(f: impl FnOnce(T) -> ValidatorResult, v: T) -> ValidatorResult {
      f(v)
    }

    fn call_fn_mut<T: ScalarValue>(f: &mut impl FnMut(T) -> ValidatorResult, v: T) -> ValidatorResult {
      f(v)
    }

    // Test FnMut
    let mut vldtr_mut = vldtr.clone();
    let result_mut = call_fn_mut(&mut vldtr_mut, 99);
    assert_eq!(result_mut, Ok(()));

    // Test FnOnce
    let result_once = call_fn_once(vldtr, 99);
    assert_eq!(result_once, Ok(()));
  }

  #[test]
  fn test_display_and_debug() {
    let vldtr = RangeValidatorBuilder::<i32>::default()
      .min(1)
      .max(10)
      .build()
      .unwrap();

    let display_output = format!("{}", vldtr);
    let debug_output = format!("{:?}", vldtr);

    let expected_output = "RangeValidator { min: 1, max: 10 }";

    assert_eq!(display_output, expected_output);
    assert_eq!(debug_output, expected_output);
  }
}
