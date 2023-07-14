use std::fmt::Display;
use std::sync::Arc;

use crate::input::{ConstraintViolation, ValidationError, ValidationResult};
use crate::input::ConstraintViolation::{StepMismatch, RangeOverflow, RangeUnderflow, ValueMissing};

pub type NumberViolationCallback<T: Display + PartialEq + PartialOrd + Copy> =
  dyn Fn(&NumberValidator<T>, Option<T>) -> String + Send + Sync;

#[derive(Clone)]
pub struct NumberValidator<'a, T: Display + Copy + PartialOrd + PartialEq> {
  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "None")]
  pub step: Option<T>,

  #[builder(default = "Arc::new(&range_underflow_msg)")]
  pub range_underflow: Arc<&'a NumberViolationCallback<T>>,

  #[builder(default = "Arc::new(&range_overflow_msg)")]
  pub range_overflow: Arc<&'a NumberViolationCallback<T>>,

  #[builder(default = "Arc::new(&step_mismatch_msg)")]
  pub step_mismatch: Arc<&'a NumberViolationCallback<T>>,
}

impl<'a, T> NumberValidator<'a, T>
  where T: Display + Copy + PartialOrd + PartialEq
{
  fn _validate_number(&self, v: T) -> Option<ConstraintViolation> {
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
      if step != Default::default() && v / step != Default::default() {
        return Some(StepMismatch);
      }
    }

    None
  }

  fn _get_violation_msg(&self, violation: ConstraintViolation, value: Option<T>) -> String {
    let f = match violation {
      RangeUnderflow => Some(&self.range_underflow),
      RangeOverflow => Some(&self.range_overflow),
      StepMismatch => Some(&self.step_mismatch),
      ValueMissing => Some(&self.value_missing),
      _ => unreachable!("Unsupported Constraint Violation Enum matched"),
    };

    f.map(|_f| {
      let _fn: Arc<&NumberViolationCallback<T>> = Arc::clone(_f);
      (_fn)(self, value)
    }).unwrap()
  }

  pub fn validate(&self, value: T) -> Result<(), ValidationError> {
    // Perform validation
    if let Some(violation) = self._validate_number(value) {
      return Err((violation, self._get_violation_msg(violation, value)));
    }

    Ok(())
  }
}

pub fn range_underflow_msg<T>(rules: &NumberValidator<T>, x: T) -> String
  where
    T: Display + Copy + PartialOrd + PartialEq,
{
  format!(
    "`{:}` is less than minimum `{:}`.",
    x,
    &rules.min.as_ref().unwrap()
  )
}

pub fn range_overflow_msg<T>(rules: &NumberValidator<T>, x: T) -> String
  where
    T: Display + Copy + PartialOrd + PartialEq,
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.max.as_ref().unwrap()
  )
}

pub fn step_mismatch_msg<T>(rules: &NumberValidator<T>, x: T) -> String
  where
    T: Display + Copy + PartialOrd + PartialEq,
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.step.as_ref().unwrap()
  )
}
