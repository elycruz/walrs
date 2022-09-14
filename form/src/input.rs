use crate::constants::{INPUT_SYMBOL, TEXT_SYMBOL};
use crate::traits::{FormControl, FormControlConstraints, FormControlValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use derive_builder::Builder;

/// HTML Form Control data/validation struct.
#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
pub struct HTMLInputControl<'a, Value, ValueConstraints>
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
  #[builder(setter(into), default = "Some(TEXT_SYMBOL)")]
  pub type_: Option<&'static str>,

  /// Hashmap for control's html attributes that are not defined on this struct;
  /// Other attribs.: e.g., `placeholder`, `cols` etc.;
  #[builder(setter(into), default = "None")]
  pub html_attribs: Option<HashMap<&'static str, Option<&'a str>>>,

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

impl<'a, Value, ValueConstraints> HTMLInputControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  pub fn new(name: Option<&'a str>) -> Self {
    HTMLInputControl {
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
  for HTMLInputControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&ValueConstraints> {
    self.constraints.as_ref()
  }

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>) {
    self.validation_message = msg;
  }

  /// Gets control's `value`.
  fn get_value(&self) -> Option<Value> {
    self.value.clone()
  }

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not.`
  fn set_value(&mut self, value: Option<Value>) -> bool {
    self.value = value;
    self.check_validity()
  }
}

impl<'a, Value, ValueConstraints> Display for HTMLInputControl<'a, Value, ValueConstraints>
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
  use crate::constants::{ID_SYMBOL, NAME_SYMBOL, REQUIRED_SYMBOL};
  use std::collections::HashMap;
  use std::error::Error;
  use std::ops::Deref;
  use walrs_inputconstraints::text::{too_short_msg, value_missing_msg, TextInputConstraints};

  #[test]
  fn test_html_input_new() {
    let with_defaults: HTMLInputControl<&str, TextInputConstraints> = HTMLInputControl::new(None);

    for name in [Some("test_name"), None] as [Option<&str>; 2] {
      let input: HTMLInputControl<&str, TextInputConstraints> = HTMLInputControl::new(name.clone());
      assert_eq!(&input.name, &name, "name is invalid");
      // @todo test other properties here.
    }
  }

  #[test]
  fn test_html_input_set_validation_message() {
    for validation_message in
      [Some("Some validation message".to_string()), None] as [Option<String>; 2]
    {
      let mut input: HTMLInputControl<&str, TextInputConstraints> = HTMLInputControl::new(None);
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
      let mut input: HTMLInputControl<&str, TextInputConstraints> = HTMLInputControl::new(None);
      input.value = value.clone();
      assert_eq!(input.get_value(), value, "`value` is invalid");
    }
  }

  #[test]
  fn test_html_input_get_constraints() {
    let constraint_seed: TextInputConstraints = TextInputConstraints::new();

    for (value, in_constraints) in [
      (Some("some-value"), None),
      (Some("some-value"), Some(constraint_seed.clone())),
      (None, None),
      (None, Some(constraint_seed.clone())),
    ] as [(Option<&str>, Option<TextInputConstraints>); 4]
    {
      let mut input: HTMLInputControl<&str, TextInputConstraints> = HTMLInputControl::new(None);
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
    let mut constraints: TextInputConstraints = TextInputConstraints::new();
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
      as [(
        Option<&str>,
        Option<TextInputConstraints>,
        Result<(), String>,
      ); 4]
    {
      let mut input: HTMLInputControl<&str, TextInputConstraints> =
        HTMLInputControlBuilder::default()
          .constraints(constraints)
          .build()
          .unwrap();
      let initial_validation_msg = input.validation_message.clone();
      let v_rslt = input.validate(value);

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
    let mut constraints: TextInputConstraints = TextInputConstraints::new();
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
      as [(
        Option<&str>,
        Option<TextInputConstraints>,
        bool,
        Option<String>,
      ); 4]
    {
      let mut input: HTMLInputControl<&str, TextInputConstraints> =
        HTMLInputControlBuilder::default()
          .constraints(constraints)
          .build()
          .unwrap();
      input.value = value;
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
      let mut constraints: TextInputConstraints = TextInputConstraints::new();
      constraints.required = true;

      let mut input: HTMLInputControl<&str, TextInputConstraints> =
        HTMLInputControlBuilder::default()
          .constraints(Some(constraints))
          .build()
          .unwrap();
      input.set_value(value.clone());
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
