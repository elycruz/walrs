use crate::{InputValue, Violation};

pub type ValidateResult = Result<(), Violation>;

pub trait ValidateValue<T: InputValue> {
  fn validate(&self, value: T) -> ValidateResult;
}

pub trait ValidateRefValue2<T: ?Sized> {
  fn validate_ref(&self, value: &T) -> ValidateResult;
}
