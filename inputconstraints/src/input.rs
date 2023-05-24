use std::borrow::Cow;
use std::fmt::{Debug};
use std::sync::Arc;

use crate::input::ConstraintViolation::{ValueMissing};
use crate::types::InputValue;

pub type ConstraintViolationMsg = String;

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
  TypeMismatch,
  // @todo should probably be 'format mismatch'
  ValueMissing,
  Valid,
}

pub type ConstraintViolationFn<T> =
dyn Fn(&Input<T>, Option<T>) ->
ConstraintViolationMsg + Send + Sync;

pub type ConstraintCheck<T> =
dyn Fn(T) -> bool + Send + Sync;

pub type Message = String;
pub type ValidationResult = Result<(), ConstraintViolation>;
pub type ValidationResultTuple = (ConstraintViolation, Message);
pub type Validator<'a, T> = &'a (
dyn Fn(T) -> Option<(ConstraintViolation, Message)> + Send + Sync
);
pub type Filter<'a, T> = &'a (dyn Fn(Cow<T>) -> Cow<T> + Send + Sync);

#[derive(Builder, Clone)]
pub struct Input<'a, T> where
  T: InputValue + 'a
{
  #[builder(default = "true")]
  pub break_on_failure: bool,

  #[builder(default = "None")]
  pub name: Option<Cow<'a, str>>,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(setter(strip_option), default = "None")]
  pub validators: Option<Vec<Validator<'a, T>>>,

  #[builder(setter(strip_option), default = "None")]
  pub arc_validators: Option<Arc<Vec<Arc<Validator<'a, T>>>>>,
}

impl<T: InputValue> Input<'_, T> {
  pub fn new() -> Self {
    Input {
      break_on_failure: false,
      name: None,
      required: false,
      validators: None,
      arc_validators: None,
    }
  }

  fn _validate_against_arc_validators(&self, value: T) -> Option<Vec<(ConstraintViolation, Message)>> {
    self.arc_validators.as_deref().map(|vs| {
      vs.iter().fold(vec![], |mut agg, f| {
        let _fn: Arc<Validator<T>> = Arc::clone(f);

        match (_fn)(value.clone()) {
          Some(message_tuples) => {
            agg.push(message_tuples);
            agg
          }
          None => agg
        }
      })
    })
  }

  fn _validate_against_validators(&self, value: T) -> Option<Vec<(ConstraintViolation, Message)>> {
    self.validators.as_deref().map(|vs| {
      vs.iter().fold(vec![], |mut agg, f| {

        match (f)(value.clone()) {
          Some(message_tuples) => {
            agg.push(message_tuples);
            agg
          }
          None => agg
        }
      })
    })
  }

  pub fn validate(&self, value: Option<T>) -> Result<(), Vec<(ConstraintViolation, Message)>> {
    match &value {
      None => if !self.required {
        Ok(())
      } else {
        Err(vec![(ValueMissing, "Value missing".to_string())])
      },
      Some(v) => match self._validate_against_arc_validators(v.clone()) {
        None => Ok(()),
        Some(_msgs) => if _msgs.len() > 0 {
          Err(_msgs)
        } else {
          Ok(())
        }
      }
    }
  }
}
  
#[cfg(test)]
mod test {
  use std::sync::Arc;
  use regex::Regex;
  use crate::input::{InputBuilder, ConstraintViolation};
  use std::error::Error;
  use crate::input::ConstraintViolation::PatternMismatch;

  #[test]
  fn test_input_builder() -> Result<(), Box<dyn Error>> {
    let unsized_less_100 = |x: u32| -> Option<(ConstraintViolation, String)> {
      if x >= 100 {
        return Some((ConstraintViolation::RangeOverflow, "Value greater than 100".to_string()));
      }
      None
    };

    let ymd_regex = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$")?;
    let ymd_mismatch_msg = |s: &str| -> String {
      format!("{} doesn't match pattern {}", s, ymd_regex.as_str())
    };
    let ymd_check = |s: &str| -> Option<(ConstraintViolation, String)> {
      if !ymd_regex.is_match(s) {
        return Some(
          (ConstraintViolation::PatternMismatch,
           ymd_mismatch_msg(s))
        );
      }
      None
    };

    let less_than_100_input = InputBuilder::<u32>::default()
      .arc_validators(Arc::new(vec![Arc::new(&unsized_less_100)]))
      .build()?;

    let yyyy_mm_dd_input = InputBuilder::<&str>::default()
      .arc_validators(Arc::new(vec![Arc::new(&ymd_check)]))
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

    Ok(())
  }
}
