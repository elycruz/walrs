use crate::ViolationType::ValueMissing;
use crate::{
  FilterValue, Input, InputFilterForUnsized, ValidateRef, ValidateRefOption, ValidationResult2,
  Validator, ValidatorForRef, Violation, ViolationMessage,
};
use std::fmt::{Debug, Display, Formatter};

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
  FT: From<&'b T>,
{
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
  pub filters: Option<Vec<&'a FilterValue<FT>>>,

  #[builder(default = "&ref_value_missing_msg_getter")]
  pub value_missing_msg_getter:
    &'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync),
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

impl<'a, 'b, T, FT> ValidateRef<T> for RefInput<'a, 'b, T, FT>
where
    T: ?Sized + 'b,
    FT: From<&'b T>
{
  fn validate_ref(&self, value: &T) -> ValidationResult2 {
    let mut violations = Vec::new();

    // Validate custom
    match (if let Some(custom) = self.custom {
      (custom)(value)
    } else {
      Ok(())
    }) {
      Ok(()) => (),
      Err(mut vs) => violations.append(vs.as_mut()),
    }

    if !violations.is_empty() && self.break_on_failure {
      return Err(violations);
    }

    // Else validate against validators
    self.validators.as_deref().map_or(Ok(()), |validators| {
      for validator in validators {
        match validator(value) {
          Ok(()) => continue,
          Err(mut vs) => {
            violations.append(vs.as_mut());
            if self.break_on_failure {
              break;
            }
          }
        }
      }

      // Resolve return value
      if violations.is_empty() {
        Ok(())
      } else {
        Err(violations)
      }
    })
  }
}

impl<'a, 'b, T, FT> ValidateRefOption<T> for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>
{
  fn validate_ref_option(&self, value: Option<&T>) -> ValidationResult2 {
    match value {
      Some(value) => self.validate_ref(value),
      None => {
        if self.required {
          Err(vec![Violation(
            ValueMissing,
            (self.value_missing_msg_getter)(self),
          )])
        } else {
          Ok(())
        }
      }
    }
  }
}

impl<'a, 'b, T, FT> Display for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  for<'x> FT: From<&'x T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    todo!()
  }
}

impl<'a, 'b, T, FT> Debug for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  for<'x> FT: From<&'x T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    todo!()
  }
}

impl<'a, 'b, T, FT> InputFilterForUnsized<T, FT> for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  for<'x> FT: From<&'x T>,
{
  fn filter(&self, value: &T) -> Result<FT, Vec<Violation>> {
    ValidateRef::validate_ref(self, value)?;
    Ok(self.filters.as_deref().map_or(value.into(), |filters| {
      filters
        .iter()
        .fold(value.into(), |agg, filter| (filter)(agg))
    }))
  }

  fn filter_option(&self, value: Option<&T>) -> Result<Option<FT>, Vec<Violation>> {
    match value {
      Some(value) => self.filter(value).map(Some),
      None => {
        if self.required {
          Err(vec![Violation(
            ValueMissing,
            (self.value_missing_msg_getter)(self),
          )])
        } else {
          Ok(None)
        }
      }
    }
  }
}
