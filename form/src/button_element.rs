//! Button element representation.
//!
//! This module provides the [`ButtonElement`] struct for representing HTML
//! `<button>` elements in a form structure.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::button_element::ButtonElement;
//! use walrs_form::button_type::ButtonType;
//!
//! // Create a submit button
//! let button = ButtonElement::new("submit_btn", ButtonType::Submit);
//! assert_eq!(button.name, Some("submit_btn".to_string()));
//!
//! // Create with label
//! let button = ButtonElement::with_label("Save Changes", ButtonType::Submit);
//! assert_eq!(button.label, Some("Save Changes".to_string()));
//! ```
use crate::button_type::ButtonType;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use walrs_form_core::Attributes;
/// HTML button element.
///
/// Represents the structure of an HTML `<button>` element. This is a data-only
/// representation with no interactive behavior (no click handlers, etc.) since
/// this library is designed for server-side environments.
///
/// # Example
///
/// ```rust
/// use walrs_form::ButtonElement;
/// use walrs_form::ButtonType;
///
/// let button = ButtonElement::new("action", ButtonType::Button);
/// assert_eq!(button._type, ButtonType::Button);
/// ```
#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct ButtonElement {
  /// Element name attribute.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<String>,
  /// Button type (submit, reset, button).
  #[serde(rename = "type")]
  #[builder(default = "ButtonType::Button")]
  pub _type: ButtonType,
  /// Button label/text content.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub label: Option<String>,
  /// Additional HTML attributes.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub attributes: Option<Attributes>,
  /// Whether the button is disabled.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub disabled: Option<bool>,
}
impl ButtonElement {
  /// Creates a new button with the given name and type.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::ButtonElement;
  /// use walrs_form::ButtonType;
  ///
  /// let button = ButtonElement::new("save", ButtonType::Submit);
  /// assert_eq!(button.name, Some("save".to_string()));
  /// assert_eq!(button._type, ButtonType::Submit);
  /// ```
  pub fn new(name: impl Into<String>, button_type: ButtonType) -> Self {
    Self {
      name: Some(name.into()),
      _type: button_type,
      ..Default::default()
    }
  }
  /// Creates a button with a label and type.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::ButtonElement;
  /// use walrs_form::ButtonType;
  ///
  /// let button = ButtonElement::with_label("Reset Form", ButtonType::Reset);
  /// assert_eq!(button.label, Some("Reset Form".to_string()));
  /// assert_eq!(button._type, ButtonType::Reset);
  /// ```
  pub fn with_label(label: impl Into<String>, button_type: ButtonType) -> Self {
    Self {
      _type: button_type,
      label: Some(label.into()),
      ..Default::default()
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_new() {
    let button = ButtonElement::new("btn", ButtonType::Submit);
    assert_eq!(button.name, Some("btn".to_string()));
    assert_eq!(button._type, ButtonType::Submit);
  }
  #[test]
  fn test_with_label() {
    let button = ButtonElement::with_label("Click Me", ButtonType::Button);
    assert_eq!(button.label, Some("Click Me".to_string()));
    assert_eq!(button._type, ButtonType::Button);
  }
  #[test]
  fn test_builder() {
    let button = ButtonElementBuilder::default()
      .name("submit")
      .label("Submit Form")
      ._type(ButtonType::Submit)
      .disabled(true)
      .build()
      .unwrap();
    assert_eq!(button.name, Some("submit".to_string()));
    assert_eq!(button.label, Some("Submit Form".to_string()));
    assert_eq!(button.disabled, Some(true));
  }
  #[test]
  fn test_serialization() {
    let button = ButtonElement::new("test", ButtonType::Submit);
    let json = serde_json::to_string(&button).unwrap();
    assert!(json.contains("\"name\":\"test\""));
    assert!(json.contains("\"type\":\"submit\""));
  }
}
