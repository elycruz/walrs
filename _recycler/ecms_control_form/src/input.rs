use crate::constants::TEXT_SYMBOL;
use crate::traits::{FormControl, FormControlConstraints, FormControlValue};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use derive_builder::Builder;

/// HTML Form Control data/validation struct.
#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
pub struct InputControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  /// Name associated with control's parent (html) form.
  #[builder(setter(into), default = "None")]
  pub name: Option<&'a str>,

  /// HTML `required` attrib.
  #[builder(setter(into), default = "None")]
  pub required: Option<bool>,

  /// HTML `disabled` attribute.
  #[builder(setter(into), default = "None")]
  pub disabled: Option<bool>,

  /// HTML `type` attribute.
  #[serde(rename(serialize = "type"))]
  #[builder(setter(into), default = "Some(TEXT_SYMBOL)")]
  pub type_: Option<&'static str>,

  /// Hashmap for control's html attributes that are not defined on this struct;
  /// Other attribs.: e.g., `placeholder`, `cols` etc.;
  #[builder(setter(into), default = "None")]
  pub html_attribs: Option<HashMap<&'a str, Option<&'a str>>>,

  /// Form control's `value`.
  #[builder(setter(into), default = "None")]
  pub value: Option<Value>,

  /// Associated HTML Label text.
  #[builder(setter(into), default = "None")]
  pub label: Option<&'static str>,

  /// Error message produced by control's validation.
  #[builder(setter(into), default = "None")]
  pub validation_message: Option<String>,

  /// Constraint validation ruleset checked from `validate()` method..
  #[serde(skip)]
  #[builder(setter(into), default = "None")]
  pub constraints: Option<ValueConstraints>,

  /// Help text to display below html representation of this form control.
  #[builder(setter(into), default = "None")]
  pub help_message: Option<String>,
}

impl<'a, Value, ValueConstraints> InputControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  pub fn new(name: Option<&'a str>) -> Self {
    InputControl {
      name,
      required: None,
      disabled: None,
      type_: None,
      html_attribs: None,
      value: None,
      label: None,
      validation_message: None,
      constraints: None,
      help_message: None,
    }
  }
}

impl<'a, Value: 'a, ValueConstraints: 'a> FormControl<'a, Value, ValueConstraints>
  for InputControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&ValueConstraints> {
    self.constraints.as_ref()
  }

  /// Gets ref to validation message.
  fn get_validation_message(&self) -> Option<String> {
    self.validation_message.clone()
  }

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>) {
    self.validation_message = msg;
  }

  /// Gets control's `value`.
  fn get_value(&self) -> Option<Cow<'a, Value>> {
    self.value.as_ref().map(|x| Cow::Owned(x.to_owned()))
  }

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not.`
  fn set_value(&mut self, value: Option<Value>) -> bool {
    self.value = value;
    self.check_validity()
  }

  fn get_html_attribs(&self) -> Option<&HashMap<&'a str, Option<&'a str>>> {
    self.html_attribs.as_ref()
  }

  fn set_html_attribs(&mut self, html_attribs: Option<HashMap<&'a str, Option<&'a str>>>) {
    self.html_attribs = html_attribs;
  }
}

impl<'a, Value, ValueConstraints> Display for InputControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", &self)
  }
}

#[cfg(test)]
pub mod test {
  use super::*;
  use ecms_inputconstraints::text::{value_missing_msg, TextConstraints};
  use std::borrow::Cow;

  fn _new_input(constraints: Option<TextConstraints>) -> InputControl<&str, TextConstraints> {
    if let Some(_constraints) = constraints {
      InputControlBuilder::default()
        .constraints(_constraints)
        .build()
        .unwrap()
    } else {
      InputControlBuilder::default().build().unwrap()
    }
  }

  #[test]
  fn test_html_input_new() {
    for name in [Some("test_name"), None] as [Option<&str>; 2] {
      let input: InputControl<&str, TextConstraints> = InputControl::new(name.clone());
      assert_eq!(&input.name, &name, "name is invalid");
    }
  }

  #[test]
  fn test_html_input_set_validation_message() {
    for validation_message in
      [Some("Some validation message".to_string()), None] as [Option<String>; 2]
    {
      let mut input: InputControl<&str, TextConstraints> = InputControl::new(None);
      input.set_validation_message(validation_message.clone());
      assert_eq!(
        &input.validation_message, &validation_message,
        "validation message is invalid"
      );
    }
  }

  #[test]
  fn test_html_input_get_value() {
    for value in [Some("some-value"), None] as [Option<&str>; 2] {
      let mut input: InputControl<&str, TextConstraints> = InputControl::new(None);
      input.value = value.into();
      assert_eq!(input.get_value().map(|x| *x), value, "`value` is invalid");
    }
  }

  #[test]
  fn test_html_input_get_constraints() {
    let constraint_seed: TextConstraints = TextConstraints::new();

    for in_constraints in [None, Some(constraint_seed.clone())] as [Option<TextConstraints>; 2] {
      let mut input: InputControl<&str, TextConstraints> = InputControl::new(None);
      let in_constraints_cloned = in_constraints.clone();
      input.constraints = in_constraints;
      assert_eq!(
        !input.get_constraints().is_some(),
        !in_constraints_cloned.is_some(),
        "fetched constraints are invalid"
      );
    }
  }

  #[test]
  fn test_html_input_validate() {
    let mut constraints: TextConstraints = TextConstraints::new();
    constraints.required = true;
    for (value, constraints, rslt) in [
      (None, None, Ok(())),
      (Some("some-value"), None, Ok(())),
      (Some("some-value"), Some(constraints.clone()), Ok(())),
      (
        None,
        Some(constraints.clone()),
        Err(value_missing_msg(&constraints, None)),
      ),
    ]
      as [(Option<&str>, Option<TextConstraints>, Result<(), String>); 4]
    {
      let mut input: InputControl<&str, TextConstraints> = _new_input(constraints);
      let initial_validation_msg = input.validation_message.clone();
      let v_rslt = input.validate(value.as_deref().map(|x| Cow::Owned(x.clone())));

      assert_eq!(&v_rslt, &rslt, "result is invalid");

      // Validity state should not have changed.
      assert_eq!(
        &input.validation_message, &initial_validation_msg,
        "validity state is invalid"
      );
    }
  }

  #[test]
  fn test_html_input_check_validity() {
    let mut constraints: TextConstraints = TextConstraints::new();
    constraints.required = true;
    for (value, constraints, rslt, expected_validation_msg) in [
      (None, None, true, None),
      (Some("some-value"), None, true, None),
      (Some("some-value"), Some(constraints.clone()), true, None),
      (
        None,
        Some(constraints.clone()),
        false,
        Some(value_missing_msg(&constraints, None)),
      ),
    ]
      as [(Option<&str>, Option<TextConstraints>, bool, Option<String>); 4]
    {
      let mut input: InputControl<&str, TextConstraints> = _new_input(constraints);
      input.value = value.into();
      let v_rslt = input.check_validity();

      assert_eq!(v_rslt, rslt, "result is invalid");

      // Validity state should not have changed.
      assert_eq!(
        input.validation_message, expected_validation_msg,
        "validity state is invalid"
      );
    }
  }

  #[test]
  fn test_html_input_set_value() {
    for value in [Some("some-value"), None] as [Option<&str>; 2] {
      let mut constraints: TextConstraints = TextConstraints::new();
      constraints.required = true;

      let mut input: InputControl<&str, TextConstraints> = InputControlBuilder::default()
        .constraints(constraints)
        .build()
        .unwrap();
      input.set_value(value.into());
      assert_eq!(&input.value, &value, "`value` is invalid");

      // Since we marked `value` as `required` we can check for control's 'validity state'.
      assert_eq!(
        input.validation_message.is_some(),
        value.is_none(),
        "validity state is invalid"
      );
    }
  }
}
