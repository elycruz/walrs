use crate::{InputValue, Violation};

pub type ValidatorResult = Result<(), Violation>;

pub trait Validate<T: InputValue> {
  fn validate(&self, value: T) -> ValidatorResult;
}

pub trait ValidateRef<T: ?Sized> {
  fn validate_ref(&self, value: &T) -> ValidatorResult;
}
