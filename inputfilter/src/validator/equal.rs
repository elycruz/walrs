use std::fmt::Display;
use crate::ToAttributesList;
use crate::types::ConstraintViolation;
use crate::types::{InputValue, ValidateValue, ValidationResult};

#[derive(Builder, Clone)]
pub struct EqualityValidator<'a, T>
  where T: InputValue + Clone
{
  pub rhs_value: T,
  pub not_equal_msg: &'a (dyn Fn(&EqualityValidator<'a, T>, T) -> String + Send + Sync),
}

impl<'a, T> ValidateValue<T> for EqualityValidator<'a, T>
  where T: InputValue + Clone,
{
  fn validate(&self, x: T) -> ValidationResult {
    if x == self.rhs_value {
      Ok(())
    } else {
      Err(vec![(ConstraintViolation::NotEqual, (self.not_equal_msg)(self, x))])
    }
  }
}

impl<T> Display for EqualityValidator<'_, T>
  where T: InputValue + Clone,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "EqualValidator {{rhs_value: {}}}", &self.rhs_value.to_string())
  }
}

impl<T: InputValue + Clone> ToAttributesList for EqualityValidator<'_, T> {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    Some(vec![(
      "pattern".to_string(),
      serde_json::to_value(self.rhs_value.clone()).unwrap(),
    )])
  }
}

pub fn not_equal_msg<T: InputValue + Clone>(_: &EqualityValidator<T>, value: T) -> String
  where T: InputValue,
{
  format!("Value must equal {}", value)
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_construction() {
    let _ = EqualityValidatorBuilder::<&str>::default()
      .rhs_value("foo".into())
      .not_equal_msg(&not_equal_msg)
      .build();
  }

  fn test_types() {
    let _ = EqualityValidatorBuilder::<&str>::default()
      .rhs_value("foo".into())
      .not_equal_msg(&not_equal_msg)
      .build();


  }
}
