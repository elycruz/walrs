use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use crate::types::{ConstraintViolation, Filter, InputConstraints, InputValue, ValidationError, ValidationResult, Validator, ViolationMessage};

pub type ValueMissingViolationCallback<'a, T> =
  dyn Fn(&Input<'a, T>) -> ViolationMessage + Send + Sync;

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct Input<'a, T>
where
  T: InputValue,
{
  #[builder(default = "true")]
  pub break_on_failure: bool,

  #[builder(setter(into), default = "None")]
  pub name: Option<&'a str>,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(setter(strip_option), default = "None")]
  pub validators: Option<Vec<Arc<&'a Validator<T>>>>,

  #[builder(setter(strip_option), default = "None")]
  pub filters: Option<Vec<&'a Filter<T>>>,

  #[builder(default = "&value_missing_msg")]
  pub value_missing: &'a (dyn Fn(&Input<'a, T>) -> ViolationMessage + Send + Sync),

  // @todo Add support for `io_validators` (e.g., validators that return futures).
}

impl<'a, T> Input<'a, T>
where
  T: InputValue,
{
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
}

impl<'a, T: InputValue> InputConstraints<'a, T> for Input<'a, T> {
  fn get_should_break_on_failure(&self) -> bool {
    self.break_on_failure
  }

  fn get_required(&self) -> bool {
    self.required
  }

  fn get_value_missing_handler(&self) -> &'a (dyn Fn(&Self) -> ViolationMessage + Send + Sync) {
    self.value_missing
  }

  fn get_validators(&self) -> Option<&[Arc<&Validator<T>>]> {
    self.validators.as_deref()
  }

  fn get_filters(&self) -> Option<&[&Filter<T>]> {
    self.filters.as_deref()
  }
}

impl<T: InputValue> Default for Input<'_, T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: InputValue> Display for Input<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Input {{ name: {}, required: {}, validators: {}, filters: {} }}",
      self.name.unwrap_or("None"),
      self.required,
      self.validators.as_deref().map(|vs|
        format!("Some([Validator; {}])", vs.len())
      ).unwrap_or("None".to_string()),
      self.filters.as_deref().map(|fs|
        format!("Some([Filter; {}])", fs.len())
      ).unwrap_or("None".to_string()),
    )
  }
}

impl<T: InputValue> Debug for Input<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

pub fn value_missing_msg<T: InputValue>(_: &Input<T>) -> String {
  "Value is missing.".to_string()
}

#[cfg(test)]
mod test {
  use regex::Regex;
  use std::{borrow::Cow, error::Error, sync::Arc, thread};

  use super::ValidationResult;
  use crate::types::{ConstraintViolation, ConstraintViolation::{PatternMismatch, RangeOverflow},
                     InputConstraints};
  use crate::input::{InputBuilder};
  use crate::validator::number::{NumberValidatorBuilder, step_mismatch_msg};
  use crate::validator::pattern::PatternValidator;

  // Tests setup types
  fn unsized_less_than_100_msg(value: usize) -> String {
    format!("{} is greater than 100", value)
  }

  fn ymd_mismatch_msg(s: &str, pattern_str: &str) -> String {
    format!("{} doesn't match pattern {}", s, pattern_str)
  }

  fn unsized_less_100(x: &usize) -> ValidationResult {
    if *x >= 100 {
      return Err(vec![(
        RangeOverflow,
        unsized_less_than_100_msg(*x)
      )]);
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

    let ymd_check = move |s: &&str| -> ValidationResult {
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

    let less_than_100_input = InputBuilder::<usize>::default()
      .validators(vec![Arc::new(&unsized_less_100)])
      .build()?;

    let yyyy_mm_dd_input = InputBuilder::<&str>::default()
      .validators(vec![Arc::new(&ymd_check)])
      .build()?;

    let even_0_to_100 = NumberValidatorBuilder::<usize>::default()
      .min(0)
      .max(100)
      .step(2)
      .build()?;

    let even_from_0_to_100_input = InputBuilder::<usize>::default()
      .name("even-0-to-100")
      .validators(vec![Arc::new(&even_0_to_100)])
      .build()?;

    let yyyy_mm_dd_input2 = InputBuilder::<&str>::default()
      .validators(vec![Arc::new(&pattern_validator)])
      .build()?;

    // Missing value check
    match less_than_100_input.validate(None) {
      Err(errs) => panic!("Expected Ok(());  Received Err({:#?})", &errs),
      Ok(()) => (),
    }

    // `Rem` (Remainder) trait check
    match even_from_0_to_100_input.validate(Some(&3)) {
      Err(errs) => errs.iter().for_each(|v_err| {
        assert_eq!(v_err.0, ConstraintViolation::StepMismatch);
        assert_eq!(v_err.1, step_mismatch_msg(&even_0_to_100, 3));
      }),
      _ => panic!("Expected Err(...);  Received Ok(())")
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
    fn ymd_mismatch_msg(s: &str, pattern_str: &str) -> String {
      format!("{} doesn't match pattern {}", s, pattern_str)
    }

    fn ymd_check(s: &&str) -> ValidationResult {
      // Simplified ISO year-month-date regex
      let rx = Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap();
      if !rx.is_match(s) {
        return Err(vec![(PatternMismatch, ymd_mismatch_msg(s, rx.as_str()))]);
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

    let handle =
      thread::spawn(
        move || match usize_input_instance.validate(Some(&101)) {
          Err(x) => {
            assert_eq!(x[0].1.as_str(), unsized_less_than_100_msg(101));
          }
          _ => panic!("Expected `Err(...)`"),
        },
      );

    let handle2 =
      thread::spawn(
        move || match str_input_instance.validate(Some(&"")) {
          Err(x) => {
            assert_eq!(
              x[0].1.as_str(),
              ymd_mismatch_msg(
                "",
                Regex::new(r"^\d{1,4}-\d{1,2}-\d{1,2}$").unwrap().as_str()
              )
            );
          }
          _ => panic!("Expected `Err(...)`"),
        },
      );

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

    let ymd_check = move |s: &&str| -> ValidationResult {
      // Simplified ISO year-month-date regex
      if !ymd_rx_clone.is_match(s) {
        return Err(vec![(
          PatternMismatch,
          ymd_mismatch_msg(s, ymd_rx_clone.as_str()),
        )]);
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
      scope.spawn(
        || match usize_input_instance.validate(Some(&101)) {
          Err(x) => {
            assert_eq!(x[0].1.as_str(), &unsized_less_than_100_msg(101));
          }
          _ => panic!("Expected `Err(...)`"),
        },
      );

      scope.spawn(
        || match str_input_instance.validate(Some(&"")) {
          Err(x) => {
            assert_eq!(x[0].1.as_str(), ymd_mismatch_msg("", ymd_rx.as_str()));
          }
          _ => panic!("Expected `Err(...)`"),
        },
      );

      scope.spawn(
        || if let Err(_err_tuple) = str_input_instance.validate(Some(&"2013-08-31")) {
          panic!("Expected `Ok(());  Received Err(...)`")
        },
      );
    });

    Ok(())
  }

  #[test]
  fn test_validate_and_filter() {
    let input = InputBuilder::<usize>::default()
      .name("hello")
      .required(true)
      .validators(vec![Arc::new(&unsized_less_100)])
      .build()
      .unwrap();

    assert_eq!(input.validate_and_filter(Some(&101)), Err(vec![(RangeOverflow, unsized_less_than_100_msg(101))]));
    assert_eq!(input.validate_and_filter(Some(&99)), Ok(Some(Cow::Borrowed(&99))));
  }

  #[test]
  fn test_value_type() {
    let callback1 = |xs: &&str| -> ValidationResult {
      if !xs.is_empty() {
        Ok(())
      } else {
        Err(vec![(
          ConstraintViolation::TypeMismatch,
          "Error".to_string(),
        )])
      }
    };

    let _input = InputBuilder::default()
      .name("hello")
      .validators(vec![Arc::new(&callback1)])
      .build()
      .unwrap();
  }

  #[test]
  fn test_display() {
    let input = InputBuilder::<usize>::default()
      .name("hello")
      .validators(vec![Arc::new(&unsized_less_100)])
      .build()
      .unwrap();

    assert_eq!(
      input.to_string(),
      "Input { name: hello, required: false, validators: Some([Validator; 1]), filters: None }"
    );
  }
}
