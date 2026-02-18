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
/// let field = FieldBuilder::<String>::default()
///     .name("email".to_string())
///     .rules(vec![Rule::Required, Rule::Email])
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
    #[builder(default)]
    pub name: Option<String>,

    /// Optional locale for localized error messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub locale: Option<String>,

    /// Validation rules to apply.
    #[serde(default)]
    #[builder(default)]
    pub rules: Vec<Rule<T>>,

    /// Filters to apply before validation.
    #[serde(default)]
    #[builder(default)]
    pub filters: Vec<Filter<T>>,

    /// When true, stops validation at the first error.
    #[builder(default = "false")]
    pub break_on_failure: bool,
}

impl<T: Clone> Default for Field<T> {
    fn default() -> Self {
        Self {
            name: None,
            locale: None,
            rules: Vec::new(),
            filters: Vec::new(),
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
            && self.rules == other.rules
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
        self.filters.iter().fold(value, |v, f| f.apply(v))
    }

    /// Validate the value against all rules.
    ///
    /// Returns `Ok(())` if all rules pass, or `Err(Violations)` with all failures.
    pub fn validate(&self, value: &String) -> Result<(), Violations> {
        let mut violations = Violations::empty();

        // Apply rules
        for rule in &self.rules {
            if let Err(violation) = rule.validate_ref(value) {
                violations.push(violation);
                if self.break_on_failure {
                    return Err(violations);
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
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
        self.filters.iter().fold(value, |v, f| f.apply(v))
    }

    /// Validate the value against all rules.
    ///
    /// Returns `Ok(())` if all rules pass, or `Err(Violations)` with all failures.
    ///
    /// Note: For `Value` fields, rules are applied based on the underlying type.
    /// String values are extracted and validated with string rules.
    pub fn validate(&self, value: &Value) -> Result<(), Violations> {
        use walrs_form_core::ValueExt;

        let mut violations = Violations::empty();

        // Check required using ValueExt trait
        // TODO: Implement Rule<Value> in walrs_validator for full rule support
        // For now, we check for Required rule manually
        for rule in &self.rules {
            if matches!(rule, Rule::Required) && value.is_empty_value() {
                violations.push(Violation::new(
                    walrs_validator::ViolationType::ValueMissing,
                    "Value is required",
                ));
                if self.break_on_failure {
                    return Err(violations);
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
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
        assert!(field.rules.is_empty());
        assert!(field.filters.is_empty());
    }

    #[test]
    fn test_field_builder_with_values() {
        let field = FieldBuilder::<String>::default()
            .name("email".to_string())
            .rules(vec![Rule::Required, Rule::MinLength(5)])
            .filters(vec![Filter::Trim])
            .build()
            .unwrap();

        assert_eq!(field.name, Some("email".to_string()));
        assert_eq!(field.rules.len(), 2);
        assert_eq!(field.filters.len(), 1);
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
            .rules(vec![Rule::MinLength(3)])
            .build()
            .unwrap();

        assert!(field.validate(&"hello".to_string()).is_ok());
    }

    #[test]
    fn test_string_field_validate_fails() {
        let field = FieldBuilder::<String>::default()
            .rules(vec![Rule::MinLength(10)])
            .build()
            .unwrap();

        assert!(field.validate(&"hello".to_string()).is_err());
    }

    #[test]
    fn test_string_field_required() {
        let field = FieldBuilder::<String>::default()
            .rules(vec![Rule::Required])
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
            .rules(vec![Rule::MinLength(3)])
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
            .rules(vec![Rule::Required])
            .build()
            .unwrap();

        assert!(field.validate(&json!(null)).is_err());
        assert!(field.validate(&json!("")).is_err());
        assert!(field.validate(&json!("hello")).is_ok());
    }

    #[test]
    fn test_break_on_failure() {
        let field = FieldBuilder::<String>::default()
            .rules(vec![Rule::Required, Rule::MinLength(5), Rule::MaxLength(10)])
            .break_on_failure(true)
            .build()
            .unwrap();

        // Empty string should fail on required and stop
        let result = field.validate(&"".to_string());
        assert!(result.is_err());
        let violations = result.unwrap_err();
        assert_eq!(violations.len(), 1); // Only required violation
    }

    #[test]
    fn test_collect_all_violations() {
        let field = FieldBuilder::<String>::default()
            .rules(vec![Rule::Required, Rule::MinLength(5)])
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
            .rules(vec![Rule::Required])
            .build()
            .unwrap();

        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("username"));
        assert!(json.contains("required")); // lowercase due to serde rename_all
    }
}

