use std::borrow::Cow;
use crate::constants::{TEXT_SYMBOL};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::fmt::{Debug, Display, Formatter};

use derive_builder::Builder;
use crate::traits::FormControl;
use crate::walrs_inputfilter::{types::{InputConstraints, InputValue}};

/// HTML Input Control Draft - meant as a data struct only - not as an actual DOM Node.
/// ----
#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
pub struct HTMLInput<'a, 'b, Value, Constraints>
where
  Value: InputValue,
  Constraints: 'a + InputConstraints<'a, 'b, Value>,
{
  /// Name associated with control's parent (html) form.
  #[builder(setter(into), default = "None")]
  pub name: Option<Cow<'a, str>>,

  /// HTML `required` attrib.
  #[builder(setter(into), default = "false")]
  pub required: bool,

  /// HTML `disabled` attribute.
  #[builder(setter(into), default = "false")]
  pub disabled: bool,

  /// HTML `type` attribute.
  #[serde(rename(serialize = "type"))]
  #[builder(setter(into), default = "Some(TEXT_SYMBOL)")]
  pub type_: Option<&'a str>,

  /// Hashmap for control's html attributes that are not defined on this struct;
  /// Other attribs.: e.g., `placeholder`, `cols` etc.;
  /// **Note:** This field gets flattened into parent struct.
  #[serde(flatten)]
  #[builder(setter(into), default = "None")]
  pub attributes: Option<Map<String, serde_json::Value>>,

  /// Form control's `value`.
  #[builder(setter(into), default = "None")]
  pub value: Option<Value>,

  /// Associated HTML Label text.
  #[builder(setter(into), default = "None")]
  pub label: Option<Cow<'a, str>>,

  /// Error message produced by control's validation.
  #[builder(setter(into), default = "None")]
  pub validation_message: Option<String>,

  /// Constraint validation ruleset checked from `validate()` method..
  /// **Note:** In the future this could be serializable (would require an update to
  ///  `walrs_inputfilter::input` module).`
  ///
  #[serde(skip)]
  #[builder(setter(into), default = "None")]
  pub constraints: Option<Constraints>,

  /// Help text to display below html representation of this form control.
  #[builder(setter(into), default = "None")]
  pub help_message: Option<String>,

  _phantom: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b, Value, Constraints> HTMLInput<'a, 'b, Value, Constraints>
where
  Value: InputValue,
  Constraints: 'a + InputConstraints<'a, 'b, Value>,
{
  pub fn new(name: Option<Cow<'a, str>>) -> Self {
    HTMLInput {
      name,
      required: false,
      disabled: false,
      type_: None,
      attributes: None,
      value: None,
      label: None,
      validation_message: None,
      constraints: None,
      help_message: None,
      _phantom: Default::default(),
    }
  }
}

impl<'a, 'b, Value: 'a + 'b, ValueConstraints: 'a> FormControl<'a, 'b, Value, ValueConstraints>
for HTMLInput<'a, 'b, Value, ValueConstraints>
  where
    Value: InputValue,
    ValueConstraints: InputConstraints<'a, 'b, Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&ValueConstraints> {
    self.constraints.as_ref()
  }

  /// Gets ref to validation message.
  fn get_validation_message(&self) -> Option<Cow<'a, str>> {
    self.validation_message.as_deref().map(|x| Cow::Owned(x.to_string()))
  }

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>) {
    self.validation_message = msg;
  }

  /// Gets control's `value`.
  fn get_value<'c: 'b>(&self) -> Option<Cow<'c, Value>> {
    self.value.as_ref().map(|x| Cow::Owned(x.to_owned()))
  }

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not.`
  fn set_value(&mut self, value: Option<Value>) -> bool {
    self.value = value;
    self.check_validity()
  }

  fn get_attributes(&self) -> Option<&Map<String, serde_json::Value>> {
    self.attributes.as_ref()
  }

  fn get_attributes_mut(&mut self) -> Option<&mut Map<String, serde_json::Value>> {
    self.attributes.as_mut()
  }

  fn set_attributes(&mut self, attributes: Option<Map<String, serde_json::Value>>) {
    self.attributes = attributes;
  }
}

impl<'a, 'b, Value, ValueConstraints> Display for HTMLInput<'a, 'b, Value, ValueConstraints>
  where
    Value: InputValue,
    ValueConstraints: InputConstraints<'a, 'b, Value>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", &self)
  }
}

#[cfg(test)]
pub mod test {
  use std::borrow::Cow;
  use walrs_inputfilter::input::{Input, InputBuilder, value_missing_msg};
  use super::*;

  fn _new_input<'a, 'b, T: Default + Clone + InputValue>(constraints: Option<Input<'a, 'b, T>>) -> HTMLInput<'a, 'b, T, Input<'a, 'b, T>> {
    if let Some(_constraints) = constraints {
      HTMLInputBuilder::default()
        .constraints(_constraints)
        .build()
        .unwrap()
    } else {
      HTMLInputBuilder::default()
        .build()
        .unwrap()
    }
  }

  #[test]
  fn test_html_input_new() {
    for name in [Some(Cow::Borrowed(&"test_name")), None] as [Option<Cow<str>>; 2] {
      let input: HTMLInput<&str, Input<&str>> = HTMLInput::new(name.clone());
      assert_eq!(&input.name, &name, "name is invalid");
    }
  }

  #[test]
  fn test_html_input_set_validation_message() {
    for validation_message in
    [Some("Some validation message".to_string()), None]
    {
      let mut input: HTMLInput<&str, Input<&str>> = HTMLInput::new(None);
      input.set_validation_message(validation_message.clone());
      assert_eq!(
        &input.validation_message, &validation_message,
        "validation message is invalid"
      );
    }
  }

  #[test]
  fn test_html_input_get_value() {
    for value in [Some("some-value"), None] {
      let mut input: HTMLInput<&str, Input<&str>> = HTMLInput::new(None);
      input.value = value.into();
      assert_eq!(input.get_value().map(|x| *x), value, "`value` is invalid");
    }
  }

  #[test]
  fn test_html_input_get_constraints() {
    let constraint_seed: Input<&str> = Input::new(None);

    for in_constraints in [
      None,
      Some(constraint_seed.clone()),
    ]
    {
      let mut input: HTMLInput<&str, Input<&str>> = HTMLInput::new(None);
      let in_constraints_cloned = in_constraints.clone();
      input.constraints = in_constraints;
      println!("input.constraints: {:?}, in_constraints_cloned: {:?}",
               &input.constraints, &in_constraints_cloned);
      if input.constraints.is_none() && in_constraints_cloned.is_none() {
        assert!(true);
        continue;
      }
      assert_eq!(
        format!("{}", input.constraints.unwrap()),
        format!("{}", in_constraints_cloned.unwrap()),
                 "constraints are invalid"
      );
    }
  }

  #[test]
  fn test_html_input_validate() {
    let constraints: Input<&str> = InputBuilder::default()
      .required(true)
      .build()
      .unwrap();

    for (value, _constraints, expected_rslt) in [
      (None, None, Ok(())),
      (Some("some-value"), None, Ok(())),
      (Some("some-value"), Some(constraints.clone()), Ok(())),
      (
        None,
        Some(constraints.clone()),
        Err(value_missing_msg(&constraints)),
      ),
    ]
    {
      let mut input: HTMLInput<&str, Input<&str>> = _new_input(_constraints);
      let initial_validation_msg = input.validation_message.clone();
      let rslt = input.validate(value.as_ref());

      assert_eq!(&rslt, &expected_rslt, "result is invalid");

      // Validity state should not have changed.
      assert_eq!(
        &input.validation_message, &initial_validation_msg,
        "validity state is invalid"
      );
    }
  }

  #[test]
  fn test_html_input_check_validity() {
    let mut constraints: Input<&str> = Input::new(None);
    constraints.required = true;
    for (value, constraints, rslt, expected_validation_msg) in [
      (None, None, true, None),
      (Some("some-value"), None, true, None),
      (Some("some-value"), Some(constraints.clone()), true, None),
      (
        None,
        Some(constraints.clone()),
        false,
        Some(value_missing_msg(&constraints)),
      ),
    ]
    {
      let mut input: HTMLInput<&str, Input<&str>> = _new_input(constraints);
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
    for value in [Some("some-value"), None] {
      let mut constraints: Input<&str> = Input::new(None);
      constraints.required = true;

      let mut input: HTMLInput<&str, Input<&str>> =
        HTMLInputBuilder::default()
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
