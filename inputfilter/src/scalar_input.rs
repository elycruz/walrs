use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use crate::types::{Filter, InputConstraints, Validator, ViolationMessage};
use crate::{
  value_missing_msg, ViolationEnum, ScalarValue, ViolationTuple, ValueMissingCallback,
  WithName,
};

pub fn range_underflow_msg<T: ScalarValue>(rules: &ScalarInput<T>, x: T) -> String {
  format!(
    "`{:}` is less than minimum `{:}`.",
    x,
    &rules.min.unwrap()
  )
}

pub fn range_overflow_msg<T: ScalarValue>(rules: &ScalarInput<T>, x: T) -> String {
  format!(
    "`{:}` is greater than maximum `{:}`.",
    x,
    &rules.max.unwrap()
  )
}

pub fn scalar_not_equal_msg<T: ScalarValue>(rules: &ScalarInput<T>, x: T) -> String {
  format!(
    "`{:}` is not equal to `{:}`.",
    x,
    &rules.equal.unwrap()
  )
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct ScalarInput<'a, T: ScalarValue> {
  #[builder(default = "true")]
  pub break_on_failure: bool,

  #[builder(setter(into), default = "None")]
  pub name: Option<&'a str>,

  #[builder(default = "None")]
  pub min: Option<T>,

  #[builder(default = "None")]
  pub max: Option<T>,

  #[builder(default = "None")]
  pub equal: Option<T>,

  #[builder(default = "false")]
  pub required: bool,

  #[builder(default = "None")]
  pub default_value: Option<T>,

  #[builder(default = "None")]
  pub validators: Option<Vec<&'a Validator<T>>>,

  #[builder(default = "None")]
  pub filters: Option<Vec<&'a Filter<Option<T>>>>,

  #[builder(default = "&range_underflow_msg")]
  pub range_underflow: &'a (dyn Fn(&ScalarInput<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&range_overflow_msg")]
  pub range_overflow: &'a (dyn Fn(&ScalarInput<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&scalar_not_equal_msg")]
  pub not_equal: &'a (dyn Fn(&ScalarInput<'a, T>, T) -> String + Send + Sync),

  #[builder(default = "&value_missing_msg")]
  pub value_missing: &'a ValueMissingCallback,
}

impl<'a, T> ScalarInput<'a, T>
where
  T: ScalarValue,
{
  /// Returns a new instance containing defaults.
  pub fn new(name: Option<&'a str>) -> Self {
    ScalarInput {
      break_on_failure: false,
      name,
      min: None,
      max: None,
      equal: None,
      required: false,
      default_value: None,
      validators: None,
      filters: None,
      range_underflow: &(range_underflow_msg),
      range_overflow: &(range_overflow_msg),
      not_equal: &(scalar_not_equal_msg),
      value_missing: &value_missing_msg,
    }
  }

  fn _run_own_validators_on(&self, value: T) -> Result<(), Vec<ViolationTuple>> {
    let mut errs = vec![];

    // Test lower bound
    if let Some(min) = self.min {
      if value < min {
        errs.push((
          ViolationEnum::RangeUnderflow,
          (self.range_underflow)(self, value),
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
          (self.range_overflow)(self, value),
        ));

        if self.break_on_failure {
          return Err(errs);
        }
      }
    }

    // Test equality
    if let Some(equal) = self.equal {
      if value != equal {
        errs.push((
          ViolationEnum::NotEqual,
          (self.not_equal)(self, value),
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

  fn _run_validators_on(&self, value: T) -> Result<(), Vec<ViolationTuple>> {
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

impl<'a, 'b, T: 'b> InputConstraints<'a, 'b, T, T> for ScalarInput<'a, T>
where
  T: ScalarValue,
{
  /// Validates given value against contained constraints and returns a result of unit and/or a Vec of violation tuples.
  ///
  /// ```rust
  /// use walrs_inputfilter::{
  ///   ScalarInput, InputConstraints, ViolationEnum, ScalarInputBuilder,
  ///   range_underflow_msg, range_overflow_msg,
  ///   ScalarValue
  /// };
  /// use walrs_inputfilter::equal::not_equal_msg;
  ///
  /// ```
  fn validate_detailed(&self, value: Option<T>) -> Result<(), Vec<ViolationTuple>> {
    match value {
      None => {
        if self.required {
          Err(vec![(
            ViolationEnum::ValueMissing,
            (self.value_missing)(self),
          )])
        } else {
          Ok(())
        }
      }
      // Else if value is populated validate it
      Some(v) =>
        match self._run_own_validators_on(v) {
          Ok(_) => self._run_validators_on(v),
          Err(messages1) =>
            if self.break_on_failure {
              Err(messages1)
            } else if let Err(mut messages2) = self._run_validators_on(v) {
              let mut agg = messages1;
              agg.append(messages2.as_mut());
              Err(agg)
            } else {
              Err(messages1)
            }
        }
    }
  }

  /// Validates given value against contained constraints, and returns a result of unit, and/or, a Vec of
  /// Violation messages.
  fn validate(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>> {
    match self.validate_detailed(value) {
      // If errors, extract messages and return them
      Err(messages) => Err(messages.into_iter().map(|(_, message)| message).collect()),
      Ok(_) => Ok(()),
    }
  }

  /// Filters value against contained filters.
  fn filter(&self, value: Option<T>) -> Option<T> {
    let v = match value {
      None => self.default_value,
      Some(x) => Some(x),
    };

    match self.filters.as_deref() {
      None => v,
      Some(fs) => fs.iter().fold(v, |agg, f| f(agg)),
    }
  }

  /// Validates, and filters, given value against contained rules, validators, and filters, respectively.
  fn validate_and_filter(&self, x: Option<T>) -> Result<Option<T>, Vec<ViolationMessage>> {
    match self.validate_and_filter_detailed(x) {
      Err(messages) => Err(messages.into_iter().map(|(_, message)| message).collect()),
      Ok(filtered) => Ok(filtered),
    }
  }

  /// Validates, and filters, given value against contained rules, validators, and filters, respectively and
  /// returns a result of filtered value or a Vec of Violation tuples.
  fn validate_and_filter_detailed(&self, x: Option<T>) -> Result<Option<T>, Vec<ViolationTuple>> {
    self.validate_detailed(x).map(|_| self.filter(x))
  }
}

impl<'a, T: ScalarValue> WithName<'a> for ScalarInput<'a, T> {
  fn get_name(&self) -> Option<Cow<'a, str>> {
    self.name.map(Cow::Borrowed)
  }
}

impl<T: ScalarValue> Default for ScalarInput<'_, T> {
  fn default() -> Self {
    Self::new(None)
  }
}

impl<T: ScalarValue> Display for ScalarInput<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ScalarInput {{ name: {}, required: {}, validators: {}, filters: {} }}",
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

impl<T: ScalarValue> Debug for ScalarInput<'_, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{ViolationEnum, InputConstraints};

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

    let usize_required = (|| -> ScalarInput<usize> {
        let mut new_input = usize_not_required.clone();
        new_input.required = true;
        new_input
    })();

    let _usize_no_break_on_failure = (|| -> ScalarInput<usize> {
      let mut new_input = usize_required.clone();
      // new_input.validators.push(&|x: usize| if x % 2 != 0 {
      //   Err(vec![(ConstraintViolation::CustomError, "Must be even".to_string())])
      // } else {
      //   Ok(())
      // });
      new_input.break_on_failure = true;
      new_input
    })();

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
        (ViolationEnum::RangeOverflow,
         range_overflow_msg(&usize_not_required, 11)),
      ])),
      ("1-10, Even, with invalid value (3)", &usize_not_required, Some(7), Err(vec![
        (ViolationEnum::CustomError,
         "Must be even".to_string()),
      ])),
      ("1-10, Even, with value value", &usize_not_required, Some(8), Ok(())),

      // Required
      // ----
      ("1-10, Even, required, no value", &usize_required, None, Err(vec![
        (ViolationEnum::ValueMissing,
         value_missing_msg(&usize_required)),
      ])),
      ("1-10, Even, required, with valid value", &usize_required, Some(2), Ok(())),
      ("1-10, Even, required, with valid value (2)", &usize_required, Some(10), Ok(())),
      ("1-10, Even, required, with invalid value", &usize_required, Some(0), Err(vec![
        (ViolationEnum::RangeUnderflow,
         range_underflow_msg(&usize_required, 0)),
      ])),
      ("1-10, Even, required, with invalid value(2)", &usize_required, Some(11), Err(vec![
        (ViolationEnum::RangeOverflow,
         range_overflow_msg(&usize_required, 11)),
      ])),
      ("1-10, Even, required, with invalid value (3)", &usize_required, Some(7), Err(vec![
        (ViolationEnum::CustomError,
         "Must be even".to_string()),
      ])),
      ("1-10, Even, required, with value value", &usize_required, Some(8), Ok(())),
      // ("1-10, Even, 'break-on-failure: true' false", &usize_no_break_on_failure, Some(7), Err(vec![
      //   (ConstraintViolation::CustomError,
      //    "Must be even".to_string()),
      // ])),
    ];

    for (i, (test_name, input, subj, expected)) in test_cases.into_iter().enumerate() {
      println!("Case {}: {}", i + 1, test_name);

      assert_eq!(input.validate_detailed(subj), expected);
    }

    // Test basic usage with other types
    // ----
    // Validates `f64`, and `f32` types
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
       value_missing_msg(&f64_input_required)),
    ]));
    assert_eq!(f64_input_required.validate_detailed(Some(2.0)), Ok(()));
    assert_eq!(f64_input_required.validate_detailed(Some(11.0)), Err(vec![
      (ViolationEnum::RangeOverflow,
       range_overflow_msg(&f64_input_required, 11.0)),
    ]));

    // Test `char` type usage
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

    // Test `equal` field usage
    let char_input_equal = ScalarInputBuilder::<char>::default()
      .equal('a')
      .build()
      .unwrap();

    assert_eq!(char_input_equal.validate_detailed(None), Ok(()));
    assert_eq!(char_input_equal.validate_detailed(Some('b')), Err(vec![
      (ViolationEnum::NotEqual,
       "`b` is not equal to `a`.".to_string()),
    ]));
  }
}
