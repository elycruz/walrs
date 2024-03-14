use std::fmt::{Debug, Display, Formatter};
use crate::{Filter, InputConstraints, InputValue,
            Validator, ViolationEnum, ViolationMessage, ViolationTuple};

type ValueMissingCallback<T, FT> = dyn Fn(&Input<T, FT>) -> ViolationMessage + Send + Sync;

pub fn value_missing_msg_getter<T: InputValue, FT: From<T>>(_: &Input<T, FT>) -> ViolationMessage {
    "Value is missing".to_string()
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct Input<'a, 'b, T, FT>
    where T: InputValue + 'b,
          FT: 'b + From<T>,
{
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
    pub name: Option<&'a str>,

    #[builder(default = "None")]
    pub default_value: Option<T>,

    #[builder(default = "None")]
    pub validators: Option<Vec<&'a Validator<T>>>,

    #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<Option<FT>>>>,

    #[builder(default = "&range_underflow_msg_getter")]
    pub range_underflow_msg: &'a (dyn Fn(&Input<'a, 'b, T, FT>, T) -> String + Send + Sync),

    #[builder(default = "&range_overflow_msg_getter")]
    pub range_overflow_msg: &'a (dyn Fn(&Input<'a, 'b, T, FT>, T) -> String + Send + Sync),

    #[builder(default = "&value_missing_msg_getter")]
    pub value_missing_msg: &'a (dyn Fn(&Input<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync)
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
    /// assert_eq!(input.min, None);
    /// assert_eq!(input.max, None);
    /// assert_eq!(input.required, false);
    /// assert!(input.custom.is_none());
    /// assert_eq!(input.name, None);
    /// assert_eq!(input.default_value, None);
    /// assert!(input.validators.is_none());
    /// assert!(input.filters.is_none());
    /// ```
    pub fn new() -> Self {
        Input {
            break_on_failure: false,
            min: None,
            max: None,
            required: false,
            custom: None,
            name: None,
            default_value: None,
            validators: None,
            filters: None,
            range_underflow_msg: &(range_underflow_msg_getter),
            range_overflow_msg: &(range_overflow_msg_getter),
            value_missing_msg: &value_missing_msg_getter,
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


impl<'a, 'b, T: 'b, FT: 'b + From<T>> InputConstraints<'a, 'b, T, FT> for Input<'a, 'b, T, FT>
where
  T: InputValue,
{
  /// Validates given value against contained constraints, and returns a result of unit, and/or, a Vec of
  /// Violation messages.
  ///
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   Input, InputConstraints, ViolationEnum,
  ///   InputBuilder,
  ///   range_underflow_msg_getter, range_overflow_msg_getter, value_missing_msg_getter,
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
  /// let usize_required = InputBuilder::<usize, usize>::default()
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
  ///      value_missing_msg_getter(&usize_required),
  ///   ])),
  ///   ("With valid value", &usize_required, Some(4), Ok(())),
  ///   ("With \"out of lower bounds\" value", &usize_required, Some(0), Err(vec![
  ///      range_underflow_msg_getter(&usize_required, 0),
  ///   ])),
  ///   ("With \"out of upper bounds\" value", &usize_required, Some(11), Err(vec![
  ///     range_overflow_msg_getter(&usize_required, 11),
  ///     "Must be even".to_string(),
  ///   ])),
  ///   ("With \"out of upper bounds\" value, and 'break_on_failure: true'", &usize_break_on_failure, Some(11), Err(vec![
  ///     range_overflow_msg_getter(&usize_required, 11),
  ///   ])),
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
  ///   Input, InputConstraints, ViolationEnum,
  ///   InputBuilder,
  ///   range_underflow_msg_getter, range_overflow_msg_getter, value_missing_msg_getter,
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
  /// let usize_required = InputBuilder::<usize, usize>::default()
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
  ///      value_missing_msg_getter(&usize_required)),
  ///   ])),
  ///   ("With valid value", &usize_required, Some(4), Ok(())),
  ///   ("With \"out of lower bounds\" value", &usize_required, Some(0), Err(vec![
  ///     (ViolationEnum::RangeUnderflow,
  ///      range_underflow_msg_getter(&usize_required, 0)),
  ///   ])),
  ///   ("With \"out of upper bounds\" value", &usize_required, Some(11), Err(vec![
  ///     (ViolationEnum::RangeOverflow, range_overflow_msg_getter(&usize_required, 11)),
  ///     (ViolationEnum::CustomError, "Must be even".to_string()),
  ///   ])),
  ///   ("With \"out of upper bounds\" value, and 'break_on_failure: true'", &usize_break_on_failure, Some(11), Err(vec![
  ///     (ViolationEnum::RangeOverflow, range_overflow_msg_getter(&usize_required, 11)),
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
  ///   InputConstraints,
  /// };
  ///
  /// // Setup input constraints
  /// let usize_input = InputBuilder::<usize, usize>::default()
  ///   .filters(vec![&|x: Option<usize>| x.map(|_x| _x * 2usize)])
  ///   .build()
  ///   .unwrap();
  ///
  /// let test_cases = [
  ///   (&usize_input, None, None),
  ///   (&usize_input, Some(0), Some(0)),
  ///   (&usize_input, Some(2), Some(4)),
  ///   (&usize_input, Some(4), Some(8)),
  /// ];
  ///
  /// // Run test cases
  /// for (i, (input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
  ///   println!("Case {}: `(usize_input.filter)({:?}) == {:?}`", i + 1, value.clone(), expected_rslt.clone());
  ///   assert_eq!(input.filter(value), expected_rslt);
  /// }
  /// ```
  ///
  fn filter(&self, value: Option<FT>) -> Option<FT> {
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
  ///   InputConstraints,
  ///   ViolationEnum::CustomError,
  /// };
  ///
  /// // Setup input constraints
  /// let usize_input = InputBuilder::<usize, usize>::default()
  ///   .min(1)
  ///   .max(10)
  ///   .required(true)
  ///   .validators(vec![&|x: usize| if x % 2 != 0 {
  ///     Err(vec![(CustomError, "Must be even".to_string())])
  ///   } else {
  ///     Ok(())
  ///   }])
  ///   .filters(vec![&|x: Option<usize>| x.map(|_x| _x * 2usize)])
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
  ///   InputConstraints,
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
  /// // Setup input constraints
  /// let usize_input = InputBuilder::<usize, usize>::default()
  ///   .min(1)
  ///   .max(10)
  ///   .required(true)
  ///   .validators(vec![&|x: usize| if x % 2 != 0 {
  ///     Err(vec![(CustomError, "Must be even".to_string())])
  ///   } else {
  ///     Ok(())
  ///   }])
  ///   .filters(vec![&|x: Option<usize>| x.map(|_x| _x * 2usize)])
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
    self.validate_detailed(x).map(|_| self.filter(x.map(|_x| _x.into())))
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

impl<'a, 'b, T: InputValue + 'b, FT: 'b + From<T>> Display for Input<'a, 'b, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Input {{ break_on_failure: {}, min: {}, max: {}, required: {}, validators: {}, filters: {} }}",
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

impl<'a, 'b, T: InputValue + 'b, FT: 'b + From<T>> Debug for Input<'a, 'b, T, FT> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

/// Returns generic range underflow message.
///
/// ```rust
/// use walrs_inputfilter::{InputBuilder, range_underflow_msg_getter};
///
/// let input = InputBuilder::<usize, usize>::default()
///   .min(1)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_underflow_msg_getter(&input, 0), "`0` is less than minimum `1`.");
/// ```
pub fn range_underflow_msg_getter<T: InputValue, FT: From<T>>(rules: &Input<T, FT>, x: T) -> String {
    format!(
        "`{:}` is less than minimum `{:}`.",
        x,
        &rules.min.unwrap()
    )
}

/// Returns generic range overflow message.
///
/// ```rust
/// use walrs_inputfilter::{InputBuilder,
/// range_overflow_msg_getter};
///
/// let input = InputBuilder::<usize, usize>::default()
///   .max(10)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_overflow_msg_getter(&input, 100), "`100` is greater than maximum `10`.");
/// ```
pub fn range_overflow_msg_getter<T: InputValue, FT: From<T>>(rules: &Input<T, FT>, x: T) -> String {
    format!(
        "`{:}` is greater than maximum `{:}`.",
        x,
        &rules.max.unwrap()
    )
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::error::Error;
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
        // float_percent.constraints = Some(Box::new(InputBuilder::<usize>::default()
        //     .min(0)
        //     .max(100)
        //     .validators(vec![
        //         &|x| if x != 0 && x % 5 != 0 {
        //             Err(vec![(StepMismatch, format!("{} is not divisible by 5", x))])
        //         } else {
        //             Ok(())
        //         },
        //     ])
        //     .build()?
        // ));
        //
        // assert_eq!(float_percent.validate(Some(5)), Ok(()));
        // assert_eq!(float_percent.validate(Some(101)),
        //            Err(vec![
        //                // range_overflow_msg_getter(
        //                //     float_percent.constraints.as_deref().unwrap()
        //                //         .downcast_ref::<Input<usize>>().unwrap(),
        //                //     101usize
        //                // ),
        //                "`101` is greater than maximum `100`.".to_string(),
        //                "101 is not divisible by 5".to_string(),
        //            ]));
        // assert_eq!(float_percent.validate(Some(26)),
        //            Err(vec!["26 is not divisible by 5".to_string()]));
        //
        let _ = Input::<&str, Cow<str>>::new();
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
}
