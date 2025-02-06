use crate::InputValue;

pub trait FilterFn<T: InputValue> {
  fn filter(&self, value: T) -> T;
}
