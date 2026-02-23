//! Field validation configuration.
//!
//! This module provides the `Field<T>` struct for defining validation and filtering
//! rules for a single form field. It replaces the old `Input`/`RefInput` API with
//! a unified, serializable design.

use crate::filter_enum::Filter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use walrs_validation::Value;
use walrs_validation::{Rule, ValidateRef, Violations};

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
/// use walrs_validation::Rule;
///
/// // Simple field with just a rule (no filters)
/// let field = FieldBuilder::<String>::default()
///     .name("username")
///     .rule(Rule::Required)
///     .build()
///     .unwrap();
///
/// // Field with rule and filters
/// let field = FieldBuilder::<String>::default()
///     .name("email")
///     .rule(Rule::Required.and(Rule::Email(Default::default())))
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
  pub name: Option<Cow<'static, str>>,

  /// Optional locale for localized error messages.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub locale: Option<Cow<'static, str>>,

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

  /// Validate the value against the rule, short-circuiting on the first violation.
  ///
  /// Returns `Ok(())` if the rule passes, or `Err(Violations)` with the first failure.
  /// If the field has a locale set, it is applied to the rule for internationalized
  /// error messages.
  /// Whether the calling context stops processing further fields on failure is
  /// controlled by the `break_on_failure` flag (used by `FieldFilter`).
  pub fn validate(&self, value: &String) -> Result<(), Violations> {
    match &self.rule {
      Some(rule) => {
        // Apply locale to rule if set, then validate via trait method
        // @todo `locale` should be set directly on `rule`.
        let result = if let Some(locale) = &self.locale {
          rule.clone().with_locale(locale.as_ref()).validate_ref(value.as_str())
        } else {
          rule.validate_ref(value.as_str())
        };
        result.map_err(|v| {
          let mut violations = Violations::empty();
          violations.push(v);
          violations
        })
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
  /// Delegates to the full `Rule<Value>::validate_value()` implementation.
  pub fn validate(&self, value: &Value) -> Result<(), Violations> {
    match &self.rule {
      Some(rule) => rule.validate_value(value).map_err(|v| {
        let mut vs = Violations::empty();
        vs.push(v);
        vs
      }),
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
  use walrs_validation::Rule;

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
      .name("email")
      .rule(Rule::Required.and(Rule::MinLength(5)))
      .filters(vec![Filter::Trim])
      .build()
      .unwrap();

    assert_eq!(field.name.as_deref(), Some("email"));
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

    let result = field.filter(Value::Str("  HELLO  ".to_string()));
    assert_eq!(result, Value::Str("hello".to_string()));
  }

  #[test]
  fn test_value_field_required() {
    let field = FieldBuilder::<Value>::default()
      .rule(Rule::Required)
      .build()
      .unwrap();

    assert!(field.validate(&Value::Null).is_err());
    assert!(field.validate(&Value::Str("".to_string())).is_err());
    assert!(field.validate(&Value::Str("hello".to_string())).is_ok());
  }

  #[test]
  fn test_break_on_failure() {
    // `break_on_failure` signals the FieldFilter to stop processing further
    // fields when this field fails; the `validate` method itself always
    // short-circuits on the first encountered violation.
    let field = FieldBuilder::<String>::default()
      .rule(
        Rule::Required
          .and(Rule::MinLength(5))
          .and(Rule::MaxLength(10)),
      )
      .break_on_failure(true)
      .build()
      .unwrap();

    // Empty string fails on the first encountered violation
    let result = field.validate(&"".to_string());
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1); // Always returns the first violation only
  }

  #[test]
  fn test_field_serialization() {
    let field = FieldBuilder::<String>::default()
      .name("username")
      .rule(Rule::Required)
      .build()
      .unwrap();

    let json = serde_json::to_string(&field).unwrap();
    assert!(json.contains("username"));
    assert!(json.contains("required")); // lowercase due to serde rename_all
  }
}
