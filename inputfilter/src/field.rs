//! Field validation configuration.
//!
//! This module provides the `Field<T>` struct for defining validation and filtering
//! rules for a single form field. It replaces the old `Input`/`RefInput` API with
//! a unified, serializable design.

use crate::filter_enum::Filter;
use serde::{Deserialize, Serialize};
use walrs_form_core::Value;
use walrs_validator::{Rule, Violation, Violations};

/// Validation configuration for a single field.
///
/// `Field<T>` provides a unified API for field validation and filtering,
/// replacing the old `Input`/`RefInput` split. It supports:
///
/// - Rule-based validation using the `Rule<T>` enum
/// - Filter-based transformation using the `Filter<T>` enum
/// - Builder pattern via `FieldBuilder`
/// - JSON/YAML serialization for config-driven forms
///
/// # Example
///
/// ```rust
/// use walrs_inputfilter::field::{Field, FieldBuilder};
/// use walrs_inputfilter::filter_enum::Filter;
/// use walrs_validator::Rule;
///
/// // Simple field with just a rule (no filters)
/// let field = FieldBuilder::<String>::default()
///     .name("username".to_string())
///     .rule(Rule::Required)
///     .build()
///     .unwrap();
///
/// // Field with rule and filters
/// let field = FieldBuilder::<String>::default()
///     .name("email".to_string())
///     .rule(Rule::Required.and(Rule::Email))
///     .filters(vec![Filter::Trim, Filter::Lowercase])
///     .build()
///     .unwrap();
///
/// // Filter then validate
/// let value = "  TEST@EXAMPLE.COM  ".to_string();
/// let filtered = field.filter(value);
/// assert_eq!(filtered, "test@example.com");
/// assert!(field.validate(&filtered).is_ok());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option), default)]
pub struct Field<T>
where
  T: Clone,
{
  /// Optional field name for error reporting.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<String>,

  /// Optional locale for localized error messages.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub locale: Option<String>,

  /// Validation rule to apply. Use `Rule::All` for multiple rules.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub rule: Option<Rule<T>>,

  /// Filters to apply before validation. Use `Filter::Chain` for multiple filters.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub filters: Option<Vec<Filter<T>>>,

  /// When true, stops validation at the first error.
  #[builder(default = "false")]
  pub break_on_failure: bool,
}

impl<T: Clone> Default for Field<T> {
  fn default() -> Self {
    Self {
      name: None,
      locale: None,
      rule: None,
      filters: None,
      break_on_failure: false,
    }
  }
}

impl<T: Clone + PartialEq> PartialEq for Field<T>
where
  Rule<T>: PartialEq,
  Filter<T>: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
      && self.locale == other.locale
      && self.rule == other.rule
      && self.filters == other.filters
      && self.break_on_failure == other.break_on_failure
  }
}

// ============================================================================
// String Field Implementation
// ============================================================================

impl Field<String> {
  /// Apply all filters to the value sequentially.
  pub fn filter(&self, value: String) -> String {
    match &self.filters {
      Some(filters) => filters.iter().fold(value, |v, f| f.apply(v)),
      None => value,
    }
  }

  /// Validate the value against the rule.
  ///
  /// Returns `Ok(())` if the rule passes, or `Err(Violations)` with failures.
  /// Uses the field's locale for internationalized error messages.
  pub fn validate(&self, value: &String) -> Result<(), Violations> {
    match &self.rule {
      Some(rule) => {
        let locale = self.locale.as_deref();
        if self.break_on_failure {
          // Return on first error
          rule.validate_ref(value, locale).map_err(|v| {
            let mut violations = Violations::empty();
            violations.push(v);
            violations
          })
        } else {
          // Collect all violations
          rule.validate_ref_all(value, locale)
        }
      }
      None => Ok(()),
    }
  }

  /// Filter the value and then validate it.
  ///
  /// Returns `Ok(filtered_value)` if validation passes, or `Err(Violations)`.
  pub fn process(&self, value: String) -> Result<String, Violations> {
    let filtered = self.filter(value);
    self.validate(&filtered)?;
    Ok(filtered)
  }
}

// ============================================================================
// Value Field Implementation
// ============================================================================

impl Field<Value> {
  /// Apply all filters to the value sequentially.
  pub fn filter(&self, value: Value) -> Value {
    match &self.filters {
      Some(filters) => filters.iter().fold(value, |v, f| f.apply(v)),
      None => value,
    }
  }

  /// Validate the value against the rule.
  ///
  /// Returns `Ok(())` if the rule passes, or `Err(Violations)` with failures.
  ///
  /// Note: For `Value` fields, rules are applied based on the underlying type.
  /// Currently supports `Rule::Required` check via `ValueExt::is_empty_value()`.
  pub fn validate(&self, value: &Value) -> Result<(), Violations> {
    use walrs_form_core::ValueExt;

    match &self.rule {
      Some(rule) => {
        // Check if rule requires value (Required or All containing Required)
        if rule.requires_value() && value.is_empty_value() {
          let mut violations = Violations::empty();
          violations.push(Violation::new(
            walrs_validator::ViolationType::ValueMissing,
            "Value is required",
          ));
          return Err(violations);
        }
        // TODO: Implement full Rule<Value> validation in walrs_validator
        Ok(())
      }
      None => Ok(()),
    }
  }

  /// Filter the value and then validate it.
  ///
  /// Returns `Ok(filtered_value)` if validation passes, or `Err(Violations)`.
  pub fn process(&self, value: Value) -> Result<Value, Violations> {
    let filtered = self.filter(value);
    self.validate(&filtered)?;
    Ok(filtered)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use walrs_validator::Rule;

  #[test]
  fn test_field_builder_defaults() {
    let field = FieldBuilder::<String>::default().build().unwrap();
    assert_eq!(field.name, None);
    assert!(field.rule.is_none());
    assert!(field.filters.is_none());
  }

  #[test]
  fn test_field_builder_with_values() {
    let field = FieldBuilder::<String>::default()
      .name("email".to_string())
      .rule(Rule::Required.and(Rule::MinLength(5)))
      .filters(vec![Filter::Trim])
      .build()
      .unwrap();

    assert_eq!(field.name, Some("email".to_string()));
    assert!(field.rule.is_some());
    assert_eq!(field.filters.as_ref().map(|f| f.len()), Some(1));
  }

  #[test]
  fn test_string_field_filter() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![Filter::Trim, Filter::Lowercase])
      .build()
      .unwrap();

    let result = field.filter("  HELLO  ".to_string());
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_string_field_validate_passes() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::MinLength(3))
      .build()
      .unwrap();

    assert!(field.validate(&"hello".to_string()).is_ok());
  }

  #[test]
  fn test_string_field_validate_fails() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::MinLength(10))
      .build()
      .unwrap();

    assert!(field.validate(&"hello".to_string()).is_err());
  }

  #[test]
  fn test_string_field_required() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::Required)
      .build()
      .unwrap();

    assert!(field.validate(&"".to_string()).is_err());
    assert!(field.validate(&"   ".to_string()).is_err());
    assert!(field.validate(&"hello".to_string()).is_ok());
  }

  #[test]
  fn test_string_field_process() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![Filter::Trim])
      .rule(Rule::MinLength(3))
      .build()
      .unwrap();

    let result = field.process("  hello  ".to_string());
    assert_eq!(result.unwrap(), "hello");
  }

  #[test]
  fn test_value_field_filter() {
    let field = FieldBuilder::<Value>::default()
      .filters(vec![Filter::Trim, Filter::Lowercase])
      .build()
      .unwrap();

    let result = field.filter(json!("  HELLO  "));
    assert_eq!(result, json!("hello"));
  }

  #[test]
  fn test_value_field_required() {
    let field = FieldBuilder::<Value>::default()
      .rule(Rule::Required)
      .build()
      .unwrap();

    assert!(field.validate(&json!(null)).is_err());
    assert!(field.validate(&json!("")).is_err());
    assert!(field.validate(&json!("hello")).is_ok());
  }

  #[test]
  fn test_break_on_failure() {
    let field = FieldBuilder::<String>::default()
      .rule(
        Rule::Required
          .and(Rule::MinLength(5))
          .and(Rule::MaxLength(10)),
      )
      .break_on_failure(true)
      .build()
      .unwrap();

    // Empty string should fail on required and stop
    let result = field.validate(&"".to_string());
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1); // Only first violation
  }

  #[test]
  fn test_collect_all_violations() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::Required.and(Rule::MinLength(5)))
      .break_on_failure(false)
      .build()
      .unwrap();

    // Empty string should fail both required and min_length
    let result = field.validate(&"".to_string());
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 2);
  }

  #[test]
  fn test_field_serialization() {
    let field = FieldBuilder::<String>::default()
      .name("username".to_string())
      .rule(Rule::Required)
      .build()
      .unwrap();

    let json = serde_json::to_string(&field).unwrap();
    assert!(json.contains("username"));
    assert!(json.contains("required")); // lowercase due to serde rename_all
  }
}
