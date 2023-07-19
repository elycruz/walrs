use std::borrow::Cow;
use std::sync::Arc;
use regex::Regex;

use crate::input::ConstraintViolation;
use crate::input::ConstraintViolation::PatternMismatch;

pub type PatternViolationCallback = dyn Fn(&PatternValidator, &str) -> String + Send + Sync;

pub struct PatternValidator<'a> {
  pub pattern: Cow<'a, Regex>,
  pub pattern_mismatch: Arc<&'a PatternViolationCallback>,
}

impl<'a> PatternValidator<'a> {
  pub fn validate(&self, value: &str) -> Result<(), (ConstraintViolation, String)> {
    match self.pattern.is_match(value) {
      false => Err((PatternMismatch, (&self.pattern_mismatch.clone())(self, value))),
      _ => Ok(())
    }
  }
}
