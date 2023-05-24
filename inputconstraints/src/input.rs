use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use regex::Regex;
use crate::input::ValidityState::{CustomError, PatternMismatch, RangeOverflow, RangeUnderflow, TooLong, TooShort, Valid, ValueMissing};
use crate::types::InputValue;

pub type ConstraintViolationMsg = String;

#[derive(PartialEq, Debug)]
pub enum ValidityState {
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
pub type ValidationResult = Result<(), ValidityState>;
pub type ValidationResultTuple = (ValidityState, Message);
pub type Validator<'a, T> = &'a (
dyn Fn(T) -> Option<Vec<(ValidityState, Message)>> + Send + Sync
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

  #[builder(default = "None")]
  pub validators: Option<Arc<Vec<Arc<Validator<'a, T>>>>>,
}

impl<T: InputValue> Input<'_, T> {
  pub fn new() -> Self {
    Input {
      break_on_failure: false,
      name: None,
      required: false,
      validators: None,
    }
  }

  pub fn validate(&self, value: Option<T>) -> Result<(), Vec<(ValidityState, Message)>> {
    match &value {
      None => if !self.required {
        Ok(())
      } else {
        Err(vec![(ValueMissing, "Value missing".to_string())])
      },
      Some(v) => match self.validators.as_deref().map(|vs| {
        vs.iter().fold(vec![], |mut agg, f| {
          let _fn: Arc<Validator<T>> = Arc::clone(f);

          match (_fn)(v.clone()) {
            Some(mut message_tuples) => {
              agg.append(message_tuples.as_mut());
              agg
            }
            None => agg
          }
        })
      }) {
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
