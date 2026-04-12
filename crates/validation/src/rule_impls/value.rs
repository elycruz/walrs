//! Validation impls for the custom `Value` enum.
//!
//! Provides `ValidateRef<Value>` and `Validate<Value>` for `Rule<Value>`,
//! enabling dynamic/heterogeneous validation of form data.

use std::cmp::Ordering;

use crate::Violation;
use crate::ViolationType;
use crate::rule::{Condition, Rule, RuleResult};
use crate::traits::{IsEmpty, Validate, ValidateRef};
use crate::value::{Value, ValueExt};

// ============================================================================
// Condition<Value> evaluation
// ============================================================================

impl Condition<Value> {
  /// Evaluates the condition against a `Value`.
  pub fn evaluate_value(&self, value: &Value) -> bool {
    match self {
      Condition::IsEmpty => value.is_empty_value(),
      Condition::IsNotEmpty => !value.is_empty_value(),
      Condition::Equals(expected) => value == expected,
      Condition::GreaterThan(threshold) => value.partial_cmp(threshold) == Some(Ordering::Greater),
      Condition::LessThan(threshold) => value.partial_cmp(threshold) == Some(Ordering::Less),
      Condition::Matches(cp) => match value {
        Value::Str(s) => cp.0.is_match(s),
        _ => false,
      },
      Condition::Custom(f) => f(value),
    }
  }
}

// ============================================================================
// ValidateRef<Value> for Rule<Value>
// ============================================================================

impl Rule<Value> {
  /// Validates a `Value` against this rule.
  pub fn validate_value(&self, value: &Value) -> RuleResult {
    self.validate_value_inner(value, None)
  }

  /// Internal validation with inherited locale.
  fn validate_value_inner(&self, value: &Value, inherited_locale: Option<&str>) -> RuleResult {
    match self {
      Rule::Required => {
        if value.is_empty() {
          Err(Violation::value_missing())
        } else {
          Ok(())
        }
      }

      // ---- Length rules (string only) ----
      Rule::MinLength(min) => match value {
        Value::Str(s) => {
          let len = s.chars().count();
          if len < *min {
            Err(Violation::too_short(*min, len))
          } else {
            Ok(())
          }
        }
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for MinLength.",
        )),
      },
      Rule::MaxLength(max) => match value {
        Value::Str(s) => {
          let len = s.chars().count();
          if len > *max {
            Err(Violation::too_long(*max, len))
          } else {
            Ok(())
          }
        }
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for MaxLength.",
        )),
      },
      Rule::ExactLength(expected) => match value {
        Value::Str(s) => {
          let len = s.chars().count();
          if len != *expected {
            Err(Violation::exact_length(*expected, len))
          } else {
            Ok(())
          }
        }
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for ExactLength.",
        )),
      },

      // ---- String rules ----
      Rule::Pattern(cp) => match value {
        Value::Str(s) => Rule::<String>::Pattern(cp.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for Pattern.",
        )),
      },
      Rule::Email(opts) => match value {
        Value::Str(s) => Rule::<String>::Email(opts.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for Email.",
        )),
      },
      Rule::Url(opts) => match value {
        Value::Str(s) => Rule::<String>::Url(opts.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for Url.",
        )),
      },
      Rule::Uri(opts) => match value {
        Value::Str(s) => Rule::<String>::Uri(opts.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for Uri.",
        )),
      },
      Rule::Ip(opts) => match value {
        Value::Str(s) => Rule::<String>::Ip(opts.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for Ip.",
        )),
      },
      Rule::Hostname(opts) => match value {
        Value::Str(s) => Rule::<String>::Hostname(opts.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for Hostname.",
        )),
      },
      Rule::Date(opts) => match value {
        Value::Str(s) => Rule::<String>::Date(opts.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for Date.",
        )),
      },
      Rule::DateRange(opts) => match value {
        Value::Str(s) => Rule::<String>::DateRange(opts.clone()).validate_str(s.as_str()),
        _ => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Expected a string for DateRange.",
        )),
      },

      // ---- Numeric rules ----
      Rule::Min(bound) => match value.partial_cmp(bound) {
        Some(Ordering::Less) => Err(Violation::range_underflow(bound)),
        Some(_) => Ok(()),
        None => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Incompatible types for Min.",
        )),
      },
      Rule::Max(bound) => match value.partial_cmp(bound) {
        Some(Ordering::Greater) => Err(Violation::range_overflow(bound)),
        Some(_) => Ok(()),
        None => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Incompatible types for Max.",
        )),
      },
      Rule::Range { min, max } => {
        match value.partial_cmp(min) {
          Some(Ordering::Less) => return Err(Violation::range_underflow(min)),
          None => {
            return Err(Violation::new(
              ViolationType::TypeMismatch,
              "Incompatible types for Range.",
            ));
          }
          _ => {}
        }
        match value.partial_cmp(max) {
          Some(Ordering::Greater) => Err(Violation::range_overflow(max)),
          None => Err(Violation::new(
            ViolationType::TypeMismatch,
            "Incompatible types for Range.",
          )),
          _ => Ok(()),
        }
      }
      Rule::Step(step) => {
        let ok = match (value, step) {
          (Value::I64(v), Value::I64(s)) => (*s != 0) && (*v % *s == 0),
          (Value::U64(v), Value::U64(s)) => (*s != 0) && (*v % *s == 0),
          (Value::F64(v), Value::F64(s)) => (*s != 0.0) && ((*v % *s).abs() < f64::EPSILON),
          _ => {
            return Err(Violation::new(
              ViolationType::TypeMismatch,
              "Incompatible types for Step.",
            ));
          }
        };
        if ok {
          Ok(())
        } else {
          Err(Violation::step_mismatch(step))
        }
      }

      // ---- Comparison ----
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(Violation::not_equal(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.iter().any(|v| v == value) {
          Ok(())
        } else {
          Err(Violation::not_one_of())
        }
      }

      // ---- Composite ----
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_value_inner(value, inherited_locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_value_inner(value, inherited_locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_value_inner(value, inherited_locale) {
        Ok(()) => Err(Violation::negation_failed()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        if condition.evaluate_value(value) {
          then_rule.validate_value_inner(value, inherited_locale)
        } else {
          match else_rule {
            Some(rule) => rule.validate_value_inner(value, inherited_locale),
            None => Ok(()),
          }
        }
      }

      // ---- Custom / Ref / WithMessage ----
      Rule::Custom(f) => f(value),
      #[cfg(feature = "async")]
      Rule::CustomAsync(_) => Ok(()),
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let eff = locale.as_deref().or(inherited_locale);
        message.wrap_result(rule.validate_value_inner(value, eff), value, eff)
      }
    }
  }
}

impl ValidateRef<Value> for Rule<Value> {
  fn validate_ref(&self, value: &Value) -> crate::ValidatorResult {
    self.validate_value(value)
  }
}

impl Validate<Value> for Rule<Value> {
  fn validate(&self, value: Value) -> crate::ValidatorResult {
    self.validate_ref(&value)
  }
}

impl Validate<Option<Value>> for Rule<Value> {
  fn validate(&self, value: Option<Value>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(ref v) => ValidateRef::<Value>::validate_ref(self, v),
    }
  }
}

impl ValidateRef<Option<Value>> for Rule<Value> {
  fn validate_ref(&self, value: &Option<Value>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => ValidateRef::<Value>::validate_ref(self, v),
    }
  }
}

// ============================================================================
// Async Value Validation
// ============================================================================

#[cfg(feature = "async")]
impl Rule<Value> {
  /// Validates a Value asynchronously.
  ///
  /// Runs all rules: sync rules execute inline, `CustomAsync` rules are awaited.
  pub(crate) async fn validate_value_async(&self, value: &Value) -> RuleResult {
    self.validate_value_async_inner(value, None).await
  }

  /// Internal async validation with inherited locale.
  /// Handles both sync and async rules in a single traversal.
  fn validate_value_async_inner<'a>(
    &'a self,
    value: &'a Value,
    inherited_locale: Option<&'a str>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RuleResult> + Send + 'a>> {
    Box::pin(async move {
      match self {
        Rule::CustomAsync(f) => f(value).await,

        Rule::All(rules) => {
          for rule in rules {
            rule
              .validate_value_async_inner(value, inherited_locale)
              .await?;
          }
          Ok(())
        }
        Rule::Any(rules) => {
          if rules.is_empty() {
            return Ok(());
          }
          let mut last_err = None;
          for rule in rules {
            match rule
              .validate_value_async_inner(value, inherited_locale)
              .await
            {
              Ok(()) => return Ok(()),
              Err(e) => last_err = Some(e),
            }
          }
          Err(last_err.unwrap())
        }
        Rule::Not(inner) => {
          match inner
            .validate_value_async_inner(value, inherited_locale)
            .await
          {
            Ok(()) => Err(Violation::negation_failed()),
            Err(_) => Ok(()),
          }
        }
        Rule::When {
          condition,
          then_rule,
          else_rule,
        } => {
          if condition.evaluate_value(value) {
            then_rule
              .validate_value_async_inner(value, inherited_locale)
              .await
          } else {
            match else_rule {
              Some(rule) => {
                rule
                  .validate_value_async_inner(value, inherited_locale)
                  .await
              }
              None => Ok(()),
            }
          }
        }
        Rule::WithMessage {
          rule,
          message,
          locale,
        } => {
          let eff = locale.as_deref().or(inherited_locale);
          message.wrap_result(
            rule.validate_value_async_inner(value, eff).await,
            value,
            eff,
          )
        }

        // All sync rules — delegate to sync validation
        other => other.validate_value_inner(value, inherited_locale),
      }
    })
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<Value> for Rule<Value> {
  async fn validate_ref_async(&self, value: &Value) -> crate::ValidatorResult {
    self.validate_value_async(value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<Value> for Rule<Value> {
  async fn validate_async(&self, value: Value) -> crate::ValidatorResult {
    self.validate_value_async(&value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<Option<Value>> for Rule<Value> {
  async fn validate_async(&self, value: Option<Value>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(ref v) => self.validate_value_async(v).await,
    }
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<Option<Value>> for Rule<Value> {
  async fn validate_ref_async(&self, value: &Option<Value>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_value_async(v).await,
    }
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use crate::value;

  #[test]
  fn test_required_null() {
    let rule = Rule::<Value>::Required;
    assert!(rule.validate_value(&Value::Null).is_err());
  }

  #[test]
  fn test_required_empty_string() {
    let rule = Rule::<Value>::Required;
    assert!(rule.validate_value(&Value::Str("".to_string())).is_err());
  }

  #[test]
  fn test_required_non_empty_string() {
    let rule = Rule::<Value>::Required;
    assert!(
      rule
        .validate_value(&Value::Str("hello".to_string()))
        .is_ok()
    );
  }

  #[test]
  fn test_required_number() {
    let rule = Rule::<Value>::Required;
    assert!(rule.validate_value(&Value::I64(0)).is_ok());
  }

  #[test]
  fn test_min_length_str() {
    let rule = Rule::<Value>::MinLength(3);
    assert!(rule.validate_value(&Value::Str("hi".to_string())).is_err());
    assert!(
      rule
        .validate_value(&Value::Str("hello".to_string()))
        .is_ok()
    );
  }

  #[test]
  fn test_min_length_non_string() {
    let rule = Rule::<Value>::MinLength(3);
    let result = rule.validate_value(&Value::I64(42));
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().violation_type(),
      ViolationType::TypeMismatch
    );
  }

  #[test]
  fn test_max_length_str() {
    let rule = Rule::<Value>::MaxLength(5);
    assert!(rule.validate_value(&Value::Str("hi".to_string())).is_ok());
    assert!(
      rule
        .validate_value(&Value::Str("hello world".to_string()))
        .is_err()
    );
  }

  #[test]
  fn test_exact_length_str() {
    let rule = Rule::<Value>::ExactLength(5);
    assert!(
      rule
        .validate_value(&Value::Str("hello".to_string()))
        .is_ok()
    );
    assert!(rule.validate_value(&Value::Str("hi".to_string())).is_err());
  }

  #[test]
  fn test_pattern() {
    let rule = Rule::<Value>::pattern(r"^\d+$").unwrap();
    assert!(rule.validate_value(&Value::Str("123".to_string())).is_ok());
    assert!(rule.validate_value(&Value::Str("abc".to_string())).is_err());
  }

  #[test]
  fn test_email() {
    let rule = Rule::<Value>::Email(Default::default());
    assert!(
      rule
        .validate_value(&Value::Str("test@example.com".to_string()))
        .is_ok()
    );
    assert!(
      rule
        .validate_value(&Value::Str("invalid".to_string()))
        .is_err()
    );
  }

  #[test]
  fn test_url() {
    let rule = Rule::<Value>::Url(Default::default());
    assert!(
      rule
        .validate_value(&Value::Str("https://example.com".to_string()))
        .is_ok()
    );
    assert!(
      rule
        .validate_value(&Value::Str("not-a-url".to_string()))
        .is_err()
    );
  }

  #[test]
  fn test_min_i64() {
    let rule = Rule::<Value>::Min(Value::I64(10));
    assert!(rule.validate_value(&Value::I64(15)).is_ok());
    assert!(rule.validate_value(&Value::I64(5)).is_err());
    assert!(rule.validate_value(&Value::I64(10)).is_ok());
  }

  #[test]
  fn test_max_f64() {
    let rule = Rule::<Value>::Max(Value::F64(100.0));
    assert!(rule.validate_value(&Value::F64(50.0)).is_ok());
    assert!(rule.validate_value(&Value::F64(150.0)).is_err());
  }

  #[test]
  fn test_range_u64() {
    let rule = Rule::<Value>::Range {
      min: Value::U64(10),
      max: Value::U64(100),
    };
    assert!(rule.validate_value(&Value::U64(50)).is_ok());
    assert!(rule.validate_value(&Value::U64(5)).is_err());
    assert!(rule.validate_value(&Value::U64(150)).is_err());
  }

  #[test]
  fn test_type_mismatch_min() {
    let rule = Rule::<Value>::Min(Value::I64(10));
    let result = rule.validate_value(&Value::Str("hello".to_string()));
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().violation_type(),
      ViolationType::TypeMismatch
    );
  }

  #[test]
  fn test_step_i64() {
    let rule = Rule::<Value>::Step(Value::I64(5));
    assert!(rule.validate_value(&Value::I64(10)).is_ok());
    assert!(rule.validate_value(&Value::I64(7)).is_err());
  }

  #[test]
  fn test_step_f64() {
    let rule = Rule::<Value>::Step(Value::F64(0.5));
    assert!(rule.validate_value(&Value::F64(1.0)).is_ok());
    assert!(rule.validate_value(&Value::F64(1.3)).is_err());
  }

  #[test]
  fn test_equals() {
    let rule = Rule::<Value>::Equals(Value::Str("hello".to_string()));
    assert!(
      rule
        .validate_value(&Value::Str("hello".to_string()))
        .is_ok()
    );
    assert!(
      rule
        .validate_value(&Value::Str("world".to_string()))
        .is_err()
    );
  }

  #[test]
  fn test_one_of() {
    let rule = Rule::<Value>::OneOf(vec![
      Value::Str("a".to_string()),
      Value::Str("b".to_string()),
    ]);
    assert!(rule.validate_value(&Value::Str("a".to_string())).is_ok());
    assert!(rule.validate_value(&Value::Str("c".to_string())).is_err());
  }

  #[test]
  fn test_all() {
    let rule = Rule::<Value>::All(vec![Rule::Required, Rule::MinLength(3)]);
    assert!(
      rule
        .validate_value(&Value::Str("hello".to_string()))
        .is_ok()
    );
    assert!(rule.validate_value(&Value::Str("hi".to_string())).is_err());
  }

  #[test]
  fn test_any() {
    let rule = Rule::<Value>::Any(vec![
      Rule::Email(Default::default()),
      Rule::Url(Default::default()),
    ]);
    assert!(
      rule
        .validate_value(&Value::Str("test@example.com".to_string()))
        .is_ok()
    );
    assert!(
      rule
        .validate_value(&Value::Str("https://example.com".to_string()))
        .is_ok()
    );
    assert!(
      rule
        .validate_value(&Value::Str("plain".to_string()))
        .is_err()
    );
  }

  #[test]
  fn test_not() {
    let rule = Rule::<Value>::Not(Box::new(Rule::Required));
    assert!(rule.validate_value(&Value::Null).is_ok());
    assert!(
      rule
        .validate_value(&Value::Str("hello".to_string()))
        .is_err()
    );
  }

  #[test]
  fn test_when() {
    let rule = Rule::<Value>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::MinLength(3)),
      else_rule: None,
    };
    // Non-empty, short string → should fail
    assert!(rule.validate_value(&Value::Str("hi".to_string())).is_err());
    // Non-empty, long enough → should pass
    assert!(
      rule
        .validate_value(&Value::Str("hello".to_string()))
        .is_ok()
    );
    // Empty → condition false, no else → pass
    assert!(rule.validate_value(&Value::Str("".to_string())).is_ok());
  }

  #[test]
  fn test_validate_ref_trait() {
    let rule = Rule::<Value>::Required;
    assert!(ValidateRef::validate_ref(&rule, &Value::Null).is_err());
    assert!(ValidateRef::validate_ref(&rule, &Value::I64(1)).is_ok());
  }

  #[test]
  fn test_validate_trait() {
    let rule = Rule::<Value>::Required;
    assert!(Validate::validate(&rule, Value::Null).is_err());
    assert!(Validate::validate(&rule, Value::I64(1)).is_ok());
  }

  #[test]
  fn test_value_macro_in_rules() {
    let rule = Rule::<Value>::Min(value!(10));
    assert!(rule.validate_value(&value!(15)).is_ok());
    assert!(rule.validate_value(&value!(5)).is_err());
  }

  // ========================================================================
  // Option<Value> Validation (trait impls)
  // ========================================================================

  #[test]
  fn test_option_value_none_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<Value>::Required;
    assert!(Validate::<Option<Value>>::validate(&rule, None).is_err());
    assert!(ValidateRef::<Option<Value>>::validate_ref(&rule, &None).is_err());
  }

  #[test]
  fn test_option_value_none_not_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<Value>::MinLength(3);
    assert!(Validate::<Option<Value>>::validate(&rule, None).is_ok());
    assert!(ValidateRef::<Option<Value>>::validate_ref(&rule, &None).is_ok());
  }

  #[test]
  fn test_option_value_some_valid() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<Value>::MinLength(3);
    let val = Value::Str("hello".to_string());
    assert!(Validate::<Option<Value>>::validate(&rule, Some(val.clone())).is_ok());
    assert!(ValidateRef::<Option<Value>>::validate_ref(&rule, &Some(val)).is_ok());
  }

  #[test]
  fn test_option_value_some_invalid() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<Value>::MinLength(5);
    let val = Value::Str("hi".to_string());
    assert!(Validate::<Option<Value>>::validate(&rule, Some(val.clone())).is_err());
    assert!(ValidateRef::<Option<Value>>::validate_ref(&rule, &Some(val)).is_err());
  }

  #[test]
  fn test_option_value_none_all_with_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<Value>::Required.and(Rule::MinLength(3));
    assert!(Validate::<Option<Value>>::validate(&rule, None).is_err());
    assert!(ValidateRef::<Option<Value>>::validate_ref(&rule, &None).is_err());
  }

  #[test]
  fn test_option_value_none_all_without_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<Value>::MinLength(3).and(Rule::MaxLength(10));
    assert!(Validate::<Option<Value>>::validate(&rule, None).is_ok());
    assert!(ValidateRef::<Option<Value>>::validate_ref(&rule, &None).is_ok());
  }

  // ==========================================================================
  // Issue #143 — Rule::Ref tests (Value)
  // ==========================================================================

  #[test]
  fn test_ref_returns_unresolved_ref_value_i64() {
    let rule = Rule::<Value>::Ref("value_ref".into());
    let err = rule.validate_value(&Value::I64(42)).unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::CustomError);
    assert!(err.message().contains("value_ref"));
    assert_eq!(err.message(), "Unresolved rule reference: value_ref.");
  }

  #[test]
  fn test_ref_returns_unresolved_ref_value_str() {
    let rule = Rule::<Value>::Ref("str_ref".into());
    let err = rule
      .validate_value(&Value::Str("hello".into()))
      .unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::CustomError);
    assert!(err.message().contains("str_ref"));
  }

  #[test]
  fn test_ref_returns_unresolved_ref_value_f64() {
    let rule = Rule::<Value>::Ref("f64_ref".into());
    let err = rule.validate_value(&Value::F64(3.14)).unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::CustomError);
    assert!(err.message().contains("f64_ref"));
  }

  #[test]
  fn test_ref_returns_unresolved_ref_value_null() {
    let rule = Rule::<Value>::Ref("null_ref".into());
    let err = rule.validate_value(&Value::Null).unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::CustomError);
    assert!(err.message().contains("null_ref"));
  }

  #[test]
  fn test_ref_inside_all_value() {
    let rule = Rule::<Value>::All(vec![Rule::MinLength(1), Rule::Ref("all_ref".into())]);
    let err = rule
      .validate_value(&Value::Str("hello".into()))
      .unwrap_err();
    assert!(err.message().contains("all_ref"));
  }

  #[test]
  fn test_ref_inside_any_value() {
    let rule = Rule::<Value>::Any(vec![
      Rule::Ref("any_ref_a".into()),
      Rule::Ref("any_ref_b".into()),
    ]);
    assert!(rule.validate_value(&Value::I64(10)).is_err());
  }

  #[test]
  fn test_ref_inside_not_value() {
    let rule = Rule::<Value>::Not(Box::new(Rule::Ref("not_ref".into())));
    assert!(rule.validate_value(&Value::I64(10)).is_ok());
  }

  #[test]
  fn test_ref_inside_when_value() {
    let rule = Rule::<Value>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::Ref("when_ref".into())),
      else_rule: None,
    };
    let err = rule
      .validate_value(&Value::Str("non_empty".into()))
      .unwrap_err();
    assert!(err.message().contains("when_ref"));
  }

  // ==========================================================================
  // Issue #145 — Deeply nested combinator tests (Value)
  // ==========================================================================

  #[test]
  fn test_nested_all_all_any_depth_2_value() {
    let rule = Rule::<Value>::All(vec![
      Rule::All(vec![Rule::MinLength(1), Rule::MaxLength(20)]),
      Rule::Any(vec![Rule::MinLength(3), Rule::MaxLength(5)]),
    ]);
    assert!(rule.validate_value(&Value::Str("hello".into())).is_ok());
    assert!(rule.validate_value(&Value::Str("".into())).is_err());
  }

  #[test]
  fn test_nested_when_all_any_depth_2_value() {
    let rule = Rule::<Value>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::All(vec![Rule::MinLength(2), Rule::MaxLength(10)])),
      else_rule: Some(Box::new(Rule::Any(vec![Rule::MinLength(0)]))),
    };
    assert!(rule.validate_value(&Value::Str("hello".into())).is_ok());
    assert!(rule.validate_value(&Value::Str("x".into())).is_err());
    // Null is empty, so else branch fires with MinLength(0); Null has no meaningful length — verify behavior
    assert!(rule.validate_value(&Value::Null).is_err());
  }

  #[test]
  fn test_nested_not_all_any_depth_3_value() {
    let rule = Rule::<Value>::Not(Box::new(Rule::All(vec![Rule::Any(vec![Rule::MinLength(
      100,
    )])])));
    assert!(rule.validate_value(&Value::Str("hello".into())).is_ok());
  }

  #[test]
  fn test_nested_any_not_all_depth_2_value() {
    let rule = Rule::<Value>::Any(vec![
      Rule::Not(Box::new(Rule::MinLength(100))),
      Rule::All(vec![Rule::MinLength(1), Rule::MaxLength(10)]),
    ]);
    assert!(rule.validate_value(&Value::Str("hello".into())).is_ok());
  }

  #[test]
  fn test_nested_all_all_all_depth_3_value() {
    let rule = Rule::<Value>::All(vec![Rule::All(vec![Rule::All(vec![
      Rule::MinLength(1),
      Rule::MaxLength(50),
    ])])]);
    assert!(rule.validate_value(&Value::Str("hello".into())).is_ok());
    assert!(rule.validate_value(&Value::Str("".into())).is_err());
  }

  #[test]
  fn test_empty_any_returns_ok_value() {
    let rule = Rule::<Value>::Any(vec![]);
    assert!(rule.validate_value(&Value::I64(42)).is_ok());
  }
}
