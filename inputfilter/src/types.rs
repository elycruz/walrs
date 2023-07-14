use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct PhantomValue {}

impl Display for PhantomValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "do not instantiate - struct is meant to be used as a `PhantomData` type")
  }
}

pub trait InputValue: Clone + Debug + Display + PartialEq + PartialOrd {}

impl InputValue for i8 {}
impl InputValue for i16 {}
impl InputValue for i32 {}
impl InputValue for i64 {}
impl InputValue for i128 {}

impl InputValue for u8 {}
impl InputValue for u16 {}
impl InputValue for u32 {}
impl InputValue for u64 {}
impl InputValue for u128 {}

impl InputValue for f32 {}
impl InputValue for f64 {}
impl InputValue for &'_ str {}
impl InputValue for Cow<'_, str> {}
impl InputValue for String {}
impl InputValue for &'_ char {}
impl InputValue for Cow<'_, char> {}
impl InputValue for PhantomValue {}

impl InputValue for bool {}

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
