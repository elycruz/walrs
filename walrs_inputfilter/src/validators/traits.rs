use crate::{InputValue, ValidationResult, ValidationResult2};

pub trait ValidateValue<T: InputValue> {
  fn validate(&self, value: T) -> ValidationResult;
}

pub trait ValidateRefValue2<T: ?Sized> {
  fn validate_ref(&self, value: &T) -> ValidationResult2;
}
