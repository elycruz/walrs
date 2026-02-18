use crate::types::ValidationResultEnum::{
  CustomError, PatternMismatch, TooLong, TooShort, Valid, ValueMissing,
};
use crate::types::{InputConstraints, ValidationResultEnum, Validator};
use regex::Regex;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

pub fn pattern_mismatch_msg(rules: &TextConstraints, xs: Option<&str>) -> String {
  format!(
    "`{}` does not match pattern `{}`.",
    &xs.as_ref().unwrap(),
    rules.pattern.as_ref().unwrap()
  )
}

pub fn too_short_msg(rules: &TextConstraints, xs: Option<&str>) -> String {
  format!(
    "Value length `{:}` is less than allowed minimum `{:}`.",
    &xs.as_ref().unwrap().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn too_long_msg(rules: &TextConstraints, xs: Option<&str>) -> String {
  format!(
    "Value length `{:}` is greater than allowed maximum `{:}`.",
    &xs.as_ref().unwrap().len(),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn value_missing_msg(_: &TextConstraints, _: Option<&str>) -> String {
  "Value is missing.".to_string()
}

pub fn custom_error_msg(_: &TextConstraints, _: Option<&str>) -> String {
  "Custom error.".to_string()
}

pub type TextConstraintsMessager<'a> =
  Arc<&'a (dyn Fn(&TextConstraints, Option<&str>) -> String + Send + Sync)>;

pub type StringFilter<'a> = Arc<&'a (dyn Fn(Cow<str>) -> Cow<str> + Send + Sync)>;

#[derive(Clone, Builder)]
pub struct TextConstraints<'a> {
  #[builder(default = "None")]
  pub min_length: Option<usize>,
  #[builder(default = "None")]
  pub max_length: Option<usize>,
  #[builder(default = "None")]
  pub pattern: Option<Regex>,
  #[builder(default = "false")]
  pub required: bool,
  #[builder(default = "None")]
  pub custom: Option<Arc<&'a (dyn Fn(&str) -> bool + Send + Sync)>>,
  #[builder(default = "None")]
  pub validators: Option<Arc<Vec<Arc<Validator<'a, dyn AsRef<str>>>>>>,
  #[builder(default = "None")]
  pub filters: Option<Arc<Vec<StringFilter<'a>>>>,

  #[builder(default = "Arc::new(&pattern_mismatch_msg)")]
  pub pattern_mismatch: TextConstraintsMessager<'a>,
  #[builder(default = "Arc::new(&too_long_msg)")]
  pub too_long: TextConstraintsMessager<'a>,
  #[builder(default = "Arc::new(&too_short_msg)")]
  pub too_short: TextConstraintsMessager<'a>,
  #[builder(default = "Arc::new(&value_missing_msg)")]
  pub value_missing: TextConstraintsMessager<'a>,
  #[builder(default = "Arc::new(&custom_error_msg)")]
  pub custom_error: TextConstraintsMessager<'a>,
}

impl TextConstraints<'_> {
  pub fn new() -> Self {
    TextConstraints {
      min_length: None,
      max_length: None,
      pattern: None,
      required: false,
      custom: None,
      validators: None,
      filters: None,

      pattern_mismatch: Arc::new(&pattern_mismatch_msg),
      too_long: Arc::new(&too_long_msg),
      too_short: Arc::new(&too_short_msg),
      value_missing: Arc::new(&value_missing_msg),
      custom_error: Arc::new(&custom_error_msg),
    }
  }

  /// Applies filters in `filters` from right-to-left.
  pub fn filter<'x>(&self, xs: &'x str) -> Cow<'x, str> {
    match self.filters.as_deref() {
      Some(filters) => filters.iter().rfold(Cow::Borrowed(xs), |_xs, f| f(_xs)),
      _ => Cow::Borrowed(xs),
    }
  }

  pub fn validate_and_filter<'x>(
    &self,
    value: &'x str,
  ) -> Result<Cow<'x, str>, (ValidationResultEnum, String)> {
    self.validate(Some(value))?;

    Ok(self.filter(value))
  }
}

fn _validate_text(rules: &TextConstraints, xs: &str) -> ValidationResultEnum {
  // Run custom test
  if let Some(custom) = &rules.custom {
    let _fn = Arc::clone(custom);
    if !((_fn)(xs)) {
      return CustomError;
    }
  }

  // Test against Min Length
  if let Some(min_length) = &rules.min_length {
    if &xs.len() < min_length {
      return TooShort;
    }
  }

  // Test against Max Length
  if let Some(max_length) = &rules.max_length {
    if &xs.len() > max_length {
      return TooLong;
    }
  }

  // Test pattern
  if let Some(pattern) = &rules.pattern {
    if !pattern.is_match(xs as &'_ str) {
      return PatternMismatch;
    }
  }

  Valid
}

fn _get_validation_message(
  constraints: &TextConstraints,
  v_rslt_enum: &ValidationResultEnum,
  x: Option<&'_ str>,
) -> Option<String> {
  let f = match v_rslt_enum {
    CustomError => Some(&constraints.custom_error),
    PatternMismatch => Some(&constraints.pattern_mismatch),
    TooLong => Some(&constraints.too_long),
    TooShort => Some(&constraints.too_short),
    ValueMissing => Some(&constraints.value_missing),
    _ => None,
  };
  f.map(|_f| {
    let _fn = Arc::clone(_f);
    (_fn)(constraints, x)
  })
}

fn _validate(
  constraints: &TextConstraints,
  x: Option<&str>,
) -> Result<(), (ValidationResultEnum, String)> {
  let v_rslt = match x {
    None => {
      if constraints.required {
        ValueMissing
      } else {
        Valid
      }
    }
    Some(v) => _validate_text(constraints, v),
  };

  if v_rslt != Valid {
    return match _get_validation_message(constraints, &v_rslt, x) {
      Some(msg) => Err((v_rslt, msg)),
      _ => Ok(()),
    };
  }

  Ok(())
}

impl InputConstraints<&str> for TextConstraints<'_> {
  fn validate(&self, x: Option<&str>) -> Result<(), (ValidationResultEnum, String)> {
    _validate(self, x)
  }
}

impl InputConstraints<Cow<'_, str>> for TextConstraints<'_> {
  fn validate(&self, x: Option<Cow<'_, str>>) -> Result<(), (ValidationResultEnum, String)> {
    _validate(self, x.as_deref())
  }
}

impl InputConstraints<&char> for TextConstraints<'_> {
  fn validate(&self, x: Option<&char>) -> Result<(), (ValidationResultEnum, String)> {
    _validate(self, x.map(|c| c.to_string()).as_deref())
  }
}

impl InputConstraints<String> for TextConstraints<'_> {
  fn validate(&self, x: Option<String>) -> Result<(), (ValidationResultEnum, String)> {
    _validate(self, x.as_deref())
  }
}

impl Display for TextConstraints<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "TextConstraints {{ \
       min_length: {}, \
       max_length: {}, \
       pattern: {}, \
       required: {}, \
       custom: {}\
       }}",
      self
        .min_length
        .map(|x| format!("Some({})", x))
        .unwrap_or("None".into()),
      self
        .max_length
        .map(|x| format!("Some({})", x))
        .unwrap_or("None".into()),
      self
        .pattern
        .as_ref()
        .map(|x| format!("Some({})", x))
        .unwrap_or("None".into()),
      self.required,
      if self.custom.is_some() {
        "Some(dyn fn(&str) -> bool + Send + Sync)"
      } else {
        "None"
      },
    )
  }
}

impl Debug for TextConstraints<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

impl Default for TextConstraints<'_> {
  fn default() -> Self {
    TextConstraints::new()
  }
}

#[cfg(test)]
mod test {
  use crate::text::{
    pattern_mismatch_msg, too_long_msg, too_short_msg, value_missing_msg, TextConstraints,
  };
  use crate::types::ValidationResultEnum::CustomError;
  use crate::types::ValidationResultEnum::{PatternMismatch, TooLong, TooShort, ValueMissing};
  use crate::types::{InputConstraints, ValidationResult};
  use regex::Regex;
  use std::sync::{Arc, Mutex};
  use std::thread;

  #[test]
  fn test_general_properties() {
    let mut input_1: TextConstraints = TextConstraints::new();
    input_1.max_length = Some(5);
    input_1.min_length = Some(1);
    input_1.required = true;

    let mut input_2: TextConstraints = TextConstraints::new();
    input_2.required = true;
    input_2.pattern = Some(Regex::new(r"^[a-z\\d]{1,5}$").unwrap());

    let mut input_3: TextConstraints = TextConstraints::new();
    input_3.pattern = input_2.pattern.clone();

    let mut input_4: TextConstraints = TextConstraints::new();
    let equals_aeiou = |xs: &str| -> bool { xs == "aeiou" };
    let not_equals_aeiou_err = |_: &TextConstraints, xs: Option<&str>| -> String {
      format!("\"{:}\" should equal \"{:}\"", xs.unwrap_or(""), "aeiou")
    };

    input_4.custom = Some(Arc::new(&equals_aeiou));
    input_4.custom_error = Arc::new(&not_equals_aeiou_err);

    let cases: Vec<(&str, &TextConstraints, Option<&str>, ValidationResult)> = vec![
      (
        "input_1.validate(Some(\"\")) == TooShort",
        &input_1,
        Some(""),
        Err((TooShort, too_short_msg(&input_1, Some("")))),
      ),
      (
        "input_1.validate(Some(\"aeiouy\")) == TooLong",
        &input_1,
        Some("aeiouy"),
        Err((TooLong, too_long_msg(&input_1, Some("aeiouy")))),
      ),
      (
        "input_1.validate(Some(\"aeiouy\")) == TooLong",
        &input_1,
        Some("aeiouy"),
        Err((TooLong, too_long_msg(&input_1, Some("aeiouy")))),
      ),
      (
        "input_1.validate(None) == ValueMissing",
        &input_1,
        None,
        Err((ValueMissing, value_missing_msg(&input_1, Some("")))),
      ),
      (
        "input_1.validate(Some(\"\")) == Valid",
        &input_1,
        Some("aeiou"),
        Ok(()),
      ),
      // Input with `required` and `pattern`
      (
        "input_2.validate(None) == ValueMissing",
        &input_2,
        None,
        Err((ValueMissing, value_missing_msg(&input_2, Some("")))),
      ),
      (
        "input_2.validate(Some(\"\")) == PatternMismatch",
        &input_2,
        Some(""),
        Err((PatternMismatch, pattern_mismatch_msg(&input_2, Some("")))),
      ),
      (
        "input_2.validate(Some(\"aeiouy\")) == PatternMismatch",
        &input_2,
        Some("aeiouy"),
        Err((
          PatternMismatch,
          pattern_mismatch_msg(&input_2, Some("aeiouy")),
        )),
      ),
      (
        "input_2.validate(Some(\"aeiou\") == Valid",
        &input_2,
        Some("aeiou"),
        Ok(()),
      ),
      // Input with `pattern`
      (
        "input_3.validate(None) == ValueMissing",
        &input_3,
        None,
        Ok(()),
      ),
      (
        "input_3.validate(Some(\"\")) == PatternMismatch",
        &input_3,
        Some(""),
        Err((PatternMismatch, pattern_mismatch_msg(&input_3, Some("")))),
      ),
      (
        "input_3.validate(Some(\"aeiouy\")) == PatternMismatch",
        &input_3,
        Some("aeiouy"),
        Err((
          PatternMismatch,
          pattern_mismatch_msg(&input_3, Some("aeiouy")),
        )),
      ),
      (
        "input_3.validate(Some(\"aeiou\") == Valid",
        &input_3,
        Some("aeiou"),
        Ok(()),
      ),
      // Input with `should_match`
      ("input_4.validate(None) == Valid", &input_4, None, Ok(())),
      (
        "input_4.validate(Some(\"\") == CustomError",
        &input_4,
        Some(""),
        Err((CustomError, not_equals_aeiou_err(&input_4, Some("")))),
      ),
      (
        "input_4.validate(Some(\"aeiou\") == Valid",
        &input_4,
        Some("aeiou"),
        Ok(()),
      ),
    ];

    for (test_name, rules, value, expected) in cases {
      println!("{}", test_name);

      let rslt = rules.validate(value);
      assert_eq!(rslt, expected);
    }
  }

  #[test]
  fn test_thread_safety() {
    let mut input_1: TextConstraints = TextConstraints::new();
    input_1.max_length = Some(5);
    input_1.min_length = Some(1);
    input_1.required = true;
    input_1.pattern = Some(Regex::new(r"^[a-z\\d]+$").unwrap());

    let input = Arc::new(Mutex::new(input_1));
    let input = Arc::clone(&input);

    // @todo test other props here

    let handle = thread::spawn(move || {
      let rules = input.lock().unwrap();
      let rslt = rules.validate(Some(""));
      assert_eq!(rslt, Err((TooShort, too_short_msg(&rules, Some("")))));
    });

    handle.join().unwrap();
  }
}
