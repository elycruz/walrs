//! HTML attributes storage and rendering.
//!
//! This module provides the `Attributes` struct for storing and rendering
//! HTML attributes in a type-safe way.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTML attributes storage.
///
/// A wrapper around a HashMap that provides methods for managing HTML attributes
/// and rendering them as HTML attribute strings.
///
/// # Examples
///
/// ```
/// use walrs_validation::Attributes;
///
/// let mut attrs = Attributes::new();
/// attrs.insert("class", "form-control");
/// attrs.insert("id", "email");
///
/// assert_eq!(attrs.get("class"), Some(&"form-control".to_string()));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes(HashMap<String, String>);

impl Attributes {
  /// Creates a new empty Attributes instance.
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  /// Creates Attributes with the given capacity.
  pub fn with_capacity(capacity: usize) -> Self {
    Self(HashMap::with_capacity(capacity))
  }

  /// Inserts an attribute key-value pair.
  ///
  /// Returns the previous value if the key was already present.
  pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<String>
  where
    K: Into<String>,
    V: Into<String>,
  {
    self.0.insert(key.into(), value.into())
  }

  /// Gets a reference to the value for a key.
  pub fn get(&self, key: &str) -> Option<&String> {
    self.0.get(key)
  }

  /// Removes a key from the attributes.
  ///
  /// Returns the value if the key was present.
  pub fn remove(&mut self, key: &str) -> Option<String> {
    self.0.remove(key)
  }

  /// Checks if the attributes contain a key.
  pub fn contains_key(&self, key: &str) -> bool {
    self.0.contains_key(key)
  }

  /// Returns an iterator over the key-value pairs.
  pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
    self.0.iter()
  }

  /// Returns the number of attributes.
  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// Returns true if there are no attributes.
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// Clears all attributes.
  pub fn clear(&mut self) -> &mut Self {
    self.0.clear();
    self
  }

  /// Renders attributes as an HTML attribute string.
  ///
  /// Attribute values are properly escaped for HTML.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_validation::Attributes;
  ///
  /// let mut attrs = Attributes::new();
  /// attrs.insert("class", "form-control");
  /// attrs.insert("id", "email");
  ///
  /// let html = attrs.to_html();
  /// // Result is something like: `class="form-control" id="email"`
  /// // (order may vary due to HashMap)
  /// assert!(html.contains(r#"class="form-control""#));
  /// assert!(html.contains(r#"id="email""#));
  /// ```
  pub fn to_html(&self) -> String {
    let mut parts: Vec<String> = self
      .0
      .iter()
      .map(|(k, v)| {
        format!(
          r#"{}="{}""#,
          escape_html_attr_name(k),
          escape_html_attr_value(v)
        )
      })
      .collect();
    parts.sort(); // Sort for deterministic output
    parts.join(" ")
  }

  /// Merges another Attributes instance into this one.
  ///
  /// Existing keys will be overwritten.
  pub fn merge(&mut self, other: Attributes) -> &mut Self {
    self.0.extend(other.0);
    self
  }
}

impl From<HashMap<String, String>> for Attributes {
  fn from(map: HashMap<String, String>) -> Self {
    Self(map)
  }
}

impl From<Attributes> for HashMap<String, String> {
  fn from(attrs: Attributes) -> Self {
    attrs.0
  }
}

impl<const N: usize> From<[(&str, &str); N]> for Attributes {
  fn from(arr: [(&str, &str); N]) -> Self {
    let mut attrs = Self::with_capacity(N);
    for (k, v) in arr {
      attrs.insert(k, v);
    }
    attrs
  }
}

/// Escapes special characters in HTML attribute names.
fn escape_html_attr_name(s: &str) -> String {
  // Attribute names should be alphanumeric with hyphens/underscores
  // For safety, we'll just pass through valid characters
  s.chars()
    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ':')
    .collect()
}

/// Escapes special characters in HTML attribute values.
fn escape_html_attr_value(s: &str) -> String {
  s.chars()
    .map(|c| match c {
      '&' => "&amp;".to_string(),
      '"' => "&quot;".to_string(),
      '\'' => "&#x27;".to_string(),
      '<' => "&lt;".to_string(),
      '>' => "&gt;".to_string(),
      _ => c.to_string(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_attributes() {
    let attrs = Attributes::new();
    assert!(attrs.is_empty());
  }

  #[test]
  fn test_insert_and_get() {
    let mut attrs = Attributes::new();
    attrs.insert("class", "form-control");
    assert_eq!(attrs.get("class"), Some(&"form-control".to_string()));
  }

  #[test]
  fn test_remove() {
    let mut attrs = Attributes::new();
    attrs.insert("class", "form-control");
    let removed = attrs.remove("class");
    assert_eq!(removed, Some("form-control".to_string()));
    assert!(attrs.is_empty());
  }

  #[test]
  fn test_contains_key() {
    let mut attrs = Attributes::new();
    attrs.insert("id", "email");
    assert!(attrs.contains_key("id"));
    assert!(!attrs.contains_key("class"));
  }

  #[test]
  fn test_to_html() {
    let mut attrs = Attributes::new();
    attrs.insert("class", "form-control");
    attrs.insert("id", "email");
    let html = attrs.to_html();
    assert!(html.contains(r#"class="form-control""#));
    assert!(html.contains(r#"id="email""#));
  }

  #[test]
  fn test_to_html_escapes_values() {
    let mut attrs = Attributes::new();
    attrs.insert("data-value", r#"<script>"alert"</script>"#);
    let html = attrs.to_html();
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
  }

  #[test]
  fn test_from_array() {
    let attrs = Attributes::from([("class", "btn"), ("type", "submit")]);
    assert_eq!(attrs.get("class"), Some(&"btn".to_string()));
    assert_eq!(attrs.get("type"), Some(&"submit".to_string()));
  }

  #[test]
  fn test_merge() {
    let mut attrs1 = Attributes::from([("class", "btn")]);
    let attrs2 = Attributes::from([("id", "submit"), ("class", "btn-primary")]);
    attrs1.merge(attrs2);
    assert_eq!(attrs1.get("class"), Some(&"btn-primary".to_string()));
    assert_eq!(attrs1.get("id"), Some(&"submit".to_string()));
  }

  #[test]
  fn test_serialization() {
    let attrs = Attributes::from([("class", "form-control")]);
    let json = serde_json::to_string(&attrs).unwrap();
    let deserialized: Attributes = serde_json::from_str(&json).unwrap();
    assert_eq!(attrs, deserialized);
  }
}
