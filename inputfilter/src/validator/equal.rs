use std::fmt::Display;
use crate::ToAttributesList;
use crate::traits::ValidateValue;
use crate::types::ViolationEnum;
use crate::types::{InputValue, ValidationResult};

#[derive(Builder, Clone)]
pub struct EqualityValidator<'a, T>
  where T: InputValue + Clone
{
  pub rhs_value: T,

  #[builder(default = "&not_equal_msg")]
  pub not_equal_msg: &'a (dyn Fn(&EqualityValidator<'a, T>, T) -> String + Send + Sync),
}

impl<'a, T> ValidateValue<T> for EqualityValidator<'a, T>
  where T: InputValue + Clone,
{
  fn validate(&self, x: T) -> ValidationResult {
    if x == self.rhs_value {
      Ok(())
    } else {
      Err(vec![(ViolationEnum::NotEqual, (self.not_equal_msg)(self, x))])
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

impl<T: InputValue + Clone> FnOnce<(T, )> for EqualityValidator<'_, T> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (T, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: InputValue + Clone> FnMut<(T, )> for EqualityValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (T, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: InputValue + Clone> Fn<(T, )> for EqualityValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (T, )) -> Self::Output {
    self.validate(args.0)
  }
}

pub fn not_equal_msg<T: InputValue + Clone>(_: &EqualityValidator<T>, value: T) -> String
  where T: InputValue,
{
  format!("Value must equal {}", value)
}

#[cfg(test)]
mod test {
  use std::error::Error;
  use crate::ViolationEnum::NotEqual;
  use super::*;

  #[test]
  fn test_construction() -> Result<(), Box<dyn Error>> {
    let instance = EqualityValidatorBuilder::<&str>::default()
      .rhs_value("foo")
      .build()?;

    assert_eq!(instance.rhs_value, "foo");

    assert_eq!((instance.not_equal_msg)(&instance, "foo"),
               not_equal_msg(&instance, "foo"),
    "Default 'not_equal_msg' fn should return expected value");

    Ok(())
  }

  #[test]
  fn test_validate_and_fn_trait() -> Result<(), Box<dyn Error>> {
    // Test `validate`, and `Fn*` trait
    for (lhs_value, rhs_value, should_be_ok) in [
      ("foo", "foo", true),
      ("", "abc", false),
      ("", "", true),
    ] {
      let validator = EqualityValidatorBuilder::<&str>::default()
        .rhs_value(rhs_value)
        .not_equal_msg(&not_equal_msg)
        .build()?;

      if should_be_ok {
        assert!(validator.validate(lhs_value).is_ok());
        assert!(validator(lhs_value).is_ok());
      } else {
        assert_eq!(
          validator.validate(lhs_value),
          Err(vec![(NotEqual, not_equal_msg(&validator, lhs_value))])
        );
        assert_eq!(
          validator(lhs_value),
          Err(vec![(NotEqual, not_equal_msg(&validator, lhs_value))])
        );
      }
    }

    Ok(())
  }
}
