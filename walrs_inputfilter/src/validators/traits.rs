use crate::{InputValue, ValidationResult, Violation};

pub type ValidatorResult = Result<(), Violation>;

pub trait ValidateValue<T: InputValue> {
  fn validate(&self, value: T) -> ValidationResult;
}

pub trait ValidateRefValue2<T: ?Sized> {
  fn validate_ref(&self, value: &T) -> ValidatorResult;
}
