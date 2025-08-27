use crate::violation::{Violation, ViolationMessage, Violations};
use std::fmt::{Debug, Display};

/// For Owned values.
pub type ValidatorForSized<T> = dyn Fn(T) -> Result<(), Violation> + Send + Sync;

/// For referenced/Unsized values.
pub type ValidatorForRef<T> = dyn Fn(&T) -> Result<(), Violation> + Send + Sync;

/// A trait for performing validations, and filtering (transformations), all in one,
/// for unsized types.
pub trait FilterForUnsized<'a, T, FT>: Display + Debug
where
  T: ?Sized + 'a,
  FT: From<&'a T>, // Filtered type - Returned by `Filter` components.
{
  fn validate_ref_detailed(&self, x: &T) -> Result<(), Violations>;

  fn validate_ref(&self, x: &T) -> Result<(), Vec<ViolationMessage>>;

  fn validate_ref_option_detailed(&self, x: Option<&T>) -> Result<(), Violations>;

  fn validate_ref_option(&self, x: Option<&T>) -> Result<(), Vec<ViolationMessage>>;

  fn filter_ref_detailed(&self, value: &'a T) -> Result<FT, Violations>;

  fn filter_ref(&self, value: &'a T) -> Result<FT, Vec<ViolationMessage>>;

  fn filter_ref_option_detailed(&self, value: Option<&'a T>) -> Result<Option<FT>, Violations>;

  fn filter_ref_option(&self, value: Option<&'a T>) -> Result<Option<FT>, Vec<ViolationMessage>>;
}

pub trait FilterForSized<T, FT = T>: Display + Debug
where
  T: Copy,
  FT: From<T>,
{
  fn validate_detailed(&self, x: T) -> Result<(), Violations>;

  fn validate(&self, x: T) -> Result<(), Vec<ViolationMessage>>;

  fn validate_option_detailed(&self, x: Option<T>) -> Result<(), Violations>;

  fn validate_option(&self, x: Option<T>) -> Result<(), Vec<ViolationMessage>>;

  fn filter_detailed(&self, value: T) -> Result<FT, Violations>;

  fn filter(&self, value: T) -> Result<FT, Vec<ViolationMessage>>;

  fn filter_option_detailed(&self, value: Option<T>) -> Result<Option<FT>, Violations>;

  fn filter_option(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>>;
}

pub type FilterFn<T> = dyn Fn(T) -> T + Send + Sync;

/// Allows serialization of properties that can be used for html form control contexts.
pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}