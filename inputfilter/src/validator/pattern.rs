use std::borrow::Cow;
use std::fmt::Display;
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

impl ValidateValue<&str> for PatternValidator<'_>
where {
  fn validate(&self, value: &&str) -> ValidationResult {
    match self.pattern.is_match(value) {
      false => Err(vec![(PatternMismatch, (self.pattern_mismatch)(self, value))]),
      _ => Ok(())
    }
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

impl Display for PatternValidator<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "PatternValidator {{pattern: {}}}", &self.pattern.to_string())
  }
}

pub fn pattern_mismatch_msg(rules: &PatternValidator, x: &str) -> String {
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.pattern.to_string()
  )
}
