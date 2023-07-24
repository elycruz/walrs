use std::borrow::Cow;
use crate::input::ConstraintViolation;
use crate::types::{InputValue, ValidateValue, ValidationResult};

#[derive(Builder, Clone)]
pub struct EqualValidator<'a, T>
  where T: InputValue
{
  pub rhs_value: Cow<'a, T>,
  pub not_equal_msg: &'a (dyn Fn(&EqualValidator<'a, T>, Cow<'_, T>) -> String + Send + Sync),
}

impl<'a, T> ValidateValue<T> for EqualValidator<'a, T>
  where T: InputValue,
{
  fn validate(&self, x: Cow<'_, T>) -> ValidationResult {
    if x == self.rhs_value {
      Ok(())
    } else {
      Err(vec![(ConstraintViolation::NotEqual, (self.not_equal_msg)(self, x))])
    }
  }
}

pub fn not_equal_msg<'a, T>(_: &EqualValidator<'a, T>, value: Cow<'a, T>) -> String
  where T: InputValue,
{
  format!("Value must equal {}", &value)
}
