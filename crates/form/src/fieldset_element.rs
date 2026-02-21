//! Fieldset element for grouping form elements.
//!
//! This module provides the [`FieldsetElement`] struct for grouping related
//! form elements with an optional legend.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::{FieldsetElement, InputElement, InputType, Element};
//!
//! let mut fieldset = FieldsetElement::new("personal_info");
//! fieldset.legend = Some("Personal Information".to_string());
//! fieldset.add_element(InputElement::new("name", InputType::Text).into());
//!
//! assert_eq!(fieldset.legend, Some("Personal Information".to_string()));
//! ```
use crate::element::Element;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use walrs_validation::Attributes;
/// HTML fieldset element.
///
/// Groups related form elements together with an optional legend.
///
/// # Example
///
/// ```rust
/// use walrs_form::{FieldsetElement, InputElement, InputType, Element};
///
/// let mut fieldset = FieldsetElement::with_legend("Address");
/// fieldset.add_element(InputElement::new("street", InputType::Text).into());
/// fieldset.add_element(InputElement::new("city", InputType::Text).into());
///
/// assert_eq!(fieldset.iter_elements().count(), 2);
/// ```
#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct FieldsetElement {
  /// Fieldset name.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<Cow<'static, str>>,
  /// Legend text.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub legend: Option<String>,
  /// Whether the fieldset is disabled.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub disabled: Option<bool>,
  /// Child elements.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub elements: Option<Vec<Element>>,
  /// Additional HTML attributes.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub attributes: Option<Attributes>,
}
impl FieldsetElement {
  /// Creates a new fieldset with the given name.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::FieldsetElement;
  ///
  /// let fieldset = FieldsetElement::new("user_info");
  /// assert_eq!(fieldset.name.as_deref(), Some("user_info"));
  /// ```
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: Some(Cow::Owned(name.into())),
      ..Default::default()
    }
  }
  /// Creates a fieldset with a legend.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::FieldsetElement;
  ///
  /// let fieldset = FieldsetElement::with_legend("Contact Details");
  /// assert_eq!(fieldset.legend, Some("Contact Details".to_string()));
  /// ```
  pub fn with_legend(legend: impl Into<String>) -> Self {
    Self {
      legend: Some(legend.into()),
      ..Default::default()
    }
  }
  /// Adds an element to the fieldset.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{FieldsetElement, InputElement, InputType};
  ///
  /// let mut fieldset = FieldsetElement::new("test");
  /// fieldset.add_element(InputElement::new("field1", InputType::Text).into());
  /// assert_eq!(fieldset.elements.as_ref().unwrap().len(), 1);
  /// ```
  pub fn add_element(&mut self, element: Element) -> &mut Self {
    self.elements.get_or_insert_with(Vec::new).push(element);
    self
  }
  /// Returns an iterator over all elements.
  pub fn iter_elements(&self) -> impl Iterator<Item = &Element> {
    self.elements.iter().flatten()
  }
  /// Returns an iterator over all elements recursively, including nested fieldsets.
  pub fn iter_elements_recursive(&self) -> Box<dyn Iterator<Item = &Element> + '_> {
    Box::new(self.elements.iter().flatten().flat_map(|el| {
      let this_iter = std::iter::once(el);
      if let Element::Fieldset(fieldset) = el {
        Box::new(this_iter.chain(fieldset.iter_elements_recursive()))
          as Box<dyn Iterator<Item = &Element>>
      } else {
        Box::new(this_iter)
      }
    }))
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use crate::input_element::InputElement;
  use crate::input_type::InputType;
  #[test]
  fn test_new() {
    let fieldset = FieldsetElement::new("user_info");
    assert_eq!(fieldset.name.as_deref(), Some("user_info"));
  }
  #[test]
  fn test_with_legend() {
    let fieldset = FieldsetElement::with_legend("Personal Information");
    assert_eq!(fieldset.legend, Some("Personal Information".to_string()));
  }
  #[test]
  fn test_add_element() {
    let mut fieldset = FieldsetElement::new("test");
    fieldset.add_element(InputElement::new("email", InputType::Email).into());
    assert_eq!(fieldset.elements.as_ref().unwrap().len(), 1);
  }
  #[test]
  fn test_iter_elements() {
    let mut fieldset = FieldsetElement::new("test");
    fieldset.add_element(InputElement::new("a", InputType::Text).into());
    fieldset.add_element(InputElement::new("b", InputType::Text).into());
    assert_eq!(fieldset.iter_elements().count(), 2);
  }
  #[test]
  fn test_builder() {
    let fieldset = FieldsetElementBuilder::default()
      .name("contact")
      .legend("Contact Info")
      .disabled(true)
      .build()
      .unwrap();
    assert_eq!(fieldset.name.as_deref(), Some("contact"));
    assert_eq!(fieldset.legend, Some("Contact Info".to_string()));
    assert_eq!(fieldset.disabled, Some(true));
  }
}
