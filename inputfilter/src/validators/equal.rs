use crate::violation::ViolationType;
use crate::traits::{InputValue, ValidationResult};
use crate::ToAttributesList;
use crate::ValidateValue;
use std::fmt::Display;

#[derive(Builder, Clone)]
pub struct EqualityValidator<'a, T>
where
  T: InputValue + Display,
{
  pub rhs_value: T,

  #[builder(default = "&equal_vldr_not_equal_msg")]
  pub not_equal_msg: &'a (dyn Fn(&EqualityValidator<'a, T>, T) -> String + Send + Sync),
}

impl<T> ValidateValue<T> for EqualityValidator<'_, T>
where
  T: InputValue + Display,
{
  /// Validates implicitly sized type against contained constraints, and returns a result
  ///  of unit, and/or, a Vec of violation tuples.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   EqualityValidator,
  ///   ViolationType,
  ///   EqualityValidatorBuilder,
  ///   equal_vldr_not_equal_msg,
  ///   ValidateValue,
  ///   InputValue
  /// };
  ///
  /// let input = EqualityValidatorBuilder::<&str>::default()
  ///   .rhs_value("foo")
  ///   .not_equal_msg(&equal_vldr_not_equal_msg)
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test `validate`, and `Fn*` trait
  /// // ----
  /// // Happy path
  /// assert!(input.validate("foo").is_ok());
  /// assert!(input("foo").is_ok());
  ///
  /// // Sad path
  /// assert_eq!(
  ///   input.validate("abc"),
  ///   Err(vec![(ViolationType::NotEqual, "Value must equal abc".to_string())])
  /// );
  /// assert_eq!(
  ///   input("abc"),
  ///   Err(vec![(ViolationType::NotEqual, "Value must equal abc".to_string())])
  /// );
  /// ```
  fn validate(&self, x: T) -> ValidationResult {
    if x == self.rhs_value {
      Ok(())
    } else {
      Err(vec![(
        ViolationType::NotEqual,
        (self.not_equal_msg)(self, x),
      )])
    }
  }
}

impl<T> Display for EqualityValidator<'_, T>
where
  T: InputValue + Display,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "EqualValidator {{rhs_value: {}}}",
      &self.rhs_value.to_string()
    )
  }
}

impl<T: InputValue + Display> ToAttributesList for EqualityValidator<'_, T> {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    Some(vec![(
      "pattern".to_string(),
      serde_json::to_value(self.rhs_value).unwrap(),
    )])
  }
}

impl<T: InputValue + Display> FnOnce<(T,)> for EqualityValidator<'_, T> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: InputValue + Display> FnMut<(T,)> for EqualityValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: InputValue + Display> Fn<(T,)> for EqualityValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

pub fn equal_vldr_not_equal_msg<T: InputValue + Display>(
  _: &EqualityValidator<T>,
  value: T,
) -> String {
  format!("Value must equal {}", value)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::ViolationType::NotEqual;
  use std::error::Error;

  #[test]
  fn test_construction() -> Result<(), Box<dyn Error>> {
    let instance = EqualityValidatorBuilder::<&str>::default()
      .rhs_value("foo")
      .build()?;

    assert_eq!(instance.rhs_value, "foo");

    assert_eq!(
      (instance.not_equal_msg)(&instance, "foo"),
      equal_vldr_not_equal_msg(&instance, "foo"),
      "Default 'equal_vldr_not_equal_msg' fn should return expected value"
    );

    Ok(())
  }

  #[test]
  fn test_validate_and_fn_trait() -> Result<(), Box<dyn Error>> {
    // Test `validate`, and `Fn*` trait
    for (lhs_value, rhs_value, should_be_ok) in
      [("foo", "foo", true), ("", "abc", false), ("", "", true)]
    {
      let validator = EqualityValidatorBuilder::<&str>::default()
        .rhs_value(rhs_value)
        .not_equal_msg(&equal_vldr_not_equal_msg)
        .build()?;

      if should_be_ok {
        assert!(validator.validate(lhs_value).is_ok());
        assert!(validator(lhs_value).is_ok());
      } else {
        assert_eq!(
          validator.validate(lhs_value),
          Err(vec![(
            NotEqual,
            equal_vldr_not_equal_msg(&validator, lhs_value)
          )])
        );
        assert_eq!(
          validator(lhs_value),
          Err(vec![(
            NotEqual,
            equal_vldr_not_equal_msg(&validator, lhs_value)
          )])
        );
      }
    }

    Ok(())
  }
}
