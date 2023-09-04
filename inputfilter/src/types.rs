use std::ops::{Add, Div, Mul, Rem, Sub};
use std::borrow::Cow;
use std::fmt::{Debug, Display};
use serde::Serialize;

pub trait InputValue: Clone + Default + Debug + Display + PartialEq + PartialOrd + Serialize {}

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

pub trait NumberValue: InputValue + Copy + Add + Sub + Mul + Div + Rem<Output = Self> {}

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

pub type Filter<T> = dyn Fn(Option<Cow<T>>) -> Option<Cow<T>> + Send + Sync;

pub type Validator<T> = dyn Fn(&T) -> ValidationResult + Send + Sync;

pub trait ValidateValue<T: InputValue> {
  fn validate(&self, x: &T) -> ValidationResult;
}

pub trait FilterValue<T: InputValue> {
  fn filter(&self, x: Option<Cow<T>>) -> Option<Cow<T>>;
}

pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}

pub trait InputConstraints<'a, T: InputValue>: Display + Debug + 'a
where T: InputValue {
  fn get_should_break_on_failure(&self) -> bool;
  fn get_required(&self) -> bool;
  fn get_name(&self) -> Option<Cow<'a, str>>;
  fn get_value_missing_handler(&self) -> &'a (dyn Fn(&Self) -> ViolationMessage + Send + Sync);
  fn get_validators(&self) -> Option<&[&Validator<T>]>;
  fn get_filters(&self) -> Option<&[&Filter<T>]>;

  fn validate_with_validators(&self, value: &T, validators: Option<&[&Validator<T>]>) -> ValidationResult {
    validators.as_deref().map(|vs| {

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
      .and_then(|msgs| if msgs.is_empty() { None } else { Some(msgs) })
      .map_or(Ok(()), |msgs| Err(msgs))
  }

  fn validate(&self, value: Option<&T>) -> ValidationResult {
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

  fn filter<'b: 'c, 'c>(&self, value: Option<Cow<'b, T>>) -> Option<Cow<'c, T>> {
    match self.get_filters() {
      None => value,
      Some(fs) => fs.iter().fold(value, |agg, f| (f)(agg)),
    }
  }

  fn validate_and_filter<'b: 'c, 'c>(&self, x: Option<&'b T>) -> Result<Option<Cow<'c, T>>, Vec<ValidationError>> {
    self.validate(x).map(|_| self.filter(x.map(|_x| Cow::Borrowed(_x))))
  }
}
