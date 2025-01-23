use crate::{
  FilterFn, FilterForSized, ValidatorForSized, Violation, ViolationMessage,
  ViolationType::ValueMissing, Violations,
};

use std::fmt::{Debug, Display, Formatter};

/// Returns a generic message for "Value is missing" violation.
///
/// ```rust
/// use walrs_inputfilter::{Input, value_missing_msg_getter};
///
/// let input = Input::<usize, usize>::new();
///
/// assert_eq!(value_missing_msg_getter(&input), "Value is missing".to_string());
/// ```
pub fn value_missing_msg_getter<T: Copy, FT: From<T>>(_: &Input<T, FT>) -> ViolationMessage {
  "Value is missing".to_string()
}

/// Validation struct for validating, and/or filtering (validating and transforming) `Copy + Sized` types.
///
/// ```rust
/// use walrs_inputfilter::{FilterForSized, value_missing_msg_getter, Input, InputBuilder, Violation, Violations};
/// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
///
/// let vowels = "aeiou";
/// let vowel_validator = &|value: char| if vowels.contains(value) {
///   Ok(())
/// } else {
///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
/// };
///
/// // Generics: type to validate and type returned within `filter*` functions.
/// let input = InputBuilder::<char, char>::default()
///   .required(true)
///   .validators(vec![ vowel_validator ])
///   .build()
///   .unwrap();
///
/// let even_num_validator = |x: usize| if x & 1 == 0 {
///   Ok(())
/// } else {
///   Err(Violation(StepMismatch, format!("{} is Odd", x)))
/// };
///
/// let num_input = InputBuilder::<usize, usize>::default()
///   .required(true)
///   .validators(vec![&even_num_validator])
///   .build()
///   .unwrap();
///
/// // With filters
/// let always_a_vowel = |x: char| if !vowels.contains(x) { 'e' } else { x };
/// let always_e = || Some('e');
/// let always_a_vowel_input = InputBuilder::<char, char>::default()
///   // Triggered from resulting struct's `filter_option*` methods, which
///   // validate, and filter, incoming value
///   .get_default_value(&always_e)
///   .filters(vec![&always_a_vowel])
///   .build()
///   .unwrap();
///
/// // Test
/// assert_eq!(input.filter('a'), Ok('a'));
/// assert_eq!(input.filter('b'), Err(vec!["Only vowels allowed".to_string()]));
/// assert_eq!(always_a_vowel_input.filter('b'), Ok('e'));
/// assert_eq!(always_a_vowel_input.filter_option(None), Ok(Some('e')));
/// assert_eq!(always_a_vowel_input.filter_option(Some('b')), Ok(Some('e')));
/// // Num input
/// assert_eq!(num_input.filter(2), Ok(2));
/// assert_eq!(num_input.filter(1), Err(vec!["1 is Odd".to_string()]));
///
/// // Optional Values
/// assert_eq!(input.filter_option(None), Err(vec![value_missing_msg_getter(&input)]));
/// assert_eq!(input.filter_option(Some('a')), Ok(Some('a')));
/// assert_eq!(input.filter_option(Some('b')), Err(vec!["Only vowels allowed".to_string()]));
///
/// // Detailed violation Results
/// assert_eq!(input.filter_detailed('a'), Ok('a'));
/// assert_eq!(input.filter_detailed('b'), Err(Violations(vec![Violation(TypeMismatch, "Only vowels allowed".to_string())])));
///
/// // Detailed violation Results for optional value
/// assert_eq!(input.filter_option_detailed(Some('a')), Ok(Some('a')));
/// assert_eq!(input.filter_option_detailed(Some('b')), Err(Violations(vec![Violation(TypeMismatch, "Only vowels allowed".to_string())])));
///
/// // If just validating values (not applying transformations (filtering) or normalization):
/// assert_eq!(input.validate('a'), Ok(()));
/// assert_eq!(input.validate('b'), Err(vec!["Only vowels allowed".to_string()]));
/// // and/or, use other `validate*` methods ...
/// ```
#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct Input<'a, T, FilterT = T>
where
  T: Copy,
  FilterT: From<T>,
{
  /// Causes validation to stop at the first encountered error, and triggers early exit
  /// causing the encountered error to be returned.
  #[builder(default = "false")]
  pub break_on_failure: bool,

  /// Denotes whether value being validated is required or not.
  #[builder(default = "false")]
  pub required: bool,

  /// To be used when only a single validator is needed;  Avoids additional
  /// allocations that happen when using `validators` Vec.
  #[builder(default = "None")]
  pub custom: Option<&'a ValidatorForSized<T>>,

  /// Locale to be used in user-land validation error message getters.
  #[builder(default = "None")]
  pub locale: Option<&'a str>,

  /// Used to communicate input name in user-land use cases.
  #[builder(default = "None")]
  pub name: Option<&'a str>,

  /// Returns a default value from `filter_option*` methods, when "optional" value to be filtered
  ///   is `None`.
  #[builder(default = "None")]
  pub get_default_value: Option<&'a (dyn Fn() -> Option<FilterT> + Send + Sync)>,

  /// Validators to apply when validating, and/or filtering, values.
  #[builder(default = "None")]
  pub validators: Option<Vec<&'a ValidatorForSized<T>>>,

  /// List of transformations to apply on value being filtered.
  #[builder(default = "None")]
  pub filters: Option<Vec<&'a FilterFn<FilterT>>>,

  /// Triggered when value being validated is "required" and is missing (triggered from *_option
  ///  filter, and/or validation, methods).
  #[builder(default = "&value_missing_msg_getter")]
  pub value_missing_msg_getter:
    &'a (dyn Fn(&Input<'a, T, FilterT>) -> ViolationMessage + Send + Sync),
}

/// Input filter struct - used for keeping rules used for validating, and/or filtering values.
/// @param `T` - value type to be validated, 
/// @param `FT` - value type to be filtered.
impl<T: Copy, FT: From<T>> Input<'_, T, FT> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{ Input, value_missing_msg_getter };
  ///
  /// // Using copy types (`Input::<type-to-validate, type-returned-from-filter*-fns>`, send generic
  /// // must fit `From<arg-1>`).
  /// let _ = Input::<&str, Cow<str>>::new();
  /// let _ = Input::<char, char>::new();
  /// let _ = Input::<bool, bool>::new();
  /// let input = Input::<usize, usize>::new();
  ///
  /// // Assert defaults
  /// // ----
  /// assert_eq!(input.break_on_failure, false);
  /// assert_eq!(input.required, false);
  /// assert!(input.custom.is_none());
  /// assert_eq!(input.locale, None);
  /// assert_eq!(input.name, None);
  /// assert!(input.get_default_value.is_none());
  /// assert!(input.validators.is_none());
  /// assert!(input.filters.is_none());
  /// assert_eq!(
  ///   (&input.value_missing_msg_getter)(&input),
  ///   value_missing_msg_getter(&input)
  /// );
  /// ```
  pub fn new() -> Self {
    Input::default()
  }
}

impl<T: Copy, FT: From<T>> FilterForSized<T, FT> for Input<'_, T, FT> {
  /// Validates given value and returns detailed violation results on violation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///       Ok(())
  ///     } else {
  ///       Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  ///     };
  ///
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.validate_detailed('a'), Ok(()));
  /// // `Violations`, and `Violation` are tuple types,  E.g., inner elements can be accessed
  /// //   with tuple enumeration syntax (`tuple.0`, `tuple.1` etc), additionally there are `Deref`
  /// //   impls on them for easily accessing their inner items.
  /// assert_eq!(input.validate_detailed('b'), Err(Violations(vec![Violation(
  ///   TypeMismatch,
  ///   "Only vowels allowed".to_string()
  /// )])));
  /// ```
  fn validate_detailed(&self, value: T) -> Result<(), Violations> {
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
    self.validators.as_deref().map_or(Ok(()), |validators| {
      for validator in validators {
        match validator(value) {
          Ok(()) => continue,
          Err(err_type) => {
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
    })
  }

  /// Validates given value returning violation results on violation
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
  /// // Generics: type to validate and type returned within `filter*` functions.
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.validate('a'), Ok(()));
  /// assert_eq!(input.validate('b'), Err(vec!["Only vowels allowed".to_string()]));
  /// ```
  fn validate(&self, value: T) -> Result<(), Vec<ViolationMessage>> {
    match self.validate_detailed(value) {
      Ok(()) => Ok(()),
      Err(violations) => Err(violations.to_string_vec()),
    }
  }

  /// Validates given optional value and returns detailed violation results on violation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{value_missing_msg_getter, FilterForSized, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{ValueMissing, TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  /// // Generics: type to validate, type returned within `filter*` functions.
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.validate_option_detailed(None), Err(Violations(vec![Violation(ValueMissing, value_missing_msg_getter(&input))])));
  /// assert_eq!(input.validate_option_detailed(Some('a')), Ok(()));
  /// assert_eq!(input.validate_option_detailed(Some('b')), Err(Violations(vec![Violation(TypeMismatch, "Only vowels allowed".to_string())])));
  /// ```
  fn validate_option_detailed(&self, value: Option<T>) -> Result<(), Violations> {
    match value {
      Some(v) => self.validate_detailed(v),
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

  /// Validates given "optional" value.
  ///
  /// ```rust
  /// use walrs_inputfilter::{value_missing_msg_getter, FilterForSized, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  /// // Generics: type to validate, type returned within `filter*` functions.
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.validate_option(None), Err(vec![value_missing_msg_getter(&input)]));
  /// assert_eq!(input.validate_option(Some('a')), Ok(()));
  /// assert_eq!(input.validate_option(Some('b')), Err(vec!["Only vowels allowed".to_string()]));
  /// ```
  fn validate_option(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>> {
    match self.validate_option_detailed(value) {
      Ok(()) => Ok(()),
      Err(violations) => Err(violations.to_string_vec()),
    }
  }

  /// Validates and transforms given value, and returns transformed value on validation success,
  /// and/or detailed violation details on violation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, value_missing_msg_getter, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
  /// // Generics: type to validate and type returned within `filter*` functions.
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.filter_detailed('a'), Ok('a'));
  /// assert_eq!(input.filter_detailed('b'), Err(Violations(vec![Violation(TypeMismatch, "Only vowels allowed".to_string())])));
  /// ```
  fn filter_detailed(&self, value: T) -> Result<FT, Violations> {
    self.validate_detailed(value)?;

    Ok(self.filters.as_deref().map_or(value.into(), |filters| {
      filters
        .iter()
        .fold(value.into(), |agg, filter| (filter)(agg))
    }))
  }

  /// Validates and transforms given value, and returns transformed value on successful validation,
  /// and/or generated violation message(s) on validation violation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, value_missing_msg_getter, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
  /// // Generics: type to validate and type returned within `filter*` functions.
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// let always_a_vowel = |x| if !vowels.contains(x) { 'e' } else { x };
  /// let vowel_input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .filters(vec![ &always_a_vowel ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.filter('a'), Ok('a'));
  /// assert_eq!(input.filter('b'), Err(vec!["Only vowels allowed".to_string()]));
  /// assert_eq!(vowel_input.filter('b'), Ok('e'));
  /// ```
  fn filter(&self, value: T) -> Result<FT, Vec<ViolationMessage>> {
    match self.filter_detailed(value) {
      Ok(value) => Ok(value),
      Err(violations) => Err(violations.to_string_vec()),
    }
  }

  /// Validates and transforms given value, and returns transformed value on successful validation,
  /// and/or generated violation message(s) on validation violation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, value_missing_msg_getter, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
  /// // Generics: type to validate and type returned within `filter*` functions.
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// let always_a_vowel = |x| if !vowels.contains(x) { 'e' } else { x };
  /// let vowel_input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .filters(vec![ &always_a_vowel ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.filter_option_detailed(Some('a')), Ok(Some('a')));
  /// assert_eq!(input.filter_option_detailed(Some('b')), Err(Violations(vec![Violation(TypeMismatch, "Only vowels allowed".to_string())])));
  /// assert_eq!(vowel_input.filter_option_detailed(Some('b')), Ok(Some('e')));
  /// ```
  fn filter_option_detailed(&self, value: Option<T>) -> Result<Option<FT>, Violations> {
    match value {
      Some(value) => self.filter_detailed(value).map(Some),
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

  /// Validates and transforms given value, and returns transformed value on validation success,
  /// and/or detailed violation details on violation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, value_missing_msg_getter, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch};
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = |value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
  /// // Generics: type to validate and type returned within `filter*` functions.
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ &vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// let always_a_vowel = |x| if !vowels.contains(x) { 'e' } else { x };
  /// let vowel_input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .filters(vec![ &always_a_vowel ])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Test
  /// assert_eq!(input.filter_option(Some('a')), Ok(Some('a')));
  /// assert_eq!(input.filter_option(Some('b')), Err(vec!["Only vowels allowed".to_string()]));
  /// assert_eq!(vowel_input.filter_option(Some('b')), Ok(Some('e')));
  /// ```
  fn filter_option(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>> {
    match self.filter_option_detailed(value) {
      Ok(value) => Ok(value),
      Err(violations) => Err(violations.to_string_vec()),
    }
  }
}

impl<T: Copy, FT: From<T>> Default for Input<'_, T, FT> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   Input, InputConstraints, ViolationType,
  ///   value_missing_msg_getter
  /// };
  ///
  /// let input = Input::<usize, usize>::default();
  ///
  /// // Assert defaults
  /// // ----
  /// assert_eq!(input.break_on_failure, false);
  /// assert_eq!(input.required, false);
  /// assert!(input.custom.is_none());
  /// assert_eq!(input.locale, None);
  /// assert_eq!(input.name, None);
  /// assert!(input.get_default_value.is_none());
  /// assert!(input.validators.is_none());
  /// assert!(input.filters.is_none());
  /// assert_eq!(
  ///   (&input.value_missing_msg_getter)(&input),
  ///   value_missing_msg_getter(&input)
  /// );
  /// ```
  fn default() -> Self {
    Input {
      break_on_failure: false,
      required: false,
      custom: None,
      locale: None,
      name: None,
      get_default_value: None,
      validators: None,
      filters: None,
      value_missing_msg_getter: &value_missing_msg_getter,
    }
  }
}

impl<T: Copy, FT: From<T>> Display for Input<'_, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Input")
      .field("break_on_failure", &self.break_on_failure)
      .field("required", &self.required)
      .field(
        "validators",
        &self
          .validators
          .as_deref()
          .map(|vs| format!("Some([Validator; {}])", vs.len()))
          .unwrap_or("None".to_string()),
      )
      .field(
        "filters",
        &self
          .filters
          .as_deref()
          .map(|fs| format!("Some([Filter; {}])", fs.len()))
          .unwrap_or("None".to_string()),
      )
      .finish()
  }
}

impl<T: Copy, FT: From<T>> Debug for Input<'_, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Input")
      .field("break_on_failure", &self.break_on_failure)
      .field("required", &self.required)
      .field_with("custom", |fmtr| {
        let val = if self.custom.is_some() {
          "Some(&ValidatorForSized)"
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
          format!("Some(Vec<&ValidatorForSized>{{ len: {} }})", vs.len())
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

#[cfg(test)]
mod test {
  use super::*;
  use crate::ViolationType::{CustomError, StepMismatch};
  use std::borrow::Cow;
  use std::error::Error;

  /*
  // From previous implementation
  // ----
        use crate::ViolationType::StepMismatch;
        use crate::{
            range_overflow_msg_getter, LengthValidatorBuilder, PatternValidatorBuilder,
            RangeValidatorBuilder, SlugFilter,
        };
        use regex::Regex;
        use std::borrow::Cow;
        use std::error::Error;
        // use crate::{InputBuilder, StringConstraintsBuilder};
        // use crate::ViolationType::StepMismatch;
        use super::*;

        #[test]
        fn test_validate() {
            // Setup a custom validator
            let validate_is_even = |x: usize| {
                if x % 2 != 0 {
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )])
                } else {
                    Ok(())
                }
            };

            let one_to_ten = RangeValidatorBuilder::<usize>::default()
                .min(1)
                .max(10)
                .build()
                .unwrap();

            // Setup input constraints
            let usize_required = InputBuilder::<usize, usize>::default()
                .required(true)
                .validators(vec![&one_to_ten, &validate_is_even])
                .build()
                .unwrap();

            let test_cases = [
                (
                    "No value",
                    &usize_required,
                    None,
                    Err(vec![value_missing_msg_getter(&usize_required)]),
                ),
                ("With valid value", &usize_required, Some(4), Ok(())),
                (
                    "With \"not Even\" value",
                    &usize_required,
                    Some(7),
                    Err(vec!["Must be even".to_string()]),
                ),
            ];

            // Run test cases
            for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
                println!("Case {}: {}", i + 1, test_name);
                assert_eq!(input.validate(value), expected_rslt);
            }
        }

        #[test]
        fn test_validate_detailed() {
            // Ensure each logic case in method is sound, and that method is callable for each scalar type:
            // 1) Test method logic
            // ----
            let validate_is_even = |x: usize| {
                if x % 2 != 0 {
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )])
                } else {
                    Ok(())
                }
            };

            let zero_to_ten = RangeValidatorBuilder::<usize>::default()
                .min(0)
                .max(10)
                .build()
                .unwrap();

            let usize_input_default = InputBuilder::<usize, usize>::default().build().unwrap();

            let usize_not_required = InputBuilder::<usize, usize>::default()
                .validators(vec![&validate_is_even, &zero_to_ten])
                .build()
                .unwrap();

            let usize_required = {
                let mut new_input = usize_not_required.clone();
                new_input.required = true;
                new_input
            };

            let usize_break_on_failure = {
                let mut new_input = usize_required.clone();
                new_input.break_on_failure = true;
                new_input
            };

            let test_cases = vec![
                ("Default, with no value", &usize_input_default, None, Ok(())),
                ("Default, with value", &usize_input_default, Some(1), Ok(())),
                // Not required
                // ----
                ("1-10, Even, no value", &usize_not_required, None, Ok(())),
                (
                    "1-10, Even, with valid value",
                    &usize_not_required,
                    Some(2),
                    Ok(()),
                ),
                (
                    "1-10, Even, with valid value (2)",
                    &usize_not_required,
                    Some(10),
                    Ok(()),
                ),
                (
                    "1-10, Even, with invalid value (3)",
                    &usize_not_required,
                    Some(7),
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )]),
                ),
                (
                    "1-10, Even, with valid value",
                    &usize_not_required,
                    Some(8),
                    Ok(()),
                ),
                // Required
                // ----
                (
                    "1-10, Even, required, no value",
                    &usize_required,
                    None,
                    Err(vec![(
                        ViolationType::ValueMissing,
                        value_missing_msg_getter(&usize_required),
                    )]),
                ),
                (
                    "1-10, Even, required, with valid value",
                    &usize_required,
                    Some(2),
                    Ok(()),
                ),
                (
                    "1-10, Even, required, with valid value (1)",
                    &usize_required,
                    Some(4),
                    Ok(()),
                ),
                (
                    "1-10, Even, required, with valid value (2)",
                    &usize_required,
                    Some(8),
                    Ok(()),
                ),
                (
                    "1-10, Even, required, with valid value (3)",
                    &usize_required,
                    Some(10),
                    Ok(()),
                ),
                (
                    "1-10, Even, required, with invalid value (3)",
                    &usize_required,
                    Some(7),
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )]),
                ),
                (
                    "1-10, Even, required, with invalid value (3)",
                    &usize_break_on_failure,
                    Some(7),
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )]),
                ),
            ];

            for (i, (test_name, input, subj, expected)) in test_cases.into_iter().enumerate() {
                println!("Case {}: {}", i + 1, test_name);

                assert_eq!(input.validate_detailed(subj), expected);
            }

            // Test basic usage with other types
            // ----
            let zero_to_ten = RangeValidatorBuilder::<f64>::default()
                .min(0.0)
                .max(10.0)
                .build()
                .unwrap();

            // Validates `f64`, and `f32` usage
            let f64_input_required = InputBuilder::<f64, f64>::default()
                .required(true)
                .validators(vec![&zero_to_ten, &|x: f64| {
                    if x % 2.0 != 0.0 {
                        Err(vec![(
                            ViolationType::CustomError,
                            "Must be even".to_string(),
                        )])
                    } else {
                        Ok(())
                    }
                }])
                .build()
                .unwrap();

            assert_eq!(
                f64_input_required.validate_detailed(None),
                Err(vec![(
                             ViolationType::ValueMissing,
                             value_missing_msg_getter(&f64_input_required)
                         ), ])
            );
            assert_eq!(f64_input_required.validate_detailed(Some(2.0)), Ok(()));

            let ay_to_eff = RangeValidatorBuilder::<char>::default()
                .min('a')
                .max('f')
                .build()
                .unwrap();

            // Test `char` usage
            let char_input = InputBuilder::<char, char>::default()
                .validators(vec![&ay_to_eff])
                .build()
                .unwrap();

            assert_eq!(char_input.validate_detailed(None), Ok(()));
            assert_eq!(char_input.validate_detailed(Some('a')), Ok(()));
            assert_eq!(char_input.validate_detailed(Some('f')), Ok(()));
            assert_eq!(
                char_input.validate_detailed(Some('g')),
                Err(vec![(
                             ViolationType::RangeOverflow,
                             "`g` is greater than maximum `f`.".to_string()
                         ), ])
            );
        }

        #[test]
        fn test_filter() -> Result<(), Box<dyn Error>> {
            // Setup input constraints
            // ----
            // 1. With no filters.
            let usize_input_default = InputBuilder::<usize, usize>::default().build()?;

            // 2. With one filter.
            let usize_input_twofold = InputBuilder::<usize, usize>::default()
                .filters(vec![&|x: usize| x * 2usize])
                .build()?;

            // 3. With two filters.
            let usize_input_gte_four = InputBuilder::<usize, usize>::default()
                .filters(vec![&|x: usize| if x < 4 { 4 } else { x }, &|x: usize| {
                    x * 2usize
                }])
                .build()?;

            let test_cases = [
                // No filters
                (&usize_input_default, 100, 100),
                // With one filter
                (&usize_input_twofold, 0, 0),
                (&usize_input_twofold, 2, 4),
                (&usize_input_twofold, 4, 8),
                // With multiple filters
                (&usize_input_gte_four, 0, 8),
                (&usize_input_gte_four, 2, 8),
                (&usize_input_gte_four, 4, 8),
                (&usize_input_gte_four, 6, 12),
            ];

            // Run test cases
            for (i, (input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
                println!(
                    "Case {}: `(usize_input.filter)({:?}) == {:?}`",
                    i + 1,
                    value.clone(),
                    expected_rslt.clone()
                );
                assert_eq!(input.filter(value), expected_rslt);
            }

            Ok(())
        }

        #[test]
        fn test_validate_and_filter_detailed() -> Result<(), Box<dyn Error>> {
            // Ensure each logic case in method is sound, and that method is callable for each scalar type:
            // 1) Test method logic
            // ----
            let validate_is_even = |x: usize| {
                if x % 2 != 0 {
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )])
                } else {
                    Ok(())
                }
            };

            let one_to_ten = RangeValidatorBuilder::<usize>::default()
                .min(1)
                .max(10)
                .build()
                .unwrap();

            let usize_input_default = InputBuilder::<usize, usize>::default().build().unwrap();

            let usize_not_required_with_rules = InputBuilder::<usize, usize>::default()
                .validators(vec![&one_to_ten, &validate_is_even])
                .build()
                .unwrap();

            let usize_required_with_rules = {
                let mut new_input = usize_not_required_with_rules.clone();
                new_input.required = true;
                new_input
            };

            let usize_break_on_failure_with_rules = {
                let mut new_input = usize_required_with_rules.clone();
                new_input.break_on_failure = true;
                new_input
            };

            let test_cases = vec![
                // Default
                // ----
                (
                    "Default, with no value",
                    &usize_input_default,
                    None,
                    Ok(None),
                ),
                (
                    "Default, with value",
                    &usize_input_default,
                    Some(1),
                    Ok(Some(1)),
                ),
                // Not required
                // ----
                (
                    "1-10, Even, no value",
                    &usize_not_required_with_rules,
                    None,
                    Ok(None),
                ),
                (
                    "1-10, Even, with valid value",
                    &usize_not_required_with_rules,
                    Some(2),
                    Ok(Some(2)),
                ),
                (
                    "1-10, Even, with valid value (2)",
                    &usize_not_required_with_rules,
                    Some(10),
                    Ok(Some(10)),
                ),
                (
                    "1-10, Even, with invalid value (3)",
                    &usize_not_required_with_rules,
                    Some(7),
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )]),
                ),
                (
                    "1-10, Even, with valid value",
                    &usize_not_required_with_rules,
                    Some(8),
                    Ok(Some(8)),
                ),
                // Required
                // ----
                (
                    "1-10, Even, required, no value",
                    &usize_required_with_rules,
                    None,
                    Err(vec![(
                        ViolationType::ValueMissing,
                        value_missing_msg_getter(&usize_required_with_rules),
                    )]),
                ),
                (
                    "1-10, Even, required, with valid value",
                    &usize_required_with_rules,
                    Some(2),
                    Ok(Some(2)),
                ),
                (
                    "1-10, Even, required, with valid value (1)",
                    &usize_required_with_rules,
                    Some(4),
                    Ok(Some(4)),
                ),
                (
                    "1-10, Even, required, with valid value (2)",
                    &usize_required_with_rules,
                    Some(8),
                    Ok(Some(8)),
                ),
                (
                    "1-10, Even, required, with valid value (3)",
                    &usize_required_with_rules,
                    Some(10),
                    Ok(Some(10)),
                ),
                (
                    "1-10, Even, required, with invalid value (3)",
                    &usize_required_with_rules,
                    Some(7),
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )]),
                ),
                (
                    "1-10, Even, required, with invalid value (3)",
                    &usize_break_on_failure_with_rules,
                    Some(7),
                    Err(vec![(
                        ViolationType::CustomError,
                        "Must be even".to_string(),
                    )]),
                ),
            ];

            for (i, (test_name, input, subj, expected)) in test_cases.into_iter().enumerate() {
                println!("Case {}: {}", i + 1, test_name);

                assert_eq!(input.validate_and_filter_detailed(subj), expected);
            }

            // Test basic usage with other types
            // ----

            let one_to_ten = RangeValidatorBuilder::<f64>::default()
                .min(1.0)
                .max(10.0)
                .build()
                .unwrap();

            // Validates `f64`, and `f32` usage
            let f64_input_required = InputBuilder::<f64, f64>::default()
                .required(true)
                .validators(vec![&one_to_ten, &|x: f64| {
                    if x % 2.0 != 0.0 {
                        Err(vec![(
                            ViolationType::CustomError,
                            "Must be even".to_string(),
                        )])
                    } else {
                        Ok(())
                    }
                }])
                .build()
                .unwrap();

            assert_eq!(
                f64_input_required.validate_detailed(None),
                Err(vec![(
                             ViolationType::ValueMissing,
                             value_missing_msg_getter(&f64_input_required)
                         ), ])
            );
            assert_eq!(f64_input_required.validate_detailed(Some(2.0)), Ok(()));

            let a_to_f = RangeValidatorBuilder::<char>::default()
                .min('a')
                .max('f')
                .build()
                .unwrap();

            // Test `char` usage
            let char_input = InputBuilder::<char, char>::default()
                .validators(vec![&a_to_f])
                .build()
                .unwrap();

            assert_eq!(char_input.validate_detailed(None), Ok(()));
            assert_eq!(char_input.validate_detailed(Some('a')), Ok(()));
            assert_eq!(char_input.validate_detailed(Some('f')), Ok(()));
            assert_eq!(
                char_input.validate_detailed(Some('g')),
                Err(vec![(
                             ViolationType::RangeOverflow,
                             "`g` is greater than maximum `f`.".to_string()
                         ), ])
            );

            Ok(())
        }

  #[test]
  fn test_debug() {
    let input = RefInputBuilder::<str, Cow<str>>::default().build().unwrap();;
    println!("{:#?}", &input);

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
        .value_missing_msg_getter(&|_: &RefInput<'_, '_, str, Cow<str>>| "Value is missing".to_string())
        .build()
        .unwrap();

    println!("{:#?}", &input);
  }
  */
}
