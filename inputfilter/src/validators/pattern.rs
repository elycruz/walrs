use crate::traits::ToAttributesList;
use crate::ViolationType::PatternMismatch;
use crate::{Validate, ValidateRef, ValidatorResult, Violation};
use regex::Regex;
use std::borrow::Cow;
use std::fmt::Display;

pub type PatternViolationCallback = dyn Fn(&PatternValidator, &str) -> String + Send + Sync;

/// A validator for checking that a string matches a specified regex pattern.
///
/// ```rust
///  use walrs_inputfilter::{PatternValidator, PatternValidatorBuilder, Validate, ValidateRef};
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
///
///  // As a function (Fn* trait object).
///  assert_eq!(vldtr("abc"), Ok(()));
///  assert!(vldtr("!@#)(*").is_err());
/// ```
#[derive(Builder, Clone)]
pub struct PatternValidator<'a> {
  pub pattern: Cow<'a, Regex>,

  #[builder(default = "&pattern_vldr_pattern_mismatch_msg")]
  pub pattern_mismatch: &'a PatternViolationCallback,
}

impl<'a> PatternValidator<'a> {
  /// Returns a new instance of `PatternValidator` with passed in Regex value.
  ///
  /// ```rust
  ///  use walrs_inputfilter::validators::{PatternValidator, PatternValidatorBuilder};
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
}

// @todo Should implement default - requires making `pattern` attrib. Maybe (Some|None),
//   also requires 'validate*' methods update to take this into account.

impl Validate<&str> for PatternValidator<'_> {
  /// Validates input string against regex.
  ///
  /// ```rust
  ///  use walrs_inputfilter::validators::{PatternValidator, PatternValidatorBuilder, Validate};
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
  /// Same as `validate` but exists to appease `ValidateRef` trait, which is [currently] required
  /// in some special use cases.
  ///
  /// ```rust
  ///  use walrs_inputfilter::validators::{PatternValidator, PatternValidatorBuilder, ValidateRef};
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
      false => Err(Violation(
        PatternMismatch,
        (self.pattern_mismatch)(self, value),
      )),
      _ => Ok(()),
    }
  }
}

impl ToAttributesList for PatternValidator<'_> {
  /// Returns list of attributes to be used in HTML form input element.
  ///
  /// ```rust
  ///  use walrs_inputfilter::validators::{PatternValidator, PatternValidatorBuilder};
  ///  use walrs_inputfilter::traits::ToAttributesList;
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

impl FnOnce<(&str,)> for PatternValidator<'_> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl FnMut<(&str,)> for PatternValidator<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

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
///  use walrs_inputfilter::{PatternValidatorBuilder, pattern_vldr_pattern_mismatch_msg};
///  use regex::Regex;
///  use std::borrow::Cow;
///
///  let rx = Regex::new(r"^\w{2,55}$").unwrap();
///
///  let vldtr = PatternValidatorBuilder::default()
///    .pattern(Cow::Owned(rx))
///    .build()
///    .unwrap();
///
///  assert_eq!(
///   pattern_vldr_pattern_mismatch_msg(&vldtr, "!@#)(*"),
///   "`!@#)(*` does not match pattern `^\\w{2,55}$`."
///  );
/// ```
///
pub fn pattern_vldr_pattern_mismatch_msg(rules: &PatternValidator, xs: &str) -> String {
  format!(
    "`{}` does not match pattern `{}`.",
    xs,
    &rules.pattern.to_string()
  )
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

    fn on_custom_pattern_mismatch(_: &PatternValidator, _: &str) -> String {
      "custom pattern mismatch err message".into()
    }

    for (name, instance, passing_value, failing_value, _err_callback) in [
      (
        "Default",
        PatternValidatorBuilder::default()
          .pattern(Cow::Owned(_rx.clone()))
          .build()?,
        "abc",
        "!@#)(*",
        &pattern_vldr_pattern_mismatch_msg,
      ),
      (
        "Custom ",
        PatternValidatorBuilder::default()
          .pattern(Cow::Owned(_rx.clone()))
          .pattern_mismatch(&on_custom_pattern_mismatch)
          .build()?,
        "abc",
        "!@#)(*",
        &on_custom_pattern_mismatch,
      ),
    ]
      as [(
        &str,
        PatternValidator,
        &str,
        &str,
        &PatternViolationCallback,
      ); 2]
    {
      println!("{}", name);

      // Test as an `Fn*` trait
      assert_eq!((&instance)(passing_value), Ok(()));
      assert_eq!(
        (&instance)(failing_value),
        Err(Violation(
          PatternMismatch,
          (instance.pattern_mismatch)(&instance, failing_value)
        ))
      );

      // Test `validate` method directly
      assert_eq!(instance.validate(passing_value), Ok(()));
      assert_eq!(instance.validate_ref(passing_value), Ok(()));
      assert_eq!(
        instance.validate(failing_value),
        Err(Violation(
          PatternMismatch,
          (instance.pattern_mismatch)(&instance, failing_value)
        ))
      );
      assert_eq!(
        instance.validate_ref(failing_value),
        Err(Violation(
          PatternMismatch,
          (instance.pattern_mismatch)(&instance, failing_value)
        ))
      );
    }

    Ok(())
  }

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
