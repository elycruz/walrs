use crate::violation::{Violation, ViolationMessage, Violations};
use std::fmt::{Debug, Display};

// ============================================================================
// Validator Type Aliases
// ============================================================================

/// Validator function type for owned/copied values.
/// 
/// Used with `Input` struct for `Copy` types.
/// 
/// # Example
/// ```rust
/// use walrs_inputfilter::{OwnedValidator, Violation, ViolationType};
/// 
/// let is_positive: &OwnedValidator<i32> = &|value: i32| {
///     if value > 0 {
///         Ok(())
///     } else {
///         Err(Violation::new(ViolationType::RangeUnderflow, "Value must be positive"))
///     }
/// };
/// ```
pub type OwnedValidator<T> = dyn Fn(T) -> Result<(), Violation> + Send + Sync;

/// Validator function type for referenced values.
/// 
/// Used with `RefInput` struct for unsized/referenced types like `str`, `[T]`, etc.
/// 
/// # Example
/// ```rust
/// use walrs_inputfilter::{RefValidator, Violation, ViolationType};
/// 
/// let is_not_empty: &RefValidator<str> = &|value: &str| {
///     if !value.is_empty() {
///         Ok(())
///     } else {
///         Err(Violation::new(ViolationType::ValueMissing, "Value cannot be empty"))
///     }
/// };
/// ```
pub type RefValidator<T> = dyn Fn(&T) -> Result<(), Violation> + Send + Sync;

// Backwards compatibility aliases
#[deprecated(since = "0.2.0", note = "Use `OwnedValidator` instead")]
pub type ValidatorForSized<T> = OwnedValidator<T>;

#[deprecated(since = "0.2.0", note = "Use `RefValidator` instead")]
pub type ValidatorForRef<T> = RefValidator<T>;

// ============================================================================
// Filter Type Aliases
// ============================================================================

/// Filter/transformation function type.
/// 
/// Takes an owned value and returns a transformed value of the same type.
pub type FilterFn<T> = dyn Fn(T) -> T + Send + Sync;

// ============================================================================
// Result Type Aliases
// ============================================================================

/// Result type for validation operations returning detailed violations.
pub type ValidationResult = Result<(), Violations>;

/// Result type for filter operations returning detailed violations.
pub type FilterResult<T> = Result<T, Violations>;

/// Result type for validation operations returning string messages.
pub type SimpleValidationResult = Result<(), Vec<ViolationMessage>>;

/// Result type for filter operations returning string messages.
pub type SimpleFilterResult<T> = Result<T, Vec<ViolationMessage>>;

// ============================================================================
// Traits
// ============================================================================

/// A trait for performing validations, and filtering (transformations), all in one,
/// for unsized types.
///
/// Only `validate_ref_detailed`, `validate_ref_option_detailed`, `filter_ref_detailed`,
/// and `filter_ref_option_detailed` are required; the other methods have default
/// implementations that delegate to these core methods.
pub trait FilterForUnsized<'a, T, FT>: Display + Debug
where
  T: ?Sized + 'a,
  FT: From<&'a T>,
{
  /// Validates the given reference and returns detailed violation results.
  /// This is a required method that implementors must provide.
  fn validate_ref_detailed(&self, x: &T) -> Result<(), Violations>;

  /// Validates the given optional reference and returns detailed violation results.
  /// This is a required method - implementors need this for custom "required" field handling.
  fn validate_ref_option_detailed(&self, x: Option<&T>) -> Result<(), Violations>;

  /// Validates, and filters, the given reference and returns the filtered value
  /// or detailed violation results.
  /// This is a required method that implementors must provide.
  fn filter_ref_detailed(&self, value: &'a T) -> Result<FT, Violations>;

  /// Validates, and filters, the given optional reference and returns the filtered value
  /// or detailed violation results.
  /// This is a required method - implementors need this for custom "required" field or default value handling.
  fn filter_ref_option_detailed(&self, value: Option<&'a T>) -> Result<Option<FT>, Violations>;

  /// Validates the given reference and returns violation messages.
  /// Default implementation delegates to `validate_ref_detailed`.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForUnsized, RefInput, RefInputBuilder, Violation};
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
  /// assert_eq!(input.validate_ref("Hello, World!"), Ok(()));
  /// assert_eq!(input.validate_ref("Hi!"), Err(vec!["Value is too short".to_string()]));
  /// assert_eq!(input.validate_ref(""), Err(vec!["Value is too short".to_string()]));
  /// ```
  fn validate_ref(&self, x: &T) -> Result<(), Vec<ViolationMessage>> {
    self.validate_ref_detailed(x).map_err(|v| v.to_string_vec())
  }

  /// Validates the given optional reference and returns violation messages.
  /// Default implementation delegates to `validate_ref_option_detailed`.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForUnsized, RefInput, RefInputBuilder, Violation};
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
  /// assert_eq!(input.validate_ref_option(Some("Hello, World!")), Ok(()));
  /// assert_eq!(input.validate_ref_option(Some("Hi!")), Err(vec!["Value is too short".to_string()]));
  /// assert_eq!(input.validate_ref_option(Some("")), Err(vec!["Value is too short".to_string()]));
  /// assert_eq!(input.validate_ref_option(None), Err(vec!["Value is missing".to_string()]));
  /// ```
  fn validate_ref_option(&self, x: Option<&T>) -> Result<(), Vec<ViolationMessage>> {
    self.validate_ref_option_detailed(x).map_err(|v| v.to_string_vec())
  }

  /// Validates, and filters, the given reference and returns the filtered value
  /// or violation messages.
  /// Default implementation delegates to `filter_ref_detailed`.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{RefInput, FilterForUnsized, Violation, ViolationType::TypeMismatch};
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
  /// alnum_input.validators = Some(vec![&alnum_only]);
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
  /// assert_eq!(input_num_list.filter_ref(&value).unwrap(), vec![2, 4, 6]);
  /// assert_eq!(input_num_list.filter_ref(&vec![]), Err(vec!["Value is empty".to_string()]));
  ///
  /// let value = "Hello, World!";
  ///
  /// assert_eq!(input.filter_ref(value).unwrap(), Cow::Borrowed(value));
  /// assert_eq!(input2.filter_ref(value).unwrap(), value.to_lowercase());
  /// assert_eq!(alnum_input.filter_ref(value), Err(vec!["Value is not alpha-numeric".to_string()]));
  /// ```
  fn filter_ref(&self, value: &'a T) -> Result<FT, Vec<ViolationMessage>> {
    self.filter_ref_detailed(value).map_err(|v| v.to_string_vec())
  }

  /// Validates, and filters, the given optional reference and returns the filtered value
  /// or violation messages.
  /// Default implementation delegates to `filter_ref_option_detailed`.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use walrs_inputfilter::{RefInput, FilterForUnsized, Violation, ViolationType::TypeMismatch};
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
  /// alnum_input.validators = Some(vec![&alnum_only]);
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
  /// assert_eq!(input_num_list.filter_ref_option(Some(&value)).unwrap(), Some(vec![2, 4, 6]));
  /// assert_eq!(input_num_list.filter_ref_option(Some(&vec![])), Err(vec!["Value is empty".to_string()]));
  ///
  /// let value = "Hello, World!";
  ///
  /// assert_eq!(input.filter_ref_option(Some(value)).unwrap(), Some(Cow::Borrowed(value)));
  /// assert_eq!(input2.filter_ref_option(Some(value)).unwrap(), Some(value.to_lowercase()));
  /// assert_eq!(alnum_input.filter_ref_option(Some(value)), Err(vec!["Value is not alpha-numeric".to_string()]));
  /// ```
  fn filter_ref_option(&self, value: Option<&'a T>) -> Result<Option<FT>, Vec<ViolationMessage>> {
    self.filter_ref_option_detailed(value).map_err(|v| v.to_string_vec())
  }
}

/// A trait for performing validations, and filtering (transformations), all in one,
/// for sized types.
///
/// Only `validate_detailed`, `validate_option_detailed`, `filter_detailed`,
/// and `filter_option_detailed` are required; the other methods have default
/// implementations that delegate to these core methods.
pub trait FilterForSized<T, FT = T>: Display + Debug
where
  T: Copy,
  FT: From<T>,
{
  /// Validates the given value and returns detailed violation results.
  /// This is a required method that implementors must provide.
  fn validate_detailed(&self, x: T) -> Result<(), Violations>;

  /// Validates the given optional value and returns detailed violation results.
  /// This is a required method - implementors need this for custom "required" field handling.
  fn validate_option_detailed(&self, x: Option<T>) -> Result<(), Violations>;

  /// Validates, and filters, the given value and returns the filtered value
  /// or detailed violation results.
  /// This is a required method that implementors must provide.
  fn filter_detailed(&self, value: T) -> Result<FT, Violations>;

  /// Validates, and filters, the given optional value and returns the filtered value
  /// or detailed violation results.
  /// This is a required method - implementors need this for custom "required" field or default value handling.
  fn filter_option_detailed(&self, value: Option<T>) -> Result<Option<FT>, Violations>;

  /// Validates the given value and returns violation messages.
  /// Default implementation delegates to `validate_detailed`.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, Input, InputBuilder, Violation};
  /// use walrs_inputfilter::ViolationType::TypeMismatch;
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// assert_eq!(input.validate('a'), Ok(()));
  /// assert_eq!(input.validate('b'), Err(vec!["Only vowels allowed".to_string()]));
  /// ```
  fn validate(&self, x: T) -> Result<(), Vec<ViolationMessage>> {
    self.validate_detailed(x).map_err(|v| v.to_string_vec())
  }

  /// Validates the given optional value and returns violation messages.
  /// Default implementation delegates to `validate_option_detailed`.
  ///
  /// ```rust
  /// use walrs_inputfilter::{value_missing_msg_getter, FilterForSized, Input, InputBuilder, Violation};
  /// use walrs_inputfilter::ViolationType::TypeMismatch;
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
  /// let input = InputBuilder::<char, char>::default()
  ///   .required(true)
  ///   .validators(vec![ vowel_validator ])
  ///   .build()
  ///   .unwrap();
  ///
  /// assert_eq!(input.validate_option(None), Err(vec![value_missing_msg_getter(&input)]));
  /// assert_eq!(input.validate_option(Some('a')), Ok(()));
  /// assert_eq!(input.validate_option(Some('b')), Err(vec!["Only vowels allowed".to_string()]));
  /// ```
  fn validate_option(&self, x: Option<T>) -> Result<(), Vec<ViolationMessage>> {
    self.validate_option_detailed(x).map_err(|v| v.to_string_vec())
  }

  /// Validates, and filters, the given value and returns the filtered value
  /// or violation messages.
  /// Default implementation delegates to `filter_detailed`.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, InputBuilder, Violation};
  /// use walrs_inputfilter::ViolationType::TypeMismatch;
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = &|value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
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
  /// assert_eq!(input.filter('a'), Ok('a'));
  /// assert_eq!(input.filter('b'), Err(vec!["Only vowels allowed".to_string()]));
  /// assert_eq!(vowel_input.filter('b'), Ok('e'));
  /// ```
  fn filter(&self, value: T) -> Result<FT, Vec<ViolationMessage>> {
    self.filter_detailed(value).map_err(|v| v.to_string_vec())
  }

  /// Validates, and filters, the given optional value and returns the filtered value
  /// or violation messages.
  /// Default implementation delegates to `filter_option_detailed`.
  ///
  /// ```rust
  /// use walrs_inputfilter::{FilterForSized, InputBuilder, Violation};
  /// use walrs_inputfilter::ViolationType::TypeMismatch;
  ///
  /// let vowels = "aeiou";
  /// let vowel_validator = |value: char| if vowels.contains(value) {
  ///   Ok(())
  /// } else {
  ///   Err(Violation(TypeMismatch, "Only vowels allowed".to_string()))
  /// };
  ///
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
  /// assert_eq!(input.filter_option(Some('a')), Ok(Some('a')));
  /// assert_eq!(input.filter_option(Some('b')), Err(vec!["Only vowels allowed".to_string()]));
  /// assert_eq!(vowel_input.filter_option(Some('b')), Ok(Some('e')));
  /// ```
  fn filter_option(&self, value: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>> {
    self.filter_option_detailed(value).map_err(|v| v.to_string_vec())
  }
}

/// Allows serialization of properties that can be used for html form control contexts.
pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}
