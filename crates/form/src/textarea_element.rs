//! Textarea element representation.
//!
//! This module provides the [`TextareaElement`] struct for representing HTML
//! `<textarea>` elements.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::TextareaElement;
//!
//! let textarea = TextareaElement::new("bio");
//! assert_eq!(textarea.name.as_deref(), Some("bio"));
//! ```
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use walrs_form_core::{Attributes, Value};
use walrs_inputfilter::Field;
use walrs_validation::Violations;
/// HTML textarea element.
///
/// Represents a `<textarea>` element for multi-line text input.
///
/// # Example
///
/// ```rust
/// use walrs_form::TextareaElement;
///
/// let mut textarea = TextareaElement::new("description");
/// textarea.rows = Some(5);
/// textarea.cols = Some(40);
/// textarea.value = Some("Default text".to_string());
///
/// assert_eq!(textarea.rows, Some(5));
/// ```
#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct TextareaElement {
  /// Element name attribute.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<Cow<'static, str>>,
  /// Current value.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub value: Option<String>,
  /// Number of visible text rows.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub rows: Option<u32>,
  /// Number of visible text columns.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub cols: Option<u32>,
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
}
impl TextareaElement {
  /// Creates a new textarea element with the given name.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::TextareaElement;
  ///
  /// let textarea = TextareaElement::new("comments");
  /// assert_eq!(textarea.name.as_deref(), Some("comments"));
  /// ```
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: Some(Cow::Owned(name.into())),
      ..Default::default()
    }
  }
  /// Creates a textarea with specified dimensions.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::TextareaElement;
  ///
  /// let textarea = TextareaElement::with_size("notes", 10, 60);
  /// assert_eq!(textarea.rows, Some(10));
  /// assert_eq!(textarea.cols, Some(60));
  /// ```
  pub fn with_size(name: impl Into<String>, rows: u32, cols: u32) -> Self {
    Self {
      name: Some(Cow::Owned(name.into())),
      rows: Some(rows),
      cols: Some(cols),
      ..Default::default()
    }
  }
  /// Validates the given value against the field configuration.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::TextareaElement;
  /// use serde_json::json;
  ///
  /// let textarea = TextareaElement::new("test");
  /// assert!(textarea.validate_value(&json!("some text")).is_ok());
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
    let textarea = TextareaElement::new("bio");
    assert_eq!(textarea.name.as_deref(), Some("bio"));
  }
  #[test]
  fn test_with_size() {
    let textarea = TextareaElement::with_size("notes", 5, 40);
    assert_eq!(textarea.rows, Some(5));
    assert_eq!(textarea.cols, Some(40));
  }
  #[test]
  fn test_builder() {
    let textarea = TextareaElementBuilder::default()
      .name("description")
      .rows(10u32)
      .cols(80u32)
      .required(true)
      .build()
      .unwrap();
    assert_eq!(textarea.rows, Some(10));
    assert_eq!(textarea.cols, Some(80));
    assert_eq!(textarea.required, Some(true));
  }
  #[test]
  fn test_validate_without_field() {
    let textarea = TextareaElement::new("test");
    assert!(textarea.validate_value(&json!("text")).is_ok());
  }
  #[test]
  fn test_serialization() {
    let textarea = TextareaElement::new("test");
    let json = serde_json::to_string(&textarea).unwrap();
    assert!(json.contains("\"name\":\"test\""));
  }
}
