//! Form element enum for polymorphic element handling.
//!
//! This module provides the [`Element`] enum which allows storing different
//! element types in a single collection.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::{Element, InputElement, InputType, ButtonElement, ButtonType};
//!
//! let elements: Vec<Element> = vec![
//!     InputElement::new("email", InputType::Email).into(),
//!     ButtonElement::new("submit", ButtonType::Submit).into(),
//! ];
//!
//! for el in &elements {
//!     println!("Element: {:?}", el.name());
//! }
//! ```
use crate::button_element::ButtonElement;
use crate::fieldset_element::FieldsetElement;
use crate::input_element::InputElement;
use crate::select_element::SelectElement;
use crate::textarea_element::TextareaElement;
use serde::{Deserialize, Serialize};
/// Form element types.
///
/// A tagged enum representing all supported form element types. Allows
/// pattern matching based on element type.
///
/// # Example
///
/// ```rust
/// use walrs_form::{Element, InputElement, InputType};
///
/// let element: Element = InputElement::new("email", InputType::Email).into();
///
/// match &element {
///     Element::Input(input) => {
///         println!("Input element: {:?}", input.name);
///     }
///     _ => {}
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "element")]
pub enum Element {
  /// Button element.
  Button(ButtonElement),
  /// Input element.
  Input(InputElement),
  /// Select element.
  Select(SelectElement),
  /// Textarea element.
  Textarea(TextareaElement),
  /// Fieldset element containing other elements.
  Fieldset(FieldsetElement),
}
impl Element {
  /// Returns the element name if available.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{Element, InputElement, InputType};
  ///
  /// let element: Element = InputElement::new("email", InputType::Email).into();
  /// assert_eq!(element.name(), Some("email"));
  /// ```
  pub fn name(&self) -> Option<&str> {
    match self {
      Element::Button(el) => el.name.as_deref(),
      Element::Input(el) => el.name.as_deref(),
      Element::Select(el) => el.name.as_deref(),
      Element::Textarea(el) => el.name.as_deref(),
      Element::Fieldset(el) => el.name.as_deref(),
    }
  }
  /// Returns true if the element is disabled.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{Element, InputElement, InputType};
  ///
  /// let mut input = InputElement::new("test", InputType::Text);
  /// input.disabled = Some(true);
  /// let element: Element = input.into();
  /// assert!(element.is_disabled());
  /// ```
  pub fn is_disabled(&self) -> bool {
    match self {
      Element::Button(el) => el.disabled.unwrap_or(false),
      Element::Input(el) => el.disabled.unwrap_or(false),
      Element::Select(el) => el.disabled.unwrap_or(false),
      Element::Textarea(el) => el.disabled.unwrap_or(false),
      Element::Fieldset(el) => el.disabled.unwrap_or(false),
    }
  }
  /// Returns true if the element is required.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_form::{Element, InputElement, InputType};
  ///
  /// let mut input = InputElement::new("test", InputType::Text);
  /// input.required = Some(true);
  /// let element: Element = input.into();
  /// assert!(element.is_required());
  /// ```
  pub fn is_required(&self) -> bool {
    match self {
      Element::Button(_) => false,
      Element::Input(el) => el.required.unwrap_or(false),
      Element::Select(el) => el.required.unwrap_or(false),
      Element::Textarea(el) => el.required.unwrap_or(false),
      Element::Fieldset(_) => false,
    }
  }
}
impl From<InputElement> for Element {
  fn from(el: InputElement) -> Self {
    Element::Input(el)
  }
}
impl From<SelectElement> for Element {
  fn from(el: SelectElement) -> Self {
    Element::Select(el)
  }
}
impl From<TextareaElement> for Element {
  fn from(el: TextareaElement) -> Self {
    Element::Textarea(el)
  }
}
impl From<ButtonElement> for Element {
  fn from(el: ButtonElement) -> Self {
    Element::Button(el)
  }
}
impl From<FieldsetElement> for Element {
  fn from(el: FieldsetElement) -> Self {
    Element::Fieldset(el)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use crate::button_type::ButtonType;
  use crate::input_type::InputType;
  #[test]
  fn test_from_input() {
    let input = InputElement::new("email", InputType::Email);
    let element: Element = input.into();
    assert_eq!(element.name(), Some("email"));
  }
  #[test]
  fn test_from_button() {
    let button = ButtonElement::new("submit", ButtonType::Submit);
    let element: Element = button.into();
    assert_eq!(element.name(), Some("submit"));
  }
  #[test]
  fn test_is_disabled() {
    let mut input = InputElement::new("test", InputType::Text);
    input.disabled = Some(true);
    let element: Element = input.into();
    assert!(element.is_disabled());
  }
  #[test]
  fn test_is_required() {
    let mut input = InputElement::new("test", InputType::Text);
    input.required = Some(true);
    let element: Element = input.into();
    assert!(element.is_required());
  }
  #[test]
  fn test_serialization() {
    let element: Element = InputElement::new("test", InputType::Text).into();
    let json = serde_json::to_string(&element).unwrap();
    assert!(json.contains("\"element\":\"Input\""));
  }
}
