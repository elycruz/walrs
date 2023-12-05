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

impl InputValue for char {}
impl InputValue for str {}

impl InputValue for &str {}

pub trait NumberValue: InputValue + Default + Copy + Add + Sub + Mul + Div + Rem<Output = Self> {}

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
  TypeMismatch,
  ValueMissing,
}

pub type ViolationMessage = String;

pub type ValidationErrTuple = (ConstraintViolation, ViolationMessage);

pub type ValidationResult = Result<(), Vec<ValidationErrTuple>>;

pub type Filter<T> = dyn Fn(T) -> T + Send + Sync;

pub type Validator<T> = dyn Fn(T) -> ValidationResult + Send + Sync;

pub type ValueMissingCallback<RulesStruct> = dyn Fn(RulesStruct) -> ViolationMessage + Send + Sync;

pub trait WithName<'a> {
  fn get_name() -> Option<Cow<'a, str>>;
}

pub trait ValidateValue<T: InputValue> {
  fn validate(&self, value: T) -> ValidationResult;
}

pub trait FilterValue<T: InputValue> {
  fn filter(&self, value: T) -> T;
}

pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}

pub trait InputConstraints<'a, 'b, T: 'b, FT: 'b>: Display + Debug
  where T: InputValue + ?Sized {

  fn validate(&self, value: Option<T>) -> Result<(), Vec<ValidationErrTuple>>;

  fn validate1(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>>;

  fn filter(&self, value: FT) -> FT;

  fn validate_and_filter(&self, x: Option<T>) -> Result<Option<FT>, Vec<ValidationErrTuple>>;

  fn validate_and_filter1(&self, x: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>>;
}
