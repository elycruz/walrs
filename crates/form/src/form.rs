//! HTML form.
#![allow(deprecated)]

use crate::element::Element;
use crate::form_data::FormData;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use walrs_fieldfilter::FieldFilter;
#[cfg(feature = "async")]
use walrs_fieldfilter::IndexMap;
#[cfg(feature = "async")]
use walrs_validation::Value;
use walrs_validation::{Attributes, FieldsetViolations};
/// HTTP form method.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormMethod {
  #[default]
  #[serde(rename = "GET")]
  Get,
  #[serde(rename = "POST")]
  Post,
}
/// Form encoding type.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormEnctype {
  #[default]
  #[serde(rename = "application/x-www-form-urlencoded")]
  UrlEncoded,
  #[serde(rename = "multipart/form-data")]
  MultipartFormData,
  #[serde(rename = "text/plain")]
  TextPlain,
}
/// HTML form.
#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct Form {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<Cow<'static, str>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub action: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub method: Option<FormMethod>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub enctype: Option<FormEnctype>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub elements: Option<Vec<Element>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub attributes: Option<Attributes>,
  #[serde(skip)]
  #[builder(default = "None")]
  pub field_filter: Option<FieldFilter>,
}
impl Form {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: Some(Cow::Owned(name.into())),
      ..Default::default()
    }
  }
  pub fn bind_data(&mut self, data: FormData) -> &mut Self {
    if let Some(ref mut elements) = self.elements {
      Self::bind_data_recursive(elements, &data);
    }
    self
  }
  fn bind_data_recursive(elements: &mut [Element], data: &FormData) {
    for element in elements.iter_mut() {
      match element {
        Element::Fieldset(fs) => {
          if let Some(ref mut children) = fs.elements {
            Self::bind_data_recursive(children, data);
          }
        }
        _ => {
          if let Some(name) = element.name()
            && let Some(value) = data.get(name)
          {
            Self::set_element_value(element, value.clone());
          }
        }
      }
    }
  }
  fn set_element_value(element: &mut Element, value: walrs_validation::Value) {
    match element {
      Element::Input(el) => el.value = Some(value),
      Element::Select(el) => el.value = Some(value),
      Element::Textarea(el) => {
        if let Some(s) = value.as_str() {
          el.value = Some(s.to_string());
        }
      }
      _ => {}
    }
  }
  pub fn validate(&self, data: &FormData) -> Result<(), FieldsetViolations> {
    if let Some(ref filter) = self.field_filter {
      filter.validate(data.as_inner())
    } else {
      let mut violations = FieldsetViolations::new();
      if let Some(ref elements) = self.elements {
        Self::validate_recursive(elements, data, &mut violations);
      }
      if violations.is_empty() {
        Ok(())
      } else {
        Err(violations)
      }
    }
  }
  fn validate_recursive(
    elements: &[Element],
    data: &FormData,
    violations: &mut FieldsetViolations,
  ) {
    for element in elements {
      match element {
        Element::Fieldset(fs) => {
          if let Some(ref children) = fs.elements {
            Self::validate_recursive(children, data, violations);
          }
        }
        _ => {
          if let Some(name) = element.name() {
            let value = data
              .get(name)
              .cloned()
              .unwrap_or(walrs_validation::Value::Null);
            let result = match element {
              Element::Input(el) => el.validate_value(&value),
              Element::Select(el) => el.validate_value(&value),
              Element::Textarea(el) => el.validate_value(&value),
              _ => Ok(()),
            };
            if let Err(field_violations) = result {
              violations.add_many(name, field_violations);
            }
          }
        }
      }
    }
  }
  pub fn add_element(&mut self, element: Element) -> &mut Self {
    self.elements.get_or_insert_with(Vec::new).push(element);
    self
  }
  pub fn get_element(&self, name: &str) -> Option<&Element> {
    self
      .elements
      .as_ref()?
      .iter()
      .find(|el| el.name() == Some(name))
  }
  pub fn iter_elements(&self) -> impl Iterator<Item = &Element> {
    self.elements.iter().flatten()
  }
}

#[cfg(feature = "async")]
impl Form {
  /// Validates form data asynchronously.
  ///
  /// When a `field_filter` is set, delegates to
  /// [`FieldFilter::validate_async`]. Otherwise, walks the element tree
  /// and calls `validate_value_async` on each input, select, and textarea.
  pub async fn validate_async(&self, data: &FormData) -> Result<(), FieldsetViolations> {
    if let Some(ref filter) = self.field_filter {
      filter.validate_async(data.as_inner()).await
    } else {
      let mut violations = FieldsetViolations::new();
      if let Some(ref elements) = self.elements {
        Self::validate_recursive_async(elements, data, &mut violations).await;
      }
      if violations.is_empty() {
        Ok(())
      } else {
        Err(violations)
      }
    }
  }

  /// Cleans (filters + validates) form data asynchronously.
  ///
  /// When a `field_filter` is set, delegates to
  /// [`FieldFilter::clean_async`] (sync filter + async validate).
  /// Otherwise, validates asynchronously and returns the data.
  pub async fn clean_async(
    &self,
    data: &FormData,
  ) -> Result<IndexMap<String, Value>, FieldsetViolations> {
    if let Some(ref filter) = self.field_filter {
      filter.clean_async(data.as_inner().clone()).await
    } else {
      self.validate_async(data).await?;
      Ok(data.as_inner().clone())
    }
  }

  fn validate_recursive_async<'a>(
    elements: &'a [Element],
    data: &'a FormData,
    violations: &'a mut FieldsetViolations,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a>> {
    Box::pin(async move {
      for element in elements {
        match element {
          Element::Fieldset(fs) => {
            if let Some(ref children) = fs.elements {
              Self::validate_recursive_async(children, data, violations).await;
            }
          }
          _ => {
            if let Some(name) = element.name() {
              let value = data
                .get(name)
                .cloned()
                .unwrap_or(walrs_validation::Value::Null);
              let result = match element {
                Element::Input(el) => el.validate_value_async(&value).await,
                Element::Select(el) => el.validate_value_async(&value).await,
                Element::Textarea(el) => el.validate_value_async(&value).await,
                _ => Ok(()),
              };
              if let Err(field_violations) = result {
                violations.add_many(name, field_violations);
              }
            }
          }
        }
      }
    })
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use crate::fieldset_element::FieldsetElement;
  use crate::input_element::InputElement;
  use crate::input_type::InputType;
  use walrs_validation::Value;
  #[test]
  fn test_new() {
    let form = Form::new("login");
    assert_eq!(form.name.as_deref(), Some("login"));
  }
  #[test]
  fn test_add_element() {
    let mut form = Form::new("test");
    form.add_element(InputElement::new("email", InputType::Email).into());
    assert_eq!(form.elements.as_ref().unwrap().len(), 1);
  }
  #[test]
  fn test_bind_data() {
    let mut form = Form::new("test");
    form.add_element(InputElement::new("email", InputType::Email).into());
    let mut data = FormData::new();
    data.insert("email", Value::Str("test@example.com".to_string()));
    form.bind_data(data);
    if let Some(Element::Input(input)) = form.get_element("email") {
      assert_eq!(
        input.value.as_ref().unwrap().as_str(),
        Some("test@example.com")
      );
    } else {
      panic!("Element not found");
    }
  }
  #[test]
  fn test_bind_data_nested_fieldset() {
    let mut form = Form::new("test");
    let mut fieldset = FieldsetElement::new("personal");
    fieldset.add_element(InputElement::new("name", InputType::Text).into());
    fieldset.add_element(InputElement::new("age", InputType::Number).into());
    form.add_element(fieldset.into());
    let mut data = FormData::new();
    data.insert("name", Value::Str("Alice".to_string()));
    data.insert("age", Value::from(30i32));
    form.bind_data(data);
    // Verify nested elements were bound
    if let Some(Element::Fieldset(fs)) = form.elements.as_ref().and_then(|e| e.first()) {
      let children = fs.elements.as_ref().unwrap();
      if let Element::Input(input) = &children[0] {
        assert_eq!(input.value.as_ref().unwrap().as_str(), Some("Alice"));
      } else {
        panic!("Expected Input element");
      }
      if let Element::Input(input) = &children[1] {
        assert_eq!(input.value.as_ref().unwrap().as_i64(), Some(30));
      } else {
        panic!("Expected Input element");
      }
    } else {
      panic!("Expected Fieldset element");
    }
  }
  #[test]
  fn test_bind_data_deeply_nested_fieldset() {
    let mut form = Form::new("test");
    let mut inner_fs = FieldsetElement::new("inner");
    inner_fs.add_element(InputElement::new("deep_field", InputType::Text).into());
    let mut outer_fs = FieldsetElement::new("outer");
    outer_fs.add_element(inner_fs.into());
    form.add_element(outer_fs.into());
    let mut data = FormData::new();
    data.insert("deep_field", Value::Str("deep_value".to_string()));
    form.bind_data(data);
    if let Some(Element::Fieldset(outer)) = form.elements.as_ref().and_then(|e| e.first()) {
      if let Some(Element::Fieldset(inner)) = outer.elements.as_ref().and_then(|e| e.first()) {
        if let Some(Element::Input(input)) = inner.elements.as_ref().and_then(|e| e.first()) {
          assert_eq!(input.value.as_ref().unwrap().as_str(), Some("deep_value"));
        } else {
          panic!("Expected Input element");
        }
      } else {
        panic!("Expected inner Fieldset");
      }
    } else {
      panic!("Expected outer Fieldset");
    }
  }
  #[test]
  fn test_validate_nested_fieldset() {
    use walrs_fieldfilter::FieldBuilder;
    use walrs_validation::Rule;
    let mut form = Form::new("test");
    let mut email_input = InputElement::new("email", InputType::Email);
    email_input.field = Some(
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );
    let mut fieldset = FieldsetElement::new("contact");
    fieldset.add_element(email_input.into());
    form.add_element(fieldset.into());
    // Validate with empty data - should fail since email is required
    let data = FormData::new();
    let result = form.validate(&data);
    assert!(result.is_err());
    // Validate with valid data
    let mut data = FormData::new();
    data.insert("email", Value::Str("test@example.com".to_string()));
    let result = form.validate(&data);
    assert!(result.is_ok());
  }
}

#[cfg(test)]
#[cfg(feature = "async")]
mod async_tests {
  use super::*;
  use crate::fieldset_element::FieldsetElement;
  use crate::form_data::FormData;
  use crate::input_element::InputElement;
  use crate::input_type::InputType;
  use crate::select_element::SelectElement;
  use crate::textarea_element::TextareaElement;
  use walrs_fieldfilter::{FieldBuilder, FieldFilter};
  use walrs_validation::{Rule, Value};

  #[tokio::test]
  async fn test_validate_async_passes() {
    let mut form = Form::new("test");
    let mut input = InputElement::new("name", InputType::Text);
    input.field = Some(
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );
    form.add_element(input.into());

    let mut data = FormData::new();
    data.insert("name", Value::Str("Alice".to_string()));
    let result = form.validate_async(&data).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_validate_async_fails() {
    let mut form = Form::new("test");
    let mut input = InputElement::new("name", InputType::Text);
    input.field = Some(
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );
    form.add_element(input.into());

    let data = FormData::new();
    let result = form.validate_async(&data).await;
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_validate_async_nested_fieldset() {
    let mut form = Form::new("test");
    let mut email_input = InputElement::new("email", InputType::Email);
    email_input.field = Some(
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );
    let mut fieldset = FieldsetElement::new("contact");
    fieldset.add_element(email_input.into());
    form.add_element(fieldset.into());

    // Empty data - should fail
    let data = FormData::new();
    let result = form.validate_async(&data).await;
    assert!(result.is_err());

    // Valid data
    let mut data = FormData::new();
    data.insert("email", Value::Str("test@example.com".to_string()));
    let result = form.validate_async(&data).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_validate_async_with_field_filter() {
    let mut filter = FieldFilter::default();
    filter.fields.insert(
      "email".to_string(),
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );

    let mut form = Form::new("test");
    form.field_filter = Some(filter);

    // Empty data - should fail
    let data = FormData::new();
    let result = form.validate_async(&data).await;
    assert!(result.is_err());

    // Valid data
    let mut data = FormData::new();
    data.insert("email", Value::Str("test@example.com".to_string()));
    let result = form.validate_async(&data).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_clean_async() {
    let mut form = Form::new("test");
    let mut input = InputElement::new("name", InputType::Text);
    input.field = Some(
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );
    form.add_element(input.into());

    // Invalid - should fail
    let data = FormData::new();
    let result = form.clean_async(&data).await;
    assert!(result.is_err());

    // Valid - should return data
    let mut data = FormData::new();
    data.insert("name", Value::Str("Alice".to_string()));
    let result = form.clean_async(&data).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert_eq!(processed.get("name").unwrap().as_str(), Some("Alice"));
  }

  #[tokio::test]
  async fn test_clean_async_with_field_filter() {
    let mut filter = FieldFilter::default();
    filter.fields.insert(
      "name".to_string(),
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );

    let mut form = Form::new("test");
    form.field_filter = Some(filter);

    // Invalid - should fail
    let data = FormData::new();
    let result = form.clean_async(&data).await;
    assert!(result.is_err());

    // Valid - should return filtered data
    let mut data = FormData::new();
    data.insert("name", Value::Str("Bob".to_string()));
    let result = form.clean_async(&data).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert_eq!(processed.get("name").unwrap().as_str(), Some("Bob"));
  }

  #[tokio::test]
  async fn test_validate_async_select_element() {
    let mut form = Form::new("test");
    let mut select = SelectElement::new("color");
    select.field = Some(
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );
    form.add_element(select.into());

    let data = FormData::new();
    let result = form.validate_async(&data).await;
    assert!(result.is_err());

    let mut data = FormData::new();
    data.insert("color", Value::Str("red".to_string()));
    let result = form.validate_async(&data).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_validate_async_textarea_element() {
    let mut form = Form::new("test");
    let mut textarea = TextareaElement::new("bio");
    textarea.field = Some(
      FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
    );
    form.add_element(textarea.into());

    let data = FormData::new();
    let result = form.validate_async(&data).await;
    assert!(result.is_err());

    let mut data = FormData::new();
    data.insert("bio", Value::Str("Hello world".to_string()));
    let result = form.validate_async(&data).await;
    assert!(result.is_ok());
  }
}
