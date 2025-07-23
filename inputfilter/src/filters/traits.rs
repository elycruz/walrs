use crate::InputValue;

pub trait Filter<T: InputValue> {
  fn filter(&self, value: T) -> T;
}
