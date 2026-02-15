use std::fmt::{Debug, Display, Formatter};

use crate::{Message, MessageContext, MessageParams, ScalarValue, Validate, ValidatorResult, Violation, ViolationType};

/// A validator for checking that a scalar value falls within a specified range.
///
/// ```rust
///  use walrs_validator::{RangeValidator, RangeValidatorBuilder, Validate};
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
///
///  assert!(vldtr.validate(0).is_err());
///  assert!(vldtr.validate(11).is_err());
/// ```
///
#[must_use]
#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct RangeValidator<T: ScalarValue> {
  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "default_range_underflow_msg()")]
  pub range_underflow_msg: Message<T>,

  #[builder(default = "default_range_overflow_msg()")]
  pub range_overflow_msg: Message<T>,
}

impl<T: ScalarValue> RangeValidator<T> {
  ///
  /// ```rust
  /// use walrs_validator::{
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
      range_underflow_msg: default_range_underflow_msg(),
      range_overflow_msg: default_range_overflow_msg(),
    }
  }
}

impl<T: ScalarValue> Validate<T> for RangeValidator<T> {
  /// Validates given value against contained constraints and returns a result of unit and/or a Vec of violation tuples
  /// if value doesn't pass validation.
  ///
  /// ```rust
  /// use walrs_validator::{
  ///   RangeValidator, ViolationType,
  ///   RangeValidatorBuilder,
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
  /// // Test valid values
  /// assert_eq!(usize_vldtr.validate(1), Ok(()));
  /// assert_eq!(usize_vldtr.validate(4), Ok(()));
  /// assert_eq!(usize_vldtr.validate(10), Ok(()));
  ///
  /// // Test invalid values
  /// assert!(usize_vldtr.validate(0).is_err());
  /// assert!(usize_vldtr.validate(11).is_err());
  /// ```
  fn validate(&self, value: T) -> ValidatorResult {
    // Test lower bound
    if let Some(min) = self.min {
      if value < min {
        let params = MessageParams::new("RangeValidator")
          .with_min(min)
          .with_max(self.max.map(|m| m.to_string()).unwrap_or_default());
        let ctx = MessageContext::new(&value, params);
        return Err(Violation(
          ViolationType::RangeUnderflow,
          self.range_underflow_msg.resolve_with_context(&ctx),
        ));
      }
    }

    // Test upper bound
    if let Some(max) = self.max {
      if value > max {
        let params = MessageParams::new("RangeValidator")
          .with_min(self.min.map(|m| m.to_string()).unwrap_or_default())
          .with_max(max);
        let ctx = MessageContext::new(&value, params);
        return Err(Violation(
          ViolationType::RangeOverflow,
          self.range_overflow_msg.resolve_with_context(&ctx),
        ));
      }
    }

    Ok(())
  }
}

#[cfg(feature = "fn_traits")]
impl<T: ScalarValue> FnMut<(T,)> for RangeValidator<T> {
  extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: ScalarValue> Fn<(T,)> for RangeValidator<T> {
  extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: ScalarValue> FnOnce<(T,)> for RangeValidator<T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

/// Returns generic range underflow message.
///
/// ```rust
/// use walrs_validator::range_underflow_msg_getter;
///
/// assert_eq!(range_underflow_msg_getter(0, 1), "`0` is less than minimum `1`.");
/// ```
pub fn range_underflow_msg_getter<T: ScalarValue>(value: T, min: T) -> String {
  format!("`{}` is less than minimum `{}`.", value, min)
}

/// Returns generic range overflow message.
///
/// ```rust
/// use walrs_validator::range_overflow_msg_getter;
///
/// assert_eq!(range_overflow_msg_getter(100, 10), "`100` is greater than maximum `10`.");
/// ```
pub fn range_overflow_msg_getter<T: ScalarValue>(value: T, max: T) -> String {
  format!("`{}` is greater than maximum `{}`.", value, max)
}

/// Returns default range underflow Message provider.
///
/// This wraps `range_underflow_msg_getter` in a `Message::Provider` for use with `RangeValidator`.
pub fn default_range_underflow_msg<T: ScalarValue>() -> Message<T> {
  Message::provider(|ctx: &MessageContext<T>| {
    let min_str = ctx.params.min.as_deref().unwrap_or("?");
    format!("`{}` is less than minimum `{}`.", ctx.value, min_str)
  })
}

/// Returns default range overflow Message provider.
///
/// This wraps `range_overflow_msg_getter` in a `Message::Provider` for use with `RangeValidator`.
pub fn default_range_overflow_msg<T: ScalarValue>() -> Message<T> {
  Message::provider(|ctx: &MessageContext<T>| {
    let max_str = ctx.params.max.as_deref().unwrap_or("?");
    format!("`{}` is greater than maximum `{}`.", ctx.value, max_str)
  })
}

impl<T: ScalarValue> Default for RangeValidator<T> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use walrs_validator::{
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

impl<T: ScalarValue> Display for RangeValidator<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "RangeValidator {{ min: {}, max: {} }}",
      self.min.map_or("None".to_string(), |x| x.to_string()),
      self.max.map_or("None".to_string(), |x| x.to_string()),
    )
  }
}

impl<T: ScalarValue> Debug for RangeValidator<T> {
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

    // Test valid values
    assert_eq!(usize_required.validate(1), Ok(()));
    assert_eq!(usize_required.validate(4), Ok(()));
    assert_eq!(usize_required.validate(10), Ok(()));
    assert_eq!(empty_vldtr.validate(10), Ok(()));

    // Test underflow
    let result = usize_required.validate(0);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, RangeUnderflow);
    assert_eq!(err.1, range_underflow_msg_getter(0, 1));

    // Test overflow
    let result = usize_required.validate(11);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, RangeOverflow);
    assert_eq!(err.1, range_overflow_msg_getter(11, 10));

    #[cfg(feature = "fn_traits")]
    {
      assert_eq!((&usize_required)(5), Ok(()));
      assert!((&usize_required)(0).is_err());
    }
  }

  #[test]
  fn test_custom_message() {
    let custom_msg: Message<usize> = Message::static_msg("Value too small!");
    let vldtr = RangeValidatorBuilder::<usize>::default()
      .min(10)
      .range_underflow_msg(custom_msg)
      .build()
      .unwrap();

    let result = vldtr.validate(5);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.1, "Value too small!");
  }

  #[cfg(feature = "fn_traits")]
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

