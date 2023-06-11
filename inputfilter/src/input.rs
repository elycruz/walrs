use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use regex::Regex;
use crate::input::ValidityState::{CustomError, PatternMismatch, TooLong, TooShort, Valid, ValueMissing};
use crate::types::InputValue;

pub type ConstraintViolationMsg = String;

#[derive(PartialEq, Debug)]
pub enum ValidityState {
  CustomError,
  PatternMismatch,
  RangeOverflow,
  RangeUnderflow,
  StepMismatch,
  TooLong,
  TooShort,

  /// When value is in invalid format, and not validated
  /// against `pattern` (email, url, etc.) - currently unused.
  TypeMismatch,
  // @todo should probably 'format mismatch'
  ValueMissing,
  Valid,
}

pub type ConstraintViolationFn<T> =
dyn Fn(&Input<T>, Option<T>) ->
ConstraintViolationMsg + Send + Sync;

pub type ConstraintCheck<T> =
dyn Fn(T) -> bool + Send + Sync;

pub type Message = String;
pub type ValidationResult = Result<(), ValidityState>;
pub type ValidationResultTuple = (ValidityState, Message);
pub type Validator<'a, T> = &'a (
dyn Fn(T) -> Option<Vec<(ValidityState, Message)>> + Send + Sync
);
pub type Filter<'a, T> = &'a (dyn Fn(Cow<T>) -> Cow<T> + Send + Sync);

#[derive(Builder, Clone)]
pub struct Input<'a, T> where
  T: InputValue
{
  #[builder(default = "true")]
  pub break_on_failure: bool,

  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "None")]
  pub step: Option<T>,

  #[builder(default = "None")]
  pub min_length: Option<usize>,

  #[builder(default = "None")]
  pub max_length: Option<usize>,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(default = "None")]
  pub pattern: Option<Regex>,

  #[builder(default = "None")]
  pub custom: Option<Arc<&'a ConstraintCheck<T>>>,

  #[builder(default = "None")]
  pub validators: Option<Arc<Vec<Validator<'a, T>>>>,

  #[builder(default = "None")]
  pub filters: Option<Arc<Vec<Filter<'a, T>>>>,

  #[builder(default = "Arc::new(&pattern_mismatch_msg)")]
  pub pattern_mismatch: Arc<&'a ConstraintViolationFn<T>>,

  #[builder(default = "Arc::new(&too_long_msg)")]
  pub too_long: Arc<&'a ConstraintViolationFn<T>>,

  #[builder(default = "Arc::new(&too_short_msg)")]
  pub too_short: Arc<&'a ConstraintViolationFn<T>>,

  #[builder(default = "Arc::new(&range_underflow_msg)")]
  pub range_underflow:
  Arc<&'a ConstraintViolationFn<T>>,

  #[builder(default = "Arc::new(&range_overflow_msg)")]
  pub range_overflow:
  Arc<&'a ConstraintViolationFn<T>>,

  #[builder(default = "Arc::new(&value_missing_msg)")]
  pub value_missing:
  Arc<&'a ConstraintViolationFn<T>>,

  #[builder(default = "Arc::new(&custom_error_msg)")]
  pub custom_error:
  Arc<&'a ConstraintViolationFn<T>>,
}

impl<T: InputValue> Input<'_, T> {
  pub fn new() -> Self {
    Input {
      break_on_failure: false,
      min: None,
      max: None,
      step: None,
      min_length: None,
      max_length: None,
      pattern: None,
      required: false,
      custom: None,
      validators: None,
      filters: None,

      pattern_mismatch: Arc::new(&pattern_mismatch_msg),
      too_long: Arc::new(&too_long_msg),
      too_short: Arc::new(&too_short_msg),
      range_underflow: Arc::new(&range_underflow_msg),
      range_overflow: Arc::new(&range_overflow_msg),
      value_missing: Arc::new(&value_missing_msg),
      custom_error: Arc::new(&custom_error_msg),
    }
  }

  /// Applies filters in `filters` from right-to-left.
  pub fn filter<'x>(&self, xs: Cow<'x, T>) -> Cow<'x, T> {
    match self.filters.as_deref() {
      Some(filters) => {
        filters.iter().rfold(xs, |_xs, f| f(_xs))
      }
      _ => xs
    }
  }

  pub fn validate(&self, value: Option<T>) -> Result<(), (ValidityState, String)> {
    let validity = match &value {
      None => if !self.required {
        Valid
      } else {
        ValueMissing
      },
      Some(v) => _validate(self, v.clone())
    };

    if validity != Valid {
      return match _get_validation_message(self, &validity, value) {
        Some(msg) => Err((validity, msg)),
        _ => Ok(()),
      };
    }

    Ok(())
  }

  pub fn validate_and_filter<'x>(&self, value: Option<T>) -> Result<Option<Cow<'x, T>>, (ValidityState, String)> {
    self.validate(value.clone())?;

    Ok(value.map(|v| Cow::Owned(v))) //self.filter(Cow::Borrowed(&v))))
  }
}

fn _validate<T: InputValue>(rules: &Input<T>, value: T) -> ValidityState {
  let xs = value.to_string();

  // Run custom test
  if let Some(custom) = &rules.custom {
    let _fn = Arc::clone(custom);
    if !((_fn)(value)) {
      return CustomError;
    }
  }

  // Test against Min Length
  if let Some(min_length) = &rules.min_length {
    if &xs.len() < min_length {
      return TooShort;
    }
  }

  // Test against Max Length
  if let Some(max_length) = &rules.max_length {
    if &xs.len() > max_length {
      return TooLong;
    }
  }

  // Test pattern
  if let Some(pattern) = &rules.pattern {
    if !pattern.is_match(&xs) {
      return PatternMismatch;
    }
  }

  Valid
}

fn _get_validation_message<T: InputValue>(
  constraints: &Input<T>,
  validity_enum: &ValidityState,
  x: Option<T>,
) -> Option<String> {
  let f = match validity_enum {
    CustomError => Some(&constraints.custom_error),
    PatternMismatch => Some(&constraints.pattern_mismatch),
    TooLong => Some(&constraints.too_long),
    TooShort => Some(&constraints.too_short),
    ValueMissing => Some(&constraints.value_missing),
    _ => None,
  };

  f.map(|_f| {
    let _fn = Arc::clone(_f);
    (_fn)(constraints, x)
  })
}

pub fn pattern_mismatch_msg<'a, T>(rules: &Input<T>, xs: Option<T>) -> String
  where
    T: InputValue + 'a
{
  format!(
    "`{}` does not match pattern `{}`",
    &xs.as_ref().unwrap(),
    rules.pattern.as_ref().unwrap()
  )
}

pub fn range_underflow_msg<T>(rules: &Input<T>, x: Option<T>) -> String
  where
    T: InputValue
{
  format!(
    "`{:}` is less than minimum `{:}`.",
    &x.as_ref().unwrap(),
    &rules.min.as_ref().unwrap()
  )
}

pub fn range_overflow_msg<T>(rules: &Input<T>, x: Option<T>) -> String
  where
    T: InputValue
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    &x.as_ref().unwrap(),
    &rules.max.as_ref().unwrap()
  )
}

pub fn too_short_msg<T>(rules: &Input<T>, xs: Option<T>) -> String
  where
    T: InputValue {
  format!(
    "Value length `{:}` is less than allowed minimum `{:}`.",
    &xs.as_ref().unwrap().to_string().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn too_long_msg<T>(rules: &Input<T>, xs: Option<T>) -> String
  where
    T: InputValue {
  format!(
    "Value length `{:}` is greater than allowed maximum `{:}`.",
    &xs.as_ref().unwrap().to_string().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn value_missing_msg<T>(_: &Input<T>, _: Option<T>) -> String
  where
    T: InputValue {
  "Value is missing.".to_string()
}

pub fn custom_error_msg<T>(_: &Input<T>, _: Option<T>) -> String
  where
    T: InputValue {
  "Custom error.".to_string()
}
