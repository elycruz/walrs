extern crate ecms_inputconstraints;

use crate::constants::SELECT_ONE_SYMBOL;
use crate::traits::{FormControl, FormControlConstraints, FormControlValue};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use crate::option::{is_value_in_options, OptionControl};
use derive_builder::Builder;

const VALUE_NOT_IN_OPTIONS_MSG: &'static str = "Value is not in options";

/// HTML Form Control data/validation struct.
#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
pub struct SelectControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  /// Name associated with control's parent (html) form.
  #[builder(setter(into, strip_option), default = "None")]
  pub name: Option<&'a str>,

  /// HTML `required` attrib.
  #[builder(setter(into, strip_option), default = "false")]
  pub required: bool,

  /// HTML `disabled` attribute.
  #[builder(setter(into, strip_option), default = "false")]
  pub disabled: bool,

  /// HTML `multiple` attribute.
  #[builder(setter(into, strip_option), default = "false")]
  pub multiple: bool,

  /// HTML `type` attribute.
  #[serde(rename(serialize = "type"))]
  #[builder(setter(into, strip_option), default = "Some(SELECT_ONE_SYMBOL)")]
  pub type_: Option<&'a str>,

  /// Hashmap for control's html attributes that are not defined on this struct;
  /// Other attribs.: e.g., `placeholder`, `cols` etc.;
  #[builder(setter(into, strip_option), default = "None")]
  pub html_attribs: Option<HashMap<&'a str, Option<&'a str>>>,

  /// Form control's `value`.
  #[builder(setter(into, strip_option), default = "None")]
  pub value: Option<Value>,

  /// `values` property should be used when control is functioning in `multiple` mode.`
  #[builder(setter(into, strip_option), default = "None")]
  pub values: Option<Vec<Value>>,

  /// Associated HTML Label text.
  #[builder(setter(into, strip_option), default = "None")]
  pub label: Option<&'a str>,

  /// Error message produced by control's validation.
  #[builder(setter(into, strip_option), default = "None")]
  pub validation_message: Option<String>,

  /// Constraint validation ruleset checked from `validate()` method..
  #[serde(skip)]
  #[builder(setter(into, strip_option), default = "None")]
  pub constraints: Option<ValueConstraints>,

  /// Help text to display below html representation of this form control.
  #[builder(setter(into, strip_option), default = "None")]
  pub help_message: Option<String>,

  /// Select control 'options' elements.
  #[builder(setter(into), default = "None")]
  pub options: Option<Vec<Box<OptionControl<Value>>>>,
}

impl<'a, Value, ValueConstraints> SelectControl<'a, Value, ValueConstraints>
where
  Value: FormControlValue,
  ValueConstraints: FormControlConstraints<Value>,
{
  pub fn new() -> Self {
    SelectControl {
      name: None,
      required: false,
      disabled: false,
      multiple: false,
      type_: Some(SELECT_ONE_SYMBOL),
      html_attribs: None,
      value: None,
      values: None,
      label: None,
      validation_message: None,
      constraints: None,
      help_message: None,
      options: None,
    }
  }

  pub fn is_value_in_options(&self, v: Option<&Value>) -> bool {
    is_value_in_options(self.options.as_deref(), v)
  }
}

impl<'a, Value: 'a, ValueConstraints: 'a> FormControl<'a, Value, ValueConstraints>
  for SelectControl<'a, Value, ValueConstraints>
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

  /// Runs control's validation, stores the result, and returns a bool indicating whether control's
  /// `value`/control's validity itself, is valid or not.
  fn check_validity(&mut self) -> bool {
    // @todo handle `multiple` mode.
    let value = self.get_value();
    let value_in_options = self.is_value_in_options(value.as_deref());
    let rslt = match self.validate(value.clone()) {
      Ok(()) => None,
      Err(err) => Some(err),
    };

    // Ensure value is in options, else return 'invalid' rslt
    let rslt = if (rslt.is_none() && self.required && !value_in_options)
      || (value.is_some() && !value_in_options)
    {
      Some(VALUE_NOT_IN_OPTIONS_MSG.to_string())
    } else {
      rslt
    };

    let out = rslt.is_none();
    self.set_validation_message(rslt);
    out
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

  /// Sets `values` on control - Should only be used when control is in `multiple` mode.
  fn set_values(&mut self, values: Option<&[Value]>) -> bool {
    self.values = values.map_or(None, |vs| {
      if vs.len() == 0 || self.options.is_none() {
        return None;
      }
      let mut out = vec![];
      for v in vs {
        if is_value_in_options(self.options.as_deref(), Some(v)) {
          out.push((*v).clone());
        }
      }
      Some(out)
    });
    self.check_validity()
  }

  fn get_html_attribs(&self) -> Option<&HashMap<&'a str, Option<&'a str>>> {
    self.html_attribs.as_ref()
  }

  fn set_html_attribs(&mut self, html_attribs: Option<HashMap<&'a str, Option<&'a str>>>) {
    self.html_attribs = html_attribs;
  }
}

impl<'a, Value, ValueConstraints> Display for SelectControl<'a, Value, ValueConstraints>
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
  use crate::constants::{
    CONSTRAINTS_SYMBOL, DISABLED_SYMBOL, HELP_MESSAGE_SYMBOL, HTML_ATTRIBS_SYMBOL, LABEL_SYMBOL,
    MULTIPLE_SYMBOL, NAME_SYMBOL, OPTIONS_SYMBOL, REQUIRED_SYMBOL, TYPE_SYMBOL,
    VALIDATION_MESSAGE_SYMBOL, VALUES_SYMBOL, VALUE_SYMBOL,
  };
  use crate::option::html_option_ctrl_tests::options_from_str_provider;
  use crate::option::{OptionControl, OptionControlBuilder};
  use ecms_inputconstraints::text::{value_missing_msg, TextConstraints};
  use std::borrow::Cow;
  use std::error::Error;

  type Value<'a> = &'a str;
  type ValidityBool = bool;
  type TestName = &'static str;
  type IsRequired = bool;

  const VOWELS_STR: &'static str = "aeiou";

  fn assert_defaults<T, ConstraintsT>(ctrl: &SelectControl<T, ConstraintsT>)
  where
    T: FormControlValue,
    ConstraintsT: FormControlConstraints<T>,
  {
    assert_eq!(&ctrl.name, &None, "{} is invalid", NAME_SYMBOL);
    assert_eq!(ctrl.required, false, "{} is invalid", REQUIRED_SYMBOL);
    assert_eq!(ctrl.disabled, false, "{} is invalid", DISABLED_SYMBOL);
    assert_eq!(ctrl.multiple, false, "{} is invalid", MULTIPLE_SYMBOL);
    assert_eq!(
      &ctrl.type_,
      &Some(SELECT_ONE_SYMBOL),
      "{}_ is invalid",
      TYPE_SYMBOL
    );
    assert_eq!(
      &ctrl.html_attribs, &None,
      "{} is invalid",
      HTML_ATTRIBS_SYMBOL
    );
    assert_eq!(&ctrl.value, &None, "{} is invalid", VALUE_SYMBOL);
    assert_eq!(&ctrl.values, &None, "{} is invalid", VALUES_SYMBOL);
    assert_eq!(&ctrl.label, &None, "{} is invalid", LABEL_SYMBOL);
    assert_eq!(
      &ctrl.validation_message, &None,
      "{} is invalid",
      VALIDATION_MESSAGE_SYMBOL
    );
    assert!(
      ctrl.constraints.is_none(),
      "{} is invalid",
      CONSTRAINTS_SYMBOL
    );
    assert_eq!(
      &ctrl.help_message, &None,
      "{} is invalid",
      HELP_MESSAGE_SYMBOL
    );
    assert!(ctrl.options.is_none(), "{} is invalid", OPTIONS_SYMBOL);
  }

  #[test]
  fn test_html_select_new() {
    let ctrl: SelectControl<&str, TextConstraints> = SelectControl::new();
    assert_defaults(&ctrl);
  }

  #[test]
  fn test_is_value_in_options() {
    let str_ops: Vec<Box<OptionControl<Value>>> = options_from_str_provider("aeiou");
    let in_options = true;
    type Expected = bool;
    let test_cases: Vec<(
      TestName,
      Option<Value>,
      Option<Vec<Box<OptionControl<Value>>>>,
      Expected,
    )> = vec![
      ("with no options, and no value", None, None, !in_options),
      ("with no options, and value", Some("a"), None, !in_options),
      (
        "with options of length `0`, and no value",
        None,
        Some(vec![]),
        !in_options,
      ),
      (
        "with options of length `0`, and value",
        Some("a"),
        Some(vec![]),
        !in_options,
      ),
      (
        "with options, and no value",
        None,
        Some(str_ops.clone()),
        !in_options,
      ),
      (
        "with options, and non-matching value",
        Some("x"),
        Some(str_ops.clone()),
        !in_options,
      ),
      (
        "with options, and matching value",
        Some("a"),
        Some(str_ops.clone()),
        in_options,
      ),
    ];

    for (i, (test_name, value, options, expected_rslt)) in test_cases.into_iter().enumerate() {
      println!("({}) {}", i, test_name);

      let ctrl: SelectControl<&str, TextConstraints> = SelectControlBuilder::default()
        .options(options)
        .build()
        .unwrap();

      assert_eq!(
        ctrl.is_value_in_options(value.as_ref()),
        expected_rslt,
        "Resulting `bool` doesn't match expected"
      );
    }
  }

  #[test]
  fn test_html_select_builder() -> Result<(), Box<dyn Error>> {
    let with_defaults: SelectControl<&str, TextConstraints> =
      SelectControlBuilder::default().build().unwrap();

    assert_defaults(&with_defaults);

    let options: Vec<Box<OptionControl<String>>> = VOWELS_STR
      .chars()
      .enumerate()
      .map(|(i, c)| {
        Box::new(
          OptionControlBuilder::default()
            .value(c.to_string())
            .label(c.to_string().to_uppercase())
            .selected(i & 1 == 0)
            .build()
            .unwrap(),
        )
      })
      .collect();

    let with_options: SelectControl<String, TextConstraints> = SelectControlBuilder::default()
      .name("with-options")
      .options(Some(options))
      .build()
      .unwrap();

    // Test initial assumptions
    if let Some(ops) = with_options.options.as_deref() {
      ops.iter().enumerate().for_each(|(i, o)| {
        assert_eq!(
          o.value.as_deref().unwrap(),
          &VOWELS_STR[i..i + 1],
          "`value` invalid"
        );
        assert_eq!(o.selected, i & 1 == 0, "`selected` invalid");
      });
    } else {
      panic!("Expected `options` to be set");
    }

    // Test for 'Debug' derive
    println!("{:#?}", with_defaults);
    println!("{:#?}", &with_options);

    // Test for serde 'Deserialize' trait
    println!("{}", serde_json::to_string_pretty(&with_options)?);

    Ok(())
  }

  #[test]
  fn test_html_select_set_validation_message() {
    for validation_message in
      [Some("Some validation message".to_string()), None] as [Option<String>; 2]
    {
      let mut input: SelectControl<&str, TextConstraints> = SelectControl::new();
      input.set_validation_message(validation_message.clone());
      assert_eq!(
        &input.validation_message, &validation_message,
        "validation message is invalid"
      );
    }
  }

  #[test]
  fn test_html_select_get_value() {
    for value in [Some("some-value"), None] as [Option<&str>; 2] {
      let mut input: SelectControl<&str, TextConstraints> = SelectControl::new();
      input.value = value.into();
      assert_eq!(&input.value, &value, "`value` is invalid");
    }
  }

  #[test]
  fn test_html_select_get_constraints() {
    let constraint_seed: TextConstraints = TextConstraints::new();

    for in_constraints in [None, Some(constraint_seed.clone())] as [Option<TextConstraints>; 2] {
      let mut input: SelectControl<&str, TextConstraints> =
        SelectControlBuilder::default().build().unwrap();
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
  fn test_html_select_validate() {
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
      let mut input: SelectControl<&str, TextConstraints> = if let Some(_constraints) = constraints
      {
        SelectControlBuilder::default()
          .constraints(_constraints)
          .build()
          .unwrap()
      } else {
        SelectControlBuilder::default().build().unwrap()
      };
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
  fn test_html_select_select_one_e2e() {
    let options: Vec<Box<OptionControl<&str>>> = options_from_str_provider("aeiou");
    let empty_options: Vec<Box<OptionControl<&str>>> = vec![];

    let mut required_constraints: TextConstraints = TextConstraints::new();
    required_constraints.required = true;

    let default_constraints: TextConstraints = TextConstraints::new();

    let required = true;
    let valid = true;

    let test_cases: Vec<(
      TestName,
      Option<Value>,
      Option<Vec<Box<OptionControl<Value>>>>,
      Option<TextConstraints>,
      IsRequired,
      ValidityBool,
    )> = vec![
      (
        "With no options, and value, not required",
        Some("some-value"),
        None,
        None,
        !required,
        !valid,
      ),
      (
        "With no options, or value, not required",
        None,
        None,
        None,
        !required,
        valid,
      ),
      (
        "With empty options, no value, and not required",
        None,
        Some(empty_options.clone()),
        None,
        !required,
        valid,
      ),
      (
        "With options, no value, not required",
        None,
        Some(options.clone()),
        None,
        !required,
        valid,
      ),
      (
        "With options, and 'valid' value, not required",
        Some("a"),
        Some(options.clone()),
        None,
        !required,
        valid,
      ),
      (
        "With no options, and 'invalid' value, required",
        Some("some-value"),
        None,
        None,
        required,
        !valid,
      ),
      (
        "With no options, or value, required",
        None,
        None,
        None,
        required,
        !valid,
      ),
      (
        "With options, no value, required",
        None,
        Some(options.clone()),
        None,
        required,
        !valid,
      ),
      (
        "With options, and 'valid' value, required",
        Some("a"),
        Some(options.clone()),
        None,
        required,
        valid,
      ),
      (
        "With no options, and 'invalid' value, required",
        Some("some-value"),
        None,
        Some(required_constraints.clone()),
        required,
        !valid,
      ),
      (
        "With no options, or value, required",
        None,
        None,
        Some(required_constraints.clone()),
        required,
        !valid,
      ),
      (
        "With options, no value, required",
        None,
        Some(options.clone()),
        Some(required_constraints.clone()),
        required,
        !valid,
      ),
      (
        "With options, and 'valid' value, required",
        Some("a"),
        Some(options.clone()),
        Some(required_constraints.clone()),
        required,
        valid,
      ),
      (
        "With no options, and value, required",
        Some("some-value"),
        None,
        Some(required_constraints.clone()),
        !required,
        !valid,
      ),
      (
        "With no options, or value, required",
        None,
        None,
        Some(required_constraints.clone()),
        !required,
        !valid,
      ),
      (
        "With options, no value, required",
        None,
        Some(options.clone()),
        Some(required_constraints.clone()),
        !required,
        !valid,
      ),
      (
        "With options, and valid value, required",
        Some("a"),
        Some(options.clone()),
        Some(required_constraints.clone()),
        !required,
        valid,
      ),
    ];

    for (i, (test_name, value, options, constraints, is_required, expected_validity)) in
      test_cases.into_iter().enumerate()
    {
      println!("Test {}: {}", i + 1, test_name);

      let constraints_ref = constraints.clone();

      let mut control: SelectControl<&str, TextConstraints> =
        if let Some(_constraints) = constraints {
          SelectControlBuilder::default()
            .constraints(_constraints)
            .required(is_required)
            .options(options)
            .build()
            .unwrap()
        } else {
          SelectControlBuilder::default()
            .required(is_required)
            .options(options)
            .build()
            .unwrap()
        };

      let validity = control.set_value(value.into());

      // println!("{:?}", &control);

      assert_eq!(
        validity, expected_validity,
        "expected `{}` validity",
        expected_validity
      );
      assert_eq!(&control.value, &value, "{} is invalid", VALUES_SYMBOL);
      assert_eq!(
        control.get_value().map(|x| *x),
        value.as_deref(),
        "`get_value()` is invalid"
      );
      assert_eq!(
        control.get_constraints().is_some(),
        constraints_ref.is_some(),
        "`get_constraints()` is invalid"
      );

      // Validate validation_message.
      assert!(
        if !validity {
          control.validation_message.is_some()
        } else {
          control.validation_message.is_none()
        },
        "{} is invalid",
        VALIDATION_MESSAGE_SYMBOL
      );
    }
  }
}

/*

// WHATWG's HTMLSelectElement interface (for reference):

[Exposed=Window]
interface HTMLSelectElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions] attribute DOMString autocomplete;
  [CEReactions] attribute boolean disabled;
  readonly attribute FormElement? form;
  [CEReactions] attribute boolean multiple;
  [CEReactions] attribute DOMString name;
  [CEReactions] attribute boolean required;
  [CEReactions] attribute unsigned long size;

  readonly attribute DOMString type;

  [SameObject] readonly attribute HTMLOptionControlsCollection options;
  [CEReactions] attribute unsigned long length;
  getter HTMLOptionControlElement? item(unsigned long index);
  HTMLOptionControlElement? namedItem(DOMString name);
  [CEReactions] undefined add((HTMLOptionControlElement or HTMLOptGroupElement) element, optional (HTMLElement or long)? before = null);
  [CEReactions] undefined remove(); // ChildNode overload
  [CEReactions] undefined remove(long index);
  [CEReactions] setter undefined (unsigned long index, HTMLOptionControlElement? option);

  [SameObject] readonly attribute HTMLCollection selectedOptions;
  attribute long selectedIndex;
  attribute DOMString value;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList labels;
};
*/
