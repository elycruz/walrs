use crate::{
  Message, MessageContext, MessageParams, SteppableValue, ToAttributesList, Validate,
  ValidatorResult, Violation, ViolationType,
};
use serde_json::value::to_value as to_json_value;
use std::fmt::{Debug, Display, Formatter};

/// Validator for checking that a number is a valid multiple of a specified step value.
///
/// ```rust
/// use walrs_validation::{
///   StepValidator,
///   StepValidatorBuilder,
///   Validate,
///   ValidatorResult,
///   Violation,
///   ViolationType::StepMismatch,
/// };
///
/// let vldtr = StepValidatorBuilder::<usize>::default()
///   .step(5)
///   .build()
///   .unwrap();
///
/// // Validate values
/// assert_eq!(vldtr.validate(0), Ok(()));
/// assert_eq!(vldtr.validate(5), Ok(()));
/// assert_eq!(vldtr.validate(10), Ok(()));
/// assert_eq!(vldtr.validate(100), Ok(()));
/// assert!(vldtr.validate(7).is_err());   // StepMismatch
/// assert!(vldtr.validate(26).is_err());  // StepMismatch
/// ```
///
#[must_use]
#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct StepValidator<'a, T: SteppableValue> {
  #[builder(default = "None")]
  pub step: Option<T>,

  #[builder(default = "default_step_mismatch_msg()")]
  pub step_mismatch: Message<T>,

  /// Optional locale for internationalized error messages.
  #[builder(default = "None")]
  pub locale: Option<&'a str>,
}

impl<T> StepValidator<'_, T>
where
  T: SteppableValue,
{
  fn _validate_step(&self, v: T) -> Option<ViolationType> {
    if let Some(step) = self.step {
      if step != Default::default() && v % step != Default::default() {
        return Some(ViolationType::StepMismatch);
      }
    }
    None
  }

  fn _get_violation_msg(&self, value: &T) -> String {
    let params = MessageParams::new("StepValidator")
      .with_step(self.step.map(|s| s.to_string()).unwrap_or_default());
    let ctx = MessageContext::with_locale(value, params, self.locale);
    self.step_mismatch.resolve_with_context(&ctx)
  }

  /// Creates a new `StepValidator` with default values.
  ///
  /// ```rust
  /// use walrs_validation::StepValidator;
  ///
  /// let vldtr = StepValidator::<usize>::new();
  ///
  /// // Assert defaults
  /// assert_eq!(vldtr.step, None);
  /// ```
  pub fn new() -> Self {
    StepValidator {
      step: None,
      step_mismatch: default_step_mismatch_msg(),
      locale: None,
    }
  }

  /// Returns a builder for constructing a `StepValidator`.
  ///
  /// ```rust
  /// use walrs_validation::StepValidator;
  ///
  /// let vldtr = StepValidator::<usize>::builder()
  ///   .step(5)
  ///   .build()
  ///   .unwrap();
  ///
  /// assert_eq!(vldtr.step, Some(5));
  /// ```
  pub fn builder() -> StepValidatorBuilder<'static, T> {
    StepValidatorBuilder::default()
  }
}

impl<T> Validate<T> for StepValidator<'_, T>
where
  T: SteppableValue,
{
  /// Validates given number against contained step constraint.
  ///
  /// ```rust
  /// use walrs_validation::{
  ///   StepValidator,
  ///   StepValidatorBuilder,
  ///   Validate,
  ///   ViolationType::StepMismatch,
  /// };
  ///
  /// let vldtr = StepValidatorBuilder::<usize>::default()
  ///   .step(5)
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test valid values
  /// assert_eq!(vldtr.validate(0), Ok(()));
  /// assert_eq!(vldtr.validate(5), Ok(()));
  /// assert_eq!(vldtr.validate(25), Ok(()));
  ///
  /// // Test invalid values
  /// assert!(vldtr.validate(7).is_err());
  /// assert!(vldtr.validate(26).is_err());
  /// ```
  fn validate(&self, value: T) -> ValidatorResult {
    if self._validate_step(value).is_some() {
      return Err(Violation(
        ViolationType::StepMismatch,
        self._get_violation_msg(&value),
      ));
    }
    Ok(())
  }
}

impl<T> ToAttributesList for StepValidator<'_, T>
where
  T: SteppableValue,
{
  /// Returns the validator's ruleset as a list of key/value pairs suitable for
  /// use as HTML attribute-name/attribute-value pairs.
  ///
  /// ```rust
  /// use walrs_validation::{
  ///   StepValidator,
  ///   StepValidatorBuilder,
  ///   ToAttributesList
  /// };
  ///
  /// let vldtr = StepValidatorBuilder::<usize>::default()
  ///   .step(5)
  ///   .build()
  ///   .unwrap();
  ///
  /// assert_eq!(
  ///   vldtr.to_attributes_list(),
  ///   Some(vec![
  ///     ("step".to_string(), serde_json::Value::from(5)),
  ///   ])
  /// );
  /// ```
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    if let Some(step) = self.step {
      Some(vec![("step".to_string(), to_json_value(step).unwrap())])
    } else {
      None
    }
  }
}

#[cfg(feature = "fn_traits")]
impl<T: SteppableValue> FnMut<(T,)> for StepValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: SteppableValue> Fn<(T,)> for StepValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: SteppableValue> FnOnce<(T,)> for StepValidator<'_, T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: SteppableValue> FnMut<(&T,)> for StepValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (&T,)) -> Self::Output {
    self.validate(*args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: SteppableValue> Fn<(&T,)> for StepValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (&T,)) -> Self::Output {
    self.validate(*args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: SteppableValue> FnOnce<(&T,)> for StepValidator<'_, T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&T,)) -> Self::Output {
    self.validate(*args.0)
  }
}

impl<T> Default for StepValidator<'_, T>
where
  T: SteppableValue,
{
  fn default() -> Self {
    StepValidator::new()
  }
}

impl<T: SteppableValue> Display for StepValidator<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "StepValidator {{step: {}}}",
      &self
        .step
        .map(|x| x.to_string())
        .unwrap_or("None".to_string()),
    )
  }
}

impl<T: SteppableValue> Debug for StepValidator<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

/// Returns generic step mismatch message.
///
/// ```rust
/// use walrs_validation::step_mismatch_msg;
///
/// assert_eq!(step_mismatch_msg(7, 5), "`7` is not a multiple of step `5`.");
/// ```
pub fn step_mismatch_msg<T: SteppableValue>(value: T, step: T) -> String {
  format!("`{}` is not a multiple of step `{}`.", value, step)
}

/// Returns default step mismatch Message provider for StepValidator.
pub fn default_step_mismatch_msg<T: SteppableValue>() -> Message<T> {
  Message::provider(|ctx: &MessageContext<T>| {
    let step_str = ctx.params.step.as_deref().unwrap_or("?");
    format!("`{}` is not a multiple of step `{}`.", ctx.value, step_str)
  })
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::ViolationType::StepMismatch;
  use std::error::Error;

  #[test]
  fn test_new_and_default() {
    let vldtr = StepValidator::<usize>::new();
    assert_eq!(vldtr.step, None);

    let vldtr_def = StepValidator::<usize>::default();
    assert_eq!(vldtr_def.step, None);
  }

  #[test]
  fn test_construction() -> Result<(), Box<dyn Error>> {
    for (test_name, instance, step) in [
      (
        "Default",
        StepValidatorBuilder::<usize>::default().build()?,
        None,
      ),
      ("Default 2", StepValidator::<usize>::new(), None),
      ("Default 3", StepValidator::<usize>::default(), None),
      (
        "With step",
        StepValidatorBuilder::<usize>::default().step(5).build()?,
        Some(5),
      ),
    ] {
      println!("\"{}\" test {}", test_name, &instance);
      assert_eq!(instance.step, step);
    }

    Ok(())
  }

  #[test]
  fn test_validate() -> Result<(), Box<dyn Error>> {
    let vldtr = StepValidatorBuilder::<usize>::default().step(5).build()?;

    // Valid multiples of 5
    assert_eq!(vldtr.validate(0), Ok(()));
    assert_eq!(vldtr.validate(5), Ok(()));
    assert_eq!(vldtr.validate(10), Ok(()));
    assert_eq!(vldtr.validate(25), Ok(()));
    assert_eq!(vldtr.validate(100), Ok(()));

    // Invalid - not multiples of 5
    let result = vldtr.validate(7);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, StepMismatch);
    assert_eq!(err.1, step_mismatch_msg(7, 5));

    let result = vldtr.validate(26);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, StepMismatch);
    assert_eq!(err.1, step_mismatch_msg(26, 5));

    Ok(())
  }

  #[test]
  fn test_validate_no_step() -> Result<(), Box<dyn Error>> {
    let vldtr = StepValidatorBuilder::<usize>::default().build()?;

    // Without step constraint, any value should pass
    assert_eq!(vldtr.validate(0), Ok(()));
    assert_eq!(vldtr.validate(7), Ok(()));
    assert_eq!(vldtr.validate(100), Ok(()));

    Ok(())
  }

  #[test]
  fn test_validate_step_zero() -> Result<(), Box<dyn Error>> {
    let vldtr = StepValidatorBuilder::<usize>::default().step(0).build()?;

    // Step of 0 should not validate (division by zero protection)
    assert_eq!(vldtr.validate(0), Ok(()));
    assert_eq!(vldtr.validate(7), Ok(()));
    assert_eq!(vldtr.validate(100), Ok(()));

    Ok(())
  }

  #[test]
  fn test_validate_signed_integers() -> Result<(), Box<dyn Error>> {
    let vldtr = StepValidatorBuilder::<i32>::default().step(5).build()?;

    // Valid multiples of 5 (including negative)
    assert_eq!(vldtr.validate(0), Ok(()));
    assert_eq!(vldtr.validate(5), Ok(()));
    assert_eq!(vldtr.validate(-5), Ok(()));
    assert_eq!(vldtr.validate(-10), Ok(()));

    // Invalid
    assert!(vldtr.validate(7).is_err());
    assert!(vldtr.validate(-7).is_err());

    Ok(())
  }

  #[test]
  fn test_validate_floats() -> Result<(), Box<dyn Error>> {
    let vldtr = StepValidatorBuilder::<f64>::default().step(0.5).build()?;

    // Valid multiples of 0.5
    assert_eq!(vldtr.validate(0.0), Ok(()));
    assert_eq!(vldtr.validate(0.5), Ok(()));
    assert_eq!(vldtr.validate(1.0), Ok(()));
    assert_eq!(vldtr.validate(2.5), Ok(()));

    // Invalid - not multiples of 0.5
    assert!(vldtr.validate(0.3).is_err());
    assert!(vldtr.validate(0.7).is_err());

    Ok(())
  }

  #[test]
  fn test_to_attributes_list() -> Result<(), Box<dyn Error>> {
    let vldtr = StepValidatorBuilder::<usize>::default().step(5).build()?;

    assert_eq!(
      vldtr.to_attributes_list(),
      Some(vec![("step".to_string(), serde_json::Value::from(5)),])
    );

    // No step - should return None
    let vldtr_no_step = StepValidatorBuilder::<usize>::default().build()?;
    assert_eq!(vldtr_no_step.to_attributes_list(), None);

    Ok(())
  }

  #[test]
  fn test_display() -> Result<(), Box<dyn Error>> {
    let vldtr = StepValidatorBuilder::<usize>::default().step(5).build()?;

    let display_output = format!("{}", vldtr);
    assert_eq!(display_output, "StepValidator {step: 5}");

    let vldtr_none = StepValidatorBuilder::<usize>::default().build()?;
    let display_output_none = format!("{}", vldtr_none);
    assert_eq!(display_output_none, "StepValidator {step: None}");

    Ok(())
  }

  #[test]
  fn test_step_mismatch_msg() {
    assert_eq!(
      step_mismatch_msg(7, 5),
      "`7` is not a multiple of step `5`."
    );
    assert_eq!(
      step_mismatch_msg(26, 5),
      "`26` is not a multiple of step `5`."
    );
  }
}
