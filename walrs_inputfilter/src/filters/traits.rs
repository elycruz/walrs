use crate::InputValue;

pub trait FilterValue<T: InputValue> {
    fn filter(&self, value: T) -> T;
}
