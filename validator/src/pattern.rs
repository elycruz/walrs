use crate::ViolationType::PatternMismatch;
use crate::traits::ToAttributesList;
use crate::{
  Message, MessageContext, MessageParams, Validate, ValidateRef, ValidatorResult, Violation,
};
use regex::Regex;
use std::borrow::Cow;
use std::fmt::Display;

/// A validator for checking that a string matches a specified regex pattern.
///
/// ```rust
///  use walrs_validator::{PatternValidator, PatternValidatorBuilder, Validate, ValidateRef};
///  use regex::Regex;
///  use std::borrow::Cow;
///  let rx = Regex::new(r"^\w{2,55}$").unwrap();
///  let vldtr = PatternValidatorBuilder::default()
///    .pattern(Cow::Owned(rx))
///    .build()
///    .unwrap();
///
///  assert_eq!(vldtr.pattern.as_str(), r"^\w{2,55}$");
///  assert_eq!(vldtr.validate_ref("abc"), Ok(()));
///  assert!(vldtr.validate_ref("!@#)(*").is_err());
/// ```
#[must_use]
#[derive(Builder, Clone)]
pub struct PatternValidator<'a> {
  pub pattern: Cow<'a, Regex>,

  #[builder(default = "default_pattern_mismatch_msg()")]
  pub pattern_mismatch: Message<str>,
}

impl<'a> PatternValidator<'a> {
  /// Returns a new instance of `PatternValidator` with passed in Regex value.
  ///
  /// ```rust
  ///  use walrs_validator::{PatternValidator, PatternValidatorBuilder};
  ///  use regex::Regex;
  ///  use std::borrow::Cow;
  ///
  ///  let rx = Regex::new(r"^\w{2,55}$").unwrap();
  ///  let vldtr = PatternValidator::new(Cow::Owned(rx));
  ///
  ///  assert_eq!(vldtr.pattern.as_str(), r"^\w{2,55}$");
  /// ```
  ///
  pub fn new(pattern: Cow<'a, Regex>) -> Self {
    PatternValidatorBuilder::default()
      .pattern(pattern)
      .build()
      .unwrap()
  }

  /// Returns a builder for constructing a `PatternValidator`.
  ///
  /// ```rust
  ///  use walrs_validator::PatternValidator;
  ///  use regex::Regex;
  ///  use std::borrow::Cow;
  ///
  ///  let rx = Regex::new(r"^\w{2,55}$").unwrap();
  ///  let vldtr = PatternValidator::builder()
  ///    .pattern(Cow::Owned(rx))
  ///    .build()
  ///    .unwrap();
  ///
  ///  assert_eq!(vldtr.pattern.as_str(), r"^\w{2,55}$");
  /// ```
  pub fn builder() -> PatternValidatorBuilder<'a> {
    PatternValidatorBuilder::default()
  }
}

impl Validate<&str> for PatternValidator<'_> {
  /// Validates input string against regex.
  ///
  /// ```rust
  ///  use walrs_validator::{PatternValidator, PatternValidatorBuilder, Validate};
  ///  use regex::Regex;
  ///  use std::borrow::Cow;
  ///
  ///  let rx = Regex::new(r"^\w{2,55}$").unwrap();
  ///  let vldtr = PatternValidatorBuilder::default()
  ///    .pattern(Cow::Owned(rx))
  ///    .build()
  ///    .unwrap();
  ///
  ///  assert_eq!(vldtr.validate("abc"), Ok(()));
  ///  assert!(vldtr.validate("!@#)(*").is_err());
  /// ```
  ///
  fn validate(&self, value: &str) -> ValidatorResult {
    self.validate_ref(value)
  }
}

impl ValidateRef<str> for PatternValidator<'_> {
  /// Same as `validate` but exists to appease `ValidateRef` trait.
  ///
  /// ```rust
  ///  use walrs_validator::{PatternValidator, PatternValidatorBuilder, ValidateRef};
  ///  use regex::Regex;
  ///  use std::borrow::Cow;
  ///
  ///  let rx = Regex::new(r"^\w{2,55}$").unwrap();
  ///  let vldtr = PatternValidatorBuilder::default()
  ///    .pattern(Cow::Owned(rx))
  ///    .build()
  ///    .unwrap();
  ///
  ///  assert_eq!(vldtr.validate_ref("abc"), Ok(()));
  ///  assert!(vldtr.validate_ref("!@#)(*").is_err());
  /// ```
  ///
  fn validate_ref(&self, value: &str) -> ValidatorResult {
    match self.pattern.is_match(value) {
      false => {
        let params = MessageParams::new("PatternValidator").with_pattern(self.pattern.as_str());
        let ctx = MessageContext::new(value, params);
        Err(Violation(
          PatternMismatch,
          self.pattern_mismatch.resolve_with_context(&ctx),
        ))
      }
      _ => Ok(()),
    }
  }
}

impl ToAttributesList for PatternValidator<'_> {
  /// Returns list of attributes to be used in HTML form input element.
  ///
  /// ```rust
  ///  use walrs_validator::{PatternValidator, PatternValidatorBuilder};
  ///  use walrs_validator::ToAttributesList;
  ///  use regex::Regex;
  ///  use std::borrow::Cow;
  ///
  ///  let rx = Regex::new(r"^\w{2,55}$").unwrap();
  ///  let vldtr = PatternValidatorBuilder::default()
  ///  .pattern(Cow::Owned(rx))
  ///  .build()
  ///  .unwrap();
  ///
  ///  let attrs = vldtr.to_attributes_list().unwrap();
  ///
  ///  assert_eq!(attrs.len(), 1);
  ///  assert_eq!(attrs[0].0, "pattern");
  ///  assert_eq!(attrs[0].1, r"^\w{2,55}$");
  /// ```
  ///
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    Some(vec![("pattern".into(), self.pattern.to_string().into())])
  }
}

#[cfg(feature = "fn_traits")]
impl FnOnce<(&str,)> for PatternValidator<'_> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl FnMut<(&str,)> for PatternValidator<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl Fn<(&str,)> for PatternValidator<'_> {
  extern "rust-call" fn call(&self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl Display for PatternValidator<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "PatternValidator {{pattern: {}}}",
      &self.pattern.to_string()
    )
  }
}

/// Returns generic pattern mismatch message.
///
/// ```rust
///  use walrs_validator::pattern_vldr_pattern_mismatch_msg;
///
///  assert_eq!(
///   pattern_vldr_pattern_mismatch_msg("!@#)(*", r"^\w{2,55}$"),
///   "`!@#)(*` does not match pattern `^\\w{2,55}$`."
///  );
/// ```
///
pub fn pattern_vldr_pattern_mismatch_msg(value: &str, pattern: &str) -> String {
  format!("`{}` does not match pattern `{}`.", value, pattern)
}

/// Returns default pattern mismatch Message provider.
///
/// This wraps `pattern_vldr_pattern_mismatch_msg` in a `Message::Provider` for use with `PatternValidator`.
pub fn default_pattern_mismatch_msg() -> Message<str> {
  Message::provider(|ctx: &MessageContext<str>| {
    let pattern = ctx.params.pattern.as_deref().unwrap_or("?");
    pattern_vldr_pattern_mismatch_msg(ctx.value, pattern)
  })
}

#[cfg(test)]
mod test {
  use std::error::Error;

  use super::*;

  #[test]
  fn test_construction_and_validation() -> Result<(), Box<dyn Error>> {
    let _rx = Regex::new(r"^\w{2,55}$")?;

    let standalone_instance = PatternValidator::new(Cow::Owned(_rx.clone()));
    assert_eq!(standalone_instance.pattern.as_str(), r"^\w{2,55}$");

    let default_vldtr = PatternValidatorBuilder::default()
      .pattern(Cow::Owned(_rx.clone()))
      .build()?;

    // Test passing value
    assert_eq!(default_vldtr.validate("abc"), Ok(()));
    assert_eq!(default_vldtr.validate_ref("abc"), Ok(()));

    // Test failing value
    let result = default_vldtr.validate("!@#)(*");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, PatternMismatch);
    assert_eq!(
      err.1,
      pattern_vldr_pattern_mismatch_msg("!@#)(*", r"^\w{2,55}$")
    );

    #[cfg(feature = "fn_traits")]
    {
      assert_eq!((&default_vldtr)("abc"), Ok(()));
      assert!((&default_vldtr)("!@#)(*").is_err());
    }

    Ok(())
  }

  #[test]
  fn test_custom_message() -> Result<(), Box<dyn Error>> {
    let rx = Regex::new(r"^\w{2,55}$")?;
    let custom_msg: Message<str> = Message::static_msg("Invalid format!");
    let vldtr = PatternValidatorBuilder::default()
      .pattern(Cow::Owned(rx))
      .pattern_mismatch(custom_msg)
      .build()?;

    let result = vldtr.validate("!@#)(*");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.1, "Invalid format!");

    Ok(())
  }

  #[test]
  fn test_message_provider() -> Result<(), Box<dyn Error>> {
    let rx = Regex::new(r"^\d+$")?;
    let custom_msg: Message<str> = Message::provider(|ctx: &MessageContext<str>| {
      format!(
        "'{}' must match pattern '{}'",
        ctx.value,
        ctx.params.pattern.as_deref().unwrap_or("?")
      )
    });
    let vldtr = PatternValidatorBuilder::default()
      .pattern(Cow::Owned(rx))
      .pattern_mismatch(custom_msg)
      .build()?;

    let result = vldtr.validate("abc");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.1, "'abc' must match pattern '^\\d+$'");

    Ok(())
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_trait_variations() -> Result<(), Box<dyn Error>> {
    let rx = Regex::new(r"^\w{2,55}$")?;
    let vldtr = PatternValidatorBuilder::default()
      .pattern(Cow::Owned(rx))
      .build()?;

    // As a function (Fn* trait object).
    assert_eq!(vldtr("abc"), Ok(()));
    assert!(vldtr("!@#)(*").is_err());

    let vldtr_clone = vldtr.clone();
    fn call_fn_once(v: impl FnOnce(&str) -> ValidatorResult, s: &str) -> ValidatorResult {
      v(s)
    }
    assert_eq!(call_fn_once(vldtr_clone, "abc"), Ok(()));

    let mut vldtr_mut = vldtr.clone();
    fn call_fn_mut(v: &mut impl FnMut(&str) -> ValidatorResult, s: &str) -> ValidatorResult {
      v(s)
    }
    assert_eq!(call_fn_mut(&mut vldtr_mut, "abc"), Ok(()));

    Ok(())
  }

  #[test]
  fn test_to_attributes_list() -> Result<(), Box<dyn Error>> {
    let rx = Regex::new(r"^\w{2,55}$")?;
    let vldtr = PatternValidatorBuilder::default()
      .pattern(Cow::Owned(rx))
      .build()?;

    let attrs = vldtr.to_attributes_list().unwrap();

    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "pattern");
    assert_eq!(attrs[0].1, r"^\w{2,55}$");

    Ok(())
  }

  #[test]
  fn test_display() -> Result<(), Box<dyn Error>> {
    let rx = Regex::new(r"^\w{2,55}$")?;
    let vldtr = PatternValidatorBuilder::default()
      .pattern(Cow::Owned(rx))
      .build()?;

    let disp = format!("{}", vldtr);
    assert_eq!(disp, "PatternValidator {pattern: ^\\w{2,55}$}");

    Ok(())
  }
}
