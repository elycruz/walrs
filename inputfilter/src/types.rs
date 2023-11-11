use std::ops::{Add, Div, Mul, Rem, Sub};
use std::borrow::Cow;
use std::fmt::{Debug, Display};
use serde::Serialize;

pub trait InputValue: ToOwned + Debug + Display + PartialEq + PartialOrd + Serialize {}

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

impl InputValue for str {}
impl InputValue for Box<str> {}
impl InputValue for String {}
impl<'a> InputValue for Cow<'a, str> {}
impl InputValue for &'_ str {}
impl InputValue for &&'_ str {}

pub trait NumberValue: Default + InputValue + Copy + Add + Sub + Mul + Div + Rem<Output = Self> {}

impl NumberValue for i8 {}
impl NumberValue for i16 {}
impl NumberValue for i32 {}
impl NumberValue for i64 {}
impl NumberValue for i128 {}
impl NumberValue for isize {}

impl NumberValue for u8 {}
impl NumberValue for u16 {}
impl NumberValue for u32 {}
impl NumberValue for u64 {}
impl NumberValue for u128 {}
impl NumberValue for usize {}

impl NumberValue for f32 {}
impl NumberValue for f64 {}

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

  /// Used to convey an expected string format (not necessarily a `Pattern` format;
  ///  E.g., invalid email hostname in email pattern, etc.).
  TypeMismatch,
  ValueMissing,
}

pub type ViolationMessage = String;

pub type ValidationError = (ConstraintViolation, ViolationMessage);

pub type ValidationResult = Result<(), Vec<ValidationError>>;

pub type Filter<T> = dyn Fn(Option<T>) -> Option<T> + Send + Sync;

pub type Validator<T> = dyn Fn(T) -> ValidationResult + Send + Sync;

pub trait ValidateValue<T: InputValue> {
  fn validate(&self, value: T) -> ValidationResult;
}

pub trait FilterValue<T: InputValue> {
  fn filter(&self, value: Option<Cow<T>>) -> Option<Cow<T>>;
}

pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}

pub trait InputConstraints<'a, 'call_ctx: 'a, T: InputValue>: Display + Debug + 'a {
  fn get_should_break_on_failure(&self) -> bool;
  fn get_required(&self) -> bool;
  fn get_name(&self) -> Option<Cow<'a, str>>;
  fn get_value_missing_handler(&self) -> &'a (dyn Fn(&Self) -> ViolationMessage + Send + Sync);
  fn get_validators(&self) -> Option<&[&'a Validator<&'call_ctx T>]>;
  fn get_filters(&self) -> Option<&[&'a Filter<Cow<'call_ctx, T>>]>;

  fn validate_with_validators(&self, value: &'call_ctx T, validators: Option<&[&'a Validator<&'call_ctx T>]>) -> ValidationResult {
    validators.map(|vs| {

      // If not break on failure then capture all validation errors.
      if !self.get_should_break_on_failure() {
        return vs.iter().fold(
          Vec::<ValidationError>::new(),
          |mut agg, f| match (f)(value) {
            Err(mut message_tuples) => {
              agg.append(message_tuples.as_mut());
              agg
            }
            _ => agg,
          });
      }

      // Else break on, and capture, first failure.
      let mut agg = Vec::<ValidationError>::new();
      for f in vs.iter() {
        if let Err(mut message_tuples) = (f)(value) {
          agg.append(message_tuples.as_mut());
          break;
        }
      }
      agg
    })
      .and_then(|messages| if messages.is_empty() { None } else { Some(messages) })
      .map_or(Ok(()), Err)
  }

  fn validate(&self, value: Option<&'call_ctx T>) -> ValidationResult {
    match value {
      None => {
        if self.get_required() {
          Err(vec![(
            ConstraintViolation::ValueMissing,
            (self.get_value_missing_handler())(self),
          )])
        } else {
          Ok(())
        }
      }
      Some(v) => self.validate_with_validators(v, self.get_validators()),
    }
  }

  fn filter(&self, value: Option<Cow<'call_ctx, T>>) -> Option<Cow<'call_ctx, T>> {
    match self.get_filters() {
      None => value,
      Some(fs) => fs.iter().fold(value, |agg, f| (f)(agg)),
    }
  }

  fn validate_and_filter(&self, x: Option<&'call_ctx T>) -> Result<Option<Cow<'call_ctx, T>>, Vec<ValidationError>> {
    self.validate(x).map(|_| self.filter(x.map(|_x| Cow::Borrowed(_x))))
  }
}

