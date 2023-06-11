use std::borrow::Cow;
use std::fmt::{Debug};
use std::sync::Arc;

use crate::types::InputValue;

pub type ViolationMsg = String;

#[derive(PartialEq, Debug)]
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

pub type ViolationMsgGetter<T> =
dyn Fn(&Input<T>, Option<T>) -> ViolationMsg + Send + Sync;

pub type Message = String;
pub type ValidationError = (ConstraintViolation, Message);
pub type ValidationResult = Result<(), Vec<ValidationError>>;
pub type Validator<'a, T> = &'a (
dyn Fn(T) -> Result<(), ValidationError> + Send + Sync
);
pub type Filter<'a, T> = &'a (
dyn Fn(Option<T>) -> Option<T> + Send + Sync
);

#[derive(Builder, Clone)]
pub struct Input<'a, T> where
  T: InputValue + 'a
{
  #[builder(default = "true")]
  pub break_on_failure: bool,

  #[builder(default = "None")]
  pub name: Option<Cow<'a, str>>,

  #[builder(setter(strip_option), default = "None")]
  pub validators: Option<Arc<Vec<Arc<Validator<'a, T>>>>>,

  #[builder(setter(strip_option), default = "None")]
  pub filters: Option<Arc<Vec<Arc<Filter<'a, T>>>>>

  // @todo Add support for `io_validators` (e.g., validators that return futures).
}

impl<T> Input<'_, T> where T: InputValue {
  pub fn new() -> Self {
    Input {
      break_on_failure: false,
      name: None,
      validators: None,
      filters: None,
    }
  }

  fn _validate_against_validators(&self, value: &T) -> Option<Vec<(ConstraintViolation, Message)>> {
    self.validators.as_deref().map(|vs| {
      vs.iter().fold(vec![], |mut agg, f| {
        match (f)(value.clone()) {
          Err(message_tuples) => {
            agg.push(message_tuples);
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

  fn _filter_against_filters(&self, value: Option<T>) -> Option<T> {
    self.filters.as_deref().and_then(|fs| {
      fs.iter().fold(value, |agg, f| {
        (f)(agg)
      })
    })
  }

  pub fn filter(&self, value: Option<T>) -> Option<T> {
    self._filter_against_filters(value)
  }

  pub fn validate(&self, value: Option<T>) -> ValidationResult {
    match &value {
      None => Ok(()),
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

#[cfg(test)]
mod test {
  use std::borrow::Cow;
  use std::sync::{Arc};
  use regex::Regex;
  use crate::input::{InputBuilder, ConstraintViolation};
  use std::error::Error;
  use std::thread;
  use crate::input::ConstraintViolation::{PatternMismatch, RangeOverflow};

  type PatternViolationCallback = dyn Fn(&PatternValidator, &str) -> String + Send + Sync;

  struct PatternValidator<'a> {
    pattern: Cow<'a, Regex>,
    pattern_mismatch_callback: Arc<&'a PatternViolationCallback>,
  }

  impl<'a> PatternValidator<'a> {
    pub fn validate(&self, value: &str) -> Result<(), (ConstraintViolation, String)> {
      match self.pattern.is_match(value) {
        false => Err((PatternMismatch, (&self.pattern_mismatch_callback.clone())(self, value))),
        _ => Ok(())
      }
    }
  }

  #[test]
  fn test_input_builder() -> Result<(), Box<dyn Error>> {
    let unsized_less_100 = |x: u32| -> Result<(), (ConstraintViolation, String)> {
      if x >= 100 {
        return Err((RangeOverflow, "Value greater than 100".to_string()));
      }
      Ok(())
    };

    // Simplified ISO year-month-date regex
    let ymd_regex = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$")?;
    let ymd_mismatch_msg = |s: &str| -> String {
      format!("{} doesn't match pattern {}", s, ymd_regex.as_str())
    };
    let ymd_check = |s: &str| -> Result<(), (ConstraintViolation, String)> {
      if !ymd_regex.is_match(s) {
        return Err(
          (PatternMismatch,
           ymd_mismatch_msg(s))
        );
      }
      Ok(())
    };

    let less_than_100_input = InputBuilder::<u32>::default()
      .validators(Arc::new(vec![Arc::new(&unsized_less_100)]))
      .build()?;

    let yyyy_mm_dd_input = InputBuilder::<&str>::default()
      .validators(Arc::new(vec![Arc::new(&ymd_check)]))
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

    // Validator case 1
    let pattern_validator = PatternValidator {
      pattern: Cow::Owned(ymd_regex.clone()),
      pattern_mismatch_callback: Arc::new(&|validator, s| {
        format!("{} doesn't match pattern {}", s, validator.pattern.as_str())
      }),
    };

    let ymd_validator2 = move |v| pattern_validator.validate(v);
    let yyyy_mm_dd_input2 = InputBuilder::<&str>::default()
      .validators(Arc::new(vec![Arc::new(&ymd_validator2)]))
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
    fn unsized_less_100_msg (value: u32) -> String {
      format!("{} is greater than 100", value)
    }

    fn unsized_less_100 (x: u32) -> Result<(), (ConstraintViolation, String)> {
      if x >= 100 {
        return Err((RangeOverflow, unsized_less_100_msg(x)));
      }
      Ok(())
    }

    fn ymd_mismatch_msg (s: &str, pattern_str: &str) -> String {
      format!("{} doesn't match pattern {}", s, pattern_str)
    }

    fn ymd_check(s: &str) -> Result<(), (ConstraintViolation, String)> {
      // Simplified ISO year-month-date regex
      let rx = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap();
      if !rx.is_match(s) {
        return Err(
          (PatternMismatch,
           ymd_mismatch_msg(s, rx.as_str()))
        );
      }
      Ok(())
    }

    let less_than_100_input = InputBuilder::<u32>::default()
      .validators(Arc::new(vec![Arc::new(&unsized_less_100)]))
      .build()?;

    let ymd_input = InputBuilder::<&str>::default()
      .validators(Arc::new(vec![Arc::new(&ymd_check)]))
      .build()?;

    let u32_input = Arc::new(less_than_100_input);
    let u32_input_instance = Arc::clone(&u32_input);

    let str_input = Arc::new(ymd_input);
    let str_input_instance = Arc::clone(&str_input);

    let handle = thread::spawn(move || {
      match u32_input_instance.validate(Some(101)) {
        Err(x) => {
          assert_eq!(x[0].1.as_str(), unsized_less_100_msg(101));
        },
        _ => panic!("Expected `Err(...)`")
      }
    });

    let handle2 = thread::spawn(move || {
      match str_input_instance.validate(Some("")) {
        Err(x) => {
          assert_eq!(x[0].1.as_str(), ymd_mismatch_msg("", Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap().as_str()));
        },
        _ => panic!("Expected `Err(...)`")
      }
    });

    // @note Conclusion of tests here is that validators can only (easily) be shared between threads if they're function pointers -
    //   closures are too loose and require over the top value management and planning due to the nature of multi-threaded
    //  contexts.

    handle.join().unwrap();
    handle2.join().unwrap();

    Ok(())
  }
}
