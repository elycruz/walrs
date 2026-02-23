//! Select element representation.
//!
//! This module provides the [`SelectElement`] struct for representing HTML
//! `<select>` elements.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::{SelectElement, SelectOption};
//!
//! let mut select = SelectElement::new("country");
//! select.options.push(SelectOption::new("us", "United States"));
//! select.options.push(SelectOption::new("ca", "Canada"));
//!
//! assert_eq!(select.options.len(), 2);
//! ```
use crate::select_option::SelectOption;
use crate::select_type::SelectType;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use walrs_validation::{Attributes, Value};
use walrs_inputfilter::Field;
use walrs_validation::Violations;
/// HTML select element.
///
/// Represents a `<select>` element with options. Supports both single and
/// multiple selection modes.
///
/// # Example
///
/// ```rust
/// use walrs_form::{SelectElement, SelectOption, SelectType};
///
/// let mut select = SelectElement::new("color");
/// select.options = vec![
///     SelectOption::new("red", "Red"),
///     SelectOption::new("blue", "Blue"),
/// ];
///
/// assert_eq!(select._type, SelectType::Single);
/// ```
#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct SelectElement {
  /// Element name attribute.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<Cow<'static, str>>,
  /// Select type (single or multiple).
  #[serde(rename = "type")]
  #[builder(default = "SelectType::Single")]
  pub _type: SelectType,
  /// Current value.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub value: Option<Value>,
  /// Label text.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub label: Option<String>,
  /// Available options.
  #[serde(default)]
  #[builder(default)]
  pub options: Vec<SelectOption>,
  /// Additional HTML attributes.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub attributes: Option<Attributes>,
  /// Whether the field is required.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub required: Option<bool>,
  /// Whether the field is disabled.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub disabled: Option<bool>,
  /// Validation field configuration.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub field: Option<Field<Value>>,
}
impl SelectElement {
  /// Creates a new SelectElement with the given name.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::SelectElement;
  ///
  /// let select = SelectElement::new("category");
  /// assert_eq!(select.name.as_deref(), Some("category"));
  /// ```
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: Some(Cow::Owned(name.into())),
      ..Default::default()
    }
  }
  /// Creates a multi-select element.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{SelectElement, SelectType};
  ///
  /// let select = SelectElement::multiple("tags");
  /// assert_eq!(select._type, SelectType::Multiple);
  /// ```
  pub fn multiple(name: impl Into<String>) -> Self {
    Self {
      name: Some(Cow::Owned(name.into())),
      _type: SelectType::Multiple,
      ..Default::default()
    }
  }
  /// Validates the given value against the field configuration.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::SelectElement;
  /// use walrs_validation::Value;
  ///
  /// let select = SelectElement::new("test");
  /// assert!(select.validate_value(&Value::from("option1")).is_ok());
  /// ```
  pub fn validate_value(&self, value: &Value) -> Result<(), Violations> {
    if let Some(ref field) = self.field {
      field.validate(value)
    } else {
      Ok(())
    }
  }
  /// Adds an option to the select element.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{SelectElement, SelectOption};
  ///
  /// let mut select = SelectElement::new("size");
  /// select.add_option(SelectOption::new("sm", "Small"));
  /// select.add_option(SelectOption::new("lg", "Large"));
  /// assert_eq!(select.options.len(), 2);
  /// ```
  pub fn add_option(&mut self, option: SelectOption) -> &mut Self {
    self.options.push(option);
    self
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_new() {
    let select = SelectElement::new("country");
    assert_eq!(select.name.as_deref(), Some("country"));
    assert_eq!(select._type, SelectType::Single);
  }
  #[test]
  fn test_multiple() {
    let select = SelectElement::multiple("tags");
    assert_eq!(select._type, SelectType::Multiple);
  }
  #[test]
  fn test_add_option() {
    let mut select = SelectElement::new("country");
    select.add_option(SelectOption::new("us", "United States"));
    assert_eq!(select.options.len(), 1);
  }
  #[test]
  fn test_builder() {
    let select = SelectElementBuilder::default()
      .name("priority")
      .required(true)
      .build()
      .unwrap();
    assert_eq!(select.name.as_deref(), Some("priority"));
    assert_eq!(select.required, Some(true));
  }
  #[test]
  fn test_validate_without_field() {
    let select = SelectElement::new("test");
    assert!(select.validate_value(&Value::from("value")).is_ok());
  }
}
