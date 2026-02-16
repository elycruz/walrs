use crate::traits::ToAttributesList;
use crate::violation::ViolationType;
use crate::{
  InputValue, Message, MessageContext, MessageParams, Validate, ValidatorResult, Violation,
};
use std::fmt::Display;

/// Validator for performing equality checks against contained value.
///
/// ```rust
///  use walrs_validator::{
///    EqualityValidator,
///    EqualityValidatorBuilder,
///    Validate,
///    Violation,
///    ViolationType,
///    ValidatorResult,
///  };
///
///  let vldtr = EqualityValidatorBuilder::<&str>::default()
///    .rhs_value("foo")
///    .build()
///    .unwrap();
///
///  // Happy path
///  assert!(vldtr.validate("foo").is_ok());
///
///  // Sad path
///  assert!(vldtr.validate("bar").is_err());
/// ```
///
#[must_use]
#[derive(Builder, Clone)]
pub struct EqualityValidator<T>
where
  T: InputValue,
{
  pub rhs_value: T,

  #[builder(default = "default_not_equal_msg()")]
  pub not_equal_msg: Message<T>,
}

impl<T> EqualityValidator<T>
where
  T: InputValue,
{
  /// Creates new instance of `EqualityValidator` with given rhs value.
  ///
  /// ```rust
  /// use walrs_validator::{
  ///   EqualityValidator,
  ///   EqualityValidatorBuilder,
  /// };
  ///
  /// let vldtr = EqualityValidator::<&str>::new("foo");
  ///
  /// assert_eq!(vldtr.rhs_value, "foo");
  /// ```
  ///
  pub fn new(rhs_value: T) -> Self {
    Self {
      rhs_value,
      not_equal_msg: default_not_equal_msg(),
    }
  }
}

impl<T> Validate<T> for EqualityValidator<T>
where
  T: InputValue,
{
  /// Validates implicitly sized type against contained constraints.
  ///
  /// ```rust
  /// use walrs_validator::{
  ///   EqualityValidator,
  ///   ViolationType,
  ///   EqualityValidatorBuilder,
  ///   Validate,
  ///   ValidatorResult,
  /// };
  ///
  /// let input = EqualityValidatorBuilder::<&str>::default()
  ///   .rhs_value("foo")
  ///   .build()
  ///   .unwrap();
  ///
  /// // Happy path
  /// assert!(input.validate("foo").is_ok());
  ///
  /// // Sad path
  /// assert!(input.validate("abc").is_err());
  /// ```
  ///
  fn validate(&self, x: T) -> ValidatorResult {
    if x == self.rhs_value {
      Ok(())
    } else {
      let params =
        MessageParams::new("EqualityValidator").with_expected(self.rhs_value.to_string());
      let ctx = MessageContext::new(&x, params);
      Err(Violation(
        ViolationType::NotEqual,
        self.not_equal_msg.resolve_with_context(&ctx),
      ))
    }
  }
}

impl<T> Display for EqualityValidator<T>
where
  T: InputValue,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "EqualValidator {{rhs_value: {}}}",
      &self.rhs_value.to_string()
    )
  }
}

impl<T: InputValue> ToAttributesList for EqualityValidator<T> {
  /// Returns list of attributes to be used in HTML form input element.
  ///
  /// ```rust
  ///  use walrs_validator::{EqualityValidator, EqualityValidatorBuilder};
  ///  use walrs_validator::ToAttributesList;
  ///  use std::borrow::Cow;
  ///
  ///  let vldtr = EqualityValidatorBuilder::<&str>::default()
  ///  .rhs_value("foo")
  ///  .build()
  ///  .unwrap();
  ///
  ///  let attrs = vldtr.to_attributes_list().unwrap();
  ///
  ///  assert_eq!(attrs.len(), 1);
  ///  assert_eq!(attrs[0].0, "rhs_value");
  ///  assert_eq!(attrs[0].1, "foo");
  /// ```
  ///
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    Some(vec![(
      "rhs_value".to_string(),
      serde_json::to_value(self.rhs_value).unwrap(),
    )])
  }
}

#[cfg(feature = "fn_traits")]
impl<T: InputValue> FnOnce<(T,)> for EqualityValidator<T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: InputValue> FnMut<(T,)> for EqualityValidator<T> {
  extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: InputValue> Fn<(T,)> for EqualityValidator<T> {
  extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

/// Returns generic not equal message.
///
/// ```rust
///  use walrs_validator::equal_vldr_not_equal_msg;
///
///  assert_eq!(equal_vldr_not_equal_msg("foo"), "Value must equal foo");
/// ```
///
pub fn equal_vldr_not_equal_msg<T: InputValue>(expected: T) -> String {
  format!("Value must equal {}", expected)
}

/// Returns default not equal Message provider for EqualityValidator.
pub fn default_not_equal_msg<T: InputValue>() -> Message<T> {
  Message::provider(|ctx: &MessageContext<T>| {
    let expected = ctx.params.expected.as_deref().unwrap_or("?");
    format!("Value must equal {}", expected)
  })
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

    Ok(())
  }

  #[test]
  fn test_validate() -> Result<(), Box<dyn Error>> {
    let validator = EqualityValidatorBuilder::<&str>::default()
      .rhs_value("foo")
      .build()?;

    // Happy path
    assert!(validator.validate("foo").is_ok());

    // Sad path
    let result = validator.validate("bar");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, NotEqual);
    assert_eq!(err.1, equal_vldr_not_equal_msg("foo"));

    Ok(())
  }

  #[test]
  fn test_custom_message() -> Result<(), Box<dyn Error>> {
    let custom_msg: Message<&str> = Message::static_msg("Values don't match!");
    let validator = EqualityValidatorBuilder::<&str>::default()
      .rhs_value("foo")
      .not_equal_msg(custom_msg)
      .build()?;

    let result = validator.validate("bar");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.1, "Values don't match!");

    Ok(())
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_traits() -> Result<(), Box<dyn Error>> {
    let validator = EqualityValidatorBuilder::<&str>::default()
      .rhs_value("foo")
      .build()?;

    assert!(validator("foo").is_ok());
    assert!(validator("bar").is_err());

    Ok(())
  }
}
