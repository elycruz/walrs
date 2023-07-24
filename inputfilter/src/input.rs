use std::borrow::Cow;
use std::fmt::{Debug};
use std::sync::Arc;

use crate::types::{Filter, InputValue, ValidationError, ValidationResult, ViolationMessage, Validator};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ConstraintViolation {
  CustomError,
  PatternMismatch,
  RangeOverflow,
  RangeUnderflow,
  StepMismatch,
  TooLong,
  TooShort,

  /// When value is in invalid format, and not validated
  /// against `pattern` (email, url, etc.) - currently unused.
  // @todo should probably be 'format mismatch'
  TypeMismatch,
  ValueMissing,
}

pub type ValueMissingViolationCallback = dyn Fn() -> ViolationMessage + Send + Sync;

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct Input<'a, T> where
  T: InputValue
{
  #[builder(default = "true")]
  pub break_on_failure: bool,

  #[builder(default = "None")]
  pub name: Option<Cow<'a, str>>,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(setter(strip_option), default = "None")]
  pub validators: Option<Vec<Arc<&'a Validator<T>>>>,

  #[builder(setter(strip_option), default = "None")]
  pub filters: Option<Vec<&'a Filter<T>>>,

  #[builder(default = "&value_missing_msg")]
  pub value_missing: &'a ValueMissingViolationCallback,

  // @todo Add support for `io_validators` (e.g., validators that return futures).
}

impl<'a, T> Input<'a, T> where T: InputValue {
  pub fn new() -> Self {
    Input {
      break_on_failure: false,
      name: None,
      required: false,
      validators: None,
      filters: None,
      value_missing: &value_missing_msg,
    }
  }

  fn _validate_against_validators(&self, value: &T) -> Option<Vec<ValidationError>> {
    self.validators.as_deref().map(|vs| {
      vs.iter().fold(Vec::<ValidationError>::new(), |mut agg, f| {
        match (Arc::clone(f))(Cow::Borrowed(value)) {
          Err(mut message_tuples) => {
            agg.append(message_tuples.as_mut());
            agg
          }
          _ => agg
        }
      })
    })
  }

  fn _option_rslt_to_rslt(&self, rslt: Option<Vec<ValidationError>>) -> ValidationResult {
    match rslt {
      None => Ok(()),
      Some(_msgs) => if !_msgs.is_empty() {
        Err(_msgs)
      } else {
        Ok(())
      }
    }
  }

  pub fn filter(&self, value: Option<T>) -> Option<T> {
    self.filters.as_deref().and_then(|fs| {
      fs.iter().fold(value, |agg, f| {
        (f)(agg)
      })
    })
  }

  pub fn validate(&self, value: Option<T>) -> ValidationResult {
    match &value {
      None => if self.required {
        Err(vec![(ConstraintViolation::ValueMissing, (self.value_missing)())])
      } else {
        Ok(())
      },
      Some(v) => self._option_rslt_to_rslt(
        self._validate_against_validators(v)
      )
    }
  }
}

impl<T: InputValue> Default for Input<'_, T> {
  fn default() -> Self {
    Self::new()
  }
}

pub fn value_missing_msg() -> String {
  "Value is missing.".to_string()
}

#[cfg(test)]
mod test {
  use std::{
    borrow::Cow,
    sync::{Arc},
    error::Error,
    thread,
  };
  use regex::Regex;

  use crate::input::{InputBuilder, ConstraintViolation};
  use crate::input::ConstraintViolation::{PatternMismatch, RangeOverflow};
  use super::{ValidationResult};
  use crate::validator::pattern::PatternValidator;

  // Tests setup types
  fn unsized_less_than_100_msg(value: usize) -> String {
    format!("{} is greater than 100", value)
  }

  fn ymd_mismatch_msg(s: &str, pattern_str: &str) -> String {
    format!("{} doesn't match pattern {}", s, pattern_str)
  }

  fn unsized_less_100(x: Cow<usize>) -> ValidationResult {
    if *x >= 100 {
      return Err(vec![(RangeOverflow, unsized_less_than_100_msg(*x))]);
    }
    Ok(())
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

    let ymd_check = move |s: Cow<&str>| -> ValidationResult {
      if !ymd_regex_arc.is_match(*s) {
        return Err(
          vec![(PatternMismatch,
                (&ymd_mismatch_msg_arc)(*s))]
        );
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

    let less_than_100_input = InputBuilder::<usize>::default()
      .validators(vec![Arc::new(&unsized_less_100)])
      .build()?;

    let yyyy_mm_dd_input = InputBuilder::<&str>::default()
      .validators(vec![Arc::new(&ymd_check)])
      .build()?;

    match less_than_100_input.validate(None) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => ()
    }

    let value = "1000-99-999";

    // Mismatch check
    match yyyy_mm_dd_input.validate(Some(value)) {
      Ok(_) => panic!("Expected Err(...);  Received Ok(())"),
      Err(tuples) => {
        assert_eq!(tuples[0].0, PatternMismatch);
        assert_eq!(tuples[0].1, ymd_mismatch_msg(value).as_str());
      }
    }

    // Valid check
    match yyyy_mm_dd_input.validate(None) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => ()
    }

    // Valid check 2
    let value = "1000-99-99";
    match yyyy_mm_dd_input.validate(Some(value)) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => ()
    }

    let yyyy_mm_dd_input2 = InputBuilder::<&str>::default()
      .validators(vec![Arc::new(&pattern_validator)])
      .build()?;

    // Valid check
    let value = "1000-99-99";
    match yyyy_mm_dd_input2.validate(Some(value)) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => ()
    }

    Ok(())
  }

  #[test]
  fn test_thread_safety() -> Result<(), Box<dyn Error>> {
    fn ymd_mismatch_msg(s: &str, pattern_str: &str) -> String {
      format!("{} doesn't match pattern {}", s, pattern_str)
    }

    fn ymd_check(s: Cow<&str>) -> ValidationResult {
      // Simplified ISO year-month-date regex
      let rx = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap();
      if !rx.is_match(*s) {
        return Err(
          vec![(PatternMismatch,
                ymd_mismatch_msg(*s, rx.as_str()))]
        );
      }
      Ok(())
    }

    let less_than_100_input = InputBuilder::<usize>::default()
      .validators(vec![Arc::new(&unsized_less_100)])
      .build()?;

    let ymd_input = InputBuilder::<&str>::default()
      .validators(vec![Arc::new(&ymd_check)])
      .build()?;

    let usize_input = Arc::new(less_than_100_input);
    let usize_input_instance = Arc::clone(&usize_input);

    let str_input = Arc::new(ymd_input);
    let str_input_instance = Arc::clone(&str_input);

    let handle = thread::spawn(move || {
      match usize_input_instance.validate(Some(101)) {
        Err(x) => {
          assert_eq!(x[0].1.as_str(), unsized_less_than_100_msg(101));
        }
        _ => panic!("Expected `Err(...)`")
      }
    });

    let handle2 = thread::spawn(move || {
      match str_input_instance.validate(Some("")) {
        Err(x) => {
          assert_eq!(x[0].1.as_str(), ymd_mismatch_msg("", Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap().as_str()));
        }
        _ => panic!("Expected `Err(...)`")
      }
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

  /// Example showing shared references in `Input`, and user-land, controls.
  #[test]
  fn test_thread_safety_with_scoped_threads_and_closures() -> Result<(), Box<dyn Error>> {
    let ymd_rx = Arc::new(Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap());
    let ymd_rx_clone = Arc::clone(&ymd_rx);

    let ymd_check = move |s: Cow<&str>| -> ValidationResult {
      // Simplified ISO year-month-date regex
      if !(&ymd_rx_clone).is_match(*s) {
        return Err(
          vec![(PatternMismatch,
                (&ymd_mismatch_msg)(*s, ymd_rx_clone.as_str()))]
        );
      }
      Ok(())
    };

    let less_than_100_input = InputBuilder::<usize>::default()
      .validators(vec![Arc::new(&unsized_less_100)])
      .build()?;

    let ymd_input = InputBuilder::<&str>::default()
      .validators(vec![Arc::new(&ymd_check)])
      .build()?;

    let usize_input = Arc::new(less_than_100_input);
    let usize_input_instance = Arc::clone(&usize_input);

    let str_input = Arc::new(ymd_input);
    let str_input_instance = Arc::clone(&str_input);

    thread::scope(|scope| {
      scope.spawn(|| {
        match usize_input_instance.validate(Some(101)) {
          Err(x) => {
            assert_eq!(x[0].1.as_str(), &unsized_less_than_100_msg(101));
          }
          _ => panic!("Expected `Err(...)`")
        }
      });

      scope.spawn(|| {
        match str_input_instance.validate(Some("")) {
          Err(x) => {
            assert_eq!(x[0].1.as_str(), ymd_mismatch_msg("", ymd_rx.as_str()));
          }
          _ => panic!("Expected `Err(...)`")
        }
      });
    });

    Ok(())
  }

  #[test]
  fn test_value_type() {
    let callback1 = |xs: Cow<&str>| -> ValidationResult {
      if *xs != "" {
        Ok(())
      } else { Err(vec![(ConstraintViolation::TypeMismatch, "Error".to_string())]) }
    };

    let _input = InputBuilder::default()
      .name(Some(Cow::from("hello")))
      .validators(vec![
        Arc::new(&callback1)
      ])
      .build()
      .unwrap();
  }
}
