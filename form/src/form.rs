//! HTML form.
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use walrs_form_core::Attributes;
use walrs_inputfilter::{FieldFilter, FormViolations};
use crate::element::Element;
use crate::form_data::FormData;
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
    #[builder(default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub method: Option<FormMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub enctype: Option<FormEnctype>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub elements: Option<Vec<Element>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub attributes: Option<Attributes>,
    #[serde(skip)]
    #[builder(default)]
    pub field_filter: Option<FieldFilter>,
}
impl Form {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }
    pub fn bind_data(&mut self, data: FormData) {
        if let Some(ref mut elements) = self.elements {
            for element in elements.iter_mut() {
                if let Some(name) = element.name() {
                    if let Some(value) = data.get(name) {
                        Self::set_element_value(element, value.clone());
                    }
                }
            }
        }
    }
    fn set_element_value(element: &mut Element, value: walrs_form_core::Value) {
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
    pub fn validate(&self, data: &FormData) -> Result<(), FormViolations> {
        if let Some(ref filter) = self.field_filter {
            filter.validate(data.as_inner())
        } else {
            let mut violations = FormViolations::new();
            if let Some(ref elements) = self.elements {
                for element in elements {
                    if let Some(name) = element.name() {
                        let value = data.get(name).cloned().unwrap_or(walrs_form_core::Value::Null);
                        let result = match element {
                            Element::Input(el) => el.validate_value(&value),
                            Element::Select(el) => el.validate_value(&value),
                            Element::Textarea(el) => el.validate_value(&value),
                            _ => Ok(()),
                        };
                        if let Err(field_violations) = result {
                            violations.add_field_violations(name, field_violations);
                        }
                    }
                }
            }
            if violations.is_empty() {
                Ok(())
            } else {
                Err(violations)
            }
        }
    }
    pub fn add_element(&mut self, element: Element) {
        self.elements.get_or_insert_with(Vec::new).push(element);
    }
    pub fn get_element(&self, name: &str) -> Option<&Element> {
        self.elements.as_ref()?.iter().find(|el| el.name() == Some(name))
    }
    pub fn iter_elements(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter().flatten()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_element::InputElement;
    use crate::input_type::InputType;
    use serde_json::json;
    #[test]
    fn test_new() {
        let form = Form::new("login");
        assert_eq!(form.name, Some("login".to_string()));
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
        data.insert("email", json!("test@example.com"));
        form.bind_data(data);
        if let Some(Element::Input(input)) = form.get_element("email") {
            assert_eq!(input.value.as_ref().unwrap().as_str(), Some("test@example.com"));
        } else {
            panic!("Element not found");
        }
    }
}
