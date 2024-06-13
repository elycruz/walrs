use std::fmt::{Debug, Display, Formatter};

use crate::{ViolationEnum, ScalarValue, ViolationTuple, Filter, InputConstraints, Validator, ViolationMessage};

/// Returns a generic message for "Value is missing" violation.
///
/// ```rust
/// use walrs_inputfilter::{scalar_missing_msg_getter, ScalarInput};
///
/// let input = ScalarInput::<usize>::new();
///
/// assert_eq!(scalar_missing_msg_getter(&input), "Value is missing".to_string());
/// ```
pub fn scalar_missing_msg_getter<T: ScalarValue>(_: &ScalarInput<T>) -> ViolationMessage {
    "Value is missing".to_string()
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct ScalarInput<'a, T: ScalarValue> {
    #[builder(default = "false")]
    pub break_on_failure: bool,

    #[builder(default = "None")]
    pub min: Option<T>,

    #[builder(default = "None")]
    pub max: Option<T>,

    #[builder(default = "false")]
    pub required: bool,

    #[builder(default = "None")]
    pub custom: Option<&'a Validator<T>>,

    #[builder(default = "None")]
    pub validators: Option<Vec<&'a Validator<T>>>,

    #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<T>>>,

    #[builder(default = "&range_underflow_msg")]
    pub range_underflow_msg: &'a (dyn Fn(&ScalarInput<'a, T>, T) -> String + Send + Sync),

    #[builder(default = "&range_overflow_msg")]
    pub range_overflow_msg: &'a (dyn Fn(&ScalarInput<'a, T>, T) -> String + Send + Sync),

    #[builder(default = "&scalar_missing_msg_getter")]
    pub scalar_missing_msg_getter: &'a (dyn Fn(&ScalarInput<'a, T>) -> ViolationMessage + Send + Sync),
}

impl<'a, T> ScalarInput<'a, T>
    where
        T: ScalarValue,
{
    /// Returns a new instance with all fields set defaults.
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   ScalarInput, InputConstraints, ViolationEnum,
    ///   range_overflow_msg,
    ///   range_underflow_msg, 
    ///   scalar_missing_msg_getter,
    /// };
    ///
    /// let input = ScalarInput::<usize>::new();
    ///
    /// // Assert defaults
    /// // ----
    /// assert_eq!(input.break_on_failure, false);
    /// assert_eq!(input.min, None);
    /// assert_eq!(input.max, None);
    /// assert_eq!(input.required, false);
    /// assert!(input.validators.is_none());
    /// assert!(input.filters.is_none());
    /// ```
    pub fn new() -> Self {
        ScalarInput {
            break_on_failure: false,
            min: None,
            max: None,
            required: false,
            custom: None,
            validators: None,
            filters: None,
            range_underflow_msg: &(range_underflow_msg),
            range_overflow_msg: &(range_overflow_msg),
            scalar_missing_msg_getter: &(scalar_missing_msg_getter),
        }
    }

    fn _validate_against_own_constraints(&self, value: T) -> Result<(), Vec<ViolationTuple>> {
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

        if let Some(custom) = self.custom {
            if let Err(mut custom_errs) = (custom)(value) {
                errs.append(custom_errs.as_mut());

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

    fn _validate_against_validators(&self, value: T) -> Result<(), Vec<ViolationTuple>> {
        self
            .validators
            .as_deref()
            .map(|vs| {
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
                }

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

impl<'a, 'b, T> InputConstraints<T, T> for ScalarInput<'a, T>
    where
        T: ScalarValue,
{
    /// Validates given value against contained constraints, and returns a result of unit, and/or, a Vec of
    /// Violation messages.
    ///
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   ScalarInput, InputConstraints, ViolationEnum,
    ///   ScalarInputBuilder,
    ///   range_underflow_msg, range_overflow_msg, scalar_missing_msg_getter,
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
    /// let usize_required = ScalarInputBuilder::<usize>::default()
    ///   .min(1)
    ///   .max(10)
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
    ///      scalar_missing_msg_getter(&usize_required),
    ///   ])),
    ///   ("With valid value", &usize_required, Some(4), Ok(())),
    ///   ("With \"out of lower bounds\" value", &usize_required, Some(0), Err(vec![
    ///      range_underflow_msg(&usize_required, 0),
    ///   ])),
    ///   ("With \"out of upper bounds\" value", &usize_required, Some(11), Err(vec![
    ///     range_overflow_msg(&usize_required, 11),
    ///     "Must be even".to_string(),
    ///   ])),
    ///   ("With \"out of upper bounds\" value, and 'break_on_failure: true'", &usize_break_on_failure, Some(11), Err(vec![
    ///     range_overflow_msg(&usize_required, 11),
    ///   ])),
    ///   ("With \"not Even\" value", &usize_required, Some(7), Err(vec![
    ///      "Must be even".to_string(),
    ///   ])),
    /// ];
    ///
    /// println!("Doc tests for `validate`:");
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
    ///   ScalarInput, InputConstraints, ViolationEnum,
    ///   ScalarInputBuilder,
    ///   range_underflow_msg, range_overflow_msg, scalar_missing_msg_getter,
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
    /// let usize_required = ScalarInputBuilder::<usize>::default()
    ///   .min(1)
    ///   .max(10)
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
    ///      scalar_missing_msg_getter(&usize_required)),
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
    /// println!("Doc tests for `validate_detailed`:");
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
                        (self.scalar_missing_msg_getter)(self),
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
    ///   ScalarInputBuilder,
    ///   InputConstraints,
    /// };
    ///
    /// // Setup input constraints
    /// let usize_input = ScalarInputBuilder::<usize>::default()
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
    ///   println!("Case {}: `(usize_input.filter)({:?}) == {:?}`", i + 1, value.clone(), expected_rslt.clone());
    ///   assert_eq!(input.filter(value), expected_rslt);
    /// }
    /// ```
    ///
    fn filter(&self, value: T) -> T {
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
    ///   ScalarInputBuilder,
    ///   ScalarInput,
    ///   ViolationMessage,
    ///   ViolationTuple,
    ///   InputConstraints,
    ///   ViolationEnum::CustomError,
    ///   scalar_missing_msg_getter
    /// };
    ///
    /// // Setup input constraints
    /// let u64_input = ScalarInputBuilder::<u64>::default()
    ///   .min(1)
    ///   .max(10)
    ///   .required(true)
    ///   .validators(vec![&|x: u64| if x % 2 != 0 {
    ///     Err(vec![(CustomError, "Must be even".to_string())])
    ///   } else {
    ///     Ok(())
    ///   }])
    ///   .filters(vec![&|x: u64| x| x * 2u64])
    ///   .build()
    ///   .unwrap();
    ///
    /// // Stops validation on first validation error and returns `Err` result.
    /// let u64_input_break_on_failure = {
    ///   let mut new_input = u64_input.clone();
    ///   new_input.break_on_failure = true;
    ///   new_input
    /// };
    ///
    /// let test_cases: Vec<(&str, &ScalarInput<u64>, Option<u64>, Result<Option<u64>, Vec<String>>)> = vec![
    ///   ("No value", &u64_input, None, Err(vec![ scalar_missing_msg_getter(&u64_input) ])),
    ///   ("With valid value", &u64_input, Some(4u64), Ok(Some(8u64))),
    ///   ("With \"out of lower bounds\" value", &u64_input, Some(0), Err(vec![
    ///     "`0` is less than minimum `1`.".to_string(),
    ///   ])),
    ///   ("With \"out of upper bounds\" value", &u64_input, Some(11), Err(vec![
    ///     "`11` is greater than maximum `10`.".to_string(),
    ///     "Must be even".to_string(),
    ///   ])),
    ///   ("With \"not Even\" value", &u64_input, Some(7), Err(vec![
    ///     "Must be even".to_string(),
    ///   ])),
    ///   ("With \"not Even\" value, and 'break_on_failure: true'", &u64_input_break_on_failure,
    ///     Some(7),
    ///     Err(vec![
    ///     "Must be even".to_string(),
    ///     ])
    ///   ),
    /// ];
    ///
    /// println!("Doc tests for `validate_and_filter`:");
    ///
    /// // Run test cases
    /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
    ///   println!("Case {}: {} - For: {:?}", i + 1, test_name, value);
    ///   assert_eq!(input.validate_and_filter(value), expected_rslt);
    /// }
    /// ```
    fn validate_and_filter(&self, x: Option<T>) -> Result<Option<T>, Vec<ViolationMessage>> {
        match self.validate_and_filter_detailed(x) {
            Err(messages) => Err(messages.into_iter().map(|(_, message)| message).collect()),
            Ok(filtered) => Ok(filtered),
        }
    }

    /// Validates, and filters, given value against contained rules, validators, and filters, respectively and
    /// returns a result of filtered value or a Vec of Violation tuples.
    /// ```rust
    /// use walrs_inputfilter::{
    ///   ScalarInputBuilder,
    ///   ScalarInput,
    ///   InputConstraints,
    ///   ViolationMessage,
    ///   ViolationEnum,
    ///   ViolationEnum::{
    ///     CustomError,
    ///     RangeOverflow,
    ///     RangeUnderflow,
    ///     ValueMissing,
    ///   },
    ///   scalar_missing_msg_getter,
    /// };
    ///
    /// // Setup input constraints
    /// let usize_input = ScalarInputBuilder::<usize>::default()
    ///   .min(1)
    ///   .max(10)
    ///   .required(true)
    ///   .validators(vec![&|x: usize| if x % 2 != 0 {
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
    /// type ConstraintStruct = ScalarInput<'static, usize>;
    /// type TestValue = Option<usize>;
    /// type ExpectedResult = Result<Option<usize>, Vec<(ViolationEnum, ViolationMessage)>>;
    ///
    /// let test_cases: Vec<(TestName, &ConstraintStruct, TestValue, ExpectedResult)> = vec![
    ///   ("No value", &usize_input, None, Err(vec![
    ///     (ValueMissing, scalar_missing_msg_getter(&usize_input))
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
    /// println!("Doc tests for `validate_and_filter_detailed`:");
    ///
    /// // Run test cases
    /// for (i, (test_name, input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
    ///   println!("Case {}: {}", i + 1, test_name);
    ///   assert_eq!(input.validate_and_filter_detailed(value), expected_rslt);
    /// }
    /// ```
    fn validate_and_filter_detailed(&self, x: Option<T>) -> Result<Option<T>, Vec<ViolationTuple>> {
        self.validate_detailed(x).map(|_| x.map(|_x| self.filter(_x)))
    }
}

/// Returns generic range underflow message.
///
/// ```rust
/// use walrs_inputfilter::{ScalarInputBuilder, range_underflow_msg};
///
/// let input = ScalarInputBuilder::<usize>::default()
///   .min(1)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_underflow_msg(&input, 0), "`0` is less than minimum `1`.");
/// ```
pub fn range_underflow_msg<T: ScalarValue>(rules: &ScalarInput<T>, x: T) -> String {
    format!(
        "`{}` is less than minimum `{}`.",
        x,
        &rules.min.unwrap()
    )
}

/// Returns generic range overflow message.
///
/// ```rust
/// use walrs_inputfilter::{ScalarInputBuilder, range_overflow_msg};
///
/// let input = ScalarInputBuilder::<usize>::default()
///   .max(10)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_overflow_msg(&input, 100), "`100` is greater than maximum `10`.");
/// ```
pub fn range_overflow_msg<T: ScalarValue>(rules: &ScalarInput<T>, x: T) -> String {
    format!(
        "`{}` is greater than maximum `{}`.",
        x,
        &rules.max.unwrap()
    )
}

impl<T: ScalarValue> Default for ScalarInput<'_, T> {
    /// Returns a new instance with all fields set to defaults.
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   ScalarInput, InputConstraints, ViolationEnum,
    ///   range_overflow_msg, range_underflow_msg, scalar_missing_msg_getter,
    /// };
    ///
    /// let input = ScalarInput::<usize>::default();
    ///
    /// // Assert defaults
    /// // ----
    /// assert_eq!(input.break_on_failure, false);
    /// assert_eq!(input.min, None);
    /// assert_eq!(input.max, None);
    /// assert_eq!(input.required, false);
    /// assert!(input.validators.is_none());
    /// assert!(input.filters.is_none());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ScalarValue> Display for ScalarInput<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScalarInput {{ break_on_failure: {}, min: {}, max: {}, required: {}, validators: {}, filters: {} }}",
            self.break_on_failure,
            self.min.map_or("None".to_string(), |x| x.to_string()),
            self.max.map_or("None".to_string(), |x| x.to_string()),
            self.required,
            self
                .validators
                .as_deref()
                .map(|vs| format!("Some([Validator; {}])", vs.len()))
                .unwrap_or("None".to_string()),
            self
                .filters
                .as_deref()
                .map(|fs| format!("Some([Filter; {}])", fs.len()))
                .unwrap_or("None".to_string()),
        )
    }
}

impl<T: ScalarValue> Debug for ScalarInput<'_, T> {
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
        let usize_required = ScalarInputBuilder::<usize>::default()
            .min(1)
            .max(10)
            .required(true)
            .validators(vec![&validate_is_even])
            .build()
            .unwrap();

        let usize_break_on_failure = {
            let mut new_input = usize_required.clone();
            new_input.break_on_failure = true;
            new_input
        };

        let test_cases = [
            ("No value", &usize_required, None, Err(vec![
                scalar_missing_msg_getter(&usize_required),
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

        let usize_input_default = ScalarInputBuilder::<usize>::default()
            .build()
            .unwrap();

        let usize_not_required = ScalarInputBuilder::<usize>::default()
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
                 scalar_missing_msg_getter(&usize_required)),
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
        let f64_input_required = ScalarInputBuilder::<f64>::default()
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
             scalar_missing_msg_getter(&f64_input_required)),
        ]));
        assert_eq!(f64_input_required.validate_detailed(Some(2.0)), Ok(()));
        assert_eq!(f64_input_required.validate_detailed(Some(11.0)), Err(vec![
            (ViolationEnum::RangeOverflow, range_overflow_msg(&f64_input_required, 11.0)),
            (ViolationEnum::CustomError, "Must be even".to_string()),
        ]));

        // Test `char` usage
        let char_input = ScalarInputBuilder::<char>::default()
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
        let usize_input_default = ScalarInputBuilder::<usize>::default().build()?;

        // 2. With one filter.
        let usize_input_twofold = ScalarInputBuilder::<usize>::default()
            .filters(vec![
                &|x: usize| x * 2usize,
            ])
            .build()?;

        // 3. With two filters.
        let usize_input_gte_four = ScalarInputBuilder::<usize>::default()
            .filters(vec![
                &|x: usize| if x < 4 { 4 } else { x },
                &|x: usize| x * 2usize,
            ])
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
