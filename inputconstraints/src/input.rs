use std::fmt::{Debug, Display};
use std::sync::Arc;
use regex::Regex;

pub enum InputType {
  Button,
  Checkbox,
  Color,
  Date,
  Datetime,
  DatetimeLocal,
  Email,
  File,
  Hidden,
  Image,
  Month,
  Number,
  Password,
  Radio,
  Range,
  Reset,
  Search,
  SelectMultiple,
  SelectOne,
  Submit,
  Tel,
  Text,
  TextArea,
  Time,
  URL,
  Week
}

pub type ConstraintViolationMsg = String;

pub enum ConstraintViolation {
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
  ValueMissing,
}

pub type ConstraintViolationMsgFn<T> =
  dyn Fn(&Input<T>, Option<T>) ->
    ConstraintViolationMsg + Send + Sync;

pub type ConstraintCheck<T> =
  dyn Fn(T) -> bool + Send + Sync;

#[derive(Builder, Clone, Debug)]
pub struct Input<'a, T> where
  T: Clone + Debug + Display + PartialEq + PartialOrd
{
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

  #[builder(default = "Arc::new(&pattern_mismatch_msg)")]
  pub pattern_mismatch: Arc<&'a ConstraintViolationMsgFn<T>>,

  #[builder(default = "Arc::new(&too_long_msg)")]
  pub too_long: Arc<&'a ConstraintViolationMsgFn<T>>,

  #[builder(default = "Arc::new(&too_short_msg)")]
  pub too_short: Arc<&'a ConstraintViolationMsgFn<T>>,

  #[builder(default = "Arc::new(&range_underflow_msg)")]
  pub range_underflow:
  Arc<&'a ConstraintViolationMsgFn<T>>,

  #[builder(default = "Arc::new(&range_overflow_msg)")]
  pub range_overflow:
  Arc<&'a ConstraintViolationMsgFn<T>>,

  #[builder(default = "Arc::new(&value_missing_msg)")]
  pub value_missing:
  Arc<&'a ConstraintViolationMsgFn<T>>,

  #[builder(default = "Arc::new(&custom_error_msg)")]
  pub custom_error:
  Arc<&'a ConstraintViolationMsgFn<T>>,
}


pub fn pattern_mismatch_msg<T>(rules: &Input<T>, xs: Option<T>) -> String {
  format!(
    "`{}` does not match pattern `{}`",
    xs.as_deref(),
    rules.pattern.as_ref().unwrap()
  )
}

pub fn range_underflow_msg<T>(rules: &Input<T>, x: Option<T>) -> String
  where
    T: Display + Copy + PartialOrd + PartialEq,
{
  format!(
    "`{:}` is less than minimum `{:}`.",
    &x.as_ref().unwrap(),
    &rules.min.as_ref().unwrap()
  )
}

pub fn range_overflow_msg<T>(rules: &Input<T>, x: Option<T>) -> String
  where
    T: Display + Copy + PartialOrd + PartialEq,
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    &x.as_ref().unwrap(),
    &rules.max.as_ref().unwrap()
  )
}


pub fn too_short_msg<T>(rules: &Input<T>, xs: Option<T>) -> String {
  format!(
    "Value length `{:}` is less than allowed minimum `{:}`.",
    &xs.as_ref().unwrap().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn too_long_msg<T>(rules: &Input<T>, xs: Option<T>) -> String {
  format!(
    "Value length `{:}` is greater than allowed maximum `{:}`.",
    &xs.as_ref().unwrap().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn value_missing_msg<T>(_: &Input<T>, _: Option<T>) -> String {
  "Value is missing.".to_string()
}

pub fn custom_error_msg<T>(_: &Input<T>, _: Option<T>) -> String {
  "Custom error.".to_string()
}
