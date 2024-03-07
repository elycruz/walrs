use std::borrow::Cow;
use std::fmt::Display;
use regex::Regex;
use crate::{ToAttributesList, ValidateValue};

use crate::ViolationEnum::PatternMismatch;
use crate::traits::{ValidationResult};

pub type PatternViolationCallback = dyn Fn(&PatternValidator, &str) -> String + Send + Sync;

#[derive(Builder, Clone)]
pub struct PatternValidator<'a> {
  pub pattern: Cow<'a, Regex>,

  #[builder(default = "&pattern_vldr_pattern_mismatch_msg")]
  pub pattern_mismatch: &'a PatternViolationCallback,
}

impl PatternValidator<'_> {
  pub fn new() -> Self {
    PatternValidatorBuilder::default().build().unwrap()
  }
}

impl Default for PatternValidator<'_> {
  fn default() -> Self {
    PatternValidatorBuilder::default().build().unwrap()
  }
}

impl ValidateValue<&str> for PatternValidator<'_>
where {
  fn validate(&self, value: &str) -> ValidationResult {
    match self.pattern.is_match(value) {
      false => Err(vec![(PatternMismatch, (self.pattern_mismatch)(self, value))]),
      _ => Ok(())
    }
  }
}

impl ToAttributesList for PatternValidator<'_> {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    Some(vec![("pattern".into(), self.pattern.to_string().into())])
  }
}

impl FnOnce<(&str, )> for PatternValidator<'_> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (&str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl FnMut<(&str, )> for PatternValidator<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl Fn<(&str, )> for PatternValidator<'_> {
  extern "rust-call" fn call(&self, args: (&str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl FnOnce<(&&str, )> for PatternValidator<'_> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (&&str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl FnMut<(&&str, )> for PatternValidator<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&&str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl Fn<(&&str, )> for PatternValidator<'_> {
  extern "rust-call" fn call(&self, args: (&&str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl FnOnce<(&String, )> for PatternValidator<'_> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (&String, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl FnMut<(&String, )> for PatternValidator<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (&String, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl Fn<(&String, )> for PatternValidator<'_> {
  extern "rust-call" fn call(&self, args: (&String, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl Display for PatternValidator<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "PatternValidator {{pattern: {}}}", &self.pattern.to_string())
  }
}

pub fn pattern_vldr_pattern_mismatch_msg(rules: &PatternValidator, xs: &str) -> String {
  format!(
    "`{:}` does not match pattern `{:}`.",
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

    fn on_custom_pattern_mismatch(_: &PatternValidator, _: &str) -> String {
      "custom pattern mismatch err message".into()
    }

    for (name, instance, passing_value, failing_value, _err_callback) in [
      ("Default", PatternValidatorBuilder::default()
          .pattern(Cow::Owned(_rx.clone()))
          .build()?, "abc", "!@#)(*", &pattern_vldr_pattern_mismatch_msg),
      ("Custom ", PatternValidatorBuilder::default()
          .pattern(Cow::Owned(_rx.clone()))
          .pattern_mismatch(&on_custom_pattern_mismatch)
          .build()?, "abc", "!@#)(*", &on_custom_pattern_mismatch)
    ] as [(
      &str,
      PatternValidator,
      &str,
      &str,
      &PatternViolationCallback
    ); 2] {
      println!("{}", name);

      // Test as an `Fn*` trait
      assert_eq!((&instance)(passing_value), Ok(()));
      assert_eq!((&instance)(failing_value), Err(vec![
        (PatternMismatch, (instance.pattern_mismatch)(&instance, failing_value))
      ]));

      // Test `validate` method directly
      assert_eq!(instance.validate(passing_value), Ok(()));
      assert_eq!(instance.validate(failing_value), Err(vec![
        (PatternMismatch, (instance.pattern_mismatch)(&instance, failing_value))
      ]));
    }

    Ok(())
  }
}
