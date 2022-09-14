extern crate walrs_inputconstraints;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use crate::button::HTMLButtonControl;
use crate::constants::FIELDSET_SYMBOL;
use crate::input::HTMLInputControl;
use crate::select::HTMLSelectControl;
use crate::traits::{FormControl, FormControlConstraints, FormControlValue};

#[derive(Debug, Clone)]
pub enum FieldsetElement<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  Input(HTMLInputControl<'a, Value, ValueConstraints>),
  Textarea(HTMLInputControl<'a, Value, ValueConstraints>),
  Button(HTMLButtonControl<'a, Value, ValueConstraints>),
  Select(HTMLSelectControl<'a, Value, ValueConstraints>),
  Fieldset(HTMLFieldsetControl<'a, Value, ValueConstraints>),
}

impl<'a, Value: 'a, ValueConstraints: 'a> FormControl<'a, Value, ValueConstraints>
  for FieldsetElement<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&ValueConstraints> {
    match self {
      FieldsetElement::Input(ctrl) => ctrl.get_constraints(),
      FieldsetElement::Textarea(ctrl) => ctrl.get_constraints(),
      FieldsetElement::Button(ctrl) => ctrl.get_constraints(),
      FieldsetElement::Select(ctrl) => ctrl.get_constraints(),
      FieldsetElement::Fieldset(ctrl) => ctrl.get_constraints(),
    }
  }

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>) {
    match self {
      FieldsetElement::Input(ctrl) => ctrl.validation_message = msg,
      FieldsetElement::Textarea(ctrl) => ctrl.validation_message = msg,
      FieldsetElement::Button(ctrl) => ctrl.validation_message = msg,
      FieldsetElement::Select(ctrl) => ctrl.validation_message = msg,
      FieldsetElement::Fieldset(ctrl) => ctrl.validation_message = msg,
    }
  }

  /// Gets control's `value`.
  fn get_value(&self) -> Option<Value> {
    match self {
      FieldsetElement::Input(ctrl) => ctrl.value.clone(),
      FieldsetElement::Textarea(ctrl) => ctrl.value.clone(),
      FieldsetElement::Button(ctrl) => ctrl.value.clone(),
      _ => None,
    }
  }

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not.`
  /// @note Doesn't set any value for `Fieldset(HTMLFieldsetControl)` variant.
  fn set_value(&mut self, value: Option<Value>) -> bool {
    match self {
      FieldsetElement::Input(ctrl) => {
        ctrl.value = value;
        ctrl.check_validity()
      }
      FieldsetElement::Textarea(ctrl) => {
        ctrl.value = value;
        ctrl.check_validity()
      }
      FieldsetElement::Button(ctrl) => {
        ctrl.value = value;
        ctrl.check_validity()
      }
      FieldsetElement::Select(ctrl) => {
        ctrl.value = value;
        ctrl.check_validity()
      }
      FieldsetElement::Fieldset(ctrl) => ctrl.check_validity(),
    }
  }
}

impl<'a, Value, ValueConstraints> Display for FieldsetElement<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct HTMLFieldsetControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  /// Name associated with control's parent (html) form.
  #[builder(setter(into), default = "None")]
  pub name: Option<&'a str>,

  /// HTML `disabled` attribute.
  #[builder(setter(into), default = "false")]
  pub disabled: bool,

  /// HTML `type` attribute.
  #[builder(setter(into), default = "Some(FIELDSET_SYMBOL)")]
  pub type_: Option<&'static str>,

  /// Associated HTML Label text (can be used as `legend` element text).
  #[builder(setter(into), default = "None")]
  pub label: Option<&'static str>,

  /// Error message produced by control's validation.
  #[builder(setter(into), default = "None")]
  pub validation_message: Option<String>,

  /// Hashmap for control's html attributes that are not defined on this struct;
  /// Other attribs.: e.g., `placeholder`, `cols` etc.;
  #[builder(setter(into), default = "None")]
  pub html_attribs: Option<HashMap<&'static str, Option<&'a str>>>,

  /// Constraint validation ruleset checked from `validate()` method..
  #[serde(skip)]
  #[builder(setter(into), default = "None")]
  pub constraints: Option<ValueConstraints>,

  #[serde(skip)]
  pub elements: Option<HashMap<&'static str, FieldsetElement<'a, Value, ValueConstraints>>>,
}

impl<'a, Value: 'a, ValueConstraints: 'a> FormControl<'a, Value, ValueConstraints>
  for HTMLFieldsetControl<'a, Value, ValueConstraints>
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

  fn check_validity(&mut self) -> bool {
    todo!()
  }

  /// Will always return `None`, for fieldset controls.
  fn get_value(&self) -> Option<Value> {
    None
  }

  /// Only calls `check_validity()`, for fieldset controls.
  fn set_value(&mut self, value: Option<Value>) -> bool {
    // Element has no `value` field so should not set any values here.
    // self.value = value;
    self.check_validity()
  }
}

impl<'a, Value, ValueConstraints> Display for HTMLFieldsetControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", &self)
  }
}
