//! Select option element.
//!
//! This module provides [`SelectOption`] for representing `<option>` elements
//! within a `<select>`.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::SelectOption;
//!
//! let option = SelectOption::new("us", "United States");
//! assert_eq!(option.value, Some("us".to_string()));
//! assert_eq!(option.label, Some("United States".to_string()));
//! ```
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
/// HTML option element for select elements.
///
/// Represents an `<option>` within a `<select>` element. Can also represent
/// an `<optgroup>` when the `options` field contains nested options.
///
/// # Example
///
/// ```rust
/// use walrs_form::SelectOption;
///
/// // Simple option
/// let opt = SelectOption::new("value1", "Display Label");
///
/// // Option group
/// let group = SelectOption::optgroup("North America", vec![
///     SelectOption::new("us", "United States"),
///     SelectOption::new("ca", "Canada"),
/// ]);
/// assert!(group.options.is_some());
/// ```
#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct SelectOption {
    /// Option value attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    pub value: Option<String>,
    /// Display label for the option.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    pub label: Option<String>,
    /// Whether this option is selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    pub selected: Option<bool>,
    /// Whether this option is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    pub disabled: Option<bool>,
    /// Nested options (makes this an optgroup).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    pub options: Option<Vec<SelectOption>>,
}
impl SelectOption {
    /// Creates a new option with value and label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use walrs_form::SelectOption;
    ///
    /// let opt = SelectOption::new("red", "Red Color");
    /// assert_eq!(opt.value, Some("red".to_string()));
    /// ```
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: Some(value.into()),
            label: Some(label.into()),
            ..Default::default()
        }
    }
    /// Creates an optgroup with nested options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use walrs_form::SelectOption;
    ///
    /// let group = SelectOption::optgroup("Colors", vec![
    ///     SelectOption::new("red", "Red"),
    ///     SelectOption::new("blue", "Blue"),
    /// ]);
    /// assert_eq!(group.label, Some("Colors".to_string()));
    /// assert_eq!(group.options.as_ref().unwrap().len(), 2);
    /// ```
    pub fn optgroup(label: impl Into<String>, options: Vec<SelectOption>) -> Self {
        Self {
            label: Some(label.into()),
            options: Some(options),
            ..Default::default()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let opt = SelectOption::new("us", "United States");
        assert_eq!(opt.value, Some("us".to_string()));
        assert_eq!(opt.label, Some("United States".to_string()));
    }
    #[test]
    fn test_optgroup() {
        let group = SelectOption::optgroup("North America", vec![
            SelectOption::new("us", "United States"),
            SelectOption::new("ca", "Canada"),
        ]);
        assert_eq!(group.label, Some("North America".to_string()));
        assert!(group.options.is_some());
        assert_eq!(group.options.as_ref().unwrap().len(), 2);
    }
    #[test]
    fn test_builder() {
        let opt = SelectOptionBuilder::default()
            .value("test")
            .label("Test Label")
            .selected(true)
            .build()
            .unwrap();
        assert_eq!(opt.value, Some("test".to_string()));
        assert_eq!(opt.selected, Some(true));
    }
    #[test]
    fn test_serialization() {
        let opt = SelectOption::new("val", "Label");
        let json = serde_json::to_string(&opt).unwrap();
        assert!(json.contains("\"value\":\"val\""));
        assert!(json.contains("\"label\":\"Label\""));
    }
}
