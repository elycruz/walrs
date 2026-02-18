use crate::types::ValidationResultEnum::{
  CustomError, RangeOverflow, RangeUnderflow, Valid, ValueMissing,
};
use crate::types::{InputConstraints, ValidationResultEnum};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[derive(Builder, Clone)]
pub struct NumberConstraints<'a, T: Display + Copy + PartialOrd + PartialEq> {
  #[builder(default = "None")]
  pub min: Option<T>,
  #[builder(default = "None")]
  pub max: Option<T>,
  #[builder(default = "None")]
  pub step: Option<T>,
  #[builder(default = "false")]
  pub required: bool,
  #[builder(default = "None")]
  pub custom: Option<Arc<&'a (dyn Fn(&NumberConstraints<T>, Option<T>) -> bool + Send + Sync)>>,

  #[builder(default = "Arc::new(&range_underflow_msg)")]
  pub range_underflow: Arc<&'a (dyn Fn(&NumberConstraints<T>, Option<T>) -> String + Send + Sync)>,
  #[builder(default = "Arc::new(&range_overflow_msg)")]
  pub range_overflow: Arc<&'a (dyn Fn(&NumberConstraints<T>, Option<T>) -> String + Send + Sync)>,
  #[builder(default = "Arc::new(&value_missing_msg)")]
  pub value_missing: Arc<&'a (dyn Fn(&NumberConstraints<T>, Option<T>) -> String + Send + Sync)>,
  #[builder(default = "Arc::new(&custom_error_msg)")]
  pub custom_error: Arc<&'a (dyn Fn(&NumberConstraints<T>, Option<T>) -> String + Send + Sync)>,
}

pub fn range_underflow_msg<T>(rules: &NumberConstraints<T>, x: Option<T>) -> String
where
  T: Display + Copy + PartialOrd + PartialEq,
{
  format!(
    "`{:}` is less than minimum `{:}`.",
    &x.as_ref().unwrap(),
    &rules.min.as_ref().unwrap()
  )
}

pub fn range_overflow_msg<T>(rules: &NumberConstraints<T>, x: Option<T>) -> String
where
  T: Display + Copy + PartialOrd + PartialEq,
{
  format!(
    "`{:}` is greater than maximum `{:}`.",
    &x.as_ref().unwrap(),
    &rules.max.as_ref().unwrap()
  )
}

pub fn value_missing_msg<T>(_: &NumberConstraints<T>, _: Option<T>) -> String
where
  T: Display + Copy + PartialOrd + PartialEq,
{
  "Value is missing.".to_string()
}

pub fn custom_error_msg<T: Display + Copy + PartialOrd + PartialEq>(
  _: &NumberConstraints<T>,
  _: Option<T>,
) -> String {
  "Custom error.".to_string()
}

impl<T: Display + Copy + PartialOrd + PartialEq> NumberConstraints<'_, T> {
  pub fn new() -> Self {
    NumberConstraints {
      min: None,
      max: None,
      step: None,
      required: false,
      custom: None,

      // Message getters
      range_underflow: Arc::new(&range_underflow_msg),
      range_overflow: Arc::new(&range_overflow_msg),
      value_missing: Arc::new(&value_missing_msg),
      custom_error: Arc::new(&custom_error_msg),
    }
  }
}

impl<T: Display + Copy + PartialOrd + PartialEq> Default for NumberConstraints<'_, T> {
  fn default() -> Self {
    Self::new()
  }
}

fn _get_validation_message<T: Display + Copy + PartialOrd + PartialEq>(
  constraints: &NumberConstraints<T>,
  v_rslt_enum: &ValidationResultEnum,
  x: Option<T>,
) -> Option<String> {
  let f = match v_rslt_enum {
    CustomError => Some(&constraints.custom_error),
    RangeUnderflow => Some(&constraints.range_underflow),
    RangeOverflow => Some(&constraints.range_overflow),
    ValueMissing => Some(&constraints.value_missing),
    _ => None,
  };
  f.map(|_f| {
    let _fn = Arc::clone(_f);
    (_fn)(constraints, x)
  })
}

fn _validate_number<T: Display + Copy + PartialOrd + PartialEq>(
  rules: &NumberConstraints<T>,
  x: T,
) -> ValidationResultEnum
where
  T: Display + Copy + PartialOrd + PartialEq,
{
  // Test against Min
  if let Some(min) = &rules.min {
    if x < *min {
      return RangeUnderflow;
    }
  }

  // Test against Max
  if let Some(max) = &rules.max {
    if x > *max {
      return RangeOverflow;
    }
  }

  Valid
}

fn _validate<T: Display + Copy + PartialOrd + PartialEq>(
  ctx: &NumberConstraints<T>,
  option_t: Option<T>,
) -> Result<(), (ValidationResultEnum, String)> {
  let v_rslt = option_t.map_or(if ctx.required { ValueMissing } else { Valid }, |v| {
    _validate_number(ctx, v)
  });

  if v_rslt != Valid {
    return match _get_validation_message(ctx, &v_rslt, option_t) {
      Some(msg) => Err((v_rslt, msg)),
      _ => Ok(()),
    };
  }

  Ok(())
}

impl<T> Display for NumberConstraints<'_, T>
where
  T: Display + Copy + PartialOrd + PartialEq,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "NumberConstraints {{ \
       min: {}, \
       max: {}, \
       step: {}, \
       required: {}, \
       custom: {}\
       }}",
      self
        .min
        .map(|x| format!("Some({})", x))
        .unwrap_or("None".into()),
      self
        .max
        .map(|x| format!("Some({})", x))
        .unwrap_or("None".into()),
      self
        .step
        .as_ref()
        .map(|x| format!("Some({})", x))
        .unwrap_or("None".into()),
      self.required,
      if self.custom.is_some() {
        "Some(dyn fn(T) -> bool + Send + Sync)" // @todo Should print symbol path instead.
      } else {
        "None"
      },
    )
  }
}

impl<T> Debug for NumberConstraints<'_, T>
where
  T: Copy + Display + PartialEq + PartialOrd,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

impl<T> InputConstraints<T> for NumberConstraints<'_, T>
where
  T: Debug + Display + Copy + PartialOrd + PartialEq,
{
  fn validate(&self, option_t: Option<T>) -> Result<(), (ValidationResultEnum, String)> {
    _validate(self, option_t)
  }
}

impl<T> InputConstraints<Cow<'_, T>> for NumberConstraints<'_, T>
where
  T: Debug + Display + Copy + PartialOrd + PartialEq,
{
  fn validate(&self, option_t: Option<Cow<'_, T>>) -> Result<(), (ValidationResultEnum, String)> {
    _validate(self, option_t.as_deref().copied())
  }
}

#[cfg(test)]
mod test {

  use crate::number::{
    range_overflow_msg, range_underflow_msg, value_missing_msg, NumberConstraints,
  };
  use crate::types::{
    InputConstraints, ValidationResult,
    ValidationResultEnum::{RangeOverflow, RangeUnderflow, ValueMissing},
  };
  use std::sync::{Arc, Mutex};
  use std::thread;

  #[test]
  fn test_validate_single_threaded() {
    let mut input_1: NumberConstraints<i32> = NumberConstraints::new();
    input_1.min = Some(0);
    input_1.max = Some(5);

    let mut input_2 = NumberConstraints::new();
    input_2.min = Some(0);
    input_2.max = Some(5);
    input_2.required = true;

    let mut input_3: NumberConstraints<i32> = NumberConstraints::new();
    input_3.required = true;
    input_3.custom = Some(Arc::new(&|_: &NumberConstraints<i32>,
                                     x: Option<i32>|
     -> bool { x.unwrap_or(0) == 99 }));
    input_3.custom_error = Arc::new(&|_: &NumberConstraints<i32>, x: Option<i32>| -> String {
      format!("\"{:}\" should equal \"{:}\"", x.unwrap_or(0), "99")
    });

    let cases = vec![
      (
        "input_1.validate(Some(-1)) == RangeUnderflow",
        &input_1,
        Some(-1),
        Err((RangeUnderflow, range_underflow_msg(&input_1, Some(-1)))),
      ),
      (
        "input_1.validate(Some(6)) == RangeOverflow",
        &input_1,
        Some(6),
        Err((RangeOverflow, range_overflow_msg(&input_1, Some(6)))),
      ),
      (
        "input_1.validate(Some(5)) == Valid",
        &input_1,
        Some(5),
        Ok(()),
      ),
      // ("input_3.validate(Some(-1)) == NotEqual(...)", &input_3, Some(-1),
      //  Err((NotEqual, value_mismatch_msg(&input_3, Some(-1))))),
      (
        "input_3.validate(Some(99)) == Valid",
        &input_3,
        Some(99),
        Ok(()),
      ),
      (
        "input_2.validate(None) == ValueMissing",
        &input_2,
        None,
        Err((ValueMissing, value_missing_msg(&input_2, Some(0)))),
      ),
    ];

    for (test_name, rules, value, expected) in cases {
      println!("Test: {}", test_name);
      let rslt = rules.validate(value);
      assert_eq!(rslt, expected);
    }
  }

  #[test]
  fn test_validate_multi_threaded() {
    let mut input_1 = NumberConstraints::new();
    input_1.min = Some(0);
    input_1.max = Some(5);

    let mut input_2 = NumberConstraints::new();
    input_2.min = Some(0);
    input_2.max = Some(5);
    input_2.required = true;

    let input_1 = Arc::new(Mutex::new(input_1));
    let input_2 = Arc::new(Mutex::new(input_2));

    type TestCaseGetter<'a, 'b, T = i32> =
      &'a (dyn Fn(&NumberConstraints<T>) -> (&'b str, Option<i32>, ValidationResult) + Send + Sync);

    let threads: Vec<_> = vec![
      (
        &input_1,
        &(|input: &NumberConstraints<i32>| -> (&str, Option<i32>, ValidationResult) {
          (
            "input_1.validate(Some(-1)) == RangeUnderflow",
            Some(-1),
            Err((RangeUnderflow, range_underflow_msg(input, Some(-1)))),
          )
        }) as TestCaseGetter,
      ),
      (
        &input_1,
        &(|input: &NumberConstraints<i32>| -> (&str, Option<i32>, ValidationResult) {
          (
            "input_1.validate(Some(6)) == RangeOverflow",
            Some(6),
            Err((RangeOverflow, range_overflow_msg(input, Some(6)))),
          )
        }) as TestCaseGetter,
      ),
      (
        &input_1,
        &|_: &NumberConstraints<i32>| -> (&str, Option<i32>, ValidationResult) {
          ("input_1.validate(None) == Valid", Some(5), Ok(()))
        },
      ),
      (
        &input_2,
        &(|input: &NumberConstraints<i32>| -> (&str, Option<i32>, ValidationResult) {
          (
            "input_2.validate(None) == ValueMissing",
            None,
            Err((ValueMissing, value_missing_msg(input, Some(0)))),
          )
        }),
      ),
      (
        &input_2,
        &(|input: &NumberConstraints<i32>| -> (&str, Option<i32>, ValidationResult) {
          (
            "input_2.validate(None) == RangeUnderflow",
            Some(-2),
            Err((RangeUnderflow, range_underflow_msg(input, Some(-2)))),
          )
        }),
      ),
      (
        &input_2,
        &(|input: &NumberConstraints<i32>| -> (&str, Option<i32>, ValidationResult) {
          (
            "input_2.validate(None) == RangeOverflow",
            Some(6),
            Err((RangeOverflow, range_overflow_msg(input, Some(6)))),
          )
        }),
      ),
    ]
    .into_iter()
    .map(|(rules, test_case_extractor)| {
      let rules = Arc::clone(rules);
      thread::spawn(move || {
        let rules = &rules.lock().unwrap();
        let (test_name, value, expected) = test_case_extractor(rules);
        println!("Test `{}` start.", test_name);
        let rslt = rules.validate(value);
        assert_eq!(rslt, expected);
        println!("Test `{}` end.", test_name);
      })
    })
    .collect();

    for handle in threads {
      handle.join().unwrap()
    }
  }

  #[test]
  fn test_validate_multi_threaded_actix() {}
}
