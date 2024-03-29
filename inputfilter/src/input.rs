use std::fmt::{Debug, Display, Formatter};
use crate::{Filter, InputConstraints, InputConstraints2, InputValue, Validator, ViolationEnum, ViolationMessage, ViolationTuple};

/// Returns a generic message for "Value is missing" violation.
///
/// ```rust
/// use walrs_inputfilter::{Input, value_missing_msg_getter};
///
/// let input = Input::<usize, usize>::new();
///
/// assert_eq!(value_missing_msg_getter(&input), "Value is missing".to_string());
/// ```
pub fn value_missing_msg_getter<T: InputValue, FT: From<T>>(_: &Input<T, FT>) -> ViolationMessage {
    "Value is missing".to_string()
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct Input<'a, 'b, T, FilterT>
    where T: InputValue + 'b,
          FilterT: 'b + From<T>,
{
    #[builder(default = "false")]
    pub break_on_failure: bool,

    #[builder(default = "false")]
    pub required: bool,

    #[builder(default = "None")]
    pub custom: Option<&'a Validator<T>>,

    #[builder(default = "None")]
    pub locale: Option<&'a str>,

    #[builder(default = "None")]
    pub name: Option<&'a str>,

    #[builder(default = "None")]
    pub default_value: Option<T>,

    #[builder(default = "None")]
    pub validators: Option<Vec<&'a Validator<T>>>,

    #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<FilterT>>>,

    #[builder(default = "&value_missing_msg_getter")]
    pub value_missing_msg: &'a (dyn Fn(&Input<'a, 'b, T, FilterT>) -> ViolationMessage + Send + Sync)
}

impl<'a, 'b,  T: InputValue + 'b, FT: 'b + From<T>> Input<'a, 'b, T, FT> {
    /// Returns a new instance with all fields set defaults.
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   Input,
    ///   ViolationEnum,
    /// };
    ///
    /// let input = Input::<usize, usize>::new();
    ///
    /// // Assert defaults
    /// // ----
    /// assert_eq!(input.break_on_failure, false);
    /// assert_eq!(input.required, false);
    /// assert!(input.custom.is_none());
    /// assert_eq!(input.locale, None);
    /// assert_eq!(input.name, None);
    /// assert_eq!(input.default_value, None);
    /// assert!(input.validators.is_none());
    /// assert!(input.filters.is_none());
    /// ```
    pub fn new() -> Self {
        Input {
            break_on_failure: false,
            required: false,
            custom: None,
            locale: None,
            name: None,
            default_value: None,
            validators: None,
            filters: None,
            value_missing_msg: &value_missing_msg_getter,
        }
    }

    fn _validate_against_own_constraints(&self, value: T) -> Result<(), Vec<ViolationTuple>> {
        if let Some(custom) = self.custom {
            return (custom)(value);
        }
        Ok(())
    }

    fn _validate_against_validators(&self, value: T) -> Result<(), Vec<ViolationTuple>> {
        self
            .validators
            .as_deref()
            .map(|vs|
                // If not break on failure then capture all validation errors.
                if !self.break_on_failure {
                    return vs
                        .iter()
                        .fold(Vec::<ViolationTuple>::new(), |mut agg, f| {
                            match f(value) {
                                Err(mut message_tuples) => {
                                    agg.append(message_tuples.as_mut());
                                    agg
                                }
                                _ => agg,
                            }
                        });
                } else {
                    // Else break on, and capture, first failure.
                    // ----
                    let mut agg = Vec::<ViolationTuple>::new();
                    for f in vs.iter() {
                        if let Err(mut message_tuples) = f(value) {
                            agg.append(message_tuples.as_mut());
                            break;
                        }
                    }
                    agg
                })
            .and_then(|messages| {
                if messages.is_empty() {
                    None
                } else {
                    Some(messages)
                }
            })
            .map_or(Ok(()), Err)
    }
}

impl<'a, 'b, T: 'b, FT: 'b + From<T>> InputConstraints2<'a, 'b, T, FT> for Input<'a, 'b, T, FT>
where
  T: InputValue,
{
  /// Validates given value against contained constraints, and returns a result of unit, and/or, a Vec of
  /// Violation messages.
  ///
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   Input, InputConstraints2, ViolationEnum,
  ///   InputBuilder,
  ///   value_missing_msg_getter,
  /// };
  ///
  /// // Setup a custom validator
  /// let validate_is_even = |x: usize| if x % 2 != 0 {
  ///   Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
  /// } else {
  ///   Ok(())
  /// };
  ///
  /// // Setup input constraints
  /// let usize_required = InputBuilder::<usize, usize>::default()
  ///   .required(true)
  ///   .validators(vec![&validate_is_even])
  ///   .build()
  ///   .unwrap();
  ///
  /// let usize_break_on_failure = (|| {
  ///    let mut new_input = usize_required.clone();
  ///    new_input.break_on_failure = true;
  ///    new_input
  /// })();
  ///
  /// let test_cases = [
  ///   ("No value", &usize_required, None, Err(vec![
  ///      value_missing_msg_getter(&usize_required),
  ///   ])),
  ///   ("With valid value", &usize_required, Some(4), Ok(())),
  ///   ("With \"not Even\" value", &usize_required, Some(7), Err(vec![
  ///      "Must be even".to_string(),
  ///   ])),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: {}", i + 1, test_name);
  ///   assert_eq!(input.validate(value), expected_rslt);
  /// }
  /// ```
  fn validate(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>> {
    match self.validate_detailed(value) {
      // If errors, extract messages and return them
      Err(messages) => Err(messages.into_iter().map(|(_, message)| message).collect()),
      Ok(_) => Ok(()),
    }
  }

  /// Validates given value against contained constraints and returns a result of unit and/or a Vec of violation tuples
  /// if value doesn't pass validation.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   Input, InputConstraints2, ViolationEnum,
  ///   InputBuilder,
  ///   value_missing_msg_getter,
  /// };
  ///
  /// // Setup a custom validator
  /// let validate_is_even = |x: usize| if x % 2 != 0 {
  ///   Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
  /// } else {
  ///   Ok(())
  /// };
  ///
  /// // Setup input constraints
  /// let usize_required = InputBuilder::<usize, usize>::default()
  ///   .required(true)
  ///   .validators(vec![&validate_is_even])
  ///   .build()
  ///   .unwrap();
  ///
  /// let usize_break_on_failure = (|| {
  ///    let mut new_input = usize_required.clone();
  ///    new_input.break_on_failure = true;
  ///    new_input
  /// })();
  ///
  /// let test_cases = [
  ///   ("No value", &usize_required, None, Err(vec![
  ///     (ViolationEnum::ValueMissing,
  ///      value_missing_msg_getter(&usize_required)),
  ///   ])),
  ///   ("With valid value", &usize_required, Some(4), Ok(())),
  ///   ("With \"not Even\" value", &usize_required, Some(7), Err(vec![
  ///     (ViolationEnum::CustomError,
  ///      "Must be even".to_string()),
  ///   ])),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: {}", i + 1, test_name);
  ///   assert_eq!(input.validate_detailed(value), expected_rslt);
  /// }
  /// ```
  fn validate_detailed(&self, value: Option<T>) -> Result<(), Vec<ViolationTuple>> {
    match value {
      None => {
        if self.required {
          Err(vec![(
            ViolationEnum::ValueMissing,
            (self.value_missing_msg)(self),
          )])
        } else {
          Ok(())
        }
      }
      // Else if value is populated validate it
      Some(v) =>
        match self._validate_against_own_constraints(v) {
          Ok(_) => self._validate_against_validators(v),
          Err(messages1) =>
            if self.break_on_failure {
              Err(messages1)
            } else if let Err(mut messages2) = self._validate_against_validators(v) {
              let mut agg = messages1;
              agg.append(messages2.as_mut());
              Err(agg)
            } else {
              Err(messages1)
            }
        }
    }
  }

  /// Filters value against contained filters.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   InputBuilder,
  ///   InputConstraints2,
  /// };
  ///
  /// // Setup input constraints
  /// let usize_input = InputBuilder::<usize, usize>::default()
  ///   .filters(vec![&|x: usize| x * 2usize])
  ///   .build()
  ///   .unwrap();
  ///
  /// let test_cases = [
  ///   (&usize_input, 0, 0),
  ///   (&usize_input, 2, 4),
  ///   (&usize_input, 4, 8),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: `(usize_input.filter)({:?}) == {:?}`", i + 1, value, expected_rslt);
  ///   assert_eq!(input.filter(value), expected_rslt);
  /// }
  /// ```
  ///
  fn filter(&self, value: FT) -> FT {
    match self.filters.as_deref() {
      None => value,
      Some(fs) => fs.iter().fold(value, |agg, f| f(agg)),
    }
  }

  /// Validates, and filters, given value against contained rules, validators, and filters, respectively.
  /// If value doesn't pass validation, returns a Vec of Violation messages.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   InputBuilder,
  ///   Input,
  ///   ViolationMessage,
  ///   InputConstraints2,
  ///   ViolationEnum::CustomError,
  /// };
  /// use walrs_inputfilter::ViolationEnum::{RangeOverflow, RangeUnderflow};
  ///
  /// let min_max_check = |x: usize| if x < 1 {
  ///   Err(vec![(RangeUnderflow, format!("`{}` is less than minimum `1`.", x))])
  /// } else if x > 10 {
  ///   Err(vec![(RangeOverflow, format!("`{}` is greater than maximum `10`.", x))])
  /// } else {
  ///   Ok(())
  /// };
  ///
  /// // Setup input constraints
  /// let usize_input = InputBuilder::<usize, usize>::default()
  ///   .required(true)
  ///   .validators(vec![
  ///   &min_max_check,
  ///   &|x: usize| if x % 2 != 0 {
  ///     Err(vec![(CustomError, "Must be even".to_string())])
  ///   } else {
  ///     Ok(())
  ///   }])
  ///   .filters(vec![&|x: usize| x * 2usize])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Stops validation on first validation error and returns `Err` result.
  /// let usize_input_break_on_failure = {
  ///   let mut new_input = usize_input.clone();
  ///   new_input.break_on_failure = true;
  ///   new_input
  /// };
  ///
  /// let test_cases = vec![
  ///   ("No value", &usize_input, None, Err(vec![ "Value is missing".to_string() ])),
  ///   ("With valid value", &usize_input, Some(4), Ok(Some(8))),
  ///   ("With \"out of lower bounds\" value", &usize_input, Some(0), Err(vec![
  ///     "`0` is less than minimum `1`.".to_string(),
  ///   ])),
  ///   ("With \"out of upper bounds\" value", &usize_input, Some(11), Err(vec![
  ///     "`11` is greater than maximum `10`.".to_string(),
  ///     "Must be even".to_string(),
  ///   ])),
  ///   ("With \"not Even\" value", &usize_input, Some(7), Err(vec![
  ///     "Must be even".to_string(),
  ///   ])),
  ///   ("With \"not Even\" value, and 'break_on_failure: true'", &usize_input_break_on_failure,
  ///     Some(7),
  ///     Err(vec![
  ///     "Must be even".to_string(),
  ///     ])
  ///   ),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: {}", i + 1, test_name);
  ///   assert_eq!(input.validate_and_filter(value), expected_rslt);
  /// }
  /// ```
  fn validate_and_filter(&self, x: Option<T>) -> Result<Option<FT>, Vec<ViolationMessage>> {
    match self.validate_and_filter_detailed(x) {
      Err(messages) => Err(messages.into_iter().map(|(_, message)| message).collect()),
      Ok(filtered) => Ok(filtered),
    }
  }

  /// Validates, and filters, given value against contained rules, validators, and filters, respectively and
  /// returns a result of filtered value or a Vec of Violation tuples.
  /// ```rust
  /// use walrs_inputfilter::{
  ///   InputBuilder,
  ///   Input,
  ///   InputConstraints2,
  ///   ValidationResult,
  ///   ViolationMessage,
  ///   ViolationEnum,
  ///   ViolationEnum::{
  ///     CustomError,
  ///     RangeOverflow,
  ///     RangeUnderflow,
  ///     ValueMissing,
  ///   },
  /// };
  ///
  /// let min_max_check = |x: usize| if x < 1 {
  ///   Err(vec![(RangeUnderflow, format!("`{}` is less than minimum `1`.", x))])
  /// } else if x > 10 {
  ///   Err(vec![(RangeOverflow, format!("`{}` is greater than maximum `10`.", x))])
  /// } else {
  ///   Ok(())
  /// };
  ///
  /// // Setup input constraints
  /// let usize_input = InputBuilder::<usize, usize>::default()
  ///   .required(true)
  ///   .validators(vec![
  ///   &min_max_check,
  ///   &|x: usize| if x % 2 != 0 {
  ///     Err(vec![(CustomError, "Must be even".to_string())])
  ///   } else {
  ///     Ok(())
  ///   }])
  ///   .filters(vec![&|x: usize| x * 2usize])
  ///   .build()
  ///   .unwrap();
  ///
  /// // Stops validation on first validation error and returns `Err` result.
  /// let usize_input_break_on_failure = {
  ///   let mut new_input = usize_input.clone();
  ///   new_input.break_on_failure = true;
  ///   new_input
  /// };
  ///
  /// type TestName = &'static str;
  /// type ConstraintStruct<'a, 'b> = Input<'a, 'b, usize, usize>;
  /// type TestValue = Option<usize>;
  /// type ExpectedResult = Result<Option<usize>, Vec<(ViolationEnum, ViolationMessage)>>;
  ///
  /// let test_cases: Vec<(TestName, &ConstraintStruct, TestValue, ExpectedResult)> = vec![
  ///   ("No value", &usize_input, None, Err(vec![
  ///     (ValueMissing, "Value is missing".to_string())
  ///   ])),
  ///   ("With valid value", &usize_input,
  ///     Some(4), Ok(Some(8))
  ///   ),
  ///   ("With \"out of lower bounds\" value", &usize_input, Some(0), Err(vec![
  ///     (RangeUnderflow, "`0` is less than minimum `1`.".to_string()),
  ///   ])),
  ///   ("With \"out of upper bounds\" value", &usize_input, Some(11), Err(vec![
  ///     (RangeOverflow, "`11` is greater than maximum `10`.".to_string()),
  ///     (CustomError, "Must be even".to_string()),
  ///   ])),
  ///   ("With \"not Even\" value", &usize_input, Some(7), Err(vec![
  ///     (CustomError, "Must be even".to_string()),
  ///   ])),
  ///   ("With \"not Even\" value, and 'break_on_failure: true'", &usize_input_break_on_failure,
  ///     Some(7),
  ///     Err(vec![
  ///       (CustomError, "Must be even".to_string()),
  ///     ])
  ///   ),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: {}", i + 1, test_name);
  ///   assert_eq!(input.validate_and_filter_detailed(value), expected_rslt);
  /// }
  /// ```
  fn validate_and_filter_detailed(&self, x: Option<T>) -> Result<Option<FT>, Vec<ViolationTuple>> {
    self.validate_detailed(x).map(|_| x.map(|_x| self.filter(_x.into())))
  }
}

impl<'a, 'b, T: InputValue + 'b, FT: 'b + From<T>> Default for Input<'a, 'b, T, FT> {
  /// Returns a new instance with all fields set to defaults.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   Input, InputConstraints, ViolationEnum,
  /// };
  ///
  /// let input = Input::<usize, usize>::default();
  ///
  /// // Assert defaults
  /// // ----
  /// assert_eq!(input.break_on_failure, false);
  /// assert_eq!(input.required, false);
  /// assert!(input.validators.is_none());
  /// assert!(input.filters.is_none());
  /// ```
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, 'b, T: InputValue + 'b, FT: 'b + From<T>> Display for Input<'a, 'b, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("Input")
          .field("break_on_failure", &self.break_on_failure)
          .field("required", &self.required)
          .field("validators", &self
              .validators
              .as_deref()
              .map(|vs| format!("Some([Validator; {}])", vs.len()))
              .unwrap_or("None".to_string()))
          .field("filters", &self
              .filters
              .as_deref()
              .map(|fs| format!("Some([Filter; {}])", fs.len()))
              .unwrap_or("None".to_string()))
          .finish()
  }
}

impl<'a, 'b, T: InputValue + 'b, FT: 'b + From<T>> Debug for Input<'a, 'b, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::error::Error;
    use regex::Regex;
    use crate::{
        LengthValidatorBuilder,
        PatternValidatorBuilder, 
        range_overflow_msg_getter,
        RangeValidatorBuilder,
        SlugFilter,
        SlugFilterBuilder,
        InputConstraints2,
    };
    use crate::ViolationEnum::StepMismatch;
    // use crate::{InputBuilder, StringConstraintsBuilder};
    // use crate::ViolationEnum::StepMismatch;
    use super::*;

    #[test]
    fn test_new() -> Result<(), Box<dyn Error>> {
        let _ = Input::<&str, Cow<str>>::new();
        let _ = Input::<char, char>::new();
        let _ = Input::<usize, usize>::new();
        let _ = Input::<bool, bool>::new();
        let _ = Input::<usize, usize>::new();

        let one_to_one_hundred = RangeValidatorBuilder::<usize>::default()
            .min(0)
            .max(100)
            .build().unwrap();

        let percent = InputBuilder::<usize, usize>::default()
            .validators(vec![
                &|x| if x != 0 && x % 5 != 0 {
                    Err(vec![(StepMismatch, format!("{} is not divisible by 5", x))])
                } else {
                    Ok(())
                },
                &one_to_one_hundred
            ])
            .build()
            .unwrap();

        assert_eq!(percent.validate(Some(5)), Ok(()));
        assert_eq!(percent.validate(Some(101)),
                   Err(vec![
                       "101 is not divisible by 5".to_string(),
                       range_overflow_msg_getter(&one_to_one_hundred, 101usize),
                   ]));
 
        assert_eq!(percent.validate(Some(26)),
                   Err(vec!["26 is not divisible by 5".to_string()]));

        let slug_length_validator = LengthValidatorBuilder::default()
            .min_length(2)
            .max_length(255)
            .build()?;

        let slug_pattern_validator = PatternValidatorBuilder::default()
            .pattern(Cow::Owned(Regex::new(r"(?i)^[^\w\-]{2,200}$").unwrap()))
            .build()?;
        
        let slug_filter = SlugFilter::new(200, false);

        let slug_input = InputBuilder::<&str, Cow<str>>::default()
            .validators(vec![
                &slug_length_validator,
                &slug_pattern_validator,
            ])
            .filters(vec![
                &slug_filter
            ])
            .build()?;
        
        assert_eq!(slug_input.validate_and_filter(Some("a")), Err(vec![
            (&slug_length_validator.too_short_msg)(&slug_length_validator, "a"),
            (&slug_pattern_validator.pattern_mismatch)(&slug_pattern_validator, "a"),
        ]));

        // str_input.constraints = Some(Box::new(StringConstraintsBuilder::default()
        //     .max_length(4)
        //     .build()?
        // ));
        //
        // assert_eq!(str_input.validate(Some("aeiou")),
        //            Err(vec![
        //                "Value length `5` is greater than allowed maximum `4`.".to_string(),
        //            ]));

        Ok(())
    }
/*
    #[test]
    fn test_validate() {
        // Setup a custom validator
        let validate_is_even = |x: usize| if x % 2 != 0 {
            Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
        } else {
            Ok(())
        };

        // Setup input constraints
        let usize_required = InputBuilder::<usize, usize>::default()
            .min(1)
            .max(10)
            .required(true)
            .validators(vec![&validate_is_even])
            .build()
            .unwrap();

        let usize_break_on_failure = (|| {
            let mut new_input = usize_required.clone();
            new_input.break_on_failure = true;
            new_input
        })();

        let test_cases = [
            ("No value", &usize_required, None, Err(vec![
                value_missing_msg_getter(&usize_required),
            ])),
            ("With valid value", &usize_required, Some(4), Ok(())),
            ("With \"not Even\" value", &usize_required, Some(7), Err(vec![
                "Must be even".to_string(),
            ])),
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
        let validate_is_even = |x: usize| if x % 2 != 0 {
            Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
        } else {
            Ok(())
        };

        let usize_input_default = InputBuilder::<usize, usize>::default()
            .build()
            .unwrap();

        let usize_not_required = InputBuilder::<usize, usize>::default()
            .min(1)
            .max(10)
            .validators(vec![&validate_is_even])
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
            ("1-10, Even, with valid value", &usize_not_required, Some(2), Ok(())),
            ("1-10, Even, with valid value (2)", &usize_not_required, Some(10), Ok(())),
            ("1-10, Even, with invalid value (3)", &usize_not_required, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
            ("1-10, Even, with valid value", &usize_not_required, Some(8), Ok(())),

            // Required
            // ----
            ("1-10, Even, required, no value", &usize_required, None, Err(vec![
                (ViolationEnum::ValueMissing,
                 value_missing_msg_getter(&usize_required)),
            ])),
            ("1-10, Even, required, with valid value", &usize_required, Some(2), Ok(())),
            ("1-10, Even, required, with valid value (1)", &usize_required, Some(4), Ok(())),
            ("1-10, Even, required, with valid value (2)", &usize_required, Some(8), Ok(())),
            ("1-10, Even, required, with valid value (3)", &usize_required, Some(10), Ok(())),
            ("1-10, Even, required, with invalid value (3)", &usize_required, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
            ("1-10, Even, required, with invalid value (3)", &usize_break_on_failure, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
        ];

        for (i, (test_name, input, subj, expected)) in test_cases.into_iter().enumerate() {
            println!("Case {}: {}", i + 1, test_name);

            assert_eq!(input.validate_detailed(subj), expected);
        }

        // Test basic usage with other types
        // ----
        // Validates `f64`, and `f32` usage
        let f64_input_required = InputBuilder::<f64, f64>::default()
            .required(true)
            .min(1.0)
            .max(10.0)
            .validators(vec![&|x: f64| if x % 2.0 != 0.0 {
                Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
            } else {
                Ok(())
            }])
            .build()
            .unwrap();

        assert_eq!(f64_input_required.validate_detailed(None), Err(vec![
            (ViolationEnum::ValueMissing,
             value_missing_msg_getter(&f64_input_required)),
        ]));
        assert_eq!(f64_input_required.validate_detailed(Some(2.0)), Ok(()));

        // Test `char` usage
        let char_input = InputBuilder::<char, char>::default()
            .min('a')
            .max('f')
            .build()
            .unwrap();

        assert_eq!(char_input.validate_detailed(None), Ok(()));
        assert_eq!(char_input.validate_detailed(Some('a')), Ok(()));
        assert_eq!(char_input.validate_detailed(Some('f')), Ok(()));
        assert_eq!(char_input.validate_detailed(Some('g')), Err(vec![
            (ViolationEnum::RangeOverflow,
             "`g` is greater than maximum `f`.".to_string()),
        ]));
    }

    #[test]
    fn test_filter() -> Result<(), Box<dyn Error>> {
        // Setup input constraints
        // ----
        // 1. With no filters.
        let usize_input_default = InputBuilder::<usize, usize>::default().build()?;

        // 2. With one filter.
        let usize_input_twofold = InputBuilder::<usize, usize>::default()
            .filters(vec![
                &|x: Option<usize>| x.map(|_x| _x * 2usize),
            ])
            .build()?;

        // 3. With two filters.
        let usize_input_gte_four = InputBuilder::<usize, usize>::default()
            .filters(vec![
                &|x: Option<usize>| x.map(|_x| if _x < 4 { 4 } else { _x }),
                &|x: Option<usize>| x.map(|_x| _x * 2usize),
            ])
            .build()?;

        let test_cases = [
            // No filters
            (&usize_input_default, None, None),
            (&usize_input_default, Some(100), Some(100)),

            // With one filter
            (&usize_input_twofold, None, None),
            (&usize_input_twofold, Some(0), Some(0)),
            (&usize_input_twofold, Some(2), Some(4)),
            (&usize_input_twofold, Some(4), Some(8)),

            // With multiple filters
            (&usize_input_gte_four, None, None),
            (&usize_input_gte_four, Some(0), Some(8)),
            (&usize_input_gte_four, Some(2), Some(8)),
            (&usize_input_gte_four, Some(4), Some(8)),
            (&usize_input_gte_four, Some(6), Some(12)),
        ];

        // Run test cases
        for (i, (input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
            println!("Case {}: `(usize_input.filter)({:?}) == {:?}`", i + 1,
                     value.clone(), expected_rslt.clone()
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
        let validate_is_even = |x: usize| if x % 2 != 0 {
            Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
        } else {
            Ok(())
        };

        let usize_input_default = InputBuilder::<usize, usize>::default()
            .build()
            .unwrap();

        let usize_not_required_with_rules = InputBuilder::<usize, usize>::default()
            .min(1)
            .max(10)
            .validators(vec![&validate_is_even])
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
            ("Default, with no value", &usize_input_default, None, Ok(None)),
            ("Default, with value", &usize_input_default, Some(1), Ok(Some(1))),

            // Not required
            // ----
            ("1-10, Even, no value", &usize_not_required_with_rules, None, Ok(None)),
            ("1-10, Even, with valid value", &usize_not_required_with_rules, Some(2), Ok(Some(2))),
            ("1-10, Even, with valid value (2)", &usize_not_required_with_rules, Some(10), Ok(Some(10))),
            ("1-10, Even, with invalid value (3)", &usize_not_required_with_rules, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
            ("1-10, Even, with valid value", &usize_not_required_with_rules, Some(8), Ok(Some(8))),

            // Required
            // ----
            ("1-10, Even, required, no value", &usize_required_with_rules, None, Err(vec![
                (ViolationEnum::ValueMissing,
                 value_missing_msg_getter(&usize_required_with_rules)),
            ])),
            ("1-10, Even, required, with valid value", &usize_required_with_rules, Some(2), Ok(Some(2))),
            ("1-10, Even, required, with valid value (1)", &usize_required_with_rules, Some(4), Ok(Some(4))),
            ("1-10, Even, required, with valid value (2)", &usize_required_with_rules, Some(8), Ok(Some(8))),
            ("1-10, Even, required, with valid value (3)", &usize_required_with_rules, Some(10), Ok(Some(10))),
            ("1-10, Even, required, with invalid value (3)", &usize_required_with_rules, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
            ("1-10, Even, required, with invalid value (3)", &usize_break_on_failure_with_rules, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
        ];

        for (i, (test_name, input, subj, expected)) in test_cases.into_iter().enumerate() {
            println!("Case {}: {}", i + 1, test_name);

            assert_eq!(input.validate_and_filter_detailed(subj), expected);
        }

        // Test basic usage with other types
        // ----
        // Validates `f64`, and `f32` usage
        let f64_input_required = InputBuilder::<f64, f64>::default()
            .required(true)
            .min(1.0)
            .max(10.0)
            .validators(vec![&|x: f64| if x % 2.0 != 0.0 {
                Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
            } else {
                Ok(())
            }])
            .build()
            .unwrap();

        assert_eq!(f64_input_required.validate_detailed(None), Err(vec![
            (ViolationEnum::ValueMissing,
             value_missing_msg_getter(&f64_input_required)),
        ]));
        assert_eq!(f64_input_required.validate_detailed(Some(2.0)), Ok(()));
        
        // Test `char` usage
        let char_input = InputBuilder::<char, char>::default()
            .min('a')
            .max('f')
            .build()
            .unwrap();

        assert_eq!(char_input.validate_detailed(None), Ok(()));
        assert_eq!(char_input.validate_detailed(Some('a')), Ok(()));
        assert_eq!(char_input.validate_detailed(Some('f')), Ok(()));
        assert_eq!(char_input.validate_detailed(Some('g')), Err(vec![
            (ViolationEnum::RangeOverflow,
             "`g` is greater than maximum `f`.".to_string()),
        ]));

        Ok(())
    }
    
 */
}
