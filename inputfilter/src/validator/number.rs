use std::borrow::Cow;
use crate::types::{ConstraintViolation, ConstraintViolation::{NotEqual, RangeOverflow, RangeUnderflow, StepMismatch}, NumberValue};

use crate::types::{ValidateValue, ValidationResult};

pub type NumberVldrViolationCallback<'a, T> =
  (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync);

#[derive(Builder, Clone)]
pub struct NumberValidator<'a, T: NumberValue> {
  #[builder(setter(into), default = "None")]
  pub min: Option<T>,

  #[builder(setter(into), default = "None")]
  pub max: Option<T>,

  #[builder(setter(into), default = "None")]
  pub step: Option<T>,

  #[builder(setter(into), default = "None")]
  pub equals: Option<T>,

  #[builder(default = "&range_underflow_msg")]
  pub range_underflow: &'a (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&range_overflow_msg")]
  pub range_overflow: &'a (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&step_mismatch_msg")]
  pub step_mismatch: &'a (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&step_mismatch_msg")]
  pub not_equal: &'a (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync),
}

impl<'a, T> NumberValidator<'a, T>
where
  T: NumberValue,
{
  fn _validate_integer(&self, v: T) -> Option<ConstraintViolation> {
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

    // Test Equal
    if let Some(rhs) = self.equals {
      if v == rhs {
        return Some(NotEqual);
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

  fn _get_violation_msg(&self, violation: ConstraintViolation, value: T) -> String {
    let f = match violation {
      RangeUnderflow => Some(&self.range_underflow),
      RangeOverflow => Some(&self.range_overflow),
      NotEqual => Some(&self.not_equal),
      StepMismatch => Some(&self.step_mismatch),
      _ => unreachable!("Unsupported Constraint Violation Enum matched"),
    };

    f.map(|_f| (_f)(self, value)).unwrap()
  }

  pub fn new() -> Self {
    NumberValidator {
      min: None,
      max: None,
      step: None,
      equals: None,
      range_underflow: &range_underflow_msg,
      range_overflow: &range_overflow_msg,
      step_mismatch: &step_mismatch_msg,
      not_equal: &not_equal_msg,
    }
  }
}

impl<T> ValidateValue<T> for NumberValidator<'_, T>
where
  T: NumberValue,
{
  fn validate(&self, value: Cow<'_, T>) -> ValidationResult {
    if let Some(violation) = self._validate_integer(*value) {
      return Err(vec![(
        violation,
        self._get_violation_msg(violation, *value),
      )]);
    }

    Ok(())
  }
}

impl<T: NumberValue> FnMut<(Cow<'_, T>, )> for NumberValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'_, T>, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: NumberValue> Fn<(Cow<'_, T>, )> for NumberValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (Cow<'_, T>, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: NumberValue> FnOnce<(Cow<'_, T>,)> for NumberValidator<'_, T> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (Cow<'_, T>,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<'a, T> Default for NumberValidator<'a, T>
where
  T: NumberValue,
{
  fn default() -> Self {
    NumberValidator::new()
  }
}

pub fn range_underflow_msg<T>(rules: &NumberValidator<T>, x: T) -> String
where
  T: NumberValue,
{
  format!(
    "`{:}` is less than minimum `{:}`.",
    x,
    &rules.min.as_ref().unwrap()
  )
}

pub fn range_overflow_msg<T>(rules: &NumberValidator<T>, x: T) -> String
where
  T: NumberValue,
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.max.as_ref().unwrap()
  )
}

pub fn step_mismatch_msg<T: NumberValue>(
  rules: &NumberValidator<T>,
  x: T,
) -> String {
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.step.as_ref().unwrap()
  )
}

pub fn not_equal_msg<T: NumberValue>(
  rules: &NumberValidator<T>,
  x: T,
) -> String {
  format!(
    "`{:}` is not equal to `{:}`.",
    x,
    &rules.step.as_ref().unwrap()
  )
}

#[cfg(test)]
mod test {}
