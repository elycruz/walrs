
/// General filter trait.
pub trait Filter<T> {
  fn filter(&self, value: T) -> T;
}
