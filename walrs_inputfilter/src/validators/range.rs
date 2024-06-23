use std::fmt::{Debug, Display, Formatter};

use crate::{
  ScalarValue,
  ValidateValue,
  ValidateResult,
  Violation,
  ViolationType};

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

impl<'a, T: ScalarValue> RangeValidator<'a, T> {
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

impl<'a, T: ScalarValue> ValidateValue<T> for RangeValidator<'a, T> {
  /// Validates given value against contained constraints and returns a result of unit and/or a Vec of violation tuples
  /// if value doesn't pass validation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   RangeValidator,
  ///   RangeValidatorBuilder,
  ///   range_underflow_msg_getter, range_overflow_msg_getter,
  ///   ValidateValue,
  ///   ScalarValue,
  ///   ViolationType::{RangeOverflow, RangeUnderflow},
  ///   Violation
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
  ///     Violation(RangeUnderflow, range_underflow_msg_getter(&usize_vldtr, 0)),
  ///   )),
  ///   ("With \"out of upper bounds\" value", &usize_vldtr, 11, Err(
  ///     Violation(RangeOverflow, range_overflow_msg_getter(&usize_vldtr, 11)),
  ///   )),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: {}", i + 1, test_name);
  ///   assert_eq!(usize_vldtr.validate(value), expected_rslt);
  ///   assert_eq!((usize_vldtr)(value), expected_rslt);
  /// }
  /// ```
  fn validate(&self, value: T) -> Result<(), Violation> {
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
  type Output = ValidateResult;

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

  #[test]
  fn test_validate() {
    // Setup validators and test cases
    // ---
    let usize_required = RangeValidatorBuilder::<usize>::default()
      .min(1)
      .max(10)
      .build()
      .unwrap();

    let test_cases = [
      ("With valid value", &usize_required, 1, Ok(())),
      ("With valid value", &usize_required, 4, Ok(())),
      ("With valid value", &usize_required, 10, Ok(())),
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
    }
  }
}
