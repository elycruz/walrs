extern crate ecms_inputvalidator;

use crate::constants::{SELECT_SYMBOL, SELECT_ONE_SYMBOL};
use crate::traits::{FormControl, FormControlConstraints, FormControlValue};
use ecms_inputvalidator::types::{InputConstraints};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use derive_builder::Builder;
use crate::option::{HTMLOptionCtrl, is_value_in_options};

const VALUE_NOT_IN_OPTIONS_MSG: &'static str = "Value is not in options";

/// HTML Form Control data/validation struct.
#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
pub struct HTMLSelectControl<'a, Value, ValueConstraints>
  where
    Value: FormControlValue,
    ValueConstraints: InputConstraints<Value> + Debug,
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
  #[builder(setter(into), default = "None")]
  pub constraints: Option<ValueConstraints>,

  /// Help text to display below html representation of this form control.
  #[builder(setter(into, strip_option), default = "None")]
  pub help_message: Option<String>,

  /// Select control 'options' elements.
  #[builder(setter(into), default = "None")]
  pub options: Option<Vec<Box<HTMLOptionCtrl<Value>>>>,
}

impl<'a, Value, ValueConstraints> HTMLSelectControl<'a, Value, ValueConstraints>
  where
    Value: FormControlValue,
    ValueConstraints: InputConstraints<Value> + Debug,
{
  pub fn new() -> Self {
    HTMLSelectControl {
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
for HTMLSelectControl<'a, Value, ValueConstraints>
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

  /// Runs control's validation, stores the result, and returns a bool indicating whether control's
  /// `value`/control's validity itself, is valid or not.
  fn check_validity(&mut self) -> bool {
    // @todo handle `multiple` mode.
    let value = self.get_value(); // @todo should be using references here
    let rslt = match self.validate(value.clone()) { // @todo ""
      Ok(()) => None,
      Err(err) => Some(err),
    };
    let value_in_options = self.is_value_in_options(value.as_ref());

    // Ensure value is in options, else return 'invalid' rslt
    let rslt = if (rslt.is_none() && self.required &&
      !value_in_options) || (value.is_some() && !value_in_options) {
      Some(VALUE_NOT_IN_OPTIONS_MSG.to_string())
    } else { rslt };

    let out = rslt.is_none();
    self.set_validation_message(rslt);
    out
  }

  /// Gets control's `value`.
  fn get_value(&self) -> Option<Value> {
    self.value.clone()
  }

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not.`
  fn set_value(&mut self, value: Option<Value>) -> bool { // @todo Should take `Option<&Value>` here, instead of `Option<Value>`, etc.
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
}

impl<'a, Value, ValueConstraints> Display for HTMLSelectControl<'a, Value, ValueConstraints>
  where
    Value: FormControlValue,
    ValueConstraints: InputConstraints<Value> + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", &self)
  }
}

#[cfg(test)]
pub mod test {
  use super::*;
  use crate::option::{HTMLOptionCtrl, HTMLOptionCtrlBuilder};
  use crate::option::html_option_ctrl_tests::options_from_str_provider;
  use crate::constants::{
    CONSTRAINTS_SYMBOL, DISABLED_SYMBOL,
    HELP_MESSAGE_SYMBOL, HTML_ATTRIBS_SYMBOL,
    ID_SYMBOL, LABEL_SYMBOL, NAME_SYMBOL,
    OPTIONS_SYMBOL, REQUIRED_SYMBOL,
    TYPE_SYMBOL, VALIDATION_MESSAGE_SYMBOL,
    VALUE_SYMBOL, VALUES_SYMBOL, MULTIPLE_SYMBOL,
  };
  use ecms_inputvalidator::text::{too_short_msg, value_missing_msg, TextInputConstraints};
  use std::error::Error;

  type Value<'a> = &'a str;
  type ValidityBool = bool;
  type TestName = &'static str;
  type IsRequired = bool;

  const VOWELS_STR: &'static str = "aeiou";

  fn assert_defaults<T, ConstraintsT>(ctrl: &HTMLSelectControl<T, ConstraintsT>)
    where T: FormControlValue, ConstraintsT: FormControlConstraints<T> {
    assert_eq!(&ctrl.name, &None, "{} is invalid", NAME_SYMBOL);
    assert_eq!(ctrl.required, false, "{} is invalid", REQUIRED_SYMBOL);
    assert_eq!(ctrl.disabled, false, "{} is invalid", DISABLED_SYMBOL);
    assert_eq!(ctrl.multiple, false, "{} is invalid", MULTIPLE_SYMBOL);
    assert_eq!(&ctrl.type_, &Some(SELECT_ONE_SYMBOL), "{}_ is invalid", TYPE_SYMBOL);
    assert_eq!(&ctrl.html_attribs, &None, "{} is invalid", HTML_ATTRIBS_SYMBOL);
    assert_eq!(&ctrl.value, &None, "{} is invalid", VALUE_SYMBOL);
    assert_eq!(&ctrl.values, &None, "{} is invalid", VALUES_SYMBOL);
    assert_eq!(&ctrl.label, &None, "{} is invalid", LABEL_SYMBOL);
    assert_eq!(&ctrl.validation_message, &None, "{} is invalid", VALIDATION_MESSAGE_SYMBOL);
    assert!(ctrl.constraints.is_none(), "{} is invalid", CONSTRAINTS_SYMBOL);
    assert_eq!(&ctrl.help_message, &None, "{} is invalid", HELP_MESSAGE_SYMBOL);
    assert!(ctrl.options.is_none(), "{} is invalid", OPTIONS_SYMBOL);
  }

  #[test]
  fn test_html_select_new() {
    let ctrl: HTMLSelectControl<&str, TextInputConstraints> = HTMLSelectControl::new();
    assert_defaults(&ctrl);
  }

  #[test]
  fn test_is_value_in_options() {
    let str_ops: Vec<Box<HTMLOptionCtrl<Value>>> = options_from_str_provider("aeiou");
    let str_ops_len = str_ops.len();
    let in_options = true;
    type Expected = bool;
    let test_cases: Vec<(TestName, Option<Value>,
                         Option<Vec<Box<HTMLOptionCtrl<Value>>>>, Expected)> = vec![
      ("with no options, and no value", None, None, !in_options),
      ("with no options, and value", Some("a"), None, !in_options),
      ("with options of length `0`, and no value", None, Some(vec![]), !in_options),
      ("with options of length `0`, and value", Some("a"), Some(vec![]), !in_options),
      ("with options, and no value", None, Some(str_ops.clone()), !in_options),
      ("with options, and non-matching value", Some("x"), Some(str_ops.clone()), !in_options),
      ("with options, and matching value", Some("a"), Some(str_ops.clone()), in_options),
    ];

    for (i, (
      test_name, value,
      options, expected_rslt
    )) in test_cases.into_iter().enumerate() {
      println!("({}) {}", i, test_name);

      let ctrl: HTMLSelectControl<&str, TextInputConstraints> =
        HTMLSelectControlBuilder::default()
          .options(options)
          .build()
          .unwrap();

      assert_eq!(ctrl.is_value_in_options(value.as_ref()), expected_rslt,
                 "Resulting `bool` doesn't match expected");
    }
  }

  #[test]
  fn test_html_select_builder() -> Result<(), Box<dyn Error>> {
    let with_defaults: HTMLSelectControl<&str, TextInputConstraints> =
      HTMLSelectControlBuilder::default().build().unwrap();

    assert_defaults(&with_defaults);

    let options: Vec<Box<HTMLOptionCtrl<String>>> = VOWELS_STR.chars().enumerate()
      .map(|(i, c)| Box::new(
        HTMLOptionCtrlBuilder::default()
          .value(c.to_string())
          .label(c.to_string().to_uppercase())
          .selected(i & 1 == 0)
          .build().unwrap()
      )).collect();

    let with_options: HTMLSelectControl<String, TextInputConstraints> =
      HTMLSelectControlBuilder::default()
        .name("with-options")
        .options(Some(options))
        .build()
        .unwrap();

    // Test initial assumptions
    if let Some(ops) = with_options.options.as_deref() {
      ops.iter().enumerate().for_each(|(i, o)| {
        assert_eq!(o.value.as_deref().unwrap(), &VOWELS_STR[i..i + 1], "`value` invalid");
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
      let mut input: HTMLSelectControl<&str, TextInputConstraints> = HTMLSelectControl::new();
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
      let mut input: HTMLSelectControl<&str, TextInputConstraints> = HTMLSelectControl::new();
      input.value = value.clone();
      assert_eq!(&input.value, &value, "`value` is invalid");
    }
  }

  #[test]
  fn test_html_select_get_constraints() {
    let constraint_seed: TextInputConstraints = TextInputConstraints::new();

    for (value, in_constraints) in [
      (Some("some-value"), None),
      (Some("some-value"), Some(constraint_seed.clone())),
      (None, None),
      (None, Some(constraint_seed.clone())),
    ] as [(Option<&str>, Option<TextInputConstraints>); 4]
    {
      let mut input: HTMLSelectControl<&str, TextInputConstraints> = HTMLSelectControlBuilder::default().build().unwrap();
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
      let mut input: HTMLSelectControl<&str, TextInputConstraints> =
        HTMLSelectControlBuilder::default()
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
  fn test_html_select_select_one_e2e() {
    let options: Vec<Box<HTMLOptionCtrl<&str>>> = options_from_str_provider("aeiou");

    let mut required_constraints: TextInputConstraints = TextInputConstraints::new();
    required_constraints.required = true;

    let default_constraints: TextInputConstraints = TextInputConstraints::new();

    let required = true;
    let valid = true;

    let test_cases: Vec<(
      TestName,
      Option<Value>,
      Option<Vec<Box<HTMLOptionCtrl<Value>>>>,
      Option<TextInputConstraints>,
      IsRequired,
      ValidityBool
    )> = vec![
      ("With no options, and value, not required", Some("some-value"), None, None, !required, !valid),
      ("With no options, or value, not required", None, None, None, !required, valid),
      ("With options, no value, not required", None, Some(options.clone()), None, !required, valid),
      ("With options, and 'valid' value, not required", Some("a"), Some(options.clone()), None, !required, valid),
      ("With no options, and 'invalid' value, required", Some("some-value"), None, None, required, !valid),
      ("With no options, or value, required", None, None, None, required, !valid),
      ("With options, no value, required", None, Some(options.clone()), None, required, !valid),
      ("With options, and 'valid' value, required", Some("a"), Some(options.clone()), None, required, valid),
      ("With no options, and 'invalid' value, required", Some("some-value"), None, Some(required_constraints.clone()), required, !valid),
      ("With no options, or value, required", None, None, Some(required_constraints.clone()), required, !valid),
      ("With options, no value, required", None, Some(options.clone()), Some(required_constraints.clone()), required, !valid),
      ("With options, and 'valid' value, required", Some("a"), Some(options.clone()), Some(required_constraints.clone()), required, valid),
      ("With no options, and value, required", Some("some-value"), None, Some(required_constraints.clone()), !required, !valid),
      ("With no options, or value, required", None, None, Some(required_constraints.clone()), !required, !valid),
      ("With options, no value, required", None, Some(options.clone()), Some(required_constraints.clone()), !required, !valid),
      ("With options, and valid value, required", Some("a"), Some(options.clone()), Some(required_constraints.clone()), !required, valid),
    ];

    for (i, (
      test_name,
      value,
      options,
      constraints,
      is_required,
      expected_validity
    )) in test_cases.into_iter().enumerate() {
      println!("Test {}: {}", i + 1, test_name);

      let constraints_ref = constraints.clone();

      let mut control: HTMLSelectControl<&str, TextInputConstraints> =
        HTMLSelectControlBuilder::default()
          .constraints(constraints)
          .required(is_required)
          .options(options)
          .build()
          .unwrap();

      let validity = control.set_value(value.clone());

      // println!("{:?}", &control);

      assert_eq!(validity, expected_validity, "expected `{}` validity", expected_validity);
      assert_eq!(&control.value, &value, "{} is invalid", VALUES_SYMBOL);
      assert_eq!(control.get_value(), value, "`get_value()` is invalid");
      assert_eq!(control.get_constraints().is_some(), constraints_ref.is_some(),
                 "`get_constraints()` is invalid");

      // Validate validation_message.
      assert!(
        if !validity { control.validation_message.is_some() } else {
          control.validation_message.is_none()
        },
        "{} is invalid", VALIDATION_MESSAGE_SYMBOL
      );
    }
  }
}

/*

// whatwg's HTMLSelectElement interface:

[Exposed=Window]
interface HTMLSelectElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions] attribute DOMString autocomplete;
  [CEReactions] attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
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
