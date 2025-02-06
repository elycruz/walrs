use serde::Serialize;
use std::fmt::{Debug, Display};
use std::ops::{Add, Div, Mul, Rem, Sub};
use crate::{ViolationMessage, ViolationType};

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

/// A validation violation tuple.
pub type ViolationTuple = (ViolationType, ViolationMessage);

/// Returned from validators, and Input Constraint struct `validate_*detailed` methods.
pub type ValidationResult = Result<(), Vec<ViolationTuple>>;

/// Allows serialization of properties that can be used for html form control contexts.
pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}

pub type FilterFn<T> = dyn Fn(T) -> T + Send + Sync;
