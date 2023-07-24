use std::borrow::Cow;
use std::ops::Div;

use crate::input::{ConstraintViolation};
use crate::input::ConstraintViolation::{StepMismatch, RangeOverflow, RangeUnderflow};
use crate::types::{InputValue, ValidateValue, ValidationResult};

pub type IntegerViolationCallback<T> = dyn Fn(&IntegerValidator<T>, T) -> String + Send + Sync;

#[derive(Builder, Clone)]
pub struct IntegerValidator<'a, T: InputValue + Copy + Div + 'static> {
  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "None")]
  pub step: Option<T>,

  #[builder(default = "&range_underflow_msg")]
  pub range_underflow: &'a IntegerViolationCallback<T>,

  #[builder(default = "&range_overflow_msg")]
  pub range_overflow: &'a IntegerViolationCallback<T>,

  #[builder(default = "&step_mismatch_msg")]
  pub step_mismatch: &'a IntegerViolationCallback<T>,
}

impl<'a, T> IntegerValidator<'a, T>
  where T: InputValue + Copy + Div
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

    // Test Step
    // if let Some(step) = self.step {
    //   // let quotient = v / step;
    //   if step != Default::default() /*&& quotient != Default::default()*/ {
    //     return Some(StepMismatch);
    //   }
    // }

    None
  }

  fn _get_violation_msg(&self, violation: ConstraintViolation, value: T) -> String {
    let f = match violation {
      RangeUnderflow => Some(&self.range_underflow),
      RangeOverflow => Some(&self.range_overflow),
      StepMismatch => Some(&self.step_mismatch),
      _ => unreachable!("Unsupported Constraint Violation Enum matched"),
    };

    f.map(|_f| (_f)(self, value)).unwrap()
  }

  pub fn new() -> Self {
    IntegerValidator {
      min: None,
      max: None,
      step: None,
      range_underflow: &range_underflow_msg,
      range_overflow: &range_overflow_msg,
      step_mismatch: &step_mismatch_msg,
    }
  }
}

impl<T> ValidateValue<T> for IntegerValidator<'_, T>
  where T: InputValue + Copy + Div {
  fn validate(&self, value: Cow<'_, T>) -> ValidationResult {
    // Perform validation
    if let Some(violation) = self._validate_integer(*value) {
      return Err(vec![(violation, self._get_violation_msg(violation, *value))]);
    }

    Ok(())
  }
}

impl<T> FnOnce<(Cow<'_, T>, )> for IntegerValidator<'_, T>
  where T: InputValue + Copy + Div {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (Cow<'_, T>, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<'a, T> Default for IntegerValidator<'a, T>
  where T: InputValue + Copy + Div {
  fn default() -> Self {
    IntegerValidator::new()
  }
}

pub fn range_underflow_msg<T>(rules: &IntegerValidator<T>, x: T) -> String
  where
    T: InputValue + Copy + Div,
{
  format!(
    "`{:}` is less than minimum `{:}`.",
    x,
    &rules.min.as_ref().unwrap()
  )
}

pub fn range_overflow_msg<T>(rules: &IntegerValidator<T>, x: T) -> String
  where
    T: InputValue + Copy + Div,
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.max.as_ref().unwrap()
  )
}

pub fn step_mismatch_msg<T: InputValue + Copy + Div>(rules: &IntegerValidator<T>, x: T) -> String {
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.step.as_ref().unwrap()
  )
}

#[cfg(test)]
mod test {}
