use std::ops::{Add, Div, Mul, Rem, Sub};
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
impl InputValue for Cow<'_, str> {}

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
  ///  E.g., invalid email hostname, in email, etc.).
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

pub trait InputConstraints<T: InputValue>
where T: InputValue {
  fn validate(&self, x: Option<&T>) -> ValidationResult;
  fn filter<'a: 'b, 'b>(&self, x: Option<Cow<'a, T>>) -> Option<Cow<'b, T>>;
  fn validate_and_filter<'a: 'b, 'b>(&self, x: Option<&'a T>) -> Result<Option<Cow<'b, T>>, Vec<ValidationError>>;
}
