use crate::{FilterForSized, OwnedValidator, ViolationMessage, Violations};
use crate::input_common::{collect_violations, handle_missing_value, handle_missing_value_for_filter};
use crate::traits::FilterFn;
use crate::{debug_closure_field, debug_vec_closure_field};
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
#[must_use]
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
  pub custom: Option<&'a OwnedValidator<T>>,

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
  pub validators: Option<Vec<&'a OwnedValidator<T>>>,

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
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{FilterForSized, Input, InputBuilder, Violation, Violations};
  /// use walrs_inputfilter::ViolationType::{TypeMismatch, StepMismatch, TooShort};
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
  /// let min_length = |s: &str| if s.len() < 5 {
  ///   Err(Violation(TooShort, "Length is too short".to_string()))
  /// } else {
  ///   Ok(())
  /// };
  ///
  /// let to_uppercase = |s: String| s.to_uppercase();
  /// let to_uppercase_for_cow = |s: Cow<str>| -> Cow<str> {
  ///   s.to_uppercase().into()
  /// };
  ///
  /// // Note: For "invariant" lifetime scenarios, use `RefInput`
  /// // for reference types.
  /// let str_input = InputBuilder::<&str, String>::default()
  ///   .required(true)
  ///   .validators(vec![&min_length])
  ///   .filters(vec![&to_uppercase])
  ///   .build()
  ///   .unwrap();
  ///
  /// let str_input2 = InputBuilder::<&str, Cow<str>>::default()
  ///   .required(true)
  ///   .validators(vec![&min_length])
  ///   .filters(vec![&to_uppercase_for_cow])
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
  /// assert_eq!(str_input.validate("abc"), Err(vec!["Length is too short".to_string()]));
  /// assert_eq!(str_input2.validate("abc"), Err(vec!["Length is too short".to_string()]));
  /// assert_eq!(str_input.filter("abcdefg"), Ok("ABCDEFG".to_string()));
  /// assert_eq!(str_input2.filter("abcdefg"), Ok(Cow::from("ABCDEFG".to_string())));
  /// ```
  fn validate_detailed(&self, value: T) -> Result<(), Violations> {
    let custom_result = self.custom.map(|custom| custom(value));
    let validators_iter = self
      .validators
      .as_deref()
      .into_iter()
      .flatten()
      .map(|v| v(value));

    collect_violations(custom_result, validators_iter, self.break_on_failure)
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
      None => handle_missing_value(self.required, || (self.value_missing_msg_getter)(self)),
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
  /// assert_eq!(input.filter_option_detailed(Some('a')), Ok(Some('a')));
  /// assert_eq!(input.filter_option_detailed(Some('b')), Err(Violations(vec![Violation(TypeMismatch, "Only vowels allowed".to_string())])));
  /// assert_eq!(vowel_input.filter_option_detailed(Some('b')), Ok(Some('e')));
  /// ```
  fn filter_option_detailed(&self, value: Option<T>) -> Result<Option<FT>, Violations> {
    match value {
      Some(value) => self.filter_detailed(value).map(Some),
      None => handle_missing_value_for_filter(
        self.required,
        || (self.value_missing_msg_getter)(self),
        self.get_default_value.map(|f| move || f()),
      ),
    }
  }
}

impl<T: Copy, FT: From<T>> Default for Input<'_, T, FT> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   Input, ViolationType,
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

#[cfg(feature = "debug_closure_helpers")]
impl<T: Copy, FT: From<T>> Debug for Input<'_, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Input")
      .field("break_on_failure", &self.break_on_failure)
      .field("required", &self.required)
      .field_with("custom", |fmtr| {
        let val = if self.custom.is_some() {
          "Some(&OwnedValidator)"
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
          format!("Some(Vec<&OwnedValidator>{{ len: {} }})", vs.len())
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
impl<T: Copy, FT: From<T>> Debug for Input<'_, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let custom_str = debug_closure_field!(self.custom, "Some(&OwnedValidator)");
    let get_default_value_str = debug_closure_field!(
      self.get_default_value,
      "Some(&dyn Fn() -> Option<FT> + Send + Sync)"
    );
    let validators_str = debug_vec_closure_field!(self.validators, "&OwnedValidator");
    let filters_str = debug_vec_closure_field!(self.filters, "&FilterFn");

    f.debug_struct("Input")
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

#[cfg(test)]
mod test {
  use super::*;
  use crate::Violation;
  use crate::ViolationType::{CustomError, RangeOverflow, ValueMissing};
  use std::error::Error;

  #[test]
  fn test_validate_methods() {
    // Ensure each logic case in method is sound, and that method is callable for each scalar type:
    // 1) Test method logic
    // ----
    let validate_is_even = |x: usize| {
      if x % 2 != 0 {
        // @todo Populate custom error here.
        Err(Violation(CustomError, "Must be even".to_string()))
      } else {
        Ok(())
      }
    };

    let zero_to_ten = |n| {
      if n > 10 {
        Err(Violation(
          RangeOverflow,
          "Number must be between 0-10".to_string(),
        ))
      } else {
        Ok(())
      }
    };

    let usize_input_default = InputBuilder::<usize, usize>::default().build().unwrap();

    let even_zero_to_ten_not_req = InputBuilder::<usize, usize>::default()
      .validators(vec![&validate_is_even, &zero_to_ten])
      .build()
      .unwrap();

    let even_zero_to_ten_req = {
      let mut new_input = even_zero_to_ten_not_req.clone();
      new_input.required = true;
      new_input
    };

    let even_zero_to_ten_req_break_on_fail = {
      let mut new_input = even_zero_to_ten_req.clone();
      new_input.break_on_failure = true;
      new_input
    };

    let with_custom_validator = InputBuilder::<usize, usize>::default()
      .custom(&validate_is_even)
      .build()
      .unwrap();

    let with_custom_validator_req = {
      let mut new_input = with_custom_validator.clone();
      new_input.required = true;
      new_input
    };

    let with_custom_validator_two = {
      let mut new_input = with_custom_validator.clone();
      new_input.validators = Some(vec![&zero_to_ten]);
      new_input
    };

    let with_custom_validator_two_req = {
      let mut new_input = with_custom_validator_two.clone();
      new_input.required = true;
      new_input
    };

    // @todo Add test cases for some of the other scalar types to add variety.

    let test_cases: Vec<(&str, &Input<usize, usize>, usize, Result<usize, Violations>)> = vec![
      ("Default, with value", &usize_input_default, 1, Ok(1)),
      // Not required
      // ----
      (
        "1-10, Even, with valid value",
        &even_zero_to_ten_not_req,
        2,
        Ok(2),
      ),
      (
        "1-10, Even, with valid value (2)",
        &even_zero_to_ten_not_req,
        10,
        Ok(10),
      ),
      (
        "1-10, Even, with invalid value (3)",
        &even_zero_to_ten_not_req,
        7,
        Err(Violations(vec![Violation(
          CustomError,
          "Must be even".to_string(),
        )])),
      ),
      (
        "1-10, Even, with valid value",
        &even_zero_to_ten_not_req,
        8,
        Ok(8),
      ),
      // Required
      // ----
      (
        "1-10, Even, required, with valid value",
        &even_zero_to_ten_req,
        2,
        Ok(2),
      ),
      (
        "1-10, Even, required, with valid value (1)",
        &even_zero_to_ten_req,
        4,
        Ok(4),
      ),
      (
        "1-10, Even, required, with valid value (2)",
        &even_zero_to_ten_req,
        8,
        Ok(8),
      ),
      (
        "1-10, Even, required, with valid value (3)",
        &even_zero_to_ten_req,
        10,
        Ok(10),
      ),
      (
        "1-10, Even, required, with invalid value (3)",
        &even_zero_to_ten_req,
        7,
        Err(Violations(vec![Violation(
          CustomError,
          "Must be even".to_string(),
        )])),
      ),
      (
        "1-10, Even, required, with invalid out of bounds value",
        &even_zero_to_ten_req,
        77,
        Err(Violations(vec![
          Violation(CustomError, "Must be even".to_string()),
          Violation(RangeOverflow, "Number must be between 0-10".to_string()),
        ])),
      ),
      (
        "1-10, Even, required, with invalid value, and \"break_on_failure\"",
        &even_zero_to_ten_req_break_on_fail,
        7,
        Err(Violations(vec![Violation(
          CustomError,
          "Must be even".to_string(),
        )])),
      ),
      (
        "1-10, Even, required, with invalid value, and \"break_on_failure\"",
        &even_zero_to_ten_req_break_on_fail,
        12,
        Err(Violations(vec![Violation(
          RangeOverflow,
          "Number must be between 0-10".to_string(),
        )])),
      ),
      (
        "1-10, Even, required, with valid value, and \"break_on_failure\"",
        &even_zero_to_ten_req_break_on_fail,
        10,
        Ok(10),
      ),
      (
        "1-10, Even, with \"custom\" (singular) validator and invalid value",
        &with_custom_validator,
        77,
        Err(Violations(vec![Violation(
          CustomError,
          "Must be even".to_string(),
        )])),
      ),
      (
        "1-10, Even, with \"custom\" (singular) validator and valid value",
        &with_custom_validator,
        10,
        Ok(10),
      ),
      (
        "1-10, Even, with \"custom\", additional validators, and invalid value",
        &with_custom_validator_two,
        77,
        Err(Violations(vec![
          Violation(CustomError, "Must be even".to_string()),
          Violation(RangeOverflow, "Number must be between 0-10".to_string()),
        ])),
      ),
      (
        "1-10, Even, with \"custom\", additional validators, and valid value",
        &with_custom_validator_two,
        8,
        Ok(8),
      ),
    ];

    for (i, (test_name, input, subj, expected)) in test_cases.into_iter().enumerate() {
      println!("Case {}: {}", i + 1, test_name);

      match expected {
        Err(violations) => {
          let msgs_vec = violations.clone().to_string_vec();
          assert_eq!(input.validate(subj), Err(msgs_vec.clone()));
          assert_eq!(input.validate_detailed(subj), Err(violations.clone()));
          assert_eq!(input.validate_option(Some(subj)), Err(msgs_vec.clone()));
          assert_eq!(
            input.validate_option_detailed(Some(subj)),
            Err(violations.clone())
          );
          assert_eq!(input.filter(subj), Err(msgs_vec.clone()));
          assert_eq!(input.filter_detailed(subj), Err(violations.clone()));
          assert_eq!(input.filter_option(Some(subj)), Err(msgs_vec.clone()));
          assert_eq!(
            input.filter_option_detailed(Some(subj)),
            Err(violations.clone())
          );
        }
        Ok(value) => {
          assert_eq!(input.validate(subj), Ok(()));
          assert_eq!(input.validate_detailed(subj), Ok(()));
          assert_eq!(input.validate_option(Some(subj)), Ok(()));
          assert_eq!(input.validate_option_detailed(Some(subj)), Ok(()));
          assert_eq!(input.filter(subj), Ok(value));
          assert_eq!(input.filter_detailed(subj), Ok(value));
          assert_eq!(input.filter_option(Some(subj)), Ok(Some(value)));
          assert_eq!(input.filter_option_detailed(Some(subj)), Ok(Some(value)));
        }
      }
    }

    // Validate required value, with "None" value;  E.g., should always return "one" error message
    // ----
    assert_eq!(
      even_zero_to_ten_req.validate_option(None),
      Err(vec![value_missing_msg_getter(&even_zero_to_ten_req)])
    );
    assert_eq!(
      even_zero_to_ten_req.validate_option_detailed(None),
      Err(Violations(vec![Violation(
        ValueMissing,
        value_missing_msg_getter(&even_zero_to_ten_req)
      )]))
    );
    assert_eq!(
      with_custom_validator_req.validate_option(None),
      Err(vec![value_missing_msg_getter(&with_custom_validator_req)])
    );
    assert_eq!(
      with_custom_validator_req.validate_option_detailed(None),
      Err(Violations(vec![Violation(
        ValueMissing,
        value_missing_msg_getter(&with_custom_validator_req)
      )]))
    );
    assert_eq!(
      with_custom_validator_two_req.validate_option(None),
      Err(vec![value_missing_msg_getter(
        &with_custom_validator_two_req
      )])
    );
    assert_eq!(
      with_custom_validator_two_req.validate_option_detailed(None),
      Err(Violations(vec![Violation(
        ValueMissing,
        value_missing_msg_getter(&with_custom_validator_two_req)
      )]))
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
    // @todo Update this to allow Error results as well.
    for (i, (input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
      println!(
        "Case {}: `(usize_input.filter)({:?}) == {:?}`",
        i + 1,
        value.clone(),
        expected_rslt.clone()
      );
      assert_eq!(input.filter(value), Ok(expected_rslt));
    }

    Ok(())
  }
}
