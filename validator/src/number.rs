use crate::{
  Message, MessageContext, MessageParams, NumberValue, Validate, ValidatorResult, Violation,
  ViolationType,
  ViolationType::{RangeOverflow, RangeUnderflow, StepMismatch},
};
use std::fmt::{Display, Formatter};

use crate::traits::ToAttributesList;
use serde_json::value::to_value as to_json_value;

/// Validator for performing number range and step checks against given number.
///
/// ```rust
/// use walrs_validator::{
///   NumberValidator,
///   NumberValidatorBuilder,
///   Validate,
///   ValidatorResult,
///   Violation,
///   ViolationType::{RangeUnderflow, RangeOverflow, StepMismatch},
/// };
///
/// let vldtr = NumberValidatorBuilder::<usize>::default()
///   .min(1)
///   .max(100)
///   .step(5)
///   .build()
///   .unwrap();
///
/// // Validate values
/// assert_eq!(vldtr.validate(95), Ok(()));
/// assert!(vldtr.validate(0).is_err());   // RangeUnderflow
/// assert!(vldtr.validate(101).is_err()); // RangeOverflow
/// assert!(vldtr.validate(26).is_err());  // StepMismatch
/// ```
///
#[must_use]
#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct NumberValidator<T: NumberValue> {
  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "None")]
  pub step: Option<T>,

  #[builder(default = "default_num_range_underflow_msg()")]
  pub range_underflow: Message<T>,

  #[builder(default = "default_num_range_overflow_msg()")]
  pub range_overflow: Message<T>,

  #[builder(default = "default_num_step_mismatch_msg()")]
  pub step_mismatch: Message<T>,
}

impl<T> NumberValidator<T>
where
  T: NumberValue,
{
  fn _validate_number(&self, v: T) -> Option<ViolationType> {
    // Test Min
    if let Some(min) = self.min {
      if v < min {
        return Some(RangeUnderflow);
      }
    }

    // Test Max
    if let Some(max) = self.max {
      if v > max {
        return Some(RangeOverflow);
      }
    }

    // Test Step
    if let Some(step) = self.step {
      if step != Default::default() && v % step != Default::default() {
        return Some(StepMismatch);
      }
    }

    None
  }

  fn _get_violation_msg(&self, violation: ViolationType, value: &T) -> String {
    let params = MessageParams::new("NumberValidator")
      .with_min(self.min.map(|m| m.to_string()).unwrap_or_default())
      .with_max(self.max.map(|m| m.to_string()).unwrap_or_default())
      .with_step(self.step.map(|s| s.to_string()).unwrap_or_default());
    let ctx = MessageContext::new(value, params);

    match violation {
      RangeUnderflow => self.range_underflow.resolve_with_context(&ctx),
      RangeOverflow => self.range_overflow.resolve_with_context(&ctx),
      StepMismatch => self.step_mismatch.resolve_with_context(&ctx),
      _ => unreachable!("Unsupported Constraint Violation Enum matched"),
    }
  }

  pub fn new() -> Self {
    NumberValidator {
      min: None,
      max: None,
      step: None,
      range_underflow: default_num_range_underflow_msg(),
      range_overflow: default_num_range_overflow_msg(),
      step_mismatch: default_num_step_mismatch_msg(),
    }
  }
}

impl<T> Validate<T> for NumberValidator<T>
where
  T: NumberValue,
{
  /// Validates given number against contained constraints.
  fn validate(&self, value: T) -> ValidatorResult {
    if let Some(violation) = self._validate_number(value) {
      return Err(Violation(
        violation,
        self._get_violation_msg(violation, &value),
      ));
    }

    Ok(())
  }
}

impl<T> ToAttributesList for NumberValidator<T>
where
  T: NumberValue,
{
  /// Returns the validator's ruleset as a list of key/value pairs suitable for
  ///  use as HTML attribute-name/attribute-value pairs.
  ///
  /// ```rust
  /// use walrs_validator::{
  ///   NumberValidator,
  ///   NumberValidatorBuilder,
  ///   ToAttributesList
  /// };
  ///
  /// let vldtr = NumberValidatorBuilder::<usize>::default()
  ///   .min(1)
  ///   .max(100)
  ///   .step(5)
  ///   .build()
  ///   .unwrap();
  ///
  /// assert_eq!(
  ///   vldtr.to_attributes_list(),
  ///   Some(vec![
  ///     ("min".to_string(), serde_json::Value::from(1)),
  ///     ("max".to_string(), serde_json::Value::from(100)),
  ///     ("step".to_string(), serde_json::Value::from(5)),
  ///   ])
  /// );
  /// ```
  ///
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    let mut attrs = Vec::<(String, serde_json::Value)>::new();

    if let Some(min) = self.min {
      attrs.push(("min".to_string(), to_json_value(min).unwrap()));
    }

    if let Some(max) = self.max {
      attrs.push(("max".to_string(), to_json_value(max).unwrap()));
    }

    if let Some(step) = self.step {
      attrs.push(("step".to_string(), to_json_value(step).unwrap()));
    }

    if attrs.is_empty() { None } else { Some(attrs) }
  }
}

#[cfg(feature = "fn_traits")]
impl<T: NumberValue> FnMut<(T,)> for NumberValidator<T> {
  extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: NumberValue> Fn<(T,)> for NumberValidator<T> {
  extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: NumberValue> FnOnce<(T,)> for NumberValidator<T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: NumberValue> FnMut<(&T,)> for NumberValidator<T> {
  extern "rust-call" fn call_mut(&mut self, args: (&T,)) -> Self::Output {
    self.validate(*args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: NumberValue> Fn<(&T,)> for NumberValidator<T> {
  extern "rust-call" fn call(&self, args: (&T,)) -> Self::Output {
    self.validate(*args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: NumberValue> FnOnce<(&T,)> for NumberValidator<T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&T,)) -> Self::Output {
    self.validate(*args.0)
  }
}

impl<T> Default for NumberValidator<T>
where
  T: NumberValue,
{
  fn default() -> Self {
    NumberValidator::new()
  }
}

impl<T: NumberValue> Display for NumberValidator<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "NumberValidator {{min: {}, max: {}, step: {}}}",
      &self
        .min
        .map(|x| x.to_string())
        .unwrap_or("None".to_string()),
      &self
        .max
        .map(|x| x.to_string())
        .unwrap_or("None".to_string()),
      &self
        .step
        .map(|x| x.to_string())
        .unwrap_or("None".to_string()),
    )
  }
}

/// Returns generic range underflow message for numbers.
///
/// ```rust
/// use walrs_validator::num_range_underflow_msg;
///
/// assert_eq!(num_range_underflow_msg(0, 1), "`0` is less than minimum `1`.");
/// ```
pub fn num_range_underflow_msg<T: NumberValue>(value: T, min: T) -> String {
  format!("`{}` is less than minimum `{}`.", value, min)
}

/// Returns generic range overflow message for numbers.
///
/// ```rust
/// use walrs_validator::num_range_overflow_msg;
///
/// assert_eq!(num_range_overflow_msg(100, 10), "`100` is greater than maximum `10`.");
/// ```
pub fn num_range_overflow_msg<T: NumberValue>(value: T, max: T) -> String {
  format!("`{}` is greater than maximum `{}`.", value, max)
}

/// Returns generic step mismatch message for numbers.
///
/// ```rust
/// use walrs_validator::num_step_mismatch_msg;
///
/// assert_eq!(num_step_mismatch_msg(7, 5), "`7` is not a multiple of step `5`.");
/// ```
pub fn num_step_mismatch_msg<T: NumberValue>(value: T, step: T) -> String {
  format!("`{}` is not a multiple of step `{}`.", value, step)
}

/// Returns default range underflow Message provider for NumberValidator.
pub fn default_num_range_underflow_msg<T: NumberValue>() -> Message<T> {
  Message::provider(|ctx: &MessageContext<T>| {
    let min_str = ctx.params.min.as_deref().unwrap_or("?");
    format!("`{}` is less than minimum `{}`.", ctx.value, min_str)
  })
}

/// Returns default range overflow Message provider for NumberValidator.
pub fn default_num_range_overflow_msg<T: NumberValue>() -> Message<T> {
  Message::provider(|ctx: &MessageContext<T>| {
    let max_str = ctx.params.max.as_deref().unwrap_or("?");
    format!("`{}` is greater than maximum `{}`.", ctx.value, max_str)
  })
}

/// Returns default step mismatch Message provider for NumberValidator.
pub fn default_num_step_mismatch_msg<T: NumberValue>() -> Message<T> {
  Message::provider(|ctx: &MessageContext<T>| {
    let step_str = ctx.params.step.as_deref().unwrap_or("?");
    format!("`{}` is not a multiple of step `{}`.", ctx.value, step_str)
  })
}

#[cfg(test)]
mod test {

  use super::*;
  use std::error::Error;

  #[test]
  fn test_construction() -> Result<(), Box<dyn Error>> {
    // Assert all property states for different construction scenarios
    for (test_name, instance, min, max, step) in [
      (
        "Default",
        NumberValidatorBuilder::<usize>::default().build()?,
        None,
        None,
        None,
      ),
      (
        "Default 2",
        NumberValidator::<usize>::new(),
        None,
        None,
        None,
      ),
      (
        "Default 3",
        NumberValidator::<usize>::default(),
        None,
        None,
        None,
      ),
      (
        "With Range",
        NumberValidatorBuilder::<usize>::default()
          .min(0)
          .max(100)
          .build()?,
        Some(0),
        Some(100),
        None,
      ),
      (
        "With `step`",
        NumberValidatorBuilder::<usize>::default().step(5).build()?,
        None,
        None,
        Some(5),
      ),
    ] {
      println!("\"{}\" test {}", test_name, &instance);

      assert_eq!(instance.min, min);
      assert_eq!(instance.max, max);
      assert_eq!(instance.step, step);
    }

    Ok(())
  }

  #[test]
  fn test_validate() -> Result<(), Box<dyn Error>> {
    let vldtr = NumberValidatorBuilder::<usize>::default()
      .min(1)
      .max(100)
      .step(5)
      .build()?;

    // Valid values
    assert_eq!(vldtr.validate(5), Ok(()));
    assert_eq!(vldtr.validate(95), Ok(()));

    // RangeUnderflow
    let result = vldtr.validate(0);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, RangeUnderflow);
    assert_eq!(err.1, num_range_underflow_msg(0, 1));

    // RangeOverflow
    let result = vldtr.validate(101);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, RangeOverflow);
    assert_eq!(err.1, num_range_overflow_msg(101, 100));

    // StepMismatch
    let result = vldtr.validate(26);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, StepMismatch);
    assert_eq!(err.1, num_step_mismatch_msg(26, 5));

    Ok(())
  }

  #[test]
  fn test_custom_message() -> Result<(), Box<dyn Error>> {
    let custom_msg: Message<usize> = Message::static_msg("Value is too small!");
    let vldtr = NumberValidatorBuilder::<usize>::default()
      .min(10)
      .range_underflow(custom_msg)
      .build()?;

    let result = vldtr.validate(5);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.1, "Value is too small!");

    Ok(())
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_traits() -> Result<(), Box<dyn Error>> {
    let vldtr = NumberValidatorBuilder::<i32>::default().build()?;

    fn call_fn_once<T: NumberValue>(f: impl FnOnce(T) -> ValidatorResult, v: T) -> ValidatorResult {
      f(v)
    }

    fn call_fn_mut<T: NumberValue>(
      f: &mut impl FnMut(T) -> ValidatorResult,
      v: T,
    ) -> ValidatorResult {
      f(v)
    }

    // Test FnMut
    let mut vldtr_mut = vldtr.clone();
    let result_mut = call_fn_mut(&mut vldtr_mut, 99);
    assert_eq!(result_mut, Ok(()));

    // Test FnOnce
    let result_once = call_fn_once(vldtr, 99);
    assert_eq!(result_once, Ok(()));

    Ok(())
  }

  #[test]
  fn test_to_attributes_list() -> Result<(), Box<dyn Error>> {
    let vldtr = NumberValidatorBuilder::<usize>::default()
      .min(1)
      .max(100)
      .step(5)
      .build()?;

    assert_eq!(
      vldtr.to_attributes_list(),
      Some(vec![
        ("min".to_string(), serde_json::Value::from(1)),
        ("max".to_string(), serde_json::Value::from(100)),
        ("step".to_string(), serde_json::Value::from(5)),
      ])
    );

    Ok(())
  }

  #[test]
  fn test_display() -> Result<(), Box<dyn Error>> {
    let vldtr = NumberValidatorBuilder::<usize>::default()
      .min(1)
      .max(100)
      .step(5)
      .build()?;

    let display_output = format!("{}", vldtr);
    assert_eq!(
      display_output,
      "NumberValidator {min: 1, max: 100, step: 5}"
    );

    Ok(())
  }
}
