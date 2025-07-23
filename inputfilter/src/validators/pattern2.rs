use crate::{ToAttributesList, ValidateRef, ValidatorResult, Violation};
use regex::Regex;
use std::borrow::Cow;
use std::fmt::Display;

use crate::ViolationType::PatternMismatch;

pub type PatternViolationCallback2 = dyn Fn(&PatternValidator2, &str) -> String + Send + Sync;

#[derive(Builder, Clone)]
pub struct PatternValidator2<'a> {
  pub pattern: Cow<'a, Regex>,

  #[builder(default = "&pattern2_vldr_pattern_mismatch_msg")]
  pub pattern_mismatch: &'a PatternViolationCallback2,
}

impl PatternValidator2<'_> {
  pub fn new() -> Self {
    PatternValidator2Builder::default().build().unwrap()
  }
}

impl Default for PatternValidator2<'_> {
  fn default() -> Self {
    PatternValidator2Builder::default().build().unwrap()
  }
}

impl ValidateRef<str> for PatternValidator2<'_> {
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

impl ToAttributesList for PatternValidator2<'_> {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    Some(vec![("pattern".into(), self.pattern.to_string().into())])
  }
}

impl FnOnce<(&str,)> for PatternValidator2<'_> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl FnMut<(&str,)> for PatternValidator2<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl Fn<(&str,)> for PatternValidator2<'_> {
  extern "rust-call" fn call(&self, args: (&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

// @todo `Fn` traits implementation for `&&str` is not required.
impl FnOnce<(&&str,)> for PatternValidator2<'_> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl FnMut<(&&str,)> for PatternValidator2<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl Fn<(&&str,)> for PatternValidator2<'_> {
  extern "rust-call" fn call(&self, args: (&&str,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

// @todo `Fn` traits implementation for `&String` is not required.
impl FnOnce<(&String,)> for PatternValidator2<'_> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&String,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl FnMut<(&String,)> for PatternValidator2<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&String,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl Fn<(&String,)> for PatternValidator2<'_> {
  extern "rust-call" fn call(&self, args: (&String,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl Display for PatternValidator2<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "PatternValidator2 {{pattern: {}}}",
      &self.pattern.to_string()
    )
  }
}

pub fn pattern2_vldr_pattern_mismatch_msg(rules: &PatternValidator2, xs: &str) -> String {
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

    fn on_custom_pattern_mismatch(_: &PatternValidator2, _: &str) -> String {
      "custom pattern mismatch err message".into()
    }

    for (name, instance, passing_value, failing_value, _err_callback) in [
      (
        "Default",
        PatternValidator2Builder::default()
          .pattern(Cow::Owned(_rx.clone()))
          .build()?,
        "abc",
        "!@#)(*",
        &pattern2_vldr_pattern_mismatch_msg,
      ),
      (
        "Custom ",
        PatternValidator2Builder::default()
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
        PatternValidator2,
        &str,
        &str,
        &PatternViolationCallback2,
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
      assert_eq!(instance.validate_ref(passing_value), Ok(()));
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
}
