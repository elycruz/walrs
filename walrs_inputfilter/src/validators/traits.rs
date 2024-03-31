use crate::{InputValue, ValidationResult};

pub trait ValidateValue<T: InputValue> {
    fn validate(&self, value: T) -> ValidationResult;
}
