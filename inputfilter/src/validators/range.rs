use std::fmt::{Debug, Display, Formatter};

use crate::{value_missing_msg, ViolationEnum, ScalarValue, ValidateValue, ValidationResult};

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct ScalarValidator<'a, T: ScalarValue> {
    #[builder(default = "false")]
    pub break_on_failure: bool,

    #[builder(default = "None")]
    pub min: Option<T>,

    #[builder(default = "None")]
    pub max: Option<T>,

    #[builder(default = "&range_underflow_msg")]
    pub range_underflow_msg: &'a (dyn Fn(&ScalarValidator<'a, T>, T) -> String + Send + Sync),

    #[builder(default = "&range_overflow_msg")]
    pub range_overflow_msg: &'a (dyn Fn(&ScalarValidator<'a, T>, T) -> String + Send + Sync),
}

impl<'a, T> ScalarValidator<'a, T>
    where
        T: ScalarValue,
{
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   ScalarValidator, ViolationEnum,
    ///   range_overflow_msg, range_underflow_msg, value_missing_msg,
    /// };
    ///
    /// let input = ScalarValidator::<usize>::new();
    ///
    /// // Assert defaults
    /// // ----
    /// assert_eq!(input.break_on_failure, false);
    /// assert_eq!(input.min, None);
    /// assert_eq!(input.max, None);
    /// ```
    pub fn new() -> Self {
        ScalarValidator {
            break_on_failure: false,
            min: None,
            max: None,
            range_underflow_msg: &(range_underflow_msg),
            range_overflow_msg: &(range_overflow_msg),
        }
    }
}

impl<'a, T> ValidateValue<T> for ScalarValidator<'a, T>
    where
        T: ScalarValue,
{
    /// Validates given value against contained constraints and returns a result of unit and/or a Vec of violation tuples
    /// if value doesn't pass validation.
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   ScalarValidator, ViolationEnum,
    ///   ScalarValidatorBuilder,
    ///   range_underflow_msg, range_overflow_msg, value_missing_msg,
    ///   ScalarValue
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
    /// let usize_required = ScalarValidatorBuilder::<usize>::default()
    ///   .min(1)
    ///   .max(10)
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
    ///      value_missing_msg()),
    ///   ])),
    ///   ("With valid value", &usize_required, Some(4), Ok(())),
    ///   ("With \"out of lower bounds\" value", &usize_required, Some(0), Err(vec![
    ///     (ViolationEnum::RangeUnderflow,
    ///      range_underflow_msg(&usize_required, 0)),
    ///   ])),
    ///   ("With \"out of upper bounds\" value", &usize_required, Some(11), Err(vec![
    ///     (ViolationEnum::RangeOverflow, range_overflow_msg(&usize_required, 11)),
    ///     (ViolationEnum::CustomError, "Must be even".to_string()),
    ///   ])),
    ///   ("With \"out of upper bounds\" value, and 'break_on_failure: true'", &usize_break_on_failure, Some(11), Err(vec![
    ///     (ViolationEnum::RangeOverflow, range_overflow_msg(&usize_required, 11)),
    ///   ])),
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
    fn validate(&self, value: T) -> ValidationResult {
        let mut errs = vec![];

        // Test lower bound
        if let Some(min) = self.min {
            if value < min {
                errs.push((
                    ViolationEnum::RangeUnderflow,
                    (self.range_underflow_msg)(self, value),
                ));

                if self.break_on_failure {
                    return Err(errs);
                }
            }
        }

        // Test upper bound
        if let Some(max) = self.max {
            if value > max {
                errs.push((
                    ViolationEnum::RangeOverflow,
                    (self.range_overflow_msg)(self, value),
                ));

                if self.break_on_failure {
                    return Err(errs);
                }
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

/// Returns generic range underflow message.
///
/// ```rust
/// use walrs_inputfilter::{ScalarValidatorBuilder, range_underflow_msg};
///
/// let input = ScalarValidatorBuilder::<usize>::default()
///   .min(1)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_underflow_msg(&input, 0), "`0` is less than minimum `1`.");
/// ```
pub fn range_underflow_msg<T: ScalarValue>(rules: &ScalarValidator<T>, x: T) -> String {
    format!(
        "`{:}` is less than minimum `{:}`.",
        x,
        &rules.min.unwrap()
    )
}

/// Returns generic range overflow message.
///
/// ```rust
/// use walrs_inputfilter::{ScalarValidatorBuilder, range_overflow_msg};
///
/// let input = ScalarValidatorBuilder::<usize>::default()
///   .max(10)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_overflow_msg(&input, 100), "`100` is greater than maximum `10`.");
/// ```
pub fn range_overflow_msg<T: ScalarValue>(rules: &ScalarValidator<T>, x: T) -> String {
    format!(
        "`{:}` is greater than maximum `{:}`.",
        x,
        &rules.max.unwrap()
    )
}

impl<T: ScalarValue> Default for ScalarValidator<'_, T> {
    /// Returns a new instance with all fields set to defaults.
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   ScalarValidator
    /// };
    ///
    /// let input = ScalarValidator::<usize>::default();
    ///
    /// // Assert defaults
    /// // ----
    /// assert_eq!(input.break_on_failure, false);
    /// assert_eq!(input.min, None);
    /// assert_eq!(input.max, None);
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ScalarValue> Display for ScalarValidator<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScalarValidator {{ break_on_failure: {}, min: {}, max: {}, }}",
            self.break_on_failure,
            self.min.map_or("None".to_string(), |x| x.to_string()),
            self.max.map_or("None".to_string(), |x| x.to_string()),
        )
    }
}

impl<T: ScalarValue> Debug for ScalarValidator<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate() {
        // Setup a custom validator
        let validate_is_even = |x: usize| if x % 2 != 0 {
            Err(vec![(ViolationEnum::CustomError, "Must be even".to_string())])
        } else {
            Ok(())
        };

        // Setup input constraints
        let usize_required = ScalarValidatorBuilder::<usize>::default()
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
                value_missing_msg(),
            ])),
            ("With valid value", &usize_required, Some(4), Ok(())),
            ("With \"out of lower bounds\" value", &usize_required, Some(0), Err(vec![
                range_underflow_msg(&usize_required, 0),
            ])),
            ("With \"out of upper bounds\" value", &usize_required, Some(11), Err(vec![
                range_overflow_msg(&usize_required, 11),
                "Must be even".to_string(),
            ])),
            ("With \"out of upper bounds\" value, and 'break_on_failure: true'", &usize_break_on_failure,
             Some(11), Err(vec![
                range_overflow_msg(&usize_required, 11),
            ])),
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

        let usize_input_default = ScalarValidatorBuilder::<usize>::default()
            .build()
            .unwrap();

        let usize_not_required = ScalarValidatorBuilder::<usize>::default()
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
            ("1-10, Even, with invalid value", &usize_not_required, Some(0), Err(vec![
                (ViolationEnum::RangeUnderflow,
                 range_underflow_msg(&usize_not_required, 0))
            ])),
            ("1-10, Even, with invalid value(2)", &usize_not_required, Some(11), Err(vec![
                (ViolationEnum::RangeOverflow, range_overflow_msg(&usize_not_required, 11)),
                (ViolationEnum::CustomError, "Must be even".to_string()),
            ])),
            ("1-10, Even, with invalid value (3)", &usize_not_required, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
            ("1-10, Even, with valid value", &usize_not_required, Some(8), Ok(())),

            // Required
            // ----
            ("1-10, Even, required, no value", &usize_required, None, Err(vec![
                (ViolationEnum::ValueMissing,
                 value_missing_msg()),
            ])),
            ("1-10, Even, required, with valid value", &usize_required, Some(2), Ok(())),
            ("1-10, Even, required, with valid value (1)", &usize_required, Some(4), Ok(())),
            ("1-10, Even, required, with valid value (2)", &usize_required, Some(8), Ok(())),
            ("1-10, Even, required, with valid value (3)", &usize_required, Some(10), Ok(())),
            ("1-10, Even, required, with invalid value", &usize_required, Some(0), Err(vec![
                (ViolationEnum::RangeUnderflow,
                 range_underflow_msg(&usize_required, 0)),
            ])),
            ("1-10, Even, required, with invalid value(2)", &usize_required, Some(11), Err(vec![
                (ViolationEnum::RangeOverflow, range_overflow_msg(&usize_required, 11)),
                (ViolationEnum::CustomError, "Must be even".to_string()),
            ])),
            ("1-10, Even, required, with invalid value (3)", &usize_required, Some(7), Err(vec![
                (ViolationEnum::CustomError,
                 "Must be even".to_string()),
            ])),
            ("1-10, Even, required, 'break-on-failure: true', with multiple violations", &usize_break_on_failure, Some(11), Err(vec![
                (ViolationEnum::RangeOverflow, range_overflow_msg(&usize_break_on_failure, 11)),
            ])),
            ("1-10, Even, required, 'break-on-failure: true', with multiple violations (2)", &usize_break_on_failure, Some(0), Err(vec![
                (ViolationEnum::RangeUnderflow, range_underflow_msg(&usize_break_on_failure, 0)),
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
        let f64_input_required = ScalarValidatorBuilder::<f64>::default()
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
             value_missing_msg()),
        ]));
        assert_eq!(f64_input_required.validate_detailed(Some(2.0)), Ok(()));
        assert_eq!(f64_input_required.validate_detailed(Some(11.0)), Err(vec![
            (ViolationEnum::RangeOverflow, range_overflow_msg(&f64_input_required, 11.0)),
            (ViolationEnum::CustomError, "Must be even".to_string()),
        ]));

        // Test `char` usage
        let char_input = ScalarValidatorBuilder::<char>::default()
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
    fn test_filter() -> Result<(), Box<dyn std::error::Error>> {
        // Setup input constraints
        // ----
        // 1. With no filters.
        let usize_input_default = ScalarValidatorBuilder::<usize>::default().build()?;

        // 2. With one filter.
        let usize_input_twofold = ScalarValidatorBuilder::<usize>::default()
            .filters(vec![
                &|x: Option<usize>| x.map(|_x| _x * 2usize),
            ])
            .build()?;

        // 3. With two filters.
        let usize_input_gte_four = ScalarValidatorBuilder::<usize>::default()
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
    fn test_validate_and_filter() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

}
