use crate::{Filter, Input, Validator, ValidatorForRef, ViolationMessage};

/// Returns a generic message for "Value is missing" violation.
///
/// ```rust
/// use std::borrow::Cow;
/// use walrs_inputfilter::{ref_value_missing_msg_getter, RefInput, value_missing_msg_getter};
///
/// let input = RefInput::<str, Cow<str>>::default();
///
/// assert_eq!(ref_value_missing_msg_getter(&input), "Value is missing".to_string());
/// ```
pub fn ref_value_missing_msg_getter<'a, 'b, T, FT>(_: &RefInput<'a, 'b, T, FT>) -> ViolationMessage
  where
      T: ?Sized + 'b,
      FT: From<&'b T> {
  "Value is missing".to_string()
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct RefInput<'a, 'b, T, FT = T>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  #[builder(default = "false")]
  pub break_on_failure: bool,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(default = "None")]
  pub custom: Option<&'a ValidatorForRef<T>>,

  #[builder(default = "None")]
  pub locale: Option<&'a str>,

  #[builder(default = "None")]
  pub name: Option<&'a str>,

  #[builder(default = "None")]
  pub default_value: Option<FT>,

  #[builder(default = "None")]
  pub validators: Option<Vec<&'a ValidatorForRef<T>>>,

  #[builder(default = "None")]
  pub filters: Option<Vec<&'a Filter<FT>>>,

  #[builder(default = "&ref_value_missing_msg_getter")]
  pub value_missing_msg_getter: &'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync),
}

impl<'a, 'b, T: ?Sized + 'b, FT: From<&'b T>> Default for RefInput<'a, 'b, T, FT> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{  
  ///   RefInput,
  ///   ref_value_missing_msg_getter,
  ///   ViolationEnum
  /// };
  ///
  /// let input = RefInput::<str, Cow<str>>::default();
  ///
  /// // Assert defaults
  /// // ----
  /// assert_eq!(input.break_on_failure, false);
  /// assert_eq!(input.required, false);
  /// assert!(input.name.is_none());
  /// assert!(input.custom.is_none());
  /// assert!(input.default_value.is_none());
  /// assert!(input.validators.is_none());
  /// assert!(input.filters.is_none());
  /// assert_eq!(
  ///   (&input.value_missing_msg_getter)(&input),
  ///   ref_value_missing_msg_getter(&input)
  /// );
  /// ```
  fn default() -> Self {
    RefInput {
      break_on_failure: false,
      required: false,
      custom: None,
      locale: None,
      name: None,
      default_value: None,
      validators: None,
      filters: None,
      value_missing_msg_getter: &ref_value_missing_msg_getter,
    }
  }
}
