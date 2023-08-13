use std::borrow::Cow;
use std::fmt::{Display};

pub trait InputValue: Clone + Default + Display + PartialEq + PartialOrd {}

impl InputValue for i8 {}
impl InputValue for i16 {}
impl InputValue for i32 {}
impl InputValue for i64 {}
impl InputValue for i128 {}
impl InputValue for isize {}

impl InputValue for u8 {}
impl InputValue for u16 {}
impl InputValue for u32 {}
impl InputValue for u64 {}
impl InputValue for u128 {}
impl InputValue for usize {}

impl InputValue for f32 {}
impl InputValue for f64 {}
impl InputValue for bool {}
impl InputValue for &'_ str {}
impl InputValue for Box<str> {}
impl InputValue for Cow<'_, str> {}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ConstraintViolation {
  CustomError,
  PatternMismatch,
  RangeOverflow,
  RangeUnderflow,
  StepMismatch,
  TooLong,
  TooShort,
  NotEqual,

  /// When value is in an invalid format, and cannot not validated
  /// against `pattern` (email, url, etc.) - currently unused.
  // @todo should probably be 'format mismatch'
  TypeMismatch,
  ValueMissing,
}

pub type ViolationMessage = String;

pub type ValidationError = (ConstraintViolation, ViolationMessage);

pub type ValidationResult = Result<(), Vec<ValidationError>>;

pub type Filter<T> = dyn Fn(Option<T>) -> Option<T> + Send + Sync;

pub type Validator<T> = dyn Fn(Cow<T>) -> ValidationResult + Send + Sync;

pub trait ValidateValue<T: InputValue> {
  fn validate(&self, x: Cow<T>) -> ValidationResult;
}

/*
// Unused
// - Saved for reference
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
*/
