use crate::constants::BUTTON_SYMBOL;
use crate::traits::{FormControl, FormControlConstraints, FormControlValue};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

/// HTML Form Control data/validation struct.
#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
#[builder(setter(into, strip_option))]
pub struct ButtonControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  /// Name associated with control's parent (html) form.
  #[builder(default = "None")]
  pub name: Option<&'a str>,

  /// HTML `disabled` attribute.
  #[builder(setter(into), default = "false")]
  pub disabled: bool,

  /// HTML `type` attribute.
  #[serde(rename(serialize = "type"))]
  #[builder(setter(into), default = "BUTTON_SYMBOL")]
  pub type_: &'static str,

  /// Hashmap for control's html attributes that are not defined on this struct;
  /// Other attribs.: e.g., `placeholder`, `cols` etc.;
  #[builder(default = "None")]
  pub html_attribs: Option<HashMap<&'a str, Option<&'a str>>>,

  /// Form control's `value`.
  #[builder(default = "None")]
  pub value: Option<Value>,

  /// Associated HTML Label text.
  #[builder(default = "None")]
  pub label: Option<&'static str>,

  /// Error message produced by control's validation.
  #[builder(default = "None")]
  pub validation_message: Option<String>,

  /// Constraint validation ruleset checked from `validate()` method..
  #[serde(skip)]
  #[builder(default = "None")]
  pub constraints: Option<ValueConstraints>,
}

impl<'a, Value, ValueConstraints> ButtonControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  pub fn new(name: Option<&'a str>) -> Self {
    ButtonControl {
      name,
      disabled: false,
      type_: BUTTON_SYMBOL,
      html_attribs: None,
      value: None,
      label: None,
      validation_message: None,
      constraints: None,
    }
  }
}

impl<'a, Value: 'a, ValueConstraints: 'a> FormControl<'a, Value, ValueConstraints>
  for ButtonControl<'a, Value, ValueConstraints>
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
    self.value.as_ref().map(|x| Cow::Owned(x.clone()))
  }

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not, and receiving `bool` signaling
  /// control's validity.`
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

impl<'a, Value, ValueConstraints> Display for ButtonControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

/* HTML Element interface (ref:
  https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element
):
interface HTMLButtonElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions] attribute boolean disabled;
  readonly attribute FormElement? form;
  [CEReactions] attribute USVString formAction;
  [CEReactions] attribute DOMString formEnctype;
  [CEReactions] attribute DOMString formMethod;
  [CEReactions] attribute boolean formNoValidate;
  [CEReactions] attribute DOMString formTarget;
  [CEReactions] attribute DOMString name;
  [CEReactions] attribute DOMString type;
  [CEReactions] attribute DOMString value;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList labels;
};
 */

#[cfg(test)]
pub mod test {
  use super::*;
  use ecms_inputconstraints::text::{value_missing_msg, TextConstraints};
  use std::borrow::Cow;

  #[test]
  fn test_html_button_new() {
    for name in [Some("test_name"), None] as [Option<&str>; 2] {
      let input: ButtonControl<String, TextConstraints> = ButtonControl::new(name.clone());
      assert_eq!(&input.name, &name, "name is invalid");
    }
  }

  #[test]
  fn test_html_button_set_validation_message() {
    for validation_message in
      [Some("Some validation message".to_string()), None] as [Option<String>; 2]
    {
      let mut input: ButtonControl<&str, TextConstraints> = ButtonControl::new(None);
      input.set_validation_message(validation_message.clone());
      assert_eq!(
        &input.validation_message, &validation_message,
        "validation message is invalid"
      );
    }
  }

  #[test]
  fn test_html_button_get_value() {
    for value in [Some("some-value"), None] as [Option<&str>; 2] {
      let mut input: ButtonControl<&str, TextConstraints> = ButtonControl::new(None);
      input.value = value.into();
      assert_eq!(input.get_value().map(|x| *x), value, "`value` is invalid");
    }
  }

  #[test]
  fn test_html_button_get_constraints() {
    let constraint_seed: TextConstraints = TextConstraints::new();

    for in_constraints in [None, Some(constraint_seed.clone())] as [Option<TextConstraints>; 2] {
      let mut input: ButtonControl<&str, TextConstraints> = ButtonControl::new(None);
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
  fn test_html_button_validate() {
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
      let mut input: ButtonControl<&str, TextConstraints> =
        ButtonControlBuilder::default().build().unwrap();

      // Set constraints
      constraints.map(|c| input.constraints = Some(c));

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
  fn test_html_button_check_validity() {
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
      let mut input: ButtonControl<&str, TextConstraints> =
        ButtonControlBuilder::default().build().unwrap();

      constraints.map(|c| input.constraints = Some(c));

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
  fn test_html_button_set_value() {
    for value in [Some("some-value"), None] as [Option<&str>; 2] {
      let mut constraints: TextConstraints = TextConstraints::new();
      constraints.required = true;

      let mut input: ButtonControl<&str, TextConstraints> = ButtonControlBuilder::default()
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
