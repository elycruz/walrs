use crate::ViolationType::ValueMissing;
use crate::{
  FilterFn, ValidateRef, ValidateRefOption, ValidationErrType,
  ValidationResult2, ValidationRefValue,
  ValidatorForRef, Violation, ViolationMessage,
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
  pub filters: Option<Vec<&'a FilterFn<FT>>>,

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
  fn validate_ref(&self, value: ValidationRefValue<T>) -> ValidationResult2 {
    match value {
      ValidationRefValue::Element(value) => {
        let mut violations = ValidationErrType::Element(vec![]);

        // Validate custom
        match if let Some(custom) = self.custom {
          (custom)(value)
        } else {
          Ok(())
        } {
          Ok(()) => (),
          Err(err_type) => violations.extend(err_type),
        }

        if !violations.is_empty() && self.break_on_failure {
          return Err(violations);
        }

        // Else validate against validators
        self.validators.as_deref().map_or(Ok(()), |validators| {
          for validator in validators {
            match validator(value) {
              Ok(()) => continue,
              Err(err_type) => {
                violations.extend(err_type);
                if self.break_on_failure {
                  break;
                }
              },
              _ => unreachable!("Only `ValidationErrType::Element` type currently supported.")
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
      _ => unreachable!("Only `ValidationValue::Element` type currently supported.")
    }
  }
}

impl<'a, 'b, T, FT> ValidateRefOption<T> for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>
{
  fn validate_ref_option(&self, value: Option<ValidationRefValue<T>>) -> ValidationResult2 {
    match value {
      Some(v) => self.validate_ref(v),
      None => {
        if self.required {
          Err(ValidationErrType::Element(vec![Violation(
            ValueMissing,
            (self.value_missing_msg_getter)(self),
          )]))
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
  FT: From<&'b T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "RefInput {{ break_on_failure: {}, required: {}, custom: {}, locale: {:?}, name: {:?}, default_value: {}, validators: {}, filters: {}, value_missing_msg_getter: {} }}",
           self.break_on_failure,
           self.required,
           if self.custom.is_some() { "Some(&ValidatorForRef)" } else { "None" },
           self.locale,
           self.name,
           if let Some(default_value) = self.default_value.as_ref() { "Some(...)" } else { "None" },
           if let Some(vs) = self.validators.as_deref() { format!("[&ValidatorForRef<T>; {}", vs.len()) } else { "None".to_string() },
           if let Some(fs) = self.filters.as_deref() { format!("[&FilterFn<FT>; {}", fs.len()) } else { "None".to_string() },
           "&'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync)"
    )
  }
}

impl<'a, 'b, T, FT> Debug for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "RefInput {{ break_on_failure: {}, required: {}, custom: {:?}, locale: {:?}, name: {:?}, default_value: {}, validators: {}, filters: {}, value_missing_msg_getter: {} }}",
           self.break_on_failure,
           self.required,
           if self.custom.is_some() { "Some(&ValidatorForRef)" } else { "None" },
           self.locale,
           self.name,
           if let Some(default_value) = self.default_value.as_ref() { "Some(...)" } else { "None" },
           if let Some(vs) = self.validators.as_deref() { format!("[&ValidatorForRef<T>; {}", vs.len()) } else { "None".to_string() },
           if let Some(fs) = self.filters.as_deref() { format!("[&FilterFn<FT>; {}", fs.len()) } else { "None".to_string() },
           "&'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync)"
    )
  }
}
/*
impl<'a, 'b, T, FT> InputFilterForUnsized<'b, T, FT> for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  /// Validates, and filters, incoming value.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{
  ///     RefInput, ValidateRef, Filter,
  ///     InputFilterForUnsized, RefInputBuilder, Violation,
  ///     ViolationType::TypeMismatch, ValidationErrType,
  ///     ViolationMessage, ValidationRefValue
  /// };
  ///
  /// let mut input = RefInput::<str, Cow<str>>::default();
  /// let mut input2 = RefInput::<str, String>::default();
  /// let mut only_vowels = RefInput::<str, Cow<str>>::default();
  /// let alnum_regex = regex::Regex::new(r"(?i)^[a-z\d]+$").unwrap();
  ///
  /// input2.filters = Some(vec![&|value: String| value.to_lowercase()]);
  /// only_vowels.validators = Some(vec![
  ///   &|value: &str| if alnum_regex.is_match(value) {
  ///     Ok(())
  ///   } else {
  ///     Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())]))
  ///   }
  /// ]);
  ///
  /// let value = "Hello, World!";
  ///
  /// assert_eq!(input.filter(ValidationRefValue::Element(value)).unwrap(), Cow::Borrowed(value));
  /// assert_eq!(input2.filter(ValidationRefValue::Element(value)).unwrap(), value.to_lowercase());
  ///
  /// match only_vowels.filter(value) {
  ///    Ok(_) => unreachable!("Should not be reachable"),
  ///    Err(ValidationErrType::Element(violations)) => assert_eq!(
  ///      format!("{:?}", violations),
  ///      format!("{:?}", vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())])
  ///    ),
  ///    _ => unreachable!("Should not be reachable")
  /// }
  /// ```
  fn filter(&self, value: ValidationRefValue<T>) -> Result<FT, ValidationErrType> {
    ValidateRef::validate_ref(self, value)?;
    Ok(self.filters.as_deref().map_or(value.into(), |filters| {
      filters
        .iter()
        .fold(value.into(), |agg, filter| (filter)(agg))
    }))
  }

  fn filter_option(&self, value: Option<ValidationRefValue<T>>) -> Result<Option<FT>, ValidationErrType> {
    match value {
      Some(value) => self.filter(value).map(Some),
      None => {
        if self.required {
          Err(ValidationErrType::Element(vec![
            Violation(
              ValueMissing,
              (self.value_missing_msg_getter)(self),
            )
          ]))
        } else {
          Ok(None)
        }
      }
    }
  }
}
*/