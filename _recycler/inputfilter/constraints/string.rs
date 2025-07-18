use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use regex::Regex;

use crate::{
    ViolationType, ViolationTuple, ValidationResult, value_missing_msg, ValueMissingCallback,
    Filter, InputConstraints, Validator, ViolationMessage
};

pub type StringConstraintsViolationCallback = dyn Fn(&StringConstraints, Option<&str>) -> ViolationMessage + Send + Sync;

pub fn pattern_mismatch_msg(rules: &StringConstraints, xs: Option<&str>) -> String {
    format!(
        "`{}` does not match pattern `{}`",
        &xs.as_ref().unwrap(),
        rules.pattern.as_ref().unwrap().as_str()
    )
}

pub fn too_short_msg(rules: &StringConstraints, xs: Option<&str>) -> String {
    format!(
        "Value length `{}` is less than allowed minimum `{}`.",
        &xs.as_ref().unwrap().len(),
        &rules.min_length.unwrap_or(0)
    )
}

pub fn too_long_msg(rules: &StringConstraints, xs: Option<&str>) -> String {
    format!(
        "Value length `{}` is greater than allowed maximum `{}`.",
        &xs.as_ref().unwrap().len(),
        &rules.max_length.unwrap_or(0)
    )
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct StringConstraints<'a, 'b> {
    #[builder(default = "false")]
    pub break_on_failure: bool,

    #[builder(default = "None")]
    pub min_length: Option<usize>,

    #[builder(default = "None")]
    pub max_length: Option<usize>,

    #[builder(default = "None")]
    pub pattern: Option<Regex>,

    #[builder(default = "false")]
    pub required: bool,

    #[builder(default = "None")]
    pub custom: Option<&'a Validator<&'b str>>,

    #[builder(default = "None")]
    pub validators: Option<Vec<&'a Validator<&'b str>>>,

    #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<Option<Cow<'b, str>>>>>,

    #[builder(default = "&too_short_msg")]
    pub too_short_msg: &'a StringConstraintsViolationCallback,

    #[builder(default = "&too_long_msg")]
    pub too_long_msg: &'a StringConstraintsViolationCallback,

    #[builder(default = "&pattern_mismatch_msg")]
    pub pattern_mismatch_msg: &'a StringConstraintsViolationCallback,

    #[builder(default = "&value_missing_msg")]
    pub value_missing_msg: &'a ValueMissingCallback,
}

impl<'a, 'b> StringConstraints<'a, 'b> {
    pub fn new() -> Self {
        StringConstraints {
            break_on_failure: false,
            min_length: None,
            max_length: None,
            pattern: None,
            required: false,
            custom: None,
            validators: None,
            filters: None,
            too_short_msg: &(too_long_msg),
            too_long_msg: &(too_long_msg),
            pattern_mismatch_msg: &(pattern_mismatch_msg),
            value_missing_msg: &value_missing_msg,
        }
    }

    fn _validate_against_own_constraints(&self, value: &'b str) -> ValidationResult {
        let mut errs = vec![];

        if let Some(min_length) = self.min_length {
            if value.len() < min_length {
                errs.push((
                    ViolationType::TooShort,
                    (self.too_short_msg)(self, Some(value)),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        if let Some(max_length) = self.max_length {
            if value.len() > max_length {
                errs.push((
                    ViolationType::TooLong,
                    (self.too_long_msg)(self, Some(value)),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        if let Some(pattern) = &self.pattern {
            if !pattern.is_match(value) {
                errs.push((
                    ViolationType::PatternMismatch,
                    (self.pattern_mismatch_msg)(self, Some(value)),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }

    fn _validate_against_validators(&self, value: &'b str) -> Result<(), Vec<ViolationTuple>> {
        self.validators.as_deref().map(|vs| {

            // If not break on failure then capture all validation errors.
            if !self.break_on_failure {
                return vs.iter().fold(
                    Vec::<ViolationTuple>::new(),
                    |mut agg, f| match f(value) {
                        Err(mut message_tuples) => {
                            agg.append(message_tuples.as_mut());
                            agg
                        }
                        _ => agg,
                    });
            }

            // Else break on, and capture, first failure.
            let mut agg = Vec::<ViolationTuple>::new();
            for f in vs.iter() {
                if let Err(mut message_tuples) = f(value) {
                    agg.append(message_tuples.as_mut());
                    break;
                }
            }
            agg
        })
            .and_then(|messages| if messages.is_empty() { None } else { Some(messages) })
            .map_or(Ok(()), Err)
    }
}

impl<'a, 'b> InputConstraints<'a, 'b, &'b str, Cow<'b, str>> for StringConstraints<'a, 'b> {
    /// Same as `validate_detailed` only the violation messages are returned.
    ///
    /// ```rust
    /// use walrs_inputfilter::*;
    /// use walrs_inputfilter::pattern::PatternValidator;
    /// use walrs_inputfilter::violation::ViolationType::{
    ///   ValueMissing, TooShort, TooLong, TypeMismatch, CustomError,
    ///   RangeOverflow, RangeUnderflow, StepMismatch
    /// };
    ///
    /// let str_input = StringConstraintsBuilder::default()
    ///  .required(true)
    ///  .value_missing_msg(&|| "Value missing".to_string())
    ///  .min_length(3usize)
    ///  .too_short_msg(&|_, _| "Too short".to_string())
    ///  .max_length(200usize) // Default violation message callback used here.
    ///   // Naive email pattern validator (naive for this example).
    ///  .validators(vec![&|x: &str| {
    ///     if !x.contains('@') {
    ///       return Err(vec![(TypeMismatch, "Invalid email".to_string())]);
    ///     }
    ///     Ok(())
    ///   }])
    ///  .build()
    ///  .unwrap();
    ///
    /// let too_long_str = &"ab".repeat(201);
    ///
    /// assert_eq!(str_input.validate(None), Err(vec![ "Value missing".to_string() ]));
    /// assert_eq!(str_input.validate(Some(&"ab")), Err(vec![
    ///     "Too short".to_string(),
    ///      "Invalid email".to_string(),
    /// ]));
    /// assert_eq!(str_input.validate(Some(&too_long_str)), Err(vec![
    ///     too_long_msg(&str_input, Some(&too_long_str)),
    ///     "Invalid email".to_string(),
    /// ]));
    /// assert_eq!(str_input.validate(Some(&"abc")), Err(vec![ "Invalid email".to_string() ]));
    /// assert_eq!(str_input.validate(Some(&"abc@def")), Ok(()));
    /// ```
    fn validate(&self, value: Option<&'b str>) -> Result<(), Vec<ViolationMessage>> {
        match self.validate_detailed(value) {
            // If errors, extract messages and return them
            Err(messages) =>
                Err(messages.into_iter().map(|(_, message)| message).collect()),
            Ok(_) => Ok(()),
        }
    }

    /// Validates value against contained constraints and validators, and returns a result of unit and/or a Vec of
    /// Violation tuples.
    ///
    /// ```rust
    /// use walrs_inputfilter::*;
    /// use walrs_inputfilter::pattern::PatternValidator;
    /// use walrs_inputfilter::violation::ViolationType::{
    ///   ValueMissing, TooShort, TooLong, TypeMismatch, CustomError,
    ///   RangeOverflow, RangeUnderflow, StepMismatch
    /// };
    ///
    /// let str_input = StringConstraintsBuilder::default()
    ///  .required(true)
    ///  .value_missing_msg(&|| "Value missing".to_string())
    ///  .min_length(3usize)
    ///  .too_short_msg(&|_, _| "Too short".to_string())
    ///  .max_length(200usize) // Default violation message callback used here.
    ///   // Naive email pattern validator (naive for this example).
    ///  .validators(vec![&|x: &str| {
    ///     if !x.contains('@') {
    ///       return Err(vec![(TypeMismatch, "Invalid email".to_string())]);
    ///     }
    ///     Ok(())
    ///   }])
    ///  .build()
    ///  .unwrap();
    ///
    /// let too_long_str = &"ab".repeat(201);
    ///
    /// assert_eq!(str_input.validate_detailed(None), Err(vec![ (ValueMissing, "Value missing".to_string()) ]));
    /// assert_eq!(str_input.validate_detailed(Some(&"ab")), Err(vec![
    ///     (TooShort, "Too short".to_string()),
    ///     (TypeMismatch, "Invalid email".to_string()),
    /// ]));
    /// assert_eq!(str_input.validate_detailed(Some(&too_long_str)), Err(vec![
    ///     (TooLong, too_long_msg(&str_input, Some(&too_long_str))),
    ///     (TypeMismatch, "Invalid email".to_string()),
    /// ]));
    /// assert_eq!(str_input.validate_detailed(Some(&"abc")), Err(vec![ (TypeMismatch, "Invalid email".to_string()) ]));
    /// assert_eq!(str_input.validate_detailed(Some(&"abc@def")), Ok(()));
    /// ```
    fn validate_detailed(&self, value: Option<&'b str>) ->  Result<(), Vec<ViolationTuple>> {
        match value {
            None => {
                if self.required {
                    Err(vec![(
                        ViolationType::ValueMissing,
                        (self.value_missing_msg)(),
                    )])
                } else {
                    Ok(())
                }
            }
            // Else if value is populated validate it
            Some(v) => match self._validate_against_own_constraints(v) {
                Ok(_) => self._validate_against_validators(v),
                Err(messages1) =>  if self.break_on_failure {
                    Err(messages1)
                } else if let Err(mut messages2) = self._validate_against_validators(v) {
                    let mut agg = messages1;
                    agg.append(messages2.as_mut());
                    Err(agg)
                } else {
                    Err(messages1)
                }
            },
        }
    }

    fn filter(&self, value: Option<Cow<'b, str>>) -> Option<Cow<'b, str>> {
        match self.filters.as_deref() {
            None => value,
            Some(fs) =>
                fs.iter().fold(value, |agg, f| f(agg)),
        }
    }

    fn validate_and_filter_detailed(&self, x: Option<&'b str>) -> Result<Option<Cow<'b, str>>, Vec<ViolationTuple>> {
        self.validate_detailed(x).map(|_| self.filter(x.map(Cow::Borrowed)))
    }

    /// Special case of `validate_and_filter` where the error type enums are ignored (in `Err(...)`) result,
    /// and only the error messages are returned, for `Err` case.
    ///
    /// ```rust
    /// use walrs_inputfilter::*;
    /// use std::borrow::Cow;
    ///
    /// let input = StringConstraintsBuilder::default()
    ///   .required(true)
    ///   .value_missing_msg(&|| "Value missing".to_string())
    ///   .validators(vec![&|x: &str| {
    ///     if x.len() < 3 {
    ///       return Err(vec![(
    ///         ViolationType::TooShort,
    ///        "Too short".to_string(),
    ///       )]);
    ///     }
    ///     Ok(())
    ///   }])
    ///   .filters(vec![&|xs: Option<Cow<str>>| {
    ///     xs.map(|xs| Cow::Owned(xs.to_lowercase()))
    ///   }])
    ///   .build()
    ///   .unwrap()
    /// ;
    ///
    /// assert_eq!(input.validate_and_filter(Some(&"ab")), Err(vec!["Too short".to_string()]));
    /// assert_eq!(input.validate_and_filter(Some(&"Abba")), Ok(Some("Abba".to_lowercase().into())));
    /// assert_eq!(input.validate_and_filter(None), Err(vec!["Value missing".to_string()]));
    /// ```
    fn validate_and_filter(&self, x: Option<&'b str>) -> Result<Option<Cow<'b, str>>, Vec<ViolationMessage>> {
        match self.validate_and_filter_detailed(x) {
            Err(messages) =>
                Err(messages.into_iter().map(|(_, message)| message).collect()),
            Ok(filtered) => Ok(filtered),
        }
    }
}

impl Default for StringConstraints<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for StringConstraints<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StringConstraints {{ break_on_failure: {}, min_length: {}, max_length: {}, pattern: {}, required: {}, validators: {}, filters: {} }}",
            self.break_on_failure,
            self.min_length.map_or("None".to_string(), |x| x.to_string()),
            self.max_length.map_or("None".to_string(), |x| x.to_string()),
            self.pattern.as_ref().map_or("None".to_string(), |rx| rx.to_string()),
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

impl Debug for StringConstraints<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        ViolationType::{PatternMismatch, RangeOverflow},
    };
    use crate::validators::pattern::PatternValidator;
    use std::{error::Error, sync::Arc, thread};

    // Tests setup types
    fn less_than_1990_msg(value: &str) -> String {
        format!("{} is greater than 1989-12-31", value)
    }

    /// Faux validator that checks if the input is less than 1990-01-01.
    fn less_than_1990(x: &str) -> ValidationResult {
        if x >= "1989-12-31" {
            return Err(vec![(RangeOverflow, less_than_1990_msg(x))]);
        }
        Ok(())
    }

    fn ymd_mismatch_msg(s: &str, pattern_str: &str) -> String {
        format!("{} doesn't match pattern {}", s, pattern_str)
    }

    fn ymd_check(s: &str) -> ValidationResult {
        // Simplified ISO year-month-date regex
        let rx = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap();
        if !rx.is_match(s) {
            return Err(vec![(PatternMismatch, ymd_mismatch_msg(s, rx.as_str()))]);
        }
        Ok(())
    }

    /// Faux filter that returns the last date of the month.
    /// **Note:** Assumes that the input is a valid ISO year-month-date.
    fn to_last_date_of_month(x: Option<Cow<str>>) -> Option<Cow<str>> {
        x.map(|x| {
            let mut xs = x.into_owned();
            xs.replace_range(8..10, "31");
            Cow::Owned(xs)
        })
    }

    #[test]
    fn test_input_builder() -> Result<(), Box<dyn Error>> {
        // Simplified ISO year-month-date regex
        let ymd_regex = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$")?;
        let ymd_regex_2 = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$")?;
        let ymd_regex_arc_orig = Arc::new(ymd_regex);
        let ymd_regex_arc = Arc::clone(&ymd_regex_arc_orig);

        let ymd_mismatch_msg = Arc::new(move |s: &str| -> String {
            format!("{} doesn't match pattern {}", s, ymd_regex_arc.as_str())
        });

        let ymd_mismatch_msg_arc = Arc::clone(&ymd_mismatch_msg);
        let ymd_regex_arc = Arc::clone(&ymd_regex_arc_orig);

        let ymd_check = move |s: &str| -> ValidationResult {
            if !ymd_regex_arc.is_match(s) {
                return Err(vec![(PatternMismatch, ymd_mismatch_msg_arc(s))]);
            }
            Ok(())
        };

        // Validator case 1
        let pattern_validator = PatternValidator {
            pattern: Cow::Owned(ymd_regex_2),
            pattern_mismatch: &|validator, s| {
                format!("{} doesn't match pattern {}", s, validator.pattern.as_str())
            },
        };

        let less_than_1990_input = StringConstraintsBuilder::default()
            .validators(vec![&less_than_1990])
            .build()?;

        let yyyy_mm_dd_input = StringConstraintsBuilder::default()
            .validators(vec![&ymd_check])
            .build()?;

        let yyyy_mm_dd_input2 = StringConstraintsBuilder::default()
            .validators(vec![&pattern_validator])
            .build()?;

        // Missing value check
        if let Err(errs) = less_than_1990_input.validate(None) {
            panic!("Expected Ok(());  Received Err({:#?})", &errs);
        }

        // Mismatch check
        let value = "1000-99-999";
        match yyyy_mm_dd_input.validate_detailed(Some(value)) {
            Ok(_) => panic!("Expected Err(...);  Received Ok(())"),
            Err(tuples) => {
                assert_eq!(tuples[0].0, PatternMismatch);
                assert_eq!(tuples[0].1, ymd_mismatch_msg(value).as_str());
            }
        }

        // Valid check
        if let Err(errs) = yyyy_mm_dd_input.validate_detailed(None) {
            panic!("Expected Ok(());  Received Err({:#?})", &errs);
        }

        // Valid check 2
        let value = "1000-99-99";
        if let Err(errs) = yyyy_mm_dd_input.validate_detailed(Some(value)) {
            panic!("Expected Ok(());  Received Err({:#?})", &errs);
        }

        // Valid check
        let value = "1000-99-99";
        if let Err(errs) = yyyy_mm_dd_input2.validate_detailed(Some(value)) {
            panic!("Expected Ok(());  Received Err({:#?})", &errs);
        }

        Ok(())
    }

    #[test]
    fn test_thread_safety() -> Result<(), Box<dyn Error>> {
        let less_than_1990_input = StringConstraintsBuilder::default()
            .validators(vec![&less_than_1990])
            .build()?;

        let ymd_input = StringConstraintsBuilder::default()
            .validators(vec![&ymd_check])
            .build()?;

        let less_than_input = Arc::new(less_than_1990_input);
        let less_than_input_instance = Arc::clone(&less_than_input);

        let str_input = Arc::new(ymd_input);
        let str_input_instance = Arc::clone(&str_input);

        let handle =
            thread::spawn(
                move || match less_than_input_instance.validate_detailed(Some("2023-12-31")) {
                    Err(x) => {
                        assert_eq!(x[0].1.as_str(), less_than_1990_msg("2023-12-31"));
                    }
                    _ => panic!("Expected `Err(...)`"),
                },
            );

        let handle2 = thread::spawn(move || match str_input_instance.validate_detailed(Some("")) {
            Err(x) => {
                assert_eq!(
                    x[0].1.as_str(),
                    ymd_mismatch_msg(
                        "",
                        Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap().as_str(),
                    )
                );
            }
            _ => panic!("Expected `Err(...)`"),
        });

        // @note Conclusion of tests here is that validators can only (easily) be shared between threads if they are function pointers -
        //   closures are too loose and require over the top value management and planning due to the nature of multi-threaded
        //  contexts.

        // Contrary to the above, 'scoped threads', will allow variable sharing without requiring them to
        // be 'moved' first (as long as rust's lifetime rules are followed -
        //  @see https://blog.logrocket.com/using-rust-scoped-threads-improve-efficiency-safety/
        // ).

        handle.join().unwrap();
        handle2.join().unwrap();

        Ok(())
    }

    /// Example showing shared references in `StringConstraints`, and user-land, controls.
    #[test]
    fn test_thread_safety_with_scoped_threads_and_closures() -> Result<(), Box<dyn Error>> {
        let ymd_rx = Arc::new(Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap());
        let ymd_rx_clone = Arc::clone(&ymd_rx);

        let ymd_check = move |s: &str| -> ValidationResult {
            // Simplified ISO year-month-date regex
            if !ymd_rx_clone.is_match(s) {
                return Err(vec![(
                    PatternMismatch,
                    ymd_mismatch_msg(s, ymd_rx_clone.as_str()),
                )]);
            }
            Ok(())
        };

        let less_than_1990_input = StringConstraintsBuilder::default()
            .validators(vec![&less_than_1990])
            .filters(vec![&to_last_date_of_month])
            .build()?;

        let ymd_input = StringConstraintsBuilder::default()
            .validators(vec![&ymd_check])
            .build()?;

        let less_than_input = Arc::new(less_than_1990_input);
        let less_than_input_instance = Arc::clone(&less_than_input);
        let ymd_check_input = Arc::new(ymd_input);
        let ymd_check_input_instance = Arc::clone(&ymd_check_input);

        thread::scope(|scope| {
            scope.spawn(
                || match less_than_input_instance.validate_detailed(Some("2023-12-31")) {
                    Err(x) => {
                        assert_eq!(x[0].1.as_str(), &less_than_1990_msg("2023-12-31"));
                    }
                    _ => panic!("Expected `Err(...)`"),
                },
            );

            scope.spawn(
                || match less_than_input_instance.validate_and_filter_detailed(Some("1989-01-01")) {
                    Err(err) => panic!(
                        "Expected `Ok(Some({:#?})`;  Received `Err({:#?})`",
                        Cow::<str>::Owned("1989-01-31".to_string()),
                        err
                    ),
                    Ok(Some(x)) => assert_eq!(x, Cow::<str>::Owned("1989-01-31".to_string())),
                    _ => panic!("Expected `Ok(Some(Cow::Owned(99 * 2)))`;  Received `Ok(None)`"),
                },
            );

            scope.spawn(|| match ymd_check_input_instance.validate_detailed(Some("")) {
                Err(x) => {
                    assert_eq!(x[0].1.as_str(), ymd_mismatch_msg("", ymd_rx.as_str()));
                }
                _ => panic!("Expected `Err(...)`"),
            });

            scope.spawn(|| {
                if let Err(_err_tuple) = ymd_check_input_instance.validate(Some("2013-08-31")) {
                    panic!("Expected `Ok(());  Received Err(...)`")
                }
            });
        });

        Ok(())
    }

    #[test]
    fn test_validate_and_filter_detailed() {
        let input = StringConstraintsBuilder::default()
            .required(true)
            .validators(vec![&less_than_1990])
            .filters(vec![&to_last_date_of_month])
            .build()
            .unwrap();

        assert_eq!(
            input.validate_and_filter_detailed(Some("2023-12-31")),
            Err(vec![(RangeOverflow, less_than_1990_msg("2023-12-31"))])
        );
        assert_eq!(
            input.validate_and_filter_detailed(Some("1989-01-01")),
            Ok(Some(Cow::Owned("1989-01-31".to_string())))
        );
    }

    #[test]
    fn test_value_type() {
        let callback1 = |xs: &str| -> ValidationResult {
            if !xs.is_empty() {
                Ok(())
            } else {
                Err(vec![(
                    ViolationType::TypeMismatch,
                    "Error".to_string(),
                )])
            }
        };

        let _input = StringConstraintsBuilder::default()
            .validators(vec![&callback1])
            .build()
            .unwrap();
    }

    #[test]
    fn test_display() {
        let input = StringConstraintsBuilder::default()
            .validators(vec![&less_than_1990])
            .build()
            .unwrap();

        assert_eq!(
            input.to_string(),
            "StringConstraints { break_on_failure: false, min_length: None, max_length: None, pattern: None, required: false, validators: Some([Validator; 1]), filters: None }",
        );
    }
}
