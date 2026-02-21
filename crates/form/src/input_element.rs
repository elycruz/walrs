//! Input element representation.
//!
//! This module provides the [`InputElement`] struct for representing HTML
//! `<input>` elements in a form structure.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::{InputElement, InputType};
//!
//! let email = InputElement::new("email", InputType::Email);
//! assert_eq!(email.name.as_deref(), Some("email"));
//! assert_eq!(email._type, InputType::Email);
//! ```
use crate::input_type::InputType;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use walrs_form_core::{Attributes, Value};
use walrs_inputfilter::Field;
use walrs_validation::Violations;
/// HTML input element.
///
/// Represents the structure of an HTML `<input>` element with optional
/// validation configuration. This is a data-only representation with
/// no interactive behavior.
///
/// # Example
///
/// ```rust
/// use walrs_form::{InputElement, InputType};
/// use serde_json::json;
///
/// let mut input = InputElement::new("username", InputType::Text);
/// input.value = Some(json!("john_doe"));
/// input.required = Some(true);
///
/// assert_eq!(input.name.as_deref(), Some("username"));
/// ```
#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct InputElement {
  /// Element name attribute.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<Cow<'static, str>>,
  /// Input type.
  #[serde(rename = "type")]
  #[builder(default = "InputType::Text")]
  pub _type: InputType,
  /// Current value.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub value: Option<Value>,
  /// Label text.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub label: Option<String>,
  /// Additional HTML attributes.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub attributes: Option<Attributes>,
  /// Validation field configuration.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub field: Option<Field<Value>>,
  /// Whether the field is required.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub required: Option<bool>,
  /// Whether the field is disabled.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub disabled: Option<bool>,
  /// Validation error message from last validation.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub validation_message: Option<String>,
  /// Help text to display.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub help_message: Option<String>,
}
impl InputElement {
  /// Creates a new InputElement with the given name and type.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{InputElement, InputType};
  ///
  /// let input = InputElement::new("password", InputType::Password);
  /// assert_eq!(input._type, InputType::Password);
  /// ```
  pub fn new(name: impl Into<String>, input_type: InputType) -> Self {
    Self {
      name: Some(Cow::Owned(name.into())),
      _type: input_type,
      ..Default::default()
    }
  }
  /// Validates the given value against the field configuration.
  ///
  /// Returns `Ok(())` if no field configuration is set or validation passes.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{InputElement, InputType};
  /// use serde_json::json;
  ///
  /// let input = InputElement::new("test", InputType::Text);
  /// assert!(input.validate_value(&json!("hello")).is_ok());
  /// ```
  pub fn validate_value(&self, value: &Value) -> Result<(), Violations> {
    if let Some(ref field) = self.field {
      field.validate(value)
    } else {
      Ok(())
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  #[test]
  fn test_new() {
    let input = InputElement::new("email", InputType::Email);
    assert_eq!(input.name.as_deref(), Some("email"));
    assert_eq!(input._type, InputType::Email);
  }
  #[test]
  fn test_builder() {
    let input = InputElementBuilder::default()
      .name("username")
      .required(true)
      ._type(InputType::Text)
      .label("Username")
      .build()
      .unwrap();
    assert_eq!(input.name.as_deref(), Some("username"));
    assert_eq!(input.required, Some(true));
    assert_eq!(input.label, Some("Username".to_string()));
  }
  #[test]
  fn test_serialization() {
    let input = InputElement::new("email", InputType::Email);
    let json = serde_json::to_string(&input).unwrap();
    assert!(json.contains("\"name\":\"email\""));
    assert!(json.contains("\"type\":\"email\""));
  }
  #[test]
  fn test_validate_without_field() {
    let input = InputElement::new("test", InputType::Text);
    assert!(input.validate_value(&json!("hello")).is_ok());
  }
  #[test]
  fn test_with_value() {
    let mut input = InputElement::new("age", InputType::Number);
    input.value = Some(json!(25));
    assert_eq!(input.value.unwrap().as_i64(), Some(25));
  }
}
