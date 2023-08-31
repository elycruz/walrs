use std::borrow::Cow;
use std::fmt::Display;
use crate::ToAttributesList;
use crate::types::ConstraintViolation;
use crate::types::{InputValue, ValidateValue, ValidationResult};

#[derive(Builder, Clone)]
pub struct EqualityValidator<'a, T>
  where T: InputValue
{
  pub rhs_value: Cow<'a, T>,
  pub not_equal_msg: &'a (dyn Fn(&EqualityValidator<'a, T>, &T) -> String + Send + Sync),
}

impl<'a, T> ValidateValue<T> for EqualityValidator<'a, T>
  where T: InputValue,
{
  fn validate(&self, x: &T) -> ValidationResult {
    if x == &*self.rhs_value {
      Ok(())
    } else {
      Err(vec![(ConstraintViolation::NotEqual, (self.not_equal_msg)(self, x))])
    }
  }
}

impl<T> Display for EqualityValidator<'_, T>
  where T: InputValue,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "EqualValidator {{rhs_value: {}}}", &self.rhs_value.to_string())
  }
}

impl<T: InputValue> ToAttributesList for EqualityValidator<'_, T> {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    Some(vec![(
      "pattern".to_string(),
      serde_json::to_value(self.rhs_value.to_owned()).unwrap(),
    )])
  }
}

pub fn not_equal_msg<T>(_: &EqualityValidator<T>, value: &T) -> String
  where T: InputValue,
{
  format!("Value must equal {}", value)
}
