//! Multi-field validation with cross-field rules.
//!
//! This module provides `FieldFilter` for validating multiple fields at once,
//! including support for cross-field validation rules like password confirmation,
//! conditional requirements, and mutual exclusivity.

use crate::field::Field;
use crate::form_violations::FormViolations;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::Arc;
use walrs_validation::{Value, ValueExt};
use walrs_validation::{Condition, RuleResult, Violation, ViolationType};

/// Multi-field validation configuration.
///
/// `FieldFilter` validates multiple fields and can enforce cross-field
/// validation rules like password confirmation, conditional requirements,
/// and mutual exclusivity.
///
/// # Example
///
/// ```rust
/// use walrs_inputfilter::field_filter::{FieldFilter, CrossFieldRule, CrossFieldRuleType};
/// use walrs_inputfilter::field::FieldBuilder;
/// use walrs_validation::Value;
/// use walrs_validation::Rule;
/// use serde_json::json;
/// use std::collections::HashMap;
///
/// let mut field_filter = FieldFilter::new();
///
/// // Fluent API - chain add_field and add_cross_field_rule calls
/// field_filter
///     .add_field("email", FieldBuilder::<Value>::default()
///         .rule(Rule::Required)
///         .build()
///         .unwrap())
///     .add_cross_field_rule(CrossFieldRule {
///         name: Some("password_match".into()),
///         fields: vec!["password".to_string(), "password_confirm".to_string()],
///         rule: CrossFieldRuleType::FieldsEqual {
///             field_a: "password".to_string(),
///             field_b: "password_confirm".to_string(),
///         },
///     });
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FieldFilter {
  /// Field definitions keyed by name.
  #[serde(default)]
  pub fields: HashMap<String, Field<Value>>,

  /// Cross-field validation rules.
  #[serde(default)]
  pub cross_field_rules: Vec<CrossFieldRule>,
}

impl FieldFilter {
  /// Creates a new empty `FieldFilter`.
  pub fn new() -> Self {
    Self::default()
  }

  /// Adds a field definition.
  pub fn add_field<S: Into<String>>(&mut self, name: S, field: Field<Value>) -> &mut Self {
    self.fields.insert(name.into(), field);
    self
  }

  /// Removes a field definition.
  pub fn remove_field(&mut self, name: &str) -> Option<Field<Value>> {
    self.fields.remove(name)
  }

  /// Gets a field definition.
  pub fn get_field(&self, name: &str) -> Option<&Field<Value>> {
    self.fields.get(name)
  }

  /// Adds a cross-field validation rule.
  pub fn add_cross_field_rule(&mut self, rule: CrossFieldRule) -> &mut Self {
    self.cross_field_rules.push(rule);
    self
  }

  /// Validates form data against all fields and cross-field rules.
  ///
  /// Returns `Ok(())` if all validation passes, or `Err(FormViolations)` with
  /// the collected field- and form-level violations.
  ///
  /// If a field has `break_on_failure` set to `true` and fails validation,
  /// the method returns immediately with the violations collected so far,
  /// without checking the remaining fields or any cross-field rules. In that
  /// case, the returned `FormViolations` is a partial result and does not
  /// contain violations from fields or cross-field rules that were not
  /// evaluated before the early exit.
  ///
  /// **Note**: Because `fields` is a `HashMap`, iteration order is
  /// non-deterministic. When more than one field has `break_on_failure = true`,
  /// the "first" field that triggers an early return may vary between runs.
  /// For predictable early-exit behaviour, set `break_on_failure = true` on
  /// at most one field, or use a single field whose failure you wish to stop on.
  pub fn validate(&self, data: &HashMap<String, Value>) -> Result<(), FormViolations> {
    let mut violations = FormViolations::new();

    // Validate individual fields
    for (field_name, field) in &self.fields {
      let value = data.get(field_name).cloned().unwrap_or(Value::Null);
      if let Err(field_violations) = field.validate(&value) {
        violations.add_field_violations(field_name, field_violations);
        if field.break_on_failure {
          return Err(violations);
        }
      }
    }

    // Validate cross-field rules
    for rule in &self.cross_field_rules {
      if let Err(violation) = rule.evaluate(data) {
        violations.add_form_violation(violation);
      }
    }

    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Filters all field values in the data.
  ///
  /// Returns a new HashMap with filtered values.
  pub fn filter(&self, data: HashMap<String, Value>) -> HashMap<String, Value> {
    let mut result = data;
    for (field_name, field) in &self.fields {
      if let Some(value) = result.remove(field_name) {
        let filtered = field.filter(value);
        result.insert(field_name.clone(), filtered);
      }
    }
    result
  }

  /// Filters and then validates the data.
  /// @todo Consider adding a field, or a method param, that will allow to switch
  ///     the execution order:
  ///     - filtering first, then validation or
  ///     - vice-versa
  pub fn process(
    &self,
    data: HashMap<String, Value>,
  ) -> Result<HashMap<String, Value>, FormViolations> {
    let filtered = self.filter(data);
    self.validate(&filtered)?;
    Ok(filtered)
  }
}

/// Cross-field validation rule.
#[derive(Clone, Serialize, Deserialize)]
pub struct CrossFieldRule {
  /// Optional name for error messages.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<Cow<'static, str>>,

  /// Fields involved in this rule.
  pub fields: Vec<String>,

  /// The validation rule type.
  pub rule: CrossFieldRuleType,
}

impl Debug for CrossFieldRule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CrossFieldRule")
      .field("name", &self.name)
      .field("fields", &self.fields)
      .field("rule", &self.rule)
      .finish()
  }
}

impl CrossFieldRule {
  /// Evaluates the rule against the provided data.
  pub fn evaluate(&self, data: &HashMap<String, Value>) -> RuleResult {
    self.rule.evaluate(data, self.name.as_deref())
  }
}

/// Types of cross-field validation.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CrossFieldRuleType {
  /// Both fields must have equal values (e.g., password confirmation).
  FieldsEqual { field_a: String, field_b: String },

  /// Field is required if condition on another field is met.
  RequiredIf {
    field: String,
    condition: Condition<Value>,
  },

  /// Field is required unless condition on another field is met.
  RequiredUnless {
    field: String,
    condition: Condition<Value>,
  },

  /// At least one of the listed fields must have a value.
  OneOfRequired(Vec<String>),

  /// Only one of the listed fields can have a value.
  MutuallyExclusive(Vec<String>),

  /// If depends_on field has value, then field is required.
  DependentRequired { field: String, depends_on: String },

  /// Custom validation (not serializable).
  #[serde(skip)]
  Custom(Arc<dyn Fn(&HashMap<String, Value>) -> RuleResult + Send + Sync>),
}

impl Debug for CrossFieldRuleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::FieldsEqual { field_a, field_b } => f
        .debug_struct("FieldsEqual")
        .field("field_a", field_a)
        .field("field_b", field_b)
        .finish(),
      Self::RequiredIf { field, condition } => f
        .debug_struct("RequiredIf")
        .field("field", field)
        .field("condition", condition)
        .finish(),
      Self::RequiredUnless { field, condition } => f
        .debug_struct("RequiredUnless")
        .field("field", field)
        .field("condition", condition)
        .finish(),
      Self::OneOfRequired(fields) => f.debug_tuple("OneOfRequired").field(fields).finish(),
      Self::MutuallyExclusive(fields) => f.debug_tuple("MutuallyExclusive").field(fields).finish(),
      Self::DependentRequired { field, depends_on } => f
        .debug_struct("DependentRequired")
        .field("field", field)
        .field("depends_on", depends_on)
        .finish(),
      Self::Custom(_) => write!(f, "Custom(<fn>)"),
    }
  }
}

impl CrossFieldRuleType {
  /// Evaluates the rule against the provided data.
  pub fn evaluate(&self, data: &HashMap<String, Value>, rule_name: Option<&str>) -> RuleResult {
    match self {
      CrossFieldRuleType::FieldsEqual { field_a, field_b } => {
        let val_a = data.get(field_a);
        let val_b = data.get(field_b);
        if val_a == val_b {
          Ok(())
        } else {
          Err(Violation::new(
            ViolationType::NotEqual,
            format!(
              "{}: {} and {} must be equal",
              rule_name.unwrap_or("FieldsEqual"),
              field_a,
              field_b
            ),
          ))
        }
      }

      CrossFieldRuleType::RequiredIf { field, condition } => {
        let condition_met = data
          .get(field)
          .map(|v| evaluate_condition(condition, v))
          .unwrap_or(false);

        if condition_met {
          let has_value = data
            .get(field)
            .map(|v| !v.is_empty_value())
            .unwrap_or(false);

          if has_value {
            Ok(())
          } else {
            Err(Violation::new(
              ViolationType::ValueMissing,
              format!(
                "{}: {} is required when condition is met",
                rule_name.unwrap_or("RequiredIf"),
                field
              ),
            ))
          }
        } else {
          Ok(())
        }
      }

      CrossFieldRuleType::RequiredUnless { field, condition } => {
        let condition_met = data
          .get(field)
          .map(|v| evaluate_condition(condition, v))
          .unwrap_or(false);

        if !condition_met {
          let has_value = data
            .get(field)
            .map(|v| !v.is_empty_value())
            .unwrap_or(false);

          if has_value {
            Ok(())
          } else {
            Err(Violation::new(
              ViolationType::ValueMissing,
              format!(
                "{}: {} is required unless condition is met",
                rule_name.unwrap_or("RequiredUnless"),
                field
              ),
            ))
          }
        } else {
          Ok(())
        }
      }

      CrossFieldRuleType::OneOfRequired(fields) => {
        let has_any = fields
          .iter()
          .any(|f| data.get(f).map(|v| !v.is_empty_value()).unwrap_or(false));

        if has_any {
          Ok(())
        } else {
          Err(Violation::new(
            ViolationType::ValueMissing,
            format!(
              "{}: At least one of {} is required",
              rule_name.unwrap_or("OneOfRequired"),
              fields.join(", ")
            ),
          ))
        }
      }

      CrossFieldRuleType::MutuallyExclusive(fields) => {
        let filled_count = fields
          .iter()
          .filter(|f| data.get(*f).map(|v| !v.is_empty_value()).unwrap_or(false))
          .count();

        if filled_count <= 1 {
          Ok(())
        } else {
          Err(Violation::new(
            ViolationType::CustomError,
            format!(
              "{}: Only one of {} can have a value",
              rule_name.unwrap_or("MutuallyExclusive"),
              fields.join(", ")
            ),
          ))
        }
      }

      CrossFieldRuleType::DependentRequired { field, depends_on } => {
        let dependency_filled = data
          .get(depends_on)
          .map(|v| !v.is_empty_value())
          .unwrap_or(false);

        if dependency_filled {
          let field_filled = data
            .get(field)
            .map(|v| !v.is_empty_value())
            .unwrap_or(false);

          if field_filled {
            Ok(())
          } else {
            Err(Violation::new(
              ViolationType::ValueMissing,
              format!(
                "{}: {} is required when {} is provided",
                rule_name.unwrap_or("DependentRequired"),
                field,
                depends_on
              ),
            ))
          }
        } else {
          Ok(())
        }
      }

      CrossFieldRuleType::Custom(f) => f(data),
    }
  }
}

/// Helper to evaluate a Condition<Value> against a Value.
fn evaluate_condition(condition: &Condition<Value>, value: &Value) -> bool {
  match condition {
    Condition::IsEmpty => value.is_empty_value(),
    Condition::IsNotEmpty => !value.is_empty_value(),
    Condition::Equals(expected) => value == expected,
    Condition::GreaterThan(threshold) => {
      if let (Some(v), Some(t)) = (value.as_f64(), threshold.as_f64()) {
        v > t
      } else {
        false
      }
    }
    Condition::LessThan(threshold) => {
      if let (Some(v), Some(t)) = (value.as_f64(), threshold.as_f64()) {
        v < t
      } else {
        false
      }
    }
    Condition::Matches(pattern) => {
      if let Some(s) = value.as_str() {
        regex::Regex::new(pattern)
          .map(|re| re.is_match(s))
          .unwrap_or(false)
      } else {
        false
      }
    }
    Condition::Custom(f) => f(value),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::field::FieldBuilder;
  use serde_json::json;
  use walrs_validation::Rule;

  fn make_data(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs
      .iter()
      .map(|(k, v)| (k.to_string(), v.clone()))
      .collect()
  }

  #[test]
  fn test_field_filter_validate_single_field() {
    let mut filter = FieldFilter::new();
    filter.add_field(
      "email",
      FieldBuilder::<Value>::default()
        .rule(Rule::Required)
        .build()
        .unwrap(),
    );

    let data = make_data(&[("email", json!("test@example.com"))]);
    assert!(filter.validate(&data).is_ok());

    let empty_data = make_data(&[]);
    assert!(filter.validate(&empty_data).is_err());
  }

  #[test]
  fn test_fields_equal() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("password_match".into()),
      fields: vec!["password".to_string(), "password_confirm".to_string()],
      rule: CrossFieldRuleType::FieldsEqual {
        field_a: "password".to_string(),
        field_b: "password_confirm".to_string(),
      },
    });

    // Matching passwords
    let data = make_data(&[
      ("password", json!("secret123")),
      ("password_confirm", json!("secret123")),
    ]);
    assert!(filter.validate(&data).is_ok());

    // Non-matching passwords
    let data = make_data(&[
      ("password", json!("secret123")),
      ("password_confirm", json!("different")),
    ]);
    assert!(filter.validate(&data).is_err());
  }

  #[test]
  fn test_one_of_required() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("contact_required".into()),
      fields: vec!["email".to_string(), "phone".to_string()],
      rule: CrossFieldRuleType::OneOfRequired(vec!["email".to_string(), "phone".to_string()]),
    });

    // Neither present
    let data = make_data(&[]);
    assert!(filter.validate(&data).is_err());

    // Email present
    let data = make_data(&[("email", json!("test@example.com"))]);
    assert!(filter.validate(&data).is_ok());

    // Phone present
    let data = make_data(&[("phone", json!("123-456-7890"))]);
    assert!(filter.validate(&data).is_ok());
  }

  #[test]
  fn test_mutually_exclusive() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("payment_method".into()),
      fields: vec!["credit_card".to_string(), "paypal".to_string()],
      rule: CrossFieldRuleType::MutuallyExclusive(vec![
        "credit_card".to_string(),
        "paypal".to_string(),
      ]),
    });

    // Neither present - OK
    let data = make_data(&[]);
    assert!(filter.validate(&data).is_ok());

    // One present - OK
    let data = make_data(&[("credit_card", json!("1234-5678-9012-3456"))]);
    assert!(filter.validate(&data).is_ok());

    // Both present - Error
    let data = make_data(&[
      ("credit_card", json!("1234-5678-9012-3456")),
      ("paypal", json!("user@paypal.com")),
    ]);
    assert!(filter.validate(&data).is_err());
  }

  #[test]
  fn test_dependent_required() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("billing_address".into()),
      fields: vec!["billing_address".to_string(), "credit_card".to_string()],
      rule: CrossFieldRuleType::DependentRequired {
        field: "billing_address".to_string(),
        depends_on: "credit_card".to_string(),
      },
    });

    // No credit card, no billing address - OK
    let data = make_data(&[]);
    assert!(filter.validate(&data).is_ok());

    // Credit card with billing address - OK
    let data = make_data(&[
      ("credit_card", json!("1234-5678-9012-3456")),
      ("billing_address", json!("123 Main St")),
    ]);
    assert!(filter.validate(&data).is_ok());

    // Credit card without billing address - Error
    let data = make_data(&[("credit_card", json!("1234-5678-9012-3456"))]);
    assert!(filter.validate(&data).is_err());
  }

  #[test]
  fn test_filter_values() {
    let mut field_filter = FieldFilter::new();
    field_filter.add_field(
      "email",
      FieldBuilder::<Value>::default()
        .filters(vec![
          crate::filter_enum::Filter::Trim,
          crate::filter_enum::Filter::Lowercase,
        ])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("email", json!("  TEST@EXAMPLE.COM  "))]);
    let filtered = field_filter.filter(data);

    assert_eq!(filtered.get("email").unwrap(), &json!("test@example.com"));
  }

  #[test]
  fn test_process() {
    let mut field_filter = FieldFilter::new();
    field_filter.add_field(
      "email",
      FieldBuilder::<Value>::default()
        .rule(Rule::Required)
        .filters(vec![crate::filter_enum::Filter::Trim])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("email", json!("  test@example.com  "))]);
    let result = field_filter.process(data);
    assert!(result.is_ok());
    assert_eq!(
      result.unwrap().get("email").unwrap(),
      &json!("test@example.com")
    );
  }

  #[test]
  fn test_custom_cross_field_rule() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("custom_rule".into()),
      fields: vec!["age".to_string()],
      rule: CrossFieldRuleType::Custom(Arc::new(|data| {
        let age = data.get("age").and_then(|v| v.as_i64()).unwrap_or(0);
        if age >= 18 {
          Ok(())
        } else {
          Err(Violation::new(ViolationType::RangeUnderflow, "Must be 18+"))
        }
      })),
    });

    let data = make_data(&[("age", json!(21))]);
    assert!(filter.validate(&data).is_ok());

    let data = make_data(&[("age", json!(16))]);
    assert!(filter.validate(&data).is_err());
  }

  #[test]
  fn test_break_on_failure_stops_at_field_and_skips_cross_field_rules() {
    // A single field with break_on_failure = true paired with a cross-field rule
    // that would also fail. When the field fails, FieldFilter::validate must
    // return early without evaluating the cross-field rule.
    let mut filter = FieldFilter::new();
    filter.add_field(
      "email",
      FieldBuilder::<Value>::default()
        .rule(Rule::Required)
        .break_on_failure(true)
        .build()
        .unwrap(),
    );
    // Cross-field rule: password and confirm must be equal (they won't be).
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("password_match".into()),
      fields: vec!["password".to_string(), "password_confirm".to_string()],
      rule: CrossFieldRuleType::FieldsEqual {
        field_a: "password".to_string(),
        field_b: "password_confirm".to_string(),
      },
    });

    // email is missing → field fails with break_on_failure = true.
    // Cross-field rule must NOT be evaluated (no form-level violations).
    let data = make_data(&[
      ("password", json!("secret")),
      ("password_confirm", json!("different")),
    ]);
    let result = filter.validate(&data);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.for_field("email").is_some()); // field violation recorded
    assert!(violations.form.is_empty()); // cross-field rule was not reached

    // email is present → field passes, cross-field rule is evaluated and fails.
    let data = make_data(&[
      ("email", json!("test@example.com")),
      ("password", json!("secret")),
      ("password_confirm", json!("different")),
    ]);
    let result = filter.validate(&data);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.for_field("email").is_none()); // email passed
    assert!(!violations.form.is_empty()); // cross-field rule was evaluated
  }
}
