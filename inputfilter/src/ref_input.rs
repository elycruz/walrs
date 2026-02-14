use crate::traits::FilterFn;
use crate::ViolationType::ValueMissing;
use crate::{FilterForUnsized, ValidatorForRef, Violation, ViolationMessage, Violations};
use std::fmt::{Debug, Display, Formatter};

/// Returns a generic message for "Value is missing" violation.
///
/// ```rust
/// use std::borrow::Cow;
/// use walrs_inputfilter::{ref_input_value_missing_msg_getter, RefInput, value_missing_msg_getter};
///
/// let input = RefInput::<str, Cow<str>>::default();
///
/// assert_eq!(ref_input_value_missing_msg_getter(&input), "Value is missing".to_string());
/// ```
pub fn ref_input_value_missing_msg_getter<'a, 'b, T, FT>(
  _: &RefInput<'a, 'b, T, FT>,
) -> ViolationMessage
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  "Value is missing".to_string()
}

#[must_use]
#[derive(Clone)]
pub struct RefInput<'a, 'b, T, FT = T>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  /// Controls whether to run through all contained validators despite there being
  /// failures/violations and/or to stop on the first failing one.
  pub break_on_failure: bool,

  /// Whether value to be validated is required or not - Relevant only when using `*_option`
  /// methods.
  pub required: bool,

  /// Field for setting only one validator - Saves bytes when need only one validator versus
  /// using `validators` field (which requires a `Vec`).
  pub custom: Option<&'a ValidatorForRef<T>>,

  // @todo This should probably be an `Option<Cow<str>>` instead.
  /// Optional locale - Useful in validation "violation" message contexts.  Composed by the user.
  pub locale: Option<&'a str>,

  // @todo This should be an `Option<Cow<str>>` instead.
  /// Optional name - Useful in validation "violation" message contexts.  Composed by the user.
  pub name: Option<&'a str>,

  /// Returns a default value for the "input is not required, but is empty" use case.
  pub get_default_value: Option<&'a (dyn Fn() -> Option<FT> + Send + Sync)>,

  /// Validator functions to call on value to be validated.
  pub validators: Option<Vec<&'a ValidatorForRef<T>>>,

  /// Transformation functions to subsequently pass validated value through.
  pub filters: Option<Vec<&'a FilterFn<FT>>>,

  /// Supplies the error message returned by `validate_ref_option`, and/or `filter_option`,
  /// methods when the parent ref input contains `required = true` and the incoming value is `None`.
  pub value_missing_msg_getter:
    &'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync),
}

impl<'b, T, FT> RefInput<'_, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  /// Returns a new `RefInput` instance.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{
  ///   RefInput,
  ///   ref_input_value_missing_msg_getter,
  ///   ViolationType
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
  /// assert!(input.locale.is_none());
  /// assert!(input.get_default_value.is_none());
  /// assert!(input.validators.is_none());
  /// assert!(input.filters.is_none());
  /// assert_eq!(
  ///   (&input.value_missing_msg_getter)(&input),
  ///   ref_input_value_missing_msg_getter(&input)
  /// );
  /// ```
  pub fn new() -> Self {
    RefInput::default()
  }
}

impl<'b, T: ?Sized + 'b, FT: From<&'b T>> Default for RefInput<'_, 'b, T, FT> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{
  ///   RefInput,
  ///   ref_input_value_missing_msg_getter,
  ///   ViolationType
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
  /// assert!(input.locale.is_none());
  /// assert!(input.get_default_value.is_none());
  /// assert!(input.validators.is_none());
  /// assert!(input.filters.is_none());
  /// assert_eq!(
  ///   (&input.value_missing_msg_getter)(&input),
  ///   ref_input_value_missing_msg_getter(&input)
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
      value_missing_msg_getter: &ref_input_value_missing_msg_getter,
    }
  }
}

impl<'b, T, FT> Display for RefInput<'_, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

#[cfg(feature = "debug_closure_helpers")]
impl<'b, T, FT> Debug for RefInput<'_, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RefInput")
      .field("break_on_failure", &self.break_on_failure)
      .field("required", &self.required)
      .field_with("custom", |fmtr| {
        let val = if self.custom.is_some() {
          "Some(&ValidatorForRef)"
        } else {
          "None"
        };
        fmtr.write_str(val).expect("value write to succeed");
        Ok(())
      })
      .field("locale", &self.locale)
      .field("name", &self.name)
      .field_with("get_default_value", |fmtr| {
        let val = if self.get_default_value.is_some() {
          "Some(&dyn Fn() -> Option<FT> + Send + Sync)"
        } else {
          "None"
        };
        fmtr.write_str(val).expect("value write to succeed");
        Ok(())
      })
      .field_with("validators", |fmtr| {
        let val = if let Some(vs) = self.validators.as_deref() {
          format!("Some(Vec<&ValidatorForRef>{{ len: {} }})", vs.len())
        } else {
          "None".to_string()
        };
        fmtr.write_str(&val).expect("value write to succeed");
        Ok(())
      })
      .field_with("filters", |fmtr| {
        let val = if let Some(fs) = self.filters.as_deref() {
          format!("Some(Vec<&FilterFn>{{ len: {} }})", fs.len())
        } else {
          "None".to_string()
        };
        fmtr.write_str(&val).expect("value write to succeed");
        Ok(())
      })
      .finish()
  }
}

#[cfg(not(feature = "debug_closure_helpers"))]
impl<'b, T, FT> Debug for RefInput<'_, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let custom_str = if self.custom.is_some() {
      "Some(&ValidatorForRef)"
    } else {
      "None"
    };
    let get_default_value_str = if self.get_default_value.is_some() {
      "Some(&dyn Fn() -> Option<FT> + Send + Sync)"
    } else {
      "None"
    };
    let validators_str = if let Some(vs) = self.validators.as_deref() {
      format!("Some(Vec<&ValidatorForRef>{{ len: {} }})", vs.len())
    } else {
      "None".to_string()
    };
    let filters_str = if let Some(fs) = self.filters.as_deref() {
      format!("Some(Vec<&FilterFn>{{ len: {} }})", fs.len())
    } else {
      "None".to_string()
    };

    f.debug_struct("RefInput")
      .field("break_on_failure", &self.break_on_failure)
      .field("required", &self.required)
      .field("custom", &custom_str)
      .field("locale", &self.locale)
      .field("name", &self.name)
      .field("get_default_value", &get_default_value_str)
      .field("validators", &validators_str)
      .field("filters", &filters_str)
      .finish()
  }
}

impl<'b, T, FT> FilterForUnsized<'b, T, FT> for RefInput<'_, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  /// Validates given value.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForUnsized, RefInput, RefInputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::TypeMismatch;
  ///
  /// let input = RefInputBuilder::<str, String>::default()
  ///   .required(true)
  ///   .validators(vec![
  ///     &|value: &str| if value.len() > 5 {
  ///       Ok(())
  ///     } else {
  ///       Err(Violation(TypeMismatch, "Value is too short".to_string()))
  ///     }
  ///   ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.validate_ref_detailed("Hello, World!"), Ok(()));
  /// assert_eq!(input.validate_ref_detailed("Hi!"), Err(Violations(vec![Violation(TypeMismatch, "Value is too short".to_string())])));
  /// // `Violations`, and `Violation` are tuple types,  E.g., inner elements can be accessed
  /// //   with tuple enumeration syntax (`tuple.0`, `tuple.1` etc), additionally there are `Deref`
  /// //   impls on them for easily accessing their inner items.
  /// assert_eq!(input.validate_ref_detailed(""), Err(Violations(vec![Violation(TypeMismatch, "Value is too short".to_string())])));
  /// ```
  fn validate_ref_detailed(&self, value: &T) -> Result<(), Violations> {
    let mut violations = vec![];

    // Validate custom
    match if let Some(custom) = self.custom {
      (custom)(value)
    } else {
      Ok(())
    } {
      Ok(()) => (),
      Err(err_type) => violations.push(err_type),
    }

    if !violations.is_empty() && self.break_on_failure {
      return Err(Violations(violations));
    }

    // Else validate against validators
    if let Some(validators) = self.validators.as_deref() {
      for validator in validators {
        if let Err(err_type) = validator(value) {
          violations.push(err_type);
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
      Err(Violations(violations))
    }
  }

  /// Validates given "optional" value and returns detailed validation results if any violations
  /// occur.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForUnsized, RefInput, RefInputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, ValueMissing};
  ///
  /// let input = RefInputBuilder::<str, String>::default()
  ///   .required(true)
  ///   .validators(vec![
  ///     &|value: &str| if value.len() > 5 {
  ///       Ok(())
  ///     } else {
  ///       Err(Violation(TypeMismatch, "Value is too short".to_string()))
  ///     }
  ///   ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.validate_ref_option_detailed(Some("Hello, World!")), Ok(()));
  /// assert_eq!(input.validate_ref_option_detailed(Some("Hi!")), Err(Violations(vec![Violation(TypeMismatch, "Value is too short".to_string())])));
  /// assert_eq!(input.validate_ref_option_detailed(Some("")), Err(Violations(vec![Violation(TypeMismatch, "Value is too short".to_string())])));
  /// assert_eq!(input.validate_ref_option_detailed(None), Err(Violations(vec![Violation(ValueMissing, "Value is missing".to_string())])));
  /// ```
  fn validate_ref_option_detailed(&self, value: Option<&T>) -> Result<(), Violations> {
    match value {
      Some(v) => self.validate_ref_detailed(v),
      None => {
        if self.required {
          Err(Violations(vec![Violation(
            ValueMissing,
            (self.value_missing_msg_getter)(self),
          )]))
        } else {
          Ok(())
        }
      }
    }
  }

  /// Validates, and filters, incoming value, and returns detailed validation violations, if any.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{
  ///     RefInput,
  ///     FilterForUnsized,
  ///     ViolationType::TypeMismatch,
  ///     ViolationMessage,
  ///     Violation,
  ///     Violations
  /// };
  ///
  /// // Create some validators
  /// let alnum_regex = regex::Regex::new(r"(?i)^[a-z\d]+$").unwrap();
  /// let alnum_only = move |value: &str| if alnum_regex.is_match(value) {
  ///     Ok(())
  ///   } else {
  ///     Err(Violation(TypeMismatch, "Value is not alpha-numeric".to_string()))
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
  ///    Err(Violation(TypeMismatch, "Value is empty".to_string()))
  /// } else {
  ///   Ok(())
  /// }]);
  ///
  /// // Transform to even numbers only
  /// input_num_list.filters =
  ///     Some(vec![&|value: Vec<u32>| value.into_iter().filter(|v| v % 2 == 0).collect()]);
  ///
  /// // Test
  /// let value = vec![1, 2, 3, 4, 5, 6];
  /// assert_eq!(input_num_list.filter_ref_detailed(&value).unwrap(), vec![2, 4, 6]);
  /// assert_eq!(input_num_list.filter_ref_detailed(&vec![]), Err(
  ///     Violations(vec![Violation(TypeMismatch, "Value is empty".to_string())])
  /// ));
  ///
  /// let value = "Hello, World!";
  ///
  /// assert_eq!(input.filter_ref_detailed(value).unwrap(), Cow::Borrowed(value));
  /// assert_eq!(input2.filter_ref_detailed(value).unwrap(), value.to_lowercase());
  /// assert_eq!(alnum_input.filter_ref_detailed(value), Err(
  ///     Violations(vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())])
  /// ));
  ///
  /// ```
  fn filter_ref_detailed(&self, value: &'b T) -> Result<FT, Violations> {
    self.validate_ref_detailed(value)?;

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
  ///     RefInput,
  ///     FilterForUnsized, RefInputBuilder, Violation,
  ///     ViolationType::TypeMismatch,
  ///     ViolationMessage,
  ///     Violations
  /// };
  ///
  /// // Create some validators
  /// let alnum_regex = regex::Regex::new(r"(?i)^[a-z\d]+$").unwrap();
  /// let alnum_only = move |value: &str| if alnum_regex.is_match(value) {
  ///     Ok(())
  ///   } else {
  ///     Err(Violation(TypeMismatch, "Value is not alpha-numeric".to_string()))
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
  ///    Err(Violation(TypeMismatch, "Value is empty".to_string()))
  /// } else {
  ///   Ok(())
  /// }]);
  ///
  /// // Transform to even numbers only
  /// input_num_list.filters = Some(vec![&|value: Vec<u32>| value.into_iter().filter(|v| v % 2 == 0).collect()]);
  ///
  /// // Test
  /// let value = vec![1, 2, 3, 4, 5, 6];
  /// assert_eq!(input_num_list.filter_ref_option_detailed(Some(&value)).unwrap(), Some(vec![2, 4, 6]));
  /// assert_eq!(input_num_list.filter_ref_option_detailed(Some(&vec![])), Err(
  ///     Violations(vec![Violation(TypeMismatch, "Value is empty".to_string())]))
  /// );
  ///
  /// let value = "Hello, World!";
  ///
  /// assert_eq!(input.filter_ref_option_detailed(Some(value)).unwrap(), Some(Cow::Borrowed(value)));
  /// assert_eq!(input2.filter_ref_option_detailed(Some(value)).unwrap(), Some(value.to_lowercase()));
  /// assert_eq!(alnum_input.filter_ref_option_detailed(Some(value)), Err(
  ///     Violations(vec![Violation(TypeMismatch, "Value is not alpha-numeric".to_string())]))
  /// );
  /// ```
  fn filter_ref_option_detailed(&self, value: Option<&'b T>) -> Result<Option<FT>, Violations> {
    match value {
      Some(value) => self.filter_ref_detailed(value).map(Some),
      None => {
        if self.required {
          Err(Violations(vec![Violation(
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

/// Ref Input builder.
///
/// ```rust
/// use walrs_inputfilter::{RefInput, RefInputBuilder, ValidatorForRef};
/// use std::borrow::Cow;
///
/// let input = RefInputBuilder::<str, Cow<str>>::default()
///    .break_on_failure(true)
///    .required(true)
///    .custom(&|_: &str| Ok(()))
///    .locale("en_US")
///    .name("name")
///    .get_default_value(&|| Some(Cow::Borrowed("default")))
///    .validators(vec![&|_: &str| Ok(())])
///    .filters(vec![&|_: Cow<str>| Cow::Borrowed("filtered")])
///    .value_missing_msg_getter(&|_: &RefInput<str, Cow<str>>| "Value is missing".to_string())
///    .build()
///    .unwrap();
///
/// // Result
/// // ----
/// assert_eq!(input.break_on_failure, true);
/// assert_eq!(input.required, true);
/// assert!(input.custom.is_some());
/// assert_eq!(input.locale, Some("en_US"));
/// assert_eq!(input.name, Some("name"));
/// assert!(input.get_default_value.is_some());
/// assert_eq!(input.validators.as_deref().unwrap().len(), 1usize);
/// assert_eq!(input.filters.as_deref().unwrap().len(), 1usize);
/// assert_eq!(
///  (&input.value_missing_msg_getter)(&input),
///  "Value is missing".to_string()
/// );
/// ```
pub struct RefInputBuilder<'a, 'b, T, FT>
where
  T: ?Sized + 'a,
  FT: From<&'b T>,
{
  break_on_failure: Option<bool>,
  required: Option<bool>,
  custom: Option<&'a ValidatorForRef<T>>,
  locale: Option<&'a str>,
  name: Option<&'a str>,
  get_default_value: Option<&'a (dyn Fn() -> Option<FT> + Send + Sync)>,
  validators: Option<Vec<&'a ValidatorForRef<T>>>,
  filters: Option<Vec<&'a FilterFn<FT>>>,
  value_missing_msg_getter:
    Option<&'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync)>,
}

impl<'a, 'b, T, FT> RefInputBuilder<'a, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  pub fn break_on_failure(&mut self, break_on_failure: bool) -> &mut Self {
    self.break_on_failure = Some(break_on_failure);
    self
  }

  pub fn required(&mut self, required: bool) -> &mut Self {
    self.required = Some(required);
    self
  }

  pub fn custom(&mut self, custom: &'a ValidatorForRef<T>) -> &mut Self {
    self.custom = Some(custom);
    self
  }

  pub fn locale(&mut self, locale: &'a str) -> &mut Self {
    self.locale = Some(locale);
    self
  }

  pub fn name(&mut self, name: &'a str) -> &mut Self {
    self.name = Some(name);
    self
  }

  pub fn get_default_value(
    &mut self,
    get_default_value: &'a (dyn Fn() -> Option<FT> + Send + Sync),
  ) -> &mut Self {
    self.get_default_value = Some(get_default_value);
    self
  }

  pub fn validators(&mut self, validators: Vec<&'a ValidatorForRef<T>>) -> &mut Self {
    self.validators = Some(validators);
    self
  }

  pub fn filters(&mut self, filters: Vec<&'a FilterFn<FT>>) -> &mut Self {
    self.filters = Some(filters);
    self
  }

  pub fn value_missing_msg_getter(
    &mut self,
    value_missing_msg_getter: &'a (dyn Fn(&RefInput<'a, 'b, T, FT>) -> ViolationMessage
           + Send
           + Sync),
  ) -> &mut Self {
    self.value_missing_msg_getter = Some(value_missing_msg_getter);
    self
  }

  pub fn build(&mut self) -> Result<RefInput<'a, 'b, T, FT>, String> {
    Ok(RefInput {
      break_on_failure: self.break_on_failure.unwrap_or(false),
      required: self.required.unwrap_or(false),
      custom: self.custom,
      locale: self.locale,
      name: self.name,
      get_default_value: self.get_default_value,
      validators: self.validators.clone(),
      filters: self.filters.clone(),
      value_missing_msg_getter: self
        .value_missing_msg_getter
        .unwrap_or(&ref_input_value_missing_msg_getter),
    })
  }
}

impl<'b, T, FT> Default for RefInputBuilder<'_, 'b, T, FT>
where
  T: ?Sized + 'b,
  FT: From<&'b T>,
{
  fn default() -> Self {
    RefInputBuilder {
      break_on_failure: None,
      required: None,
      custom: None,
      locale: None,
      name: None,
      get_default_value: None,
      validators: None,
      filters: None,
      value_missing_msg_getter: None,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::ViolationType::{PatternMismatch, ValueMissing};
  use regex::Regex;
  use std::borrow::Cow;

  #[test]
  fn test_builder() {
    // Test default builder
    // ----
    let default_input = RefInputBuilder::<str, Cow<str>>::default().build().unwrap();

    // Test result
    // ----
    assert!(!default_input.break_on_failure);
    assert!(!default_input.required);
    assert!(default_input.custom.is_none());
    assert!(default_input.locale.is_none());
    assert!(default_input.name.is_none());
    assert!(default_input.get_default_value.is_none());
    assert!(default_input.validators.is_none());
    assert!(default_input.filters.is_none());
    assert_eq!(
      (default_input.value_missing_msg_getter)(&default_input),
      "Value is missing".to_string()
    );

    // Test builder with values
    // ----
    let input = RefInputBuilder::<str, Cow<str>>::default()
      .break_on_failure(true)
      .required(true)
      .custom(&|_: &str| Ok(()))
      .locale("en_US")
      .name("name")
      .get_default_value(&|| Some(Cow::Borrowed("default")))
      .validators(vec![&|_: &str| Ok(())])
      .filters(vec![&|_: Cow<str>| Cow::Borrowed("filtered")])
      .value_missing_msg_getter(&|_: &RefInput<'_, '_, str, Cow<str>>| {
        "Value is missing".to_string()
      })
      .build()
      .unwrap();

    // Test result
    // ----
    assert!(input.break_on_failure);
    assert!(input.required);
    assert!(input.custom.is_some());
    assert_eq!(input.locale, Some("en_US"));
    assert_eq!(input.name, Some("name"));
    assert!(input.get_default_value.is_some());
    assert!(input.validators.is_some());
    assert!(input.validators.is_some());
    assert!(input.filters.is_some());
    assert_eq!(
      (input.value_missing_msg_getter)(&input),
      "Value is missing".to_string()
    );
  }

  #[test]
  fn test_debug_and_display() {
    println!("Testing \"Debug\" and \"Display\"");

    let input = RefInputBuilder::<str, Cow<str>>::default().build().unwrap();
    println!("{:#?}", &input);
    println!("{}", &input);

    // Input with values
    // ----
    let input = RefInputBuilder::<str, Cow<str>>::default()
      .break_on_failure(true)
      .required(true)
      .custom(&|_: &str| Ok(()))
      .locale("en_US")
      .name("name")
      .get_default_value(&|| Some(Cow::Borrowed("default")))
      .validators(vec![&|_: &str| Ok(())])
      .filters(vec![&|_: Cow<str>| Cow::Borrowed("filtered")])
      .value_missing_msg_getter(&|_: &RefInput<'_, '_, str, Cow<str>>| {
        "Value is missing".to_string()
      })
      .build()
      .unwrap();

    println!("{:#?}", &input);

    // Test Display
    println!("{}", &input);
  }

  #[test]
  fn test_e2e() {
    // Prepare validators and filters
    // ----
    // Alnum validator
    let alnum_regex = Regex::new(r"(?i)^[a-z\d]+$").unwrap();
    fn get_alnum_violation() -> Violation {
      Violation(PatternMismatch, "Expected only alnum".to_string())
    }
    let validate_alnum = move |s: &str| -> Result<(), Violation> {
      if alnum_regex.is_match(s) {
        Ok(())
      } else {
        Err(get_alnum_violation())
      }
    };

    // Vowels validator
    let vowels_regex = Regex::new(r"(?i)^[aeiou]+$").unwrap();
    fn get_vowels_violation() -> Violation {
      Violation(PatternMismatch, "Expected only vowels".to_string())
    }
    let validate_only_vowels = move |s: &str| -> Result<(), Violation> {
      if vowels_regex.is_match(s) {
        Ok(())
      } else {
        Err(get_vowels_violation())
      }
    };

    fn get_empty_value_violation() -> Violation {
      Violation(ValueMissing, "Value empty".to_string())
    }
    let validate_not_empty = |s: &str| -> Result<(), Violation> {
      if !s.is_empty() {
        Ok(())
      } else {
        Err(get_empty_value_violation())
      }
    };

    let test_cases: Vec<(
      &str,
      RefInput<str, Cow<str>>,
      &str,
      Result<Cow<str>, Violations>,
    )> = vec![
      // Default
      // ----
      (
        "Default, with value",
        RefInputBuilder::<str, Cow<str>>::default().build().unwrap(),
        "abc",
        Ok("abc".into()),
      ),
      (
        "Default, with empty value",
        RefInputBuilder::<str, Cow<str>>::default().build().unwrap(),
        "",
        Ok("".into()),
      ),
      // Via `custom` field
      // ----
      (
        "Validate 'not empty' (via 'custom', with valid value)",
        RefInputBuilder::<str, Cow<str>>::default()
          .custom(&validate_not_empty)
          .build()
          .unwrap(),
        "abc",
        Ok("abc".into()),
      ),
      (
        "Validate 'not empty' (via 'custom', with invalid value)",
        RefInputBuilder::<str, Cow<str>>::default()
          .custom(&validate_not_empty)
          .build()
          .unwrap(),
        "",
        Err(Violations(vec![get_empty_value_violation()])),
      ),
      // Via `validators` field
      // ----
      (
        "Validate 'not empty' (via 'validators', with valid value)",
        RefInputBuilder::<str, Cow<str>>::default()
          .validators(vec![&validate_not_empty])
          .build()
          .unwrap(),
        "abc",
        Ok("abc".into()),
      ),
      (
        "Validate 'not empty' (via 'validators', with invalid value)",
        RefInputBuilder::<str, Cow<str>>::default()
          .validators(vec![&validate_not_empty])
          .build()
          .unwrap(),
        "",
        Err(Violations(vec![get_empty_value_violation()])),
      ),
      // Via `validators` field, with multiple validators
      // ----
      (
        "Validate with multiple 'validators', and valid value",
        RefInputBuilder::<str, Cow<str>>::default()
          .validators(vec![&validate_only_vowels, &validate_alnum])
          .build()
          .unwrap(),
        "aeiou",
        Ok("aeiou".into()),
      ),
      (
        "Validate with multiple 'validators', and invalid value",
        RefInputBuilder::<str, Cow<str>>::default()
          .validators(vec![&validate_only_vowels, &validate_alnum])
          .build()
          .unwrap(),
        "z#abce",
        Err(Violations(vec![
          get_vowels_violation(),
          get_alnum_violation(),
        ])),
      ),
      // With `custom`, and multiple, validators
      (
        "Validate with 'custom' validator, and multiple 'validators', and valid value",
        RefInputBuilder::<str, Cow<str>>::default()
          .custom(&validate_not_empty)
          .validators(vec![&validate_only_vowels, &validate_alnum])
          .build()
          .unwrap(),
        "aeiou",
        Ok("aeiou".into()),
      ),
      (
        "Validate with 'custom' validator, and multiple 'validators', and invalid value",
        RefInputBuilder::<str, Cow<str>>::default()
          .custom(&validate_not_empty)
          .validators(vec![&validate_only_vowels, &validate_alnum])
          .build()
          .unwrap(),
        "",
        Err(Violations(vec![
          get_empty_value_violation(),
          get_vowels_violation(),
          get_alnum_violation(),
        ])),
      ),
      // With `break_on_failure`
      // ----
      (
        "Break on failure with multiple 'validators'",
        RefInputBuilder::<str, Cow<str>>::default()
          .break_on_failure(true)
          .validators(vec![&validate_only_vowels, &validate_alnum])
          .build()
          .unwrap(),
        "z#abce",
        Err(Violations(vec![get_vowels_violation()])),
      ),
      (
        "Break on failure with 'custom' validator, and multiple 'validators'",
        RefInputBuilder::<str, Cow<str>>::default()
          .break_on_failure(true)
          .custom(&validate_not_empty)
          .validators(vec![&validate_only_vowels, &validate_alnum])
          .build()
          .unwrap(),
        "",
        Err(Violations(vec![get_empty_value_violation()])),
      ),
    ];

    for (i, (test_name, input, subj, expected)) in test_cases.into_iter().enumerate() {
      println!("Case {}: {}", i + 1, test_name);

      match expected {
        Err(violations) => {
          let msgs_vec = violations.clone().to_string_vec();
          assert_eq!(input.validate_ref(subj), Err(msgs_vec.clone()));
          assert_eq!(input.validate_ref_detailed(subj), Err(violations.clone()));
          assert_eq!(input.validate_ref_option(Some(subj)), Err(msgs_vec.clone()));
          assert_eq!(
            input.validate_ref_option_detailed(Some(subj)),
            Err(violations.clone())
          );
          assert_eq!(input.filter_ref(subj), Err(msgs_vec.clone()));
          assert_eq!(input.filter_ref_detailed(subj), Err(violations.clone()));
          assert_eq!(input.filter_ref_option(Some(subj)), Err(msgs_vec.clone()));
          assert_eq!(
            input.filter_ref_option_detailed(Some(subj)),
            Err(violations.clone())
          );
        }
        Ok(value) => {
          assert_eq!(input.validate_ref(subj), Ok(()));
          assert_eq!(input.validate_ref_detailed(subj), Ok(()));
          assert_eq!(input.validate_ref_option(Some(subj)), Ok(()));
          assert_eq!(input.validate_ref_option_detailed(Some(subj)), Ok(()));
          assert_eq!(input.filter_ref(subj), Ok(value.clone()));
          assert_eq!(input.filter_ref_detailed(subj), Ok(value.clone()));
          assert_eq!(input.filter_ref_option(Some(subj)), Ok(Some(value.clone())));
          assert_eq!(
            input.filter_ref_option_detailed(Some(subj)),
            Ok(Some(value.clone()))
          );
        }
      }
    }

    // Validate required value, with "None" value;  E.g., should always return "one" error message
    // ----
    let required_input = RefInputBuilder::<str, Cow<str>>::default()
      .required(true)
      .build()
      .unwrap();
    assert_eq!(
      required_input.validate_ref_option(None),
      Err(vec![ref_input_value_missing_msg_getter(&required_input)])
    );
    assert_eq!(
      required_input.validate_ref_option_detailed(None),
      Err(Violations(vec![Violation(
        ValueMissing,
        ref_input_value_missing_msg_getter(&required_input)
      )]))
    );
    assert_eq!(
      required_input.filter_ref_option(None),
      Err(vec![ref_input_value_missing_msg_getter(&required_input)])
    );
    assert_eq!(
      required_input.filter_ref_option_detailed(None),
      Err(Violations(vec![Violation(
        ValueMissing,
        ref_input_value_missing_msg_getter(&required_input)
      )]))
    );
  }
}
