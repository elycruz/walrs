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
pub struct HTMLInput<'a, Value, Constraints>
where
  Value: InputValue,
  Constraints: 'a + InputConstraints<Value>,
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
}

impl<'a, Value, Constraints> HTMLInput<'a, Value, Constraints>
where
  Value: InputValue,
  Constraints: 'a + InputConstraints<Value>,
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
    }
  }
}


impl<'a, Value: 'a, ValueConstraints: 'a> FormControl<'a, Value, ValueConstraints>
for HTMLInput<'a, Value, ValueConstraints>
  where
    Value: InputValue,
    ValueConstraints: InputConstraints<Value>,
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
  fn get_value(&self) -> Option<Cow<'a, Value>> {
    self.value.as_ref().map(|x| Cow::Owned(x.to_owned()))
  }

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not.`
  fn set_value(&mut self, value: Option<Value>) -> bool {
    self.value = value;
    self.check_validity()
  }

  fn get_attributes(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
    self.attributes.as_ref()
  }

  fn get_attributes_mut(&mut self) -> Option<&mut serde_json::Map<String, serde_json::Value>> {
    self.attributes.as_mut()
  }

  fn set_attributes(&mut self, attributes: Option<serde_json::Map<String, serde_json::Value>>) {
    self.attributes = attributes;
  }
}

impl<Value, ValueConstraints> Display for HTMLInput<'_, Value, ValueConstraints>
  where
    Value: InputValue,
    ValueConstraints: InputConstraints<Value>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self)
  }
}

#[cfg(test)]
pub mod test {}
