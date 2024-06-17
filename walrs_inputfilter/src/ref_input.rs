use crate::ViolationType::ValueMissing;
use crate::{
  FilterFn, InputFilterForUnsized, ValidateRef, ValidateRefOption, ValidationErrType,
  ValidationResult2, ValidatorForRef, Violation, ViolationMessage,
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

  /// Returns a default value for the "input is not required, but is empty" use case.
  #[builder(default = "None")]
  pub get_default_value: Option<&'a dyn Fn() -> Option<FT>>,

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
  /// assert!(input.get_default_value.is_none());
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
      get_default_value: None,
      validators: None,
      filters: None,
      value_missing_msg_getter: &ref_value_missing_msg_getter,
    }
  }
}

impl<'a, 'b, T, FT> ValidateRef<T> for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  fn validate_ref(&self, value: &T) -> ValidationResult2 {
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
  FT: From<&'b T>,
{
  fn validate_ref_option(&self, value: Option<&T>) -> ValidationResult2 {
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
    write!(f, "RefInput {{ break_on_failure: {}, required: {}, custom: {}, locale: {:?}, name: {:?}, default_value: {}, validators: {}, filters: {}, value_missing_msg_getter: &'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync) }}",
           self.break_on_failure,
           self.required,
           if self.custom.is_some() { "Some(&ValidatorForRef)" } else { "None" },
           self.locale,
           self.name,
           if self.get_default_value.as_ref().is_some() { "Some(...)" } else { "None" },
           if let Some(vs) = self.validators.as_deref() { format!("[&ValidatorForRef<T>; {}", vs.len()) } else { "None".to_string() },
           if let Some(fs) = self.filters.as_deref() { format!("[&FilterFn<FT>; {}", fs.len()) } else { "None".to_string() }
    )
  }
}

impl<'a, 'b, T, FT> Debug for RefInput<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "RefInput {{ break_on_failure: {}, required: {}, custom: {:?}, locale: {:?}, name: {:?}, default_value: {}, validators: {}, filters: {}, value_missing_msg_getter: &'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync) }}",
           self.break_on_failure,
           self.required,
           if self.custom.is_some() { "Some(&ValidatorForRef)" } else { "None" },
           self.locale,
           self.name,
           if self.get_default_value.as_ref().is_some() { "Some(...)" } else { "None" },
           if let Some(vs) = self.validators.as_deref() { format!("[&ValidatorForRef<T>; {}", vs.len()) } else { "None".to_string() },
           if let Some(fs) = self.filters.as_deref() { format!("[&FilterFn<FT>; {}", fs.len()) } else { "None".to_string() }
    )
  }
}

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
  /// // Create some validators
  /// let alnum_regex = regex::Regex::new(r"(?i)^[a-z\d]+$").unwrap();
  /// let alnum_only = move |value: &str| if alnum_regex.is_match(value) {
  ///     Ok(())
  ///   } else {
  ///     Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())]))
  ///   };
  ///
  /// // Create some input controls
  /// let mut input = RefInput::<str, Cow<str>>::default();
  ///
  /// let mut input2 = RefInput::<str, String>::default();
  /// input2.filters = Some(vec![&|value: String| value.to_lowercase()]);
  ///
  /// let mut alnum_input = RefInput::<str, Cow<str>>::default();
  /// alnum_input.validators = Some(vec![
  ///   &alnum_only
  /// ]);
  ///
  /// let mut input_alnum_to_lower = RefInput::<str, Cow<str>>::default();
  /// input_alnum_to_lower.filters = Some(vec![&|value: Cow<str>| value.to_lowercase().into()]);
  /// input_alnum_to_lower.validators = Some(vec![&alnum_only]);
  ///
  /// let mut input_num_list = RefInput::<[u32], Vec<u32>>::default();
  ///
  /// // Disallow empty lists
  /// input_num_list.validators = Some(vec![&|value: &[u32]| if value.is_empty() {
  ///    Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is empty".to_string())]))
  /// } else {
  ///   Ok(())
  /// }]);
  ///
  /// // Transform to even numbers only
  /// input_num_list.filters = Some(vec![&|value: Vec<u32>| value.into_iter().filter(|v| v % 2 == 0).collect()]);
  ///
  /// // Test
  /// let value = vec![1, 2, 3, 4, 5, 6];
  /// assert_eq!(input_num_list.filter(&value).unwrap(), vec![2, 4, 6]);
  /// assert_eq!(input_num_list.filter(&vec![]), Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is empty".to_string())])));
  ///
  /// let value = "Hello, World!";
  ///
  /// assert_eq!(input.filter(value).unwrap(), Cow::Borrowed(value));
  /// assert_eq!(input2.filter(value).unwrap(), value.to_lowercase());
  /// assert_eq!(alnum_input.filter(value), Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())])));
  ///
  /// ```
  fn filter(&self, value: &'b T) -> Result<FT, ValidationErrType> {
    ValidateRef::validate_ref(self, value)?;

    Ok(self.filters.as_deref().map_or(value.into(), |filters| {
      filters
        .iter()
        .fold(value.into(), |agg, filter| (filter)(agg))
    }))
  }

  /// Validates, and filters, incoming Option value.
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
  /// // Create some validators
  /// let alnum_regex = regex::Regex::new(r"(?i)^[a-z\d]+$").unwrap();
  /// let alnum_only = move |value: &str| if alnum_regex.is_match(value) {
  ///     Ok(())
  ///   } else {
  ///     Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())]))
  ///   };
  ///
  /// // Create some input controls
  /// let mut input = RefInput::<str, Cow<str>>::default();
  ///
  /// let mut input2 = RefInput::<str, String>::default();
  /// input2.filters = Some(vec![&|value: String| value.to_lowercase()]);
  ///
  /// let mut alnum_input = RefInput::<str, Cow<str>>::default();
  /// alnum_input.validators = Some(vec![
  ///   &alnum_only
  /// ]);
  ///
  /// let mut input_alnum_to_lower = RefInput::<str, Cow<str>>::default();
  /// input_alnum_to_lower.filters = Some(vec![&|value: Cow<str>| value.to_lowercase().into()]);
  /// input_alnum_to_lower.validators = Some(vec![&alnum_only]);
  ///
  /// let mut input_num_list = RefInput::<[u32], Vec<u32>>::default();
  ///
  /// // Disallow empty lists
  /// input_num_list.validators = Some(vec![&|value: &[u32]| if value.is_empty() {
  ///    Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is empty".to_string())]))
  /// } else {
  ///   Ok(())
  /// }]);
  ///
  /// // Transform to even numbers only
  /// input_num_list.filters = Some(vec![&|value: Vec<u32>| value.into_iter().filter(|v| v % 2 == 0).collect()]);
  ///
  /// // Test
  /// let value = vec![1, 2, 3, 4, 5, 6];
  /// assert_eq!(input_num_list.filter_option(Some(&value)).unwrap(), Some(vec![2, 4, 6]));
  /// assert_eq!(input_num_list.filter_option(Some(&vec![])), Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is empty".to_string())])));
  ///
  /// let value = "Hello, World!";
  ///
  /// assert_eq!(input.filter_option(Some(value)).unwrap(), Some(Cow::Borrowed(value)));
  /// assert_eq!(input2.filter_option(Some(value)).unwrap(), Some(value.to_lowercase()));
  /// assert_eq!(alnum_input.filter_option(Some(value)), Err(ValidationErrType::Element(vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())])));
  /// ```
  fn filter_option(&self, value: Option<&'b T>) -> Result<Option<FT>, ValidationErrType> {
    match value {
      Some(value) => self.filter(value).map(Some),
      None => {
        if self.required {
          Err(ValidationErrType::Element(vec![Violation(
            ValueMissing,
            (self.value_missing_msg_getter)(self),
          )]))
        } else {
          Ok(self.get_default_value.and_then(|f| f()))
        }
      }
    }
  }
}
