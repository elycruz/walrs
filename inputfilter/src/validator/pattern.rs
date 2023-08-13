use std::borrow::Cow;
use regex::Regex;

use crate::types::ConstraintViolation::PatternMismatch;
use crate::types::{ValidationResult, ValidateValue};

pub type PatternViolationCallback = dyn Fn(&PatternValidator, &str) -> String + Send + Sync;

#[derive(Builder, Clone)]
pub struct PatternValidator<'a> {
  pub pattern: Cow<'a, Regex>,

  #[builder(default = "&pattern_mismatch_msg")]
  pub pattern_mismatch: &'a PatternViolationCallback,
}

impl ValidateValue<&str> for PatternValidator<'_> {
  fn validate(&self, value: Cow<'_, &str>) -> ValidationResult {
    match self.pattern.is_match(value.as_ref()) {
      false => Err(vec![(PatternMismatch, (self.pattern_mismatch)(self, value.as_ref()))]),
      _ => Ok(())
    }
  }
}

impl FnOnce<(Cow<'_, &str>, )> for PatternValidator<'_> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (Cow<'_, &str>, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl FnMut<(Cow<'_, &str>, )> for PatternValidator<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'_, &str>, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl Fn<(Cow<'_, &str>, )> for PatternValidator<'_> {
  extern "rust-call" fn call(&self, args: (Cow<'_, &str>, )) -> Self::Output {
    self.validate(args.0)
  }
}

pub fn pattern_mismatch_msg(rules: &PatternValidator, x: &str) -> String {
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.pattern.to_string()
  )
}
