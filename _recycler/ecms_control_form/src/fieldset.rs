extern crate ecms_inputconstraints;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use ecms_inputconstraints::number::NumberConstraints;
use ecms_inputconstraints::text::TextConstraints;
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use crate::button::ButtonControl;
use crate::constants::FIELDSET_SYMBOL;
use crate::fieldset::FormControlVariant::*;
use crate::input::InputControl;
use crate::select::SelectControl;
use crate::traits::{
  FormControl, FormControlConstraints, FormControlValue, PhantomConstraints, PhantomValue,
};

pub trait NullControlValue {}

#[derive(Clone, Debug)]
pub enum FormControlVariant<'a> {
  // @todo Resolve the use of a `Bool{ctrl}` variant - Needed to be in compliance with possible javascript values;
  F64Button(ButtonControl<'a, f64, NumberConstraints<'a, f64>>),
  I64Button(ButtonControl<'a, i64, NumberConstraints<'a, i64>>),
  U64Button(ButtonControl<'a, u64, NumberConstraints<'a, u64>>),
  StrButton(ButtonControl<'a, &'a str, TextConstraints<'a>>),

  F64Input(InputControl<'a, f64, NumberConstraints<'a, f64>>),
  I64Input(InputControl<'a, i64, NumberConstraints<'a, i64>>),
  U64Input(InputControl<'a, u64, NumberConstraints<'a, u64>>),
  StrInput(InputControl<'a, &'a str, TextConstraints<'a>>),

  F64Select(SelectControl<'a, f64, NumberConstraints<'a, f64>>),
  I64Select(SelectControl<'a, i64, NumberConstraints<'a, i64>>),
  U64Select(SelectControl<'a, u64, NumberConstraints<'a, u64>>),
  StrSelect(SelectControl<'a, &'a str, TextConstraints<'a>>),

  Textarea(InputControl<'a, &'a str, TextConstraints<'a>>),
  Fieldset(FieldsetControl<'a, PhantomValue, PhantomConstraints<'a>>),
}

impl<'a> FormControlVariant<'a> {
  pub fn check_validity(&mut self) -> bool {
    match self {
      F64Button(ctrl) => ctrl.check_validity(),
      I64Button(ctrl) => ctrl.check_validity(),
      U64Button(ctrl) => ctrl.check_validity(),
      StrButton(ctrl) => ctrl.check_validity(),

      F64Input(ctrl) => ctrl.check_validity(),
      I64Input(ctrl) => ctrl.check_validity(),
      U64Input(ctrl) => ctrl.check_validity(),
      StrInput(ctrl) => ctrl.check_validity(),

      F64Select(ctrl) => ctrl.check_validity(),
      I64Select(ctrl) => ctrl.check_validity(),
      U64Select(ctrl) => ctrl.check_validity(),
      StrSelect(ctrl) => ctrl.check_validity(),

      Textarea(ctrl) => ctrl.check_validity(),
      Fieldset(ctrl) => ctrl.check_validity(),
    }
  }

  pub fn get_validation_message(&mut self) -> Option<String> {
    match self {
      F64Button(ctrl) => ctrl.get_validation_message(),
      I64Button(ctrl) => ctrl.get_validation_message(),
      U64Button(ctrl) => ctrl.get_validation_message(),
      StrButton(ctrl) => ctrl.get_validation_message(),

      F64Input(ctrl) => ctrl.get_validation_message(),
      I64Input(ctrl) => ctrl.get_validation_message(),
      U64Input(ctrl) => ctrl.get_validation_message(),
      StrInput(ctrl) => ctrl.get_validation_message(),

      F64Select(ctrl) => ctrl.get_validation_message(),
      I64Select(ctrl) => ctrl.get_validation_message(),
      U64Select(ctrl) => ctrl.get_validation_message(),
      StrSelect(ctrl) => ctrl.get_validation_message(),

      Textarea(ctrl) => ctrl.get_validation_message(),
      Fieldset(ctrl) => ctrl.get_validation_message(),
    }
  }
}

impl<'a> Display for FormControlVariant<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
pub struct FieldsetControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue + 'a,
  ValueConstraints: FormControlConstraints<Value> + 'a,
{
  /// Name associated with control's parent (html) form.
  #[builder(setter(into), default = "None")]
  pub name: Option<&'a str>,

  /// HTML `disabled` attribute.
  #[builder(setter(into), default = "false")]
  pub disabled: bool,

  /// HTML `type` attribute.
  #[serde(rename(serialize = "type"))]
  #[builder(setter(into), default = "Some(FIELDSET_SYMBOL)")]
  pub type_: Option<&'a str>,

  /// Associated HTML Label text (can be used as `legend` element text).
  #[builder(setter(into), default = "None")]
  pub label: Option<&'a str>,

  /// Error message produced by control's validation.
  #[builder(setter(into), default = "None")]
  pub validation_message: Option<String>,

  /// Hashmap for control's html attributes that are not defined on this struct;
  /// Other attribs.: e.g., `placeholder`, `cols` etc.;
  #[builder(setter(into), default = "None")]
  pub html_attribs: Option<HashMap<&'a str, Option<&'a str>>>,

  #[serde(skip)]
  #[builder(setter(into), default = "None")]
  pub elements: Option<Vec<FormControlVariant<'a>>>,

  /// Force allows use of "'a" lifetime and "unused" value type.
  /// #{doc(unused)]
  #[builder(setter(into), default = "None")]
  value: Option<PhantomData<Value>>,

  /// Force allows use of "'a" lifetime and "unused" value constraints type.
  /// #{doc(unused)]
  #[builder(setter(into), default = "None")]
  constraints: Option<PhantomData<ValueConstraints>>,
}

impl<'a, Value, ValueConstraints> FieldsetControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue + 'a,
  ValueConstraints: FormControlConstraints<Value>,
{
  pub fn new(name: Option<&'a str>) -> Self {
    FieldsetControl {
      name,
      disabled: false,
      type_: None,
      label: None,
      validation_message: None,
      html_attribs: None,
      elements: None,
      value: Default::default(),
      constraints: Default::default(),
    }
  }
}

impl<'a, Value, ValueConstraints> FormControl<'a, Value, ValueConstraints>
  for FieldsetControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue + 'a,
  ValueConstraints: FormControlConstraints<Value> + 'a,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&ValueConstraints> {
    None
  }

  /// Always returns nothing, for fieldset controls.
  fn get_validation_message(&self) -> Option<String> {
    None
  }

  /// Does nothing for fieldset controls.
  fn set_validation_message(&mut self, _: Option<String>) {}

  fn validate(&mut self, _: Option<Cow<'a, Value>>) -> Result<(), String> {
    self.elements.as_mut().map_or(Ok(()), |elements| {
      for element in elements.iter_mut() {
        if element.check_validity() {
          continue;
        }
        return Err(element.get_validation_message().unwrap().to_string());
      }
      Ok(())
    })
  }

  /// Always return `None`, for fieldset controls.
  fn get_value(&self) -> Option<Cow<'a, Value>> {
    None
  }

  /// Only calls `check_validity()`, for fieldset controls.
  fn set_value(&mut self, _: Option<Value>) -> bool {
    self.check_validity()
  }

  fn get_html_attribs(&self) -> Option<&HashMap<&'a str, Option<&'a str>>> {
    self.html_attribs.as_ref()
  }

  fn set_html_attribs(&mut self, html_attribs: Option<HashMap<&'a str, Option<&'a str>>>) {
    self.html_attribs = html_attribs;
  }
}

impl<'a, Value, ValueConstraints> Display for FieldsetControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue + 'a,
  ValueConstraints: FormControlConstraints<Value> + 'a,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", &self)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use ecms_inputconstraints::number::{value_missing_msg, NumberConstraintsBuilder};

  use crate::button::ButtonControlBuilder;
  use crate::constants::NUMBER_SYMBOL;
  use crate::input::InputControlBuilder;
  use ecms_inputconstraints::text::{TextConstraints, TextConstraintsBuilder};

  #[test]
  fn test_html_fieldset_builder() {
    for name in [Some("test_name"), None] as [Option<&str>; 2] {
      let input: FieldsetControl<&str, TextConstraints> = FieldsetControlBuilder::default()
        .name(name.clone())
        .build()
        .unwrap();
      assert_eq!(&input.name, &name, "name is invalid");
    }
  }

  #[test]
  fn test_html_fieldset_validation_message() {
    for validation_message in
      [Some("Some validation message".to_string()), None] as [Option<String>; 2]
    {
      let mut input: FieldsetControl<&str, TextConstraints> =
        FieldsetControlBuilder::default().build().unwrap();

      input.set_validation_message(validation_message.clone());

      assert_eq!(
        input.validation_message, None,
        "validation message should always be `None`"
      );

      assert_eq!(
        input.get_validation_message(),
        None,
        "validation message should always be `None`"
      );
    }
  }

  #[test]
  fn test_html_fieldset_get_value() {
    for value in [Some("some-value"), None] as [Option<&str>; 2] {
      let mut input: FieldsetControl<&str, TextConstraints> =
        FieldsetControlBuilder::default().build().unwrap();

      assert_eq!(
        input.get_value(),
        None,
        "`get_value()` should always return `None`"
      );
    }
  }

  #[test]
  fn test_html_fieldset_validate() {
    let required_str_constraints: TextConstraints = TextConstraintsBuilder::default()
      .required(true)
      .build()
      .unwrap();

    let required_u64_constraints = NumberConstraintsBuilder::default()
      .required(true)
      .build()
      .unwrap();

    let cases: Vec<(
      Option<&str>,
      Option<Vec<FormControlVariant>>,
      Result<(), String>,
    )> = vec![
      (None, None, Ok(())),
      (Some("hello"), None, Ok(())),
      (
        Some("abc"),
        Some(vec![
          U64Input(
            InputControlBuilder::default()
              .type_(NUMBER_SYMBOL)
              .build()
              .unwrap(),
          ),
          StrInput(
            InputControlBuilder::default()
              .value("hello-world")
              .build()
              .unwrap(),
          ),
          U64Button(
            ButtonControlBuilder::default()
              .constraints(required_u64_constraints.clone())
              .build()
              .unwrap(),
          ),
          StrInput(
            InputControlBuilder::default()
              .value("hello-world")
              .build()
              .unwrap(),
          ),
        ]),
        Err(value_missing_msg(&required_u64_constraints, None)),
      ),
      (None, None, Ok(())),
      (
        None,
        Some(vec![FormControlVariant::Textarea(InputControl::new(None))]),
        Ok(()),
      ),
    ];

    for (value, elements, rslt) in cases {
      let mut input = FieldsetControlBuilder::<&str, TextConstraints>::default()
        .elements(elements)
        .constraints(PhantomData)
        .build()
        .unwrap();

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
  fn test_html_fieldset_check_validity() {
    let mut text_constraints: TextConstraints = TextConstraints::new();
    text_constraints.required = true;
    for (value, constraints, rslt, expected_validation_msg) in vec![
      (None, None, true, None),
      (Some("some-value"), None, true, None),
      (
        Some("some-value"),
        Some(text_constraints.clone()),
        true,
        None,
      ),
      (None, Some(text_constraints.clone()), true, None),
    ]
      as Vec<(Option<&str>, Option<TextConstraints>, bool, Option<String>)>
    {
      let mut input: FieldsetControl<&str, TextConstraints> =
        FieldsetControlBuilder::default().build().unwrap();
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
  fn test_html_fieldset_set_value() {
    for value in [Some("some-value"), None] as [Option<&str>; 2] {
      let mut input: FieldsetControl<&str, TextConstraints> =
        FieldsetControlBuilder::default().build().unwrap();

      input.set_value(value);

      assert_eq!(input.get_value(), None, "`value` is invalid");

      // Since we marked `value` as `required` we can check for control's 'validity state'.
      assert!(
        input.validation_message.is_none(),
        "validity state is invalid"
      );
    }
  }
}
