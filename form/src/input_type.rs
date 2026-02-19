//! HTML5 input types.
use serde::{Deserialize, Serialize};
/// HTML5 input types.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
  #[default]
  Text,
  Email,
  Password,
  Number,
  Checkbox,
  Radio,
  File,
  Date,
  #[serde(rename = "datetime-local")]
  DateTime,
  Month,
  Week,
  Time,
  Tel,
  Url,
  Color,
  Range,
  Search,
  Hidden,
}
impl std::fmt::Display for InputType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      InputType::Text => "text",
      InputType::Email => "email",
      InputType::Password => "password",
      InputType::Number => "number",
      InputType::Checkbox => "checkbox",
      InputType::Radio => "radio",
      InputType::File => "file",
      InputType::Date => "date",
      InputType::DateTime => "datetime-local",
      InputType::Month => "month",
      InputType::Week => "week",
      InputType::Time => "time",
      InputType::Tel => "tel",
      InputType::Url => "url",
      InputType::Color => "color",
      InputType::Range => "range",
      InputType::Search => "search",
      InputType::Hidden => "hidden",
    };
    write!(f, "{}", s)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_serialization() {
    let input_type = InputType::Email;
    let json = serde_json::to_string(&input_type).unwrap();
    assert_eq!(json, r#""email""#);
  }
  #[test]
  fn test_display() {
    assert_eq!(InputType::Email.to_string(), "email");
  }
}
