use std::ops::{Add, Div, Mul, Rem, Sub};
use std::fmt::{Debug, Display};
use serde::{Serialize};

pub trait InputValue: Copy + Default + PartialEq + PartialOrd + Serialize {}

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
impl InputValue for &str {}

impl<T: InputValue> InputValue for &[T] {}

pub trait ScalarValue: InputValue + Display {}

impl ScalarValue for i8 {}
impl ScalarValue for i16 {}
impl ScalarValue for i32 {}
impl ScalarValue for i64 {}
impl ScalarValue for i128 {}
impl ScalarValue for isize {}

impl ScalarValue for u8 {}
impl ScalarValue for u16 {}
impl ScalarValue for u32 {}
impl ScalarValue for u64 {}
impl ScalarValue for u128 {}
impl ScalarValue for usize {}

impl ScalarValue for f32 {}
impl ScalarValue for f64 {}

impl ScalarValue for bool {}
impl ScalarValue for char {}
impl ScalarValue for &str {}


pub trait NumberValue: ScalarValue + Add + Sub + Mul + Div + Rem<Output = Self> {}

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

/// Violation Enum types represent the possible violation types that may be returned, along with error messages,
/// from any given "validation" operation.
///
/// These additionally provide a runtime opportunity to override
/// returned violation message(s), via returned validation result `Err` tuples, and the ability to provide the
/// violation type from "constraint" structures that perform validation against their own constraint props.;  E.g.,
/// `StringConstraints` (etc.) with it's `pattern`, `min_length`, `max_length` props. etc.
///
/// @todo Consider attaching messages to enum variants (will require something like `unwrap`,
///   and/or, `to_string()` to limit the `match` statement verbosity that it might add).
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ViolationEnum {
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

/// A validation violation message.
pub type ViolationMessage = String;

/// A validation violation tuple.
pub type ViolationTuple = (ViolationEnum, ViolationMessage);

/// Returned from validators, and Input Constraint struct `validate_*detailed` methods.
pub type ValidationResult = Result<(), Vec<ViolationTuple>>;

/// Allows serialization of properties that can be used for html form control contexts.
pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}

pub type Filter<T> = dyn Fn(T) -> T + Send + Sync;

pub type Validator<T> = dyn Fn(T) -> ValidationResult + Send + Sync;

pub trait ValidatorRefT<T: ?Sized>: Fn(&T) -> ValidationResult {}

impl<T: ?Sized, F: ?Sized> ValidatorRefT<T> for F 
  where F: Fn(&T) -> ValidationResult {}

pub type ValidatorForRef<T> = dyn Fn(&T) -> ValidationResult + Send + Sync;

/// Violation message getter for `ValueMissing` Violation Enum type.
pub type ValueMissingCallback = dyn Fn() -> ViolationMessage + Send + Sync;

pub trait InputConstraints<T, FT = T>: Display + Debug
  where T: Copy,
        FT: From<T> {

  fn validate(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>>;

  fn validate_detailed(&self, value: Option<T>) -> Result<(), Vec<ViolationTuple>>;

  fn filter(&self, value: FT) -> FT;

  fn validate_and_filter(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>>;

  fn validate_and_filter_detailed(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationTuple>>;
}

pub enum Violation {
    CustomError(String),
    PatternMismatch(String),
    RangeOverflow(String),
    RangeUnderflow(String),
    StepMismatch(String),
    TooLong(String),
    TooShort(String),
    NotEqual(String),
    TypeMismatch(String),
    ValueMissing(String),
}

pub trait InputFilterForSized<T, FT = T>: Display + Debug
  where T: Copy,
        FT: From<T> {
  fn validate(&self, value: T) -> Result<(), Vec<Violation>>;

  fn validate_option(&self, value: Option<T>) -> Result<(), Vec<Violation>>;

  fn filter(&self, value: T) -> Result<Option<FT>, Vec<Violation>>;

  fn filter_option(&self, value: Option<T>) -> Result<Option<FT>, Vec<Violation>>;
}

pub trait InputFilterForUnsized<T, FT = T>: Display + Debug
  where T: ?Sized,
        for<'x> FT: From<&'x T> {
  fn validate(&self, value: &T) -> Result<(), Vec<ViolationTuple>>;
    
  fn validate_option(&self, value: Option<&T>) -> Result<(), Vec<ViolationTuple>>;

  fn filter(&self, value: &T) ->Result<Option<FT>, Vec<ViolationTuple>>;

  fn filter_option(&self, value: Option<&T>) -> Result<Option<FT>, Vec<ViolationTuple>>;
}
