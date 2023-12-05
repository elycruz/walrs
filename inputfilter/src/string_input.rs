use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use regex::Regex;

use crate::types::{Filter, InputConstraints, Validator, ViolationMessage};
use crate::{ConstraintViolation, ValidationErrTuple, ValidationResult};

pub type StrMissingViolationCallback = dyn Fn(&StringInput, Option<&str>) -> ViolationMessage + Send + Sync;

pub fn pattern_mismatch_msg(rules: &StringInput, xs: Option<&str>) -> String {
  format!(
    "`{}` does not match pattern `{}`",
    &xs.as_ref().unwrap(),
    rules.pattern.as_ref().unwrap()
  )
}

pub fn too_short_msg(rules: &StringInput, xs: Option<&str>) -> String {
  format!(
    "Value length `{:}` is less than allowed minimum `{:}`.",
    &xs.as_ref().unwrap().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn too_long_msg(rules: &StringInput, xs: Option<&str>) -> String {
  format!(
    "Value length `{:}` is greater than allowed maximum `{:}`.",
    &xs.as_ref().unwrap().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn str_not_equal_msg(rules: &StringInput, _: Option<&str>) -> String {
  format!(
    "Value is not equal to {}.",
    &rules.equal.as_deref().unwrap_or("")
  )
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct StringInput<'a, 'b> {
  #[builder(default = "true")]
  pub break_on_failure: bool,

  /// @todo This should be an `Option<Cow<'a, str>>`, for compatibility.
  #[builder(setter(into), default = "None")]
  pub name: Option<&'a str>,

  #[builder(default = "None")]
  pub min_length: Option<usize>,

  #[builder(default = "None")]
  pub max_length: Option<usize>,

  #[builder(default = "None")]
  pub pattern: Option<Regex>,

  #[builder(default = "None")]
  pub equal: Option<&'b str>,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(default = "None")]
  pub validators: Option<Vec<&'a Validator<&'b str>>>,

  // @todo Add support for `io_validators` (e.g., validators that return futures).

  #[builder(default = "None")]
  pub filters: Option<Vec<&'a Filter<Cow<'b, str>>>>,

  #[builder(default = "&too_short_msg")]
  pub too_short: &'a StrMissingViolationCallback,

  #[builder(default = "&too_long_msg")]
  pub too_long: &'a StrMissingViolationCallback,

  #[builder(default = "&pattern_mismatch_msg")]
  pub pattern_mismatch: &'a StrMissingViolationCallback,

  #[builder(default = "&str_not_equal_msg")]
  pub not_equal: &'a StrMissingViolationCallback,

  #[builder(default = "&str_missing_msg")]
  pub value_missing: &'a StrMissingViolationCallback,
}

impl<'a, 'b> StringInput<'a, 'b> {
  pub fn new(name: Option<&'a str>) -> Self {
    StringInput {
      break_on_failure: false,
      name,
      min_length: None,
      max_length: None,
      pattern: None,
      equal: None,
      required: false,
      validators: None,
      filters: None,
      too_short: &(too_long_msg),
      too_long: &(too_long_msg),
      pattern_mismatch: &(pattern_mismatch_msg),
      not_equal: &(str_not_equal_msg),
      value_missing: &str_missing_msg,
    }
  }

  fn _validate_against_self(&self, value: &'b str) -> ValidationResult {
    let mut errs = vec![];

    if let Some(min_length) = self.min_length {
      if value.len() < min_length {
        errs.push((
          ConstraintViolation::TooShort,
          (self.too_short)(self, Some(value)),
        ));

        if self.break_on_failure { return Err(errs); }
      }
    }

    if let Some(max_length) = self.max_length {
      if value.len() > max_length {
        errs.push((
          ConstraintViolation::TooLong,
          (self.too_long)(self, Some(value)),
        ));

        if self.break_on_failure { return Err(errs); }
      }
    }

    if let Some(pattern) = &self.pattern {
      if !pattern.is_match(value) {
        errs.push((
          ConstraintViolation::PatternMismatch,
          (&self.
              pattern_mismatch)(self, Some(value)),
        ));

        if self.break_on_failure { return Err(errs); }
      }
    }

    if let Some(equal) = &self.equal {
      if value != *equal {
        errs.push((
          ConstraintViolation::NotEqual,
          (&self.not_equal)(self, Some(value)),
        ));

        if self.break_on_failure { return Err(errs); }
      }
    }

    if errs.is_empty() { Ok(()) }
    else { Err(errs) }
  }
}

impl<'a, 'b> InputConstraints<'a, 'b, &'b str, Cow<'b, str>> for StringInput<'a, 'b> {
  fn validate(&self, value: Option<&'b str>) -> Result<(), Vec<ValidationErrTuple>> {
    todo!()
  }

  fn validate1(&self, value: Option<&'b str>) -> Result<(), Vec<ViolationMessage>> {
    todo!()
  }

  fn filter(&self, value: Cow<'b, str>) -> Cow<'b, str> {
    todo!()
  }

  fn validate_and_filter(&self, x: Option<&'b str>) -> Result<Option<Cow<'b, str>>, Vec<ValidationErrTuple>> {
    Ok(x.into())
  }

  fn validate_and_filter1(&self, x: Option<&'b str>) -> Result<Option<Cow<'b, str>>, Vec<ViolationMessage>> {
    todo!()
  }
}

impl Default for StringInput<'_, '_> {
  fn default() -> Self {
    Self::new(None)
  }
}

impl Display for StringInput<'_, '_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "StrInput {{ name: {}, required: {}, validators: {}, filters: {} }}",
      self.name.unwrap_or("None"),
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

impl Debug for StringInput<'_, '_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

pub fn str_missing_msg(_: &StringInput, _: Option<&str>) -> String {
  "Value is missing.".to_string()
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::types::{
    ConstraintViolation,
    ConstraintViolation::{PatternMismatch, RangeOverflow},
    InputConstraints, ValidationResult,
  };
  use crate::validator::pattern::PatternValidator;
  use regex::Regex;
  use std::{borrow::Cow, error::Error, sync::Arc, thread};

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

    let less_than_1990_input = StringInputBuilder::default()
      .validators(vec![&less_than_1990])
      .build()?;

    let yyyy_mm_dd_input = StringInputBuilder::default()
      .validators(vec![&ymd_check])
      .build()?;

    let yyyy_mm_dd_input2 = StringInputBuilder::default()
      .validators(vec![&pattern_validator])
      .build()?;

    // Missing value check
    match less_than_1990_input.validate(None) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => (),
    }

    // Mismatch check
    let value = "1000-99-999";
    match yyyy_mm_dd_input.validate(Some(&value)) {
      Ok(_) => panic!("Expected Err(...);  Received Ok(())"),
      Err(tuples) => {
        assert_eq!(tuples[0].0, PatternMismatch);
        assert_eq!(tuples[0].1, ymd_mismatch_msg(value).as_str());
      }
    }

    // Valid check
    match yyyy_mm_dd_input.validate(None) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => (),
    }

    // Valid check 2
    let value = "1000-99-99";
    match yyyy_mm_dd_input.validate(Some(&value)) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => (),
    }

    // Valid check
    let value = "1000-99-99";
    match yyyy_mm_dd_input2.validate(Some(&value)) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => (),
    }

    Ok(())
  }

  #[test]
  fn test_thread_safety() -> Result<(), Box<dyn Error>> {
    let less_than_1990_input = StringInputBuilder::default()
      .validators(vec![&less_than_1990])
      .build()?;

    let ymd_input = StringInputBuilder::default()
      .validators(vec![&ymd_check])
      .build()?;

    let less_than_input = Arc::new(less_than_1990_input);
    let less_than_input_instance = Arc::clone(&less_than_input);

    let str_input = Arc::new(ymd_input);
    let str_input_instance = Arc::clone(&str_input);

    let handle =
      thread::spawn(
        move || match less_than_input_instance.validate(Some("2023-12-31")) {
          Err(x) => {
            assert_eq!(x[0].1.as_str(), less_than_1990_msg("2023-12-31"));
          }
          _ => panic!("Expected `Err(...)`"),
        },
      );

    let handle2 = thread::spawn(move || match str_input_instance.validate(Some(&"")) {
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

  /// Example showing shared references in `StrInput`, and user-land, controls.
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

    let less_than_1990_input = StringInputBuilder::default()
      .validators(vec![&less_than_1990])
      .filters(vec![&to_last_date_of_month])
      .build()?;

    let ymd_input = StringInputBuilder::default()
      .validators(vec![&ymd_check])
      .build()?;

    let less_than_input = Arc::new(less_than_1990_input);
    let less_than_input_instance = Arc::clone(&less_than_input);
    let ymd_check_input = Arc::new(ymd_input);
    let ymd_check_input_instance = Arc::clone(&ymd_check_input);

    thread::scope(|scope| {
      scope.spawn(
        || match less_than_input_instance.validate(Some("2023-12-31")) {
          Err(x) => {
            assert_eq!(x[0].1.as_str(), &less_than_1990_msg("2023-12-31"));
          }
          _ => panic!("Expected `Err(...)`"),
        },
      );

      scope.spawn(
        || match less_than_input_instance.validate_and_filter(Some("1989-01-01")) {
          Err(err) => panic!(
            "Expected `Ok(Some({:#?})`;  Received `Err({:#?})`",
            Cow::<str>::Owned("1989-01-31".to_string()),
            err
          ),
          Ok(Some(x)) => assert_eq!(x, Cow::<str>::Owned("1989-01-31".to_string())),
          _ => panic!("Expected `Ok(Some(Cow::Owned(99 * 2)))`;  Received `Ok(None)`"),
        },
      );

      scope.spawn(|| match ymd_check_input_instance.validate(Some(&"")) {
        Err(x) => {
          assert_eq!(x[0].1.as_str(), ymd_mismatch_msg("", ymd_rx.as_str()));
        }
        _ => panic!("Expected `Err(...)`"),
      });

      scope.spawn(|| {
        if let Err(_err_tuple) = ymd_check_input_instance.validate(Some(&"2013-08-31")) {
          panic!("Expected `Ok(());  Received Err(...)`")
        }
      });
    });

    Ok(())
  }

  #[test]
  fn test_validate_and_filter() {
    let input = StringInputBuilder::default()
      .name("hello")
      .required(true)
      .validators(vec![&less_than_1990])
      .filters(vec![&to_last_date_of_month])
      .build()
      .unwrap();

    assert_eq!(
      input.validate_and_filter(Some("2023-12-31")),
      Err(vec![(RangeOverflow, less_than_1990_msg("2023-12-31"))])
    );
    assert_eq!(
      input.validate_and_filter(Some("1989-01-01")),
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
          ConstraintViolation::TypeMismatch,
          "Error".to_string(),
        )])
      }
    };

    let _input = StringInputBuilder::default()
      .name("hello")
      .validators(vec![&callback1])
      .build()
      .unwrap();
  }

  #[test]
  fn test_display() {
    let input = StringInputBuilder::default()
      .name("hello")
      .validators(vec![&less_than_1990])
      .build()
      .unwrap();

    assert_eq!(
      input.to_string(),
      "StrInput { name: hello, required: false, validators: Some([Validator; 1]), filters: None }"
    );
  }
}
