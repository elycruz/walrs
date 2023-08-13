use std::borrow::Cow;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::sync::Arc;

use crate::types::{ConstraintViolation, Filter, InputValue, ValidationError, ValidationResult, Validator, ViolationMessage};

pub trait NumberValue: InputValue + Copy + Add + Mul + Sub + Div + Rem<Output = Self> {}

pub type ValueMissingViolationCallback<'a, T> =
  dyn Fn(&NumberInput<'a, T>) -> ViolationMessage + Send + Sync;

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct NumberInput<'a, T>
where
  T: NumberValue,
{
  #[builder(default = "true")]
  pub break_on_failure: bool,

  #[builder(default = "None")]
  pub name: Option<Cow<'a, str>>,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(setter(strip_option), default = "None")]
  pub validators: Option<Vec<Arc<&'a Validator<T>>>>,

  #[builder(setter(strip_option), default = "None")]
  pub filters: Option<Vec<&'a Filter<T>>>,

  #[builder(default = "&value_missing_msg")]
  pub value_missing: &'a (dyn Fn(&NumberInput<'a, T>) -> ViolationMessage + Send + Sync),
  // @todo Add support for `io_validators` (e.g., validators that return futures).
}

impl<'a, T> NumberInput<'a, T>
where
  T: NumberValue,
{
  pub fn new() -> Self {
    NumberInput {
      break_on_failure: false,
      name: None,
      required: false,
      validators: None,
      filters: None,
      value_missing: &value_missing_msg,
    }
  }

  fn _validate_against_validators(&self, value: &T) -> Option<Vec<ValidationError>> {
    self.validators.as_deref().map(|vs| {
      vs.iter().fold(
        Vec::<ValidationError>::new(),
        |mut agg, f| match (Arc::clone(f))(Cow::Borrowed(value)) {
          Err(mut message_tuples) => {
            agg.append(message_tuples.as_mut());
            agg
          }
          _ => agg,
        },
      )
    })
  }

  fn _option_rslt_to_rslt(&self, rslt: Option<Vec<ValidationError>>) -> ValidationResult {
    match rslt {
      None => Ok(()),
      Some(_msgs) => {
        if !_msgs.is_empty() {
          Err(_msgs)
        } else {
          Ok(())
        }
      }
    }
  }

  pub fn filter(&self, value: Option<T>) -> Option<T> {
    self
      .filters
      .as_deref()
      .and_then(|fs| fs.iter().fold(value, |agg, f| (f)(agg)))
  }

  pub fn validate(&self, value: Option<Cow<T>>) -> ValidationResult {
    match &value {
      None => {
        if self.required {
          Err(vec![(
            ConstraintViolation::ValueMissing,
            (self.value_missing)(self),
          )])
        } else {
          Ok(())
        }
      }
      Some(v) => self._option_rslt_to_rslt(self._validate_against_validators(v)),
    }
  }
}

impl<T: NumberValue> Default for NumberInput<'_, T> {
  fn default() -> Self {
    Self::new()
  }
}

pub fn value_missing_msg<T: NumberValue>(_: &NumberInput<T>) -> String {
  "Value is missing.".to_string()
}

#[cfg(test)]
mod test {
  use std::{borrow::Cow, error::Error, sync::Arc, thread};

  use super::ValidationResult;
  use crate::types::ConstraintViolation::{RangeOverflow};

  // Tests setup types
  fn unsized_less_than_100_msg(value: usize) -> String {
    format!("{} is greater than 100", value)
  }

  fn ymd_mismatch_msg(s: &str, pattern_str: &str) -> String {
    format!("{} doesn't match pattern {}", s, pattern_str)
  }

  fn unsized_less_100(x: Cow<usize>) -> ValidationResult {
    if *x >= 100 {
      return Err(vec![(
        RangeOverflow,
        match x {
          Cow::Owned(v) => unsized_less_than_100_msg(v),
          Cow::Borrowed(v) => unsized_less_than_100_msg(v.clone()),
        },
      )]);
    }
    Ok(())
  }
}
