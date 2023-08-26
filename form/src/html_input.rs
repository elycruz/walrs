use std::borrow::Cow;
use crate::constants::{TEXT_SYMBOL};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug};

use derive_builder::Builder;
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
  pub attributes: Option<HashMap<&'a str, Option<&'a str>>>,

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

#[cfg(test)]
pub mod test {}
