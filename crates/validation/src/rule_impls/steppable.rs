use std::cmp::Ordering;

use crate::rule::{Rule, RuleResult};
use crate::traits::{IsEmpty, Validate, ValidateRef};
use crate::{SteppableValue, Violation, ViolationType};

impl<T: SteppableValue + IsEmpty> Rule<T> {
  /// Validates a numeric value against this rule.
  pub(crate) fn validate_step(&self, value: T) -> RuleResult {
    self.validate_step_inner(value, None)
  }

  /// Internal validation with inherited locale from an outer `WithMessage`.
  fn validate_step_inner(&self, value: T, inherited_locale: Option<&str>) -> RuleResult {
    match self {
      Rule::Required => {
        // Numeric values are always "present"
        Ok(())
      }
      Rule::Min(min) => match value.partial_cmp(min) {
        Some(Ordering::Less) => Err(Violation::range_underflow(min)),
        Some(_) => Ok(()),
        None => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Value is not a valid number.",
        )),
      },
      Rule::Max(max) => match value.partial_cmp(max) {
        Some(Ordering::Greater) => Err(Violation::range_overflow(max)),
        Some(_) => Ok(()),
        None => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Value is not a valid number.",
        )),
      },
      Rule::Range { min, max } => match value.partial_cmp(min) {
        Some(Ordering::Less) => Err(Violation::range_underflow(min)),
        None => Err(Violation::new(
          ViolationType::TypeMismatch,
          "Value is not a valid number.",
        )),
        _ => match value.partial_cmp(max) {
          Some(Ordering::Greater) => Err(Violation::range_overflow(max)),
          Some(_) => Ok(()),
          None => Err(Violation::new(
            ViolationType::TypeMismatch,
            "Value is not a valid number.",
          )),
        },
      },
      Rule::Step(step) => {
        if value.rem_check(*step) {
          Ok(())
        } else {
          Err(Violation::step_mismatch(step))
        }
      }
      Rule::Equals(expected) => {
        if value == *expected {
          Ok(())
        } else {
          Err(Violation::not_equal(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.contains(&value) {
          Ok(())
        } else {
          Err(Violation::not_one_of())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_step_inner(value, inherited_locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_step_inner(value, inherited_locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_step_inner(value, inherited_locale) {
        Ok(()) => Err(Violation::negation_failed()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate(&value);
        if should_apply {
          then_rule.validate_step_inner(value, inherited_locale)
        } else {
          match else_rule {
            Some(rule) => rule.validate_step_inner(value, inherited_locale),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value),
      #[cfg(feature = "async")]
      Rule::CustomAsync(_) => Ok(()),
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let eff = locale.as_deref().or(inherited_locale);
        message.wrap_result(rule.validate_step_inner(value, eff), &value, eff)
      }
      // String rules don't apply to numbers - pass through
      Rule::MinLength(_)
      | Rule::MaxLength(_)
      | Rule::ExactLength(_)
      | Rule::Pattern(_)
      | Rule::Email(_)
      | Rule::Url(_)
      | Rule::Uri(_)
      | Rule::Ip(_)
      | Rule::Hostname(_)
      | Rule::Date(_)
      | Rule::DateRange(_) => Ok(()),
    }
  }

  /// Validates a numeric value and collects all violations.
  pub(crate) fn validate_step_all(&self, value: T) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_violations(value, None, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional numeric value.
  #[allow(dead_code)] // Reserved for a future `validate_option` public API
  pub(crate) fn validate_option_step(&self, value: Option<T>) -> RuleResult {
    match value {
      Some(v) => self.validate_step(v),
      None => Err(Violation::value_missing()),
    }
  }

  /// Validates an optional numeric value and collects all violations.
  #[allow(dead_code)] // Reserved for a future `validate_option_all` public API
  pub(crate) fn validate_option_step_all(&self, value: Option<T>) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_step_all(v),
      None => Err(crate::Violations::from(Violation::value_missing())),
    }
  }

  /// Helper to collect all violations recursively.
  fn collect_violations(
    &self,
    value: T,
    inherited_locale: Option<&str>,
    violations: &mut crate::Violations,
  ) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_violations(value, inherited_locale, violations);
        }
      }
      Rule::Any(rules) => {
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_violations(value, inherited_locale, &mut rule_violations);
          if rule_violations.is_empty() {
            any_passed = true;
            break;
          }
          any_violations.extend(rule_violations);
        }
        if !any_passed && !rules.is_empty() {
          violations.extend(any_violations);
        }
      }
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate(&value);
        if should_apply {
          then_rule.collect_violations(value, inherited_locale, violations);
        } else if let Some(rule) = else_rule {
          rule.collect_violations(value, inherited_locale, violations);
        }
      }
      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let eff = locale.as_deref().or(inherited_locale);
        let mut inner_violations = crate::Violations::default();
        rule.collect_violations(value, eff, &mut inner_violations);
        message.wrap_violations(inner_violations, &value, eff, violations);
      }
      _ => {
        if let Err(v) = self.validate_step_inner(value, inherited_locale) {
          violations.push(v);
        }
      }
    }
  }
}

impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for Rule<T> {
  fn validate(&self, value: T) -> crate::ValidatorResult {
    Rule::validate_step(self, value)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> Validate<Option<T>> for Rule<T> {
  fn validate(&self, value: Option<T>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate(v),
    }
  }
}

impl<T: SteppableValue + IsEmpty + Clone> ValidateRef<T> for Rule<T> {
  fn validate_ref(&self, value: &T) -> crate::ValidatorResult {
    self.validate(*value)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> ValidateRef<Option<T>> for Rule<T> {
  fn validate_ref(&self, value: &Option<T>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate(*v),
    }
  }
}

// ============================================================================
// Async Numeric Validation
// ============================================================================

#[cfg(feature = "async")]
impl<T: SteppableValue + IsEmpty + Clone + Send + Sync> Rule<T> {
  /// Validates a numeric value asynchronously.
  ///
  /// Runs all rules: sync rules execute inline, `CustomAsync` rules are awaited.
  pub(crate) async fn validate_step_async(&self, value: T) -> RuleResult {
    self.validate_step_async_inner(value, None).await
  }

  /// Internal async validation with inherited locale.
  /// Handles both sync and async rules in a single traversal.
  fn validate_step_async_inner<'a>(
    &'a self,
    value: T,
    inherited_locale: Option<&'a str>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RuleResult> + Send + 'a>>
  where
    T: 'a,
  {
    Box::pin(async move {
      match self {
        Rule::CustomAsync(f) => f(&value).await,

        Rule::All(rules) => {
          for rule in rules {
            rule
              .validate_step_async_inner(value, inherited_locale)
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
              .validate_step_async_inner(value, inherited_locale)
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
            .validate_step_async_inner(value, inherited_locale)
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
          if condition.evaluate(&value) {
            then_rule
              .validate_step_async_inner(value, inherited_locale)
              .await
          } else {
            match else_rule {
              Some(rule) => {
                rule
                  .validate_step_async_inner(value, inherited_locale)
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
            rule.validate_step_async_inner(value, eff).await,
            &value,
            eff,
          )
        }

        // All sync rules — delegate to sync validation
        other => other.validate_step_inner(value, inherited_locale),
      }
    })
  }
}

#[cfg(feature = "async")]
impl<T: SteppableValue + IsEmpty + Clone + Send + Sync> crate::ValidateAsync<T> for Rule<T> {
  async fn validate_async(&self, value: T) -> crate::ValidatorResult {
    self.validate_step_async(value).await
  }
}

#[cfg(feature = "async")]
impl<T: SteppableValue + IsEmpty + Clone + Send + Sync> crate::ValidateAsync<Option<T>>
  for Rule<T>
{
  async fn validate_async(&self, value: Option<T>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_step_async(v).await,
    }
  }
}

#[cfg(feature = "async")]
impl<T: SteppableValue + IsEmpty + Clone + Send + Sync> crate::ValidateRefAsync<T> for Rule<T> {
  async fn validate_ref_async(&self, value: &T) -> crate::ValidatorResult {
    self.validate_step_async(*value).await
  }
}

#[cfg(feature = "async")]
impl<T: SteppableValue + IsEmpty + Clone + Send + Sync> crate::ValidateRefAsync<Option<T>>
  for Rule<T>
{
  async fn validate_ref_async(&self, value: &Option<T>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_step_async(*v).await,
    }
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use crate::rule::{Condition, Rule};
  use crate::{Validate, ValidateRef};

  // ========================================================================
  // Numeric Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_min() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate_step(0).is_ok());
    assert!(rule.validate_step(100).is_ok());
    assert!(rule.validate_step(-1).is_err());
  }

  #[test]
  fn test_validate_max() {
    let rule = Rule::<i32>::Max(100);
    assert!(rule.validate_step(100).is_ok());
    assert!(rule.validate_step(0).is_ok());
    assert!(rule.validate_step(101).is_err());
  }

  #[test]
  fn test_validate_range() {
    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    assert!(rule.validate_step(0).is_ok());
    assert!(rule.validate_step(50).is_ok());
    assert!(rule.validate_step(100).is_ok());
    assert!(rule.validate_step(-1).is_err());
    assert!(rule.validate_step(101).is_err());
  }

  #[test]
  fn test_validate_step() {
    let rule = Rule::<i32>::Step(5);
    assert!(rule.validate_step(0).is_ok());
    assert!(rule.validate_step(5).is_ok());
    assert!(rule.validate_step(10).is_ok());
    assert!(rule.validate_step(3).is_err());
  }

  #[test]
  fn test_validate_step_float() {
    let rule = Rule::<f64>::Step(0.5);
    assert!(rule.validate_step(0.0).is_ok());
    assert!(rule.validate_step(0.5).is_ok());
    assert!(rule.validate_step(1.0).is_ok());
    assert!(rule.validate_step(0.3).is_err());
  }

  // ========================================================================
  // NaN Validation Tests (f32 / f64)
  // ========================================================================

  #[test]
  fn test_validate_nan_f64_min() {
    let rule = Rule::<f64>::Min(0.0);
    assert!(rule.validate_step(f64::NAN).is_err());
    assert!(rule.validate(f64::NAN).is_err());
    assert!(rule.validate_ref(&f64::NAN).is_err());
  }

  #[test]
  fn test_validate_nan_f64_max() {
    let rule = Rule::<f64>::Max(100.0);
    assert!(rule.validate_step(f64::NAN).is_err());
    assert!(rule.validate(f64::NAN).is_err());
    assert!(rule.validate_ref(&f64::NAN).is_err());
  }

  #[test]
  fn test_validate_nan_f64_range() {
    let rule = Rule::<f64>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_step(f64::NAN).is_err());
    assert!(rule.validate(f64::NAN).is_err());
    assert!(rule.validate_ref(&f64::NAN).is_err());
  }

  #[test]
  fn test_validate_nan_f32_min() {
    let rule = Rule::<f32>::Min(0.0);
    assert!(rule.validate_step(f32::NAN).is_err());
    assert!(rule.validate(f32::NAN).is_err());
  }

  #[test]
  fn test_validate_nan_f32_max() {
    let rule = Rule::<f32>::Max(100.0);
    assert!(rule.validate_step(f32::NAN).is_err());
    assert!(rule.validate(f32::NAN).is_err());
  }

  #[test]
  fn test_validate_nan_f32_range() {
    let rule = Rule::<f32>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_step(f32::NAN).is_err());
    assert!(rule.validate(f32::NAN).is_err());
  }

  #[test]
  fn test_validate_nan_violation_type() {
    use crate::ViolationType;
    let rule = Rule::<f64>::Min(0.0);
    let err = rule.validate(f64::NAN).unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::TypeMismatch);
  }

  #[test]
  fn test_validate_equals_numeric() {
    let rule = Rule::<i32>::Equals(42);
    assert!(rule.validate_step(42).is_ok());
    assert!(rule.validate_step(0).is_err());
  }

  #[test]
  fn test_validate_one_of_numeric() {
    let rule = Rule::<i32>::OneOf(vec![1, 2, 3]);
    assert!(rule.validate_step(1).is_ok());
    assert!(rule.validate_step(2).is_ok());
    assert!(rule.validate_step(4).is_err());
  }

  #[test]
  fn test_validate_all_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100)).and(Rule::Step(10));
    assert!(rule.validate_step(50).is_ok());
    assert!(rule.validate_step(55).is_err()); // Not step of 10
    assert!(rule.validate_step(-10).is_err()); // Below min
  }

  #[test]
  fn test_validate_any_numeric() {
    let rule = Rule::<i32>::Equals(0).or(Rule::Equals(100));
    assert!(rule.validate_step(0).is_ok());
    assert!(rule.validate_step(100).is_ok());
    assert!(rule.validate_step(50).is_err());
  }

  #[test]
  fn test_validate_not_numeric() {
    let rule = Rule::<i32>::Min(0).not();
    assert!(rule.validate_step(-1).is_ok()); // Below 0, so NOT passes
    assert!(rule.validate_step(0).is_err()); // At 0, Min passes, so NOT fails
  }

  #[test]
  fn test_validate_with_message_numeric() {
    let rule = Rule::<i32>::Min(0).with_message("Must be non-negative.");
    let result = rule.validate_step(-5);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(violation.message(), "Must be non-negative.");
  }

  #[test]
  fn test_validate_all_numeric_multiple() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(10)).and(Rule::Step(3));

    assert!(rule.validate_step_all(6).is_ok());

    // 15 is > 10 and not a multiple of 3
    let result = rule.validate_step_all(15);
    assert!(result.is_err());
  }

  // ========================================================================
  // Option Validation (Numeric) Tests
  // ========================================================================

  #[test]
  fn test_validate_option_numeric_none() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate_option_step(None).is_err());

    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    assert!(rule.validate_option_step(None).is_err());

    let rule = Rule::<f64>::Step(0.5);
    assert!(rule.validate_option_step(None).is_err());
  }

  #[test]
  fn test_validate_option_numeric_some_valid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_option_step(Some(50)).is_ok());
  }

  #[test]
  fn test_validate_option_numeric_some_invalid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_option_step(Some(-5)).is_err());
    assert!(rule.validate_option_step(Some(150)).is_err());
  }

  #[test]
  fn test_validate_option_all_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100)).and(Rule::Step(10));

    assert!(rule.validate_option_step_all(None).is_err());
    assert!(rule.validate_option_step_all(Some(50)).is_ok());

    let result = rule.validate_option_step_all(Some(55));
    assert!(result.is_err());
  }

  // ========================================================================
  // Condition Evaluation Tests
  // ========================================================================

  #[test]
  fn test_condition_evaluate_numeric() {
    let gt = Condition::<i32>::GreaterThan(10);
    assert!(gt.evaluate(&15));
    assert!(!gt.evaluate(&5));

    let lt = Condition::<i32>::LessThan(10);
    assert!(lt.evaluate(&5));
    assert!(!lt.evaluate(&15));

    let eq = Condition::<i32>::Equals(42);
    assert!(eq.evaluate(&42));
    assert!(!eq.evaluate(&0));
  }

  // ========================================================================
  // ValidateRef<T> (non-Option) Tests
  // ========================================================================

  #[test]
  fn test_validate_ref_i32() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_ref(&50).is_ok());
    assert!(rule.validate_ref(&0).is_ok());
    assert!(rule.validate_ref(&(-1)).is_err());
    assert!(rule.validate_ref(&101).is_err());
  }

  #[test]
  fn test_validate_ref_f64() {
    let rule = Rule::<f64>::Min(0.0).and(Rule::Max(1.0));
    assert!(rule.validate_ref(&0.5).is_ok());
    assert!(rule.validate_ref(&(-0.1)).is_err());
    assert!(rule.validate_ref(&1.1).is_err());
  }

  #[test]
  fn test_validate_ref_u64() {
    let rule = Rule::<u64>::Min(10).and(Rule::Max(20));
    assert!(rule.validate_ref(&15).is_ok());
    assert!(rule.validate_ref(&5).is_err());
  }

  // ========================================================================
  // Option<T> Validation (Numeric)
  // ========================================================================

  #[test]
  fn test_option_none_required_i32() {
    let rule = Rule::<i32>::Required;
    assert!(rule.validate(None::<i32>).is_err());
    assert!(rule.validate_ref(&None::<i32>).is_err());
  }

  #[test]
  fn test_option_none_not_required_i32() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate(None::<i32>).is_ok());
    assert!(rule.validate_ref(&None::<i32>).is_ok());
  }

  #[test]
  fn test_option_some_valid_i32() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate(Some(50)).is_ok());
    assert!(rule.validate_ref(&Some(50)).is_ok());
  }

  #[test]
  fn test_option_some_invalid_i32() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate(Some(-1)).is_err());
    assert!(rule.validate_ref(&Some(-1)).is_err());
  }

  #[test]
  fn test_option_none_all_with_required() {
    let rule = Rule::<i32>::Required.and(Rule::Min(0));
    assert!(rule.validate(None::<i32>).is_err());
  }

  #[test]
  fn test_option_none_all_without_required() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate(None::<i32>).is_ok());
  }

  #[test]
  fn test_option_some_valid_f64() {
    let rule = Rule::<f64>::Min(0.0).and(Rule::Max(1.0));
    assert!(rule.validate(Some(0.5)).is_ok());
    assert!(rule.validate_ref(&Some(0.5)).is_ok());
  }

  #[test]
  fn test_option_some_invalid_f64() {
    let rule = Rule::<f64>::Min(0.0);
    assert!(rule.validate(Some(-0.1)).is_err());
    assert!(rule.validate_ref(&Some(-0.1)).is_err());
  }

  // ========================================================================
  // Rule::Ref tests (#143)
  // ========================================================================

  #[test]
  fn test_validate_step_ref_returns_err() {
    let rule = Rule::<i32>::Ref("step_ref".into());
    let result = rule.validate_step(10);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violation_type(), crate::ViolationType::CustomError);
    assert!(err.message().contains("step_ref"));
  }

  #[test]
  fn test_validate_ref_trait_ref_returns_err() {
    let rule = Rule::<i32>::Ref("step_ref".into());
    let result = rule.validate(10);
    assert!(result.is_err());
    let result_ref = rule.validate_ref(&10);
    assert!(result_ref.is_err());
    let err = result_ref.unwrap_err();
    assert_eq!(err.violation_type(), crate::ViolationType::CustomError);
    assert!(err.message().contains("step_ref"));
  }

  #[test]
  fn test_validate_step_ref_inside_all() {
    let rule = Rule::<i32>::All(vec![Rule::Min(0), Rule::Ref("step_ref".into())]);
    assert!(rule.validate_step(10).is_err());
  }

  #[test]
  fn test_validate_step_ref_inside_any() {
    let rule = Rule::<i32>::Any(vec![Rule::Ref("step_ref".into()), Rule::Min(0)]);
    assert!(rule.validate_step(10).is_ok());

    let rule_all_fail = Rule::<i32>::Any(vec![Rule::Ref("step_ref".into()), Rule::Min(100)]);
    assert!(rule_all_fail.validate_step(10).is_err());
  }

  #[test]
  fn test_validate_step_ref_inside_not() {
    let rule = Rule::<i32>::Not(Box::new(Rule::Ref("step_ref".into())));
    assert!(rule.validate_step(10).is_ok());
  }

  // ========================================================================
  // Deeply nested combinator tests (#145)
  // ========================================================================

  #[test]
  fn test_nested_depth2_all_containing_all_and_any() {
    // All([All([Min(0), Max(100)]), Any([Equals(50), Equals(75)])])
    let rule = Rule::<i32>::All(vec![
      Rule::All(vec![Rule::Min(0), Rule::Max(100)]),
      Rule::Any(vec![Rule::Equals(50), Rule::Equals(75)]),
    ]);
    assert!(rule.validate_step(50).is_ok());
    assert!(rule.validate_step(75).is_ok());
    assert!(rule.validate_step(25).is_err());
  }

  #[test]
  fn test_nested_depth2_when_with_nested_all_then() {
    // When { condition: GreaterThan(0), then: All([Min(1), Max(10), Step(1)]), else: None }
    let rule = Rule::<i32>::When {
      condition: crate::rule::Condition::GreaterThan(0),
      then_rule: Box::new(Rule::All(vec![Rule::Min(1), Rule::Max(10), Rule::Step(1)])),
      else_rule: None,
    };
    assert!(rule.validate_step(5).is_ok());
    assert!(rule.validate_step(11).is_err());
    assert!(rule.validate_step(0).is_ok()); // condition false → pass
  }

  #[test]
  fn test_nested_depth3_not_all_any() {
    // Not(All([Any([Equals(1), Equals(2)]), Min(0)]))
    // Value 1: Any passes, Min passes → All passes → Not fails
    // Value 5: Any fails → All fails → Not passes
    let rule = Rule::<i32>::Not(Box::new(Rule::All(vec![
      Rule::Any(vec![Rule::Equals(1), Rule::Equals(2)]),
      Rule::Min(0),
    ])));
    assert!(rule.validate_step(1).is_err());
    assert!(rule.validate_step(5).is_ok());
  }

  #[test]
  fn test_nested_any_with_not_and_all() {
    // Any([Not(Min(0)), All([Min(0), Max(10)])])
    // -1: Not(Min(0)) passes → Any passes
    // 5: All passes → Any passes
    // 15: Not fails, All fails → Any fails
    let rule = Rule::<i32>::Any(vec![
      Rule::Not(Box::new(Rule::Min(0))),
      Rule::All(vec![Rule::Min(0), Rule::Max(10)]),
    ]);
    assert!(rule.validate_step(-1).is_ok());
    assert!(rule.validate_step(5).is_ok());
    assert!(rule.validate_step(15).is_err());
  }

  #[test]
  fn test_nested_depth3_when_else_any_not() {
    // When { condition: GreaterThan(50), then: Max(100),
    //        else: Any([Not(Min(0)), Equals(25)]) }
    let rule = Rule::<i32>::When {
      condition: crate::rule::Condition::GreaterThan(50),
      then_rule: Box::new(Rule::Max(100)),
      else_rule: Some(Box::new(Rule::Any(vec![
        Rule::Not(Box::new(Rule::Min(0))),
        Rule::Equals(25),
      ]))),
    };
    assert!(rule.validate_step(75).is_ok());
    assert!(rule.validate_step(101).is_err());
    assert!(rule.validate_step(25).is_ok());
    assert!(rule.validate_step(-1).is_ok());
    assert!(rule.validate_step(10).is_err());
  }

  #[test]
  fn test_empty_any_passes() {
    let rule = Rule::<i32>::Any(vec![]);
    assert!(rule.validate_step(42).is_ok());
  }

  #[cfg(feature = "async")]
  mod async_option_tests {
    use crate::rule::Rule;
    use crate::{ValidateAsync, ValidateRefAsync};

    #[tokio::test]
    async fn test_async_option_none_required() {
      let rule = Rule::<i32>::Required;
      assert!(rule.validate_async(None::<i32>).await.is_err());
      assert!(rule.validate_ref_async(&None::<i32>).await.is_err());
    }

    #[tokio::test]
    async fn test_async_option_none_not_required() {
      let rule = Rule::<i32>::Min(0);
      assert!(rule.validate_async(None::<i32>).await.is_ok());
      assert!(rule.validate_ref_async(&None::<i32>).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_option_some_valid() {
      let rule = Rule::<i32>::Min(0);
      assert!(rule.validate_async(Some(5)).await.is_ok());
      assert!(rule.validate_ref_async(&Some(5)).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_validate_ref_i32() {
      let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
      assert!(rule.validate_ref_async(&50).await.is_ok());
      assert!(rule.validate_ref_async(&(-1)).await.is_err());
      assert!(rule.validate_ref_async(&101).await.is_err());
    }

    #[tokio::test]
    async fn test_async_validate_ref_f64() {
      let rule = Rule::<f64>::Min(0.0).and(Rule::Max(1.0));
      assert!(rule.validate_ref_async(&0.5).await.is_ok());
      assert!(rule.validate_ref_async(&(-0.1)).await.is_err());
    }

    #[tokio::test]
    async fn test_async_validate_i32() {
      let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
      assert!(rule.validate_async(50).await.is_ok());
      assert!(rule.validate_async(-1).await.is_err());
    }
  }
}
