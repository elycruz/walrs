use std::fmt::{Display, Formatter};
use crate::ToAttributesList;
use crate::types::{
  ConstraintViolation,
  ConstraintViolation::{
    NotEqual, RangeOverflow, RangeUnderflow, StepMismatch,
  }, NumberValue, ValidateValue, ValidationResult,
};

use serde_json::value::to_value as to_json_value;

pub type NumberVldrViolationCallback<'a, T> =
  (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync);

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct NumberValidator<'a, T: NumberValue> {
  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "None")]
  pub step: Option<T>,

  #[builder(default = "None")]
  pub equal: Option<T>,

  #[builder(default = "&range_underflow_msg")]
  pub range_underflow: &'a (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&range_overflow_msg")]
  pub range_overflow: &'a (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&step_mismatch_msg")]
  pub step_mismatch: &'a (dyn Fn(&NumberValidator<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&not_equal_msg")]
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
    if let Some(rhs) = self.equal {
      if v != rhs {
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
      equal: None,
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
  fn validate(&self, value: T) -> ValidationResult {
    if let Some(violation) = self._validate_integer(value) {
      return Err(vec![(
        violation,
        self._get_violation_msg(violation, value),
      )]);
    }

    Ok(())
  }
}

impl<T> ToAttributesList for NumberValidator<'_, T>
  where
    T: NumberValue,
{
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

    if let Some(equal) = self.equal {
      attrs.push(("pattern".to_string(), to_json_value(equal).unwrap()));
    }

    if attrs.is_empty() {
      None
    } else {
      Some(attrs)
    }
  }
}

impl<T: NumberValue> FnMut<(T, )> for NumberValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (T, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: NumberValue> Fn<(T, )> for NumberValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (T, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: NumberValue> FnOnce<(T,)> for NumberValidator<'_, T> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: NumberValue> FnMut<(&T, )> for NumberValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (&T, )) -> Self::Output {
    self.validate(*args.0)
  }
}

impl<T: NumberValue> Fn<(&T, )> for NumberValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (&T, )) -> Self::Output {
    self.validate(*args.0)
  }
}

impl<T: NumberValue> FnOnce<(&T,)> for NumberValidator<'_, T> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (&T,)) -> Self::Output {
    self.validate(*args.0)
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

impl<T: NumberValue> Display for NumberValidator<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "NumberValidator {{min: {}, max: {}, step: {}, equal: {}}}",
           &self.min.map(|x| x.to_string()).unwrap_or("None".to_string()),
           &self.max.map(|x| x.to_string()).unwrap_or("None".to_string()),
           &self.step.map(|x| x.to_string()).unwrap_or("None".to_string()),
           &self.equal.map(|x| x.to_string()).unwrap_or("None".to_string()),
    )
  }
}

pub fn range_underflow_msg<T>(rules: &NumberValidator<T>, x: T) -> String
where
  T: NumberValue,
{
  format!(
    "`{:}` is less than minimum `{:}`.",
    x,
    &rules.min.as_ref().unwrap_or(&T::default())
  )
}

pub fn range_overflow_msg<T>(rules: &NumberValidator<T>, x: T) -> String
where
  T: NumberValue,
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.max.as_ref().unwrap_or(&T::default())
  )
}

pub fn step_mismatch_msg<T: NumberValue>(
  rules: &NumberValidator<T>,
  x: T,
) -> String {
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.step.as_ref().unwrap_or(&T::default())
  )
}

pub fn not_equal_msg<T: NumberValue>(
  rules: &NumberValidator<T>,
  x: T,
) -> String {
  format!(
    "`{:}` is not equal to `{:}`.",
    x,
    &rules.equal.as_ref().unwrap_or(&T::default())
  )
}

#[cfg(test)]
mod test {

  use std::error::Error;
  use crate::ConstraintViolation::NotEqual;
  use super::*;

  #[test]
  fn test_construction() -> Result<(), Box<dyn Error>> {
    // Assert all property states for difference construction scenarios
    // ----
    for (testName, instance, min, max, step, equal) in [
      ("Default", NumberValidatorBuilder::<usize>::default()
          .build()?, None, None, None, None),
      ("With Range", NumberValidatorBuilder::<usize>::default()
           .min(0)
           .max(100)
           .build()?, Some(0), Some(100), None, None),
      ("With `equal`", NumberValidatorBuilder::<usize>::default()
           .equal(101)
           .build()?, None, None, None, Some(101)),
      ("With `step`", NumberValidatorBuilder::<usize>::default()
           .step(5)
           .build()?, None, None, Some(5), None),
    ] {
      println!("\"{}\" test {:}", testName, &instance);

      assert_eq!(instance.min, min);
      assert_eq!(instance.max, max);
      assert_eq!(instance.step, step);
      assert_eq!(instance.equal, equal);

      // Ensure default message callbacks are set
      // ----
      let test_value = 99;

      assert_eq!((instance.range_overflow)(&instance, test_value),
                 range_overflow_msg(&instance, test_value));

      assert_eq!((instance.range_underflow)(&instance, test_value),
                 range_underflow_msg(&instance, test_value));

      assert_eq!((instance.step_mismatch)(&instance, test_value),
                 step_mismatch_msg(&instance, test_value));

      assert_eq!((instance.not_equal)(&instance, test_value),
                 not_equal_msg(&instance, test_value));
    }

    Ok(())
  }

  #[test]
  fn test_validate_and_fn_trait_and_default_messengers() -> Result<(), Box<dyn Error>> {
    // Test `validate`, and `Fn*` trait
    // ----
    for (validator, value, expected) in [
      (NumberValidatorBuilder::<usize>::default().build()?, 99usize, Ok(())),
      (NumberValidatorBuilder::<usize>::default()
           .min(0)
           .build()?,
       99,
       Ok(())),
      (NumberValidatorBuilder::<usize>::default()
           .max(100)
           .build()?,
       99,
       Ok(())),
      (NumberValidatorBuilder::<usize>::default()
           .min(0)
           .max(100)
           .build()?,
       99,
       Ok(())),
      (NumberValidatorBuilder::<usize>::default()
           .step(5)
           .build()?,
       25,
       Ok(())),
      (NumberValidatorBuilder::<usize>::default()
           .equal(99)
           .build()?,
       99,
       Ok(())),
      (NumberValidatorBuilder::<usize>::default()
           .min(2)
           .build()?,
       1,
       Err(RangeUnderflow)),
      (NumberValidatorBuilder::<usize>::default()
           .min(1)
           .build()?,
       0,
       Err(RangeUnderflow)),
      (NumberValidatorBuilder::<usize>::default()
           .max(100)
           .build()?,
       101,
       Err(RangeOverflow)),
      (NumberValidatorBuilder::<usize>::default()
           .min(1)
           .max(100)
           .build()?,
       0,
       Err(RangeUnderflow)),
      (NumberValidatorBuilder::<usize>::default()
           .step(5)
           .build()?,
       26,
       Err(StepMismatch)),
      (NumberValidatorBuilder::<usize>::default()
           .equal(99)
           .build()?,
       101,
       Err(NotEqual)),
    ] {
      match expected {
        Ok(_) => {
          assert_eq!(validator.validate(value), Ok(()));
          assert_eq!((&validator)(value), Ok(()));
        },
        Err(_enum) => {
          let err_msg_tuple = match _enum {
            StepMismatch => (StepMismatch, step_mismatch_msg(&validator, value)),
            NotEqual => (NotEqual, not_equal_msg(&validator, value)),
            RangeUnderflow => (RangeUnderflow, range_underflow_msg(&validator, value)),
            RangeOverflow => (RangeOverflow, range_overflow_msg(&validator, value)),
            _ => panic!("Unknown enum variant encountered")
          };

          assert_eq!(
            validator.validate(value),
            Err(vec![err_msg_tuple.clone()])
          );
          assert_eq!(
            (&validator)(value),
            Err(vec![err_msg_tuple])
          );
        }
      }
    }

    Ok(())
  }
}
