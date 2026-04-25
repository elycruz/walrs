//! Multi-field validation with cross-field rules.
//!
//! This module provides `FieldFilter` for validating multiple fields at once,
//! including support for cross-field validation rules like password confirmation,
//! conditional requirements, and mutual exclusivity.

use crate::field::Field;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::sync::Arc;
use walrs_validation::FieldsetViolations;
use walrs_validation::{Condition, RuleResult, Violation, ViolationType};
use walrs_validation::{Value, ValueExt};

/// Multi-field validation configuration.
///
/// `FieldFilter` validates multiple fields and can enforce cross-field
/// validation rules like password confirmation, conditional requirements,
/// and mutual exclusivity.
///
/// # Example
///
/// ```rust
/// use walrs_fieldfilter::field_filter::{FieldFilter, CrossFieldRule, CrossFieldRuleType};
/// use walrs_fieldfilter::field::FieldBuilder;
/// use walrs_validation::Value;
/// use walrs_validation::Rule;
/// use serde_json::json;
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
  /// Field definitions keyed by name (insertion-ordered).
  #[serde(default)]
  pub fields: IndexMap<String, Field<Value>>,

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
    self.fields.shift_remove(name)
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
  /// Returns `Ok(())` if all validation passes, or `Err(FieldsetViolations)` with
  /// the collected field- and form-level violations.
  ///
  /// **Missing fields are treated as `Value::Null`.**  If `data` does not
  /// contain a key for a configured field, the field is validated against
  /// `Value::Null`.  This means `Rule::Required` will report a violation for
  /// any missing field.
  ///
  /// If a field has `break_on_failure` set to `true` and fails validation,
  /// the method returns immediately with the violations collected so far,
  /// without checking the remaining fields or any cross-field rules. In that
  /// case, the returned `FieldsetViolations` is a partial result and does not
  /// contain violations from fields or cross-field rules that were not
  /// evaluated before the early exit.
  ///
  /// **Note**: Because `fields` is an `IndexMap`, iteration follows insertion
  /// order. When `break_on_failure = true`, the "first" field that triggers an
  /// early return is deterministic and corresponds to the order in which fields
  /// were added.
  pub fn validate(&self, data: &IndexMap<String, Value>) -> Result<(), FieldsetViolations> {
    let null = Value::Null;
    let mut violations = FieldsetViolations::new();

    // Validate individual fields.
    // Missing fields are treated as `Value::Null`, which causes `Rule::Required`
    // to report a violation.
    for (field_name, field) in &self.fields {
      let value = data.get(field_name).unwrap_or(&null);
      if let Err(field_violations) = field.validate_ref(value) {
        violations.add_many(field_name, field_violations);
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

  /// Filters all field values in the data (infallible filters only).
  ///
  /// Returns a new IndexMap with filtered values.
  pub fn filter(&self, data: IndexMap<String, Value>) -> IndexMap<String, Value> {
    let mut result = data;
    for (field_name, field) in &self.fields {
      if let Some(value) = result.get_mut(field_name) {
        let taken = std::mem::replace(value, Value::Null);
        *value = field.filter(taken);
      }
    }
    result
  }

  /// Filters all field values without consuming the data.
  ///
  /// Returns a new `IndexMap` containing only the fields that appear in both
  /// `data` and `self.fields`, with each value run through the field's
  /// infallible filters.  Fields present in `data` but not in `self.fields`
  /// are **not** included in the result.
  pub fn filter_ref(&self, data: &IndexMap<String, Value>) -> IndexMap<String, Value> {
    let mut result = IndexMap::with_capacity(self.fields.len());
    for (field_name, field) in &self.fields {
      if let Some(value) = data.get(field_name) {
        result.insert(field_name.clone(), field.filter_ref(value));
      }
    }
    result
  }

  /// Applies fallible filters to all field values in the data.
  ///
  /// Returns `Ok(data)` with filtered values, or `Err(FieldsetViolations)` if
  /// any fallible filter fails. Short-circuits on fields with `break_on_failure`.
  pub fn try_filter(
    &self,
    data: IndexMap<String, Value>,
  ) -> Result<IndexMap<String, Value>, FieldsetViolations> {
    let mut result = data;
    let mut violations = FieldsetViolations::new();

    for (field_name, field) in &self.fields {
      if let Some(slot) = result.get_mut(field_name) {
        let taken = std::mem::replace(slot, Value::Null);
        match field.try_filter(taken) {
          Ok(filtered) => {
            *slot = filtered;
          }
          Err(field_violations) => {
            violations.add_many(field_name, field_violations);
            if field.break_on_failure {
              return Err(violations);
            }
          }
        }
      }
    }

    if violations.is_empty() {
      Ok(result)
    } else {
      Err(violations)
    }
  }

  /// Filters (infallible + fallible) and then validates the data.
  pub fn clean(
    &self,
    data: IndexMap<String, Value>,
  ) -> Result<IndexMap<String, Value>, FieldsetViolations> {
    let filtered = self.filter(data);
    let filtered = self.try_filter(filtered)?;
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
  pub fn evaluate(&self, data: &IndexMap<String, Value>) -> RuleResult {
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
  ///
  /// `condition_field` is checked against `condition`; if the condition holds,
  /// `field` must have a non-empty value.
  RequiredIf {
    field: String,
    /// The field whose value is tested against `condition`. Defaults to `field`
    /// when empty, preserving backward-compatibility with data serialized before
    /// this field was introduced.
    #[serde(default)]
    condition_field: String,
    condition: Condition<Value>,
  },

  /// Field is required unless condition on another field is met.
  ///
  /// `condition_field` is checked against `condition`; if the condition does
  /// **not** hold, `field` must have a non-empty value.
  RequiredUnless {
    field: String,
    /// The field whose value is tested against `condition`. Defaults to `field`
    /// when empty, preserving backward-compatibility with data serialized before
    /// this field was introduced.
    #[serde(default)]
    condition_field: String,
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
  #[allow(clippy::type_complexity)]
  Custom(Arc<dyn Fn(&IndexMap<String, Value>) -> RuleResult + Send + Sync>),

  /// Async custom validation (not serializable).
  #[cfg(feature = "async")]
  #[serde(skip)]
  #[allow(clippy::type_complexity)]
  CustomAsync(
    Arc<
      dyn Fn(
          &IndexMap<String, Value>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RuleResult> + Send + '_>>
        + Send
        + Sync,
    >,
  ),
}

impl Debug for CrossFieldRuleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::FieldsEqual { field_a, field_b } => f
        .debug_struct("FieldsEqual")
        .field("field_a", field_a)
        .field("field_b", field_b)
        .finish(),
      Self::RequiredIf {
        field,
        condition_field,
        condition,
      } => f
        .debug_struct("RequiredIf")
        .field("field", field)
        .field("condition_field", condition_field)
        .field("condition", condition)
        .finish(),
      Self::RequiredUnless {
        field,
        condition_field,
        condition,
      } => f
        .debug_struct("RequiredUnless")
        .field("field", field)
        .field("condition_field", condition_field)
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
      #[cfg(feature = "async")]
      Self::CustomAsync(_) => write!(f, "CustomAsync(<async fn>)"),
    }
  }
}

impl CrossFieldRuleType {
  /// Evaluates the rule against the provided data.
  pub fn evaluate(&self, data: &IndexMap<String, Value>, rule_name: Option<&str>) -> RuleResult {
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

      CrossFieldRuleType::RequiredIf {
        field,
        condition_field,
        condition,
      } => {
        let cond_field = if condition_field.is_empty() {
          field
        } else {
          condition_field
        };
        let condition_met = data
          .get(cond_field)
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
                "{}: {} is required when condition is met on {}",
                rule_name.unwrap_or("RequiredIf"),
                field,
                cond_field
              ),
            ))
          }
        } else {
          Ok(())
        }
      }

      CrossFieldRuleType::RequiredUnless {
        field,
        condition_field,
        condition,
      } => {
        let cond_field = if condition_field.is_empty() {
          field
        } else {
          condition_field
        };
        let condition_met = data
          .get(cond_field)
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
                "{}: {} is required unless condition is met on {}",
                rule_name.unwrap_or("RequiredUnless"),
                field,
                cond_field
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

      // `CustomAsync` rules are silently skipped in sync context for
      // ecosystem consistency with `walrs_validation`'s `Rule::CustomAsync`.
      // Use `validate_async()` / `evaluate_async()` to execute async rules.
      #[cfg(feature = "async")]
      CrossFieldRuleType::CustomAsync(_) => Ok(()),
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
      value.partial_cmp(threshold) == Some(std::cmp::Ordering::Greater)
    }
    Condition::LessThan(threshold) => {
      value.partial_cmp(threshold) == Some(std::cmp::Ordering::Less)
    }
    Condition::Matches(cp) => {
      if let Some(s) = value.as_str() {
        cp.0.is_match(s)
      } else {
        false
      }
    }
    Condition::Custom(f) => f(value),
  }
}

// ============================================================================
// Async Cross-Field & FieldFilter Methods
// ============================================================================

#[cfg(feature = "async")]
impl CrossFieldRuleType {
  /// Evaluates the rule asynchronously.
  ///
  /// Sync rule types are evaluated inline; `CustomAsync` is awaited.
  pub async fn evaluate_async(
    &self,
    data: &IndexMap<String, Value>,
    rule_name: Option<&str>,
  ) -> RuleResult {
    match self {
      CrossFieldRuleType::CustomAsync(f) => f(data).await,
      // All other variants are sync — delegate
      other => other.evaluate(data, rule_name),
    }
  }
}

#[cfg(feature = "async")]
impl CrossFieldRule {
  /// Evaluates the rule asynchronously.
  pub async fn evaluate_async(&self, data: &IndexMap<String, Value>) -> RuleResult {
    self.rule.evaluate_async(data, self.name.as_deref()).await
  }
}

#[cfg(feature = "async")]
impl FieldFilter {
  /// Validates form data asynchronously against all fields and cross-field rules.
  ///
  /// Works like [`validate`](Self::validate) but supports `Rule::CustomAsync`
  /// in field rules and `CrossFieldRuleType::CustomAsync` in cross-field rules.
  pub async fn validate_async(
    &self,
    data: &IndexMap<String, Value>,
  ) -> Result<(), FieldsetViolations> {
    let mut violations = FieldsetViolations::new();

    // Validate individual fields (async)
    for (field_name, field) in &self.fields {
      let value = data.get(field_name).cloned().unwrap_or(Value::Null);
      if let Err(field_violations) = field.validate_ref_async(&value).await {
        violations.add_many(field_name, field_violations);
        if field.break_on_failure {
          return Err(violations);
        }
      }
    }

    // Validate cross-field rules (async)
    for rule in &self.cross_field_rules {
      if let Err(violation) = rule.evaluate_async(data).await {
        violations.add_form_violation(violation);
      }
    }

    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Filters (infallible + fallible) and then validates the data asynchronously.
  ///
  /// Filtering is synchronous (CPU-bound); validation is async.
  pub async fn clean_async(
    &self,
    data: IndexMap<String, Value>,
  ) -> Result<IndexMap<String, Value>, FieldsetViolations> {
    let filtered = self.filter(data);
    let filtered = self.try_filter(filtered)?;
    self.validate_async(&filtered).await?;
    Ok(filtered)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::field::FieldBuilder;
  use walrs_filter::FilterOp;
  use walrs_validation::Rule;

  fn make_data(pairs: &[(&str, Value)]) -> IndexMap<String, Value> {
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

    let data = make_data(&[("email", Value::Str("test@example.com".to_string()))]);
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
      ("password", Value::Str("secret123".to_string())),
      ("password_confirm", Value::Str("secret123".to_string())),
    ]);
    assert!(filter.validate(&data).is_ok());

    // Non-matching passwords
    let data = make_data(&[
      ("password", Value::Str("secret123".to_string())),
      ("password_confirm", Value::Str("different".to_string())),
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
    let data = make_data(&[("email", Value::Str("test@example.com".to_string()))]);
    assert!(filter.validate(&data).is_ok());

    // Phone present
    let data = make_data(&[("phone", Value::Str("123-456-7890".to_string()))]);
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
    let data = make_data(&[("credit_card", Value::Str("1234-5678-9012-3456".to_string()))]);
    assert!(filter.validate(&data).is_ok());

    // Both present - Error
    let data = make_data(&[
      ("credit_card", Value::Str("1234-5678-9012-3456".to_string())),
      ("paypal", Value::Str("user@paypal.com".to_string())),
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
      ("credit_card", Value::Str("1234-5678-9012-3456".to_string())),
      ("billing_address", Value::Str("123 Main St".to_string())),
    ]);
    assert!(filter.validate(&data).is_ok());

    // Credit card without billing address - Error
    let data = make_data(&[("credit_card", Value::Str("1234-5678-9012-3456".to_string()))]);
    assert!(filter.validate(&data).is_err());
  }

  #[test]
  fn test_filter_values() {
    let mut field_filter = FieldFilter::new();
    field_filter.add_field(
      "email",
      FieldBuilder::<Value>::default()
        .filters(vec![
          walrs_filter::FilterOp::Trim,
          walrs_filter::FilterOp::Lowercase,
        ])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("email", Value::Str("  TEST@EXAMPLE.COM  ".to_string()))]);
    let filtered = field_filter.filter(data);

    assert_eq!(
      filtered.get("email").unwrap(),
      &Value::Str("test@example.com".to_string())
    );
  }

  #[test]
  fn test_clean() {
    let mut field_filter = FieldFilter::new();
    field_filter.add_field(
      "email",
      FieldBuilder::<Value>::default()
        .rule(Rule::Required)
        .filters(vec![walrs_filter::FilterOp::Trim])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("email", Value::Str("  test@example.com  ".to_string()))]);
    let result = field_filter.clean(data);
    assert!(result.is_ok());
    assert_eq!(
      result.unwrap().get("email").unwrap(),
      &Value::Str("test@example.com".to_string())
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

    let data = make_data(&[("age", Value::I64(21))]);
    assert!(filter.validate(&data).is_ok());

    let data = make_data(&[("age", Value::I64(16))]);
    assert!(filter.validate(&data).is_err());
  }

  #[test]
  fn test_break_on_failure_stops_at_field_and_skips_cross_field_rules() {
    let mut filter = FieldFilter::new();
    filter.add_field(
      "email",
      FieldBuilder::<Value>::default()
        .rule(Rule::Required)
        .break_on_failure(true)
        .build()
        .unwrap(),
    );
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("password_match".into()),
      fields: vec!["password".to_string(), "password_confirm".to_string()],
      rule: CrossFieldRuleType::FieldsEqual {
        field_a: "password".to_string(),
        field_b: "password_confirm".to_string(),
      },
    });

    let data = make_data(&[
      ("password", Value::Str("secret".to_string())),
      ("password_confirm", Value::Str("different".to_string())),
    ]);
    let result = filter.validate(&data);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.get("email").is_some());
    assert!(violations.form_violations().is_none());

    let data = make_data(&[
      ("email", Value::Str("test@example.com".to_string())),
      ("password", Value::Str("secret".to_string())),
      ("password_confirm", Value::Str("different".to_string())),
    ]);
    let result = filter.validate(&data);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.get("email").is_none());
    assert!(violations.form_violations().is_some());
  }

  // ====================================================================
  // try_filter tests
  // ====================================================================

  #[test]
  fn test_field_filter_try_filter_success() {
    use walrs_filter::TryFilterOp;

    let mut filter = FieldFilter::new();
    filter.add_field(
      "name",
      FieldBuilder::<Value>::default()
        .try_filters(vec![TryFilterOp::Infallible(FilterOp::Trim)])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("name", Value::Str("  hello  ".to_string()))]);
    let result = filter.try_filter(data).unwrap();
    assert_eq!(
      result.get("name").unwrap(),
      &Value::Str("hello".to_string())
    );
  }

  #[test]
  fn test_field_filter_try_filter_failure() {
    use std::sync::Arc;
    use walrs_filter::{FilterError, TryFilterOp};

    let mut filter = FieldFilter::new();
    filter.add_field(
      "encoded",
      FieldBuilder::<Value>::default()
        .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|v: Value| {
          if let Value::Str(ref s) = v {
            if s.contains('\0') {
              return Err(FilterError::new("null bytes not allowed"));
            }
          }
          Ok(v)
        }))])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("encoded", Value::Str("good input".to_string()))]);
    assert!(filter.try_filter(data).is_ok());

    let data = make_data(&[("encoded", Value::Str("bad\0input".to_string()))]);
    let err = filter.try_filter(data).unwrap_err();
    assert!(err.get("encoded").is_some());
  }

  #[test]
  fn test_field_filter_try_filter_break_on_failure() {
    use std::sync::Arc;
    use walrs_filter::{FilterError, TryFilterOp};

    let mut filter = FieldFilter::new();
    filter.add_field(
      "first",
      FieldBuilder::<Value>::default()
        .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|_| {
          Err(FilterError::new("first fails"))
        }))])
        .break_on_failure(true)
        .build()
        .unwrap(),
    );
    filter.add_field(
      "second",
      FieldBuilder::<Value>::default()
        .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|_| {
          panic!("should not reach second field")
        }))])
        .build()
        .unwrap(),
    );

    let data = make_data(&[
      ("first", Value::Str("a".to_string())),
      ("second", Value::Str("b".to_string())),
    ]);
    let err = filter.try_filter(data).unwrap_err();
    assert!(err.get("first").is_some());
    assert!(err.get("second").is_none());
  }

  #[test]
  fn test_field_filter_clean_with_try_filters() {
    use std::sync::Arc;
    use walrs_filter::{FilterError, TryFilterOp};

    let mut filter = FieldFilter::new();
    filter.add_field(
      "name",
      FieldBuilder::<Value>::default()
        .filters(vec![FilterOp::Trim])
        .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|v: Value| {
          if let Value::Str(ref s) = v {
            if s.is_empty() {
              return Err(FilterError::new("empty after trim"));
            }
          }
          Ok(v)
        }))])
        .rule(Rule::Required)
        .build()
        .unwrap(),
    );

    // Happy path: trim -> try_filter passes -> validation passes
    let data = make_data(&[("name", Value::Str("  hello  ".to_string()))]);
    let result = filter.clean(data).unwrap();
    assert_eq!(
      result.get("name").unwrap(),
      &Value::Str("hello".to_string())
    );

    // Try filter fails
    let data = make_data(&[("name", Value::Str("     ".to_string()))]);
    assert!(filter.clean(data).is_err());
  }

  #[test]
  fn test_field_filter_try_filter_no_try_filters() {
    let mut filter = FieldFilter::new();
    filter.add_field(
      "name",
      FieldBuilder::<Value>::default()
        .filters(vec![FilterOp::Trim])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("name", Value::Str("  hello  ".to_string()))]);
    let result = filter.try_filter(data).unwrap();
    assert_eq!(
      result.get("name").unwrap(),
      &Value::Str("  hello  ".to_string())
    );
  }

  // ====================================================================
  // filter_ref tests
  // ====================================================================

  #[test]
  fn test_field_filter_filter_ref() {
    let mut filter = FieldFilter::new();
    filter.add_field(
      "name",
      FieldBuilder::<Value>::default()
        .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
        .build()
        .unwrap(),
    );

    let data = make_data(&[("name", Value::Str("  HELLO  ".to_string()))]);
    let result = filter.filter_ref(&data);
    assert_eq!(
      result.get("name").unwrap(),
      &Value::Str("hello".to_string())
    );
    // Original data is untouched
    assert_eq!(
      data.get("name").unwrap(),
      &Value::Str("  HELLO  ".to_string())
    );
  }

  #[test]
  fn test_field_filter_filter_ref_excludes_unknown_fields() {
    let mut filter = FieldFilter::new();
    filter.add_field(
      "name",
      FieldBuilder::<Value>::default()
        .filters(vec![FilterOp::Trim])
        .build()
        .unwrap(),
    );

    let data = make_data(&[
      ("name", Value::Str("  hello  ".to_string())),
      ("extra", Value::Str("not in filter".to_string())),
    ]);
    let result = filter.filter_ref(&data);
    assert!(result.get("name").is_some());
    assert!(result.get("extra").is_none());
  }

  #[test]
  fn test_field_filter_filter_ref_no_filters() {
    let mut filter = FieldFilter::new();
    filter.add_field("name", FieldBuilder::<Value>::default().build().unwrap());

    let data = make_data(&[("name", Value::Str("unchanged".to_string()))]);
    let result = filter.filter_ref(&data);
    assert_eq!(
      result.get("name").unwrap(),
      &Value::Str("unchanged".to_string())
    );
  }

  // ====================================================================
  // validate missing-field-as-null tests
  // ====================================================================

  #[test]
  fn test_validate_missing_field_treated_as_null() {
    let mut filter = FieldFilter::new();
    filter.add_field(
      "required_field",
      FieldBuilder::<Value>::default()
        .rule(Rule::Required)
        .build()
        .unwrap(),
    );

    // Empty data — missing field should be treated as Null → Required fails
    let data: IndexMap<String, Value> = IndexMap::new();
    let result = filter.validate(&data);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.get("required_field").is_some());
  }

  #[test]
  fn test_validate_missing_field_optional_passes() {
    let mut filter = FieldFilter::new();
    // Field with no rule — missing field should pass
    filter.add_field(
      "optional_field",
      FieldBuilder::<Value>::default()
        .filters(vec![FilterOp::Trim])
        .build()
        .unwrap(),
    );

    let data: IndexMap<String, Value> = IndexMap::new();
    let result = filter.validate(&data);
    assert!(result.is_ok());
  }

  // ====================================================================
  // RequiredIf cross-field rule tests
  // ====================================================================

  #[test]
  fn test_required_if_condition_met_and_field_present() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("shipping_required".into()),
      fields: vec!["shipping_address".to_string(), "is_physical".to_string()],
      rule: CrossFieldRuleType::RequiredIf {
        field: "shipping_address".to_string(),
        condition_field: "is_physical".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition met (is_physical=true), field present → OK
    let data = make_data(&[
      ("is_physical", Value::Bool(true)),
      ("shipping_address", Value::Str("123 Main St".to_string())),
    ]);
    assert!(filter.validate(&data).is_ok());
  }

  #[test]
  fn test_required_if_condition_met_and_field_missing() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("shipping_required".into()),
      fields: vec!["shipping_address".to_string(), "is_physical".to_string()],
      rule: CrossFieldRuleType::RequiredIf {
        field: "shipping_address".to_string(),
        condition_field: "is_physical".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition met (is_physical=true), field missing → Error
    let data = make_data(&[("is_physical", Value::Bool(true))]);
    let result = filter.validate(&data);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.form_violations().is_some());
    assert!(
      violations.form_violations().unwrap()[0]
        .message()
        .contains("shipping_address")
    );
  }

  #[test]
  fn test_required_if_condition_not_met() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: None,
      fields: vec!["shipping_address".to_string(), "is_physical".to_string()],
      rule: CrossFieldRuleType::RequiredIf {
        field: "shipping_address".to_string(),
        condition_field: "is_physical".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition not met (is_physical=false), field missing → OK
    let data = make_data(&[("is_physical", Value::Bool(false))]);
    assert!(filter.validate(&data).is_ok());
  }

  #[test]
  fn test_required_if_condition_field_missing() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: None,
      fields: vec!["shipping_address".to_string(), "is_physical".to_string()],
      rule: CrossFieldRuleType::RequiredIf {
        field: "shipping_address".to_string(),
        condition_field: "is_physical".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition field missing entirely → condition not met → OK
    let data = make_data(&[]);
    assert!(filter.validate(&data).is_ok());
  }

  #[test]
  fn test_required_if_empty_condition_field_fallback() {
    // When condition_field is empty (e.g., deserialized from old data via serde(default)),
    // the fallback uses `field` as both condition and required field (backward compat).
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: None,
      fields: vec!["status".to_string()],
      rule: CrossFieldRuleType::RequiredIf {
        field: "status".to_string(),
        condition_field: String::new(), // empty → fallback to `field`
        condition: walrs_validation::Condition::Equals(Value::Str("active".into())),
      },
    });

    // Field equals condition → condition met on same field, and field has value → OK
    let data = make_data(&[("status", Value::Str("active".into()))]);
    assert!(filter.validate(&data).is_ok());

    // Field missing entirely → condition not met → OK
    let data = make_data(&[]);
    assert!(filter.validate(&data).is_ok());
  }

  #[test]
  fn test_required_unless_empty_condition_field_fallback() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: None,
      fields: vec!["status".to_string()],
      rule: CrossFieldRuleType::RequiredUnless {
        field: "status".to_string(),
        condition_field: String::new(), // empty → fallback to `field`
        condition: walrs_validation::Condition::Equals(Value::Str("exempt".into())),
      },
    });

    // Field is "exempt" → condition met → not required → OK
    let data = make_data(&[("status", Value::Str("exempt".into()))]);
    assert!(filter.validate(&data).is_ok());
  }

  // ====================================================================
  // RequiredUnless cross-field rule tests
  // ====================================================================

  #[test]
  fn test_required_unless_condition_met() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("email_unless_phone".into()),
      fields: vec!["email".to_string(), "has_phone".to_string()],
      rule: CrossFieldRuleType::RequiredUnless {
        field: "email".to_string(),
        condition_field: "has_phone".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition met (has_phone=true), field missing → OK (unless satisfied)
    let data = make_data(&[("has_phone", Value::Bool(true))]);
    assert!(filter.validate(&data).is_ok());
  }

  #[test]
  fn test_required_unless_condition_not_met_and_field_present() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: None,
      fields: vec!["email".to_string(), "has_phone".to_string()],
      rule: CrossFieldRuleType::RequiredUnless {
        field: "email".to_string(),
        condition_field: "has_phone".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition not met (has_phone=false), field present → OK
    let data = make_data(&[
      ("has_phone", Value::Bool(false)),
      ("email", Value::Str("user@example.com".to_string())),
    ]);
    assert!(filter.validate(&data).is_ok());
  }

  #[test]
  fn test_required_unless_condition_not_met_and_field_missing() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: Some("email_unless_phone".into()),
      fields: vec!["email".to_string(), "has_phone".to_string()],
      rule: CrossFieldRuleType::RequiredUnless {
        field: "email".to_string(),
        condition_field: "has_phone".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition not met (has_phone=false), field missing → Error
    let data = make_data(&[("has_phone", Value::Bool(false))]);
    let result = filter.validate(&data);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.form_violations().is_some());
    assert!(
      violations.form_violations().unwrap()[0]
        .message()
        .contains("email")
    );
  }

  #[test]
  fn test_required_unless_condition_field_missing() {
    let mut filter = FieldFilter::new();
    filter.add_cross_field_rule(CrossFieldRule {
      name: None,
      fields: vec!["email".to_string(), "has_phone".to_string()],
      rule: CrossFieldRuleType::RequiredUnless {
        field: "email".to_string(),
        condition_field: "has_phone".to_string(),
        condition: walrs_validation::Condition::Equals(Value::Bool(true)),
      },
    });

    // Condition field missing → condition not met → email required → Error
    let data = make_data(&[]);
    let result = filter.validate(&data);
    assert!(result.is_err());
  }
}
