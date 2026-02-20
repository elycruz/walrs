use crate::rule::{
    Rule, RuleResult, IsEmpty,
    value_missing_violation, range_underflow_violation, range_overflow_violation,
    step_mismatch_violation, not_equal_violation, not_one_of_violation, negation_failed_violation, unresolved_ref_violation
};
use crate::{SteppableValue, Violation};
use crate::traits::Validate;
use crate::CompiledRule;

impl<T: SteppableValue + IsEmpty> Rule<T> {
  /// Validates a numeric value against this rule.
  pub fn validate(&self, value: T, locale: Option<&str>) -> RuleResult {
    match self {
      Rule::Required => {
        // Numeric values are always "present"
        Ok(())
      }
      Rule::Min(min) => {
        if value < *min {
          Err(range_underflow_violation(min))
        } else {
          Ok(())
        }
      }
      Rule::Max(max) => {
        if value > *max {
          Err(range_overflow_violation(max))
        } else {
          Ok(())
        }
      }
      Rule::Range { min, max } => {
        if value < *min {
          Err(range_underflow_violation(min))
        } else if value > *max {
          Err(range_overflow_violation(max))
        } else {
          Ok(())
        }
      }
      Rule::Step(step) => {
        if value.rem_check(*step) {
          Ok(())
        } else {
          Err(step_mismatch_violation(step))
        }
      }
      Rule::Equals(expected) => {
        if value == *expected {
          Ok(())
        } else {
          Err(not_equal_violation(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.contains(&value) {
          Ok(())
        } else {
          Err(not_one_of_violation())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate(value, locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate(value, locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate(value, locale) {
        Ok(()) => Err(negation_failed_violation()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate(&value);
        if should_apply {
          then_rule.validate(value, locale)
        } else {
          match else_rule {
            Some(rule) => rule.validate(value, locale),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value),
      Rule::Ref(name) => Err(unresolved_ref_violation(name)),
      Rule::WithMessage { rule, message } => match rule.validate(value, locale) {
        Ok(()) => Ok(()),
        Err(violation) => {
          let custom_msg = message.resolve(&value, locale);
          Err(Violation::new(violation.violation_type(), custom_msg))
        }
      },
      // String rules don't apply to numbers - pass through
      Rule::MinLength(_)
      | Rule::MaxLength(_)
      | Rule::ExactLength(_)
      | Rule::Pattern(_)
      | Rule::Email
      | Rule::Url => Ok(()),
    }
  }

  /// Validates a numeric value and collects all violations.
  pub fn validate_all(&self, value: T, locale: Option<&str>) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_violations(value, locale, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional numeric value.
  pub fn validate_option(&self, value: Option<T>, locale: Option<&str>) -> RuleResult {
    match value {
      Some(v) => self.validate(v, locale),
      None => Err(value_missing_violation()),
    }
  }

  /// Validates an optional numeric value and collects all violations.
  pub fn validate_option_all(
    &self,
    value: Option<T>,
    locale: Option<&str>,
  ) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_all(v, locale),
      None => Err(crate::Violations::from(value_missing_violation())),
    }
  }

  /// Helper to collect all violations recursively.
  fn collect_violations(&self, value: T, locale: Option<&str>, violations: &mut crate::Violations) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_violations(value, locale, violations);
        }
      }
      Rule::Any(rules) => {
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_violations(value, locale, &mut rule_violations);
          if rule_violations.is_empty() {
            any_passed = true;
            break;
          }
          any_violations.extend(rule_violations.into_iter());
        }
        if !any_passed && !rules.is_empty() {
          if let Some(v) = any_violations.0.pop() {
            violations.push(v);
          }
        }
      }
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate(&value);
        if should_apply {
          then_rule.collect_violations(value, locale, violations);
        } else if let Some(rule) = else_rule {
          rule.collect_violations(value, locale, violations);
        }
      }
      Rule::WithMessage { rule, message } => {
        let mut inner_violations = crate::Violations::default();
        rule.collect_violations(value, locale, &mut inner_violations);
        for violation in inner_violations {
          let custom_msg = message.resolve(&value, locale);
          violations.push(Violation::new(violation.violation_type(), custom_msg));
        }
      }
      _ => {
        if let Err(v) = self.validate(value, locale) {
          violations.push(v);
        }
      }
    }
  }
}

impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for Rule<T> {
  fn validate(&self, value: T) -> crate::ValidatorResult {
    Rule::validate(self, value, None)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for CompiledRule<T> {
  fn validate(&self, value: T) -> crate::ValidatorResult {
    self.rule.validate(value, None)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> CompiledRule<T> {
  /// Validates a numeric value using the compiled rule.
  pub fn validate(&self, value: T) -> RuleResult {
    self.rule.validate(value, None)
  }

  /// Validates a numeric value and collects all violations.
  pub fn validate_all(&self, value: T) -> Result<(), crate::Violations> {
    self.rule.validate_all(value, None)
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use crate::rule::{Condition, Rule};

  // ========================================================================
  // Numeric Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_min() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate(0, None).is_ok());
    assert!(rule.validate(100, None).is_ok());
    assert!(rule.validate(-1, None).is_err());
  }

  #[test]
  fn test_validate_max() {
    let rule = Rule::<i32>::Max(100);
    assert!(rule.validate(100, None).is_ok());
    assert!(rule.validate(0, None).is_ok());
    assert!(rule.validate(101, None).is_err());
  }

  #[test]
  fn test_validate_range() {
    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    assert!(rule.validate(0, None).is_ok());
    assert!(rule.validate(50, None).is_ok());
    assert!(rule.validate(100, None).is_ok());
    assert!(rule.validate(-1, None).is_err());
    assert!(rule.validate(101, None).is_err());
  }

  #[test]
  fn test_validate_step() {
    let rule = Rule::<i32>::Step(5);
    assert!(rule.validate(0, None).is_ok());
    assert!(rule.validate(5, None).is_ok());
    assert!(rule.validate(10, None).is_ok());
    assert!(rule.validate(3, None).is_err());
  }

  #[test]
  fn test_validate_step_float() {
    let rule = Rule::<f64>::Step(0.5);
    assert!(rule.validate(0.0, None).is_ok());
    assert!(rule.validate(0.5, None).is_ok());
    assert!(rule.validate(1.0, None).is_ok());
    assert!(rule.validate(0.3, None).is_err());
  }

  #[test]
  fn test_validate_equals_numeric() {
    let rule = Rule::<i32>::Equals(42);
    assert!(rule.validate(42, None).is_ok());
    assert!(rule.validate(0, None).is_err());
  }

  #[test]
  fn test_validate_one_of_numeric() {
    let rule = Rule::<i32>::OneOf(vec![1, 2, 3]);
    assert!(rule.validate(1, None).is_ok());
    assert!(rule.validate(2, None).is_ok());
    assert!(rule.validate(4, None).is_err());
  }

  #[test]
  fn test_validate_all_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100)).and(Rule::Step(10));
    assert!(rule.validate(50, None).is_ok());
    assert!(rule.validate(55, None).is_err()); // Not step of 10
    assert!(rule.validate(-10, None).is_err()); // Below min
  }

  #[test]
  fn test_validate_any_numeric() {
    let rule = Rule::<i32>::Equals(0).or(Rule::Equals(100));
    assert!(rule.validate(0, None).is_ok());
    assert!(rule.validate(100, None).is_ok());
    assert!(rule.validate(50, None).is_err());
  }

  #[test]
  fn test_validate_not_numeric() {
    let rule = Rule::<i32>::Min(0).not();
    assert!(rule.validate(-1, None).is_ok()); // Below 0, so NOT passes
    assert!(rule.validate(0, None).is_err()); // At 0, Min passes, so NOT fails
  }

  #[test]
  fn test_validate_with_message_numeric() {
    let rule = Rule::<i32>::Min(0).with_message("Must be non-negative.");
    let result = rule.validate(-5, None);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(violation.message(), "Must be non-negative.");
  }

  #[test]
  fn test_validate_all_numeric_multiple() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(10)).and(Rule::Step(3));

    assert!(rule.validate_all(6, None).is_ok());

    // 15 is > 10 and not a multiple of 3
    let result = rule.validate_all(15, None);
    assert!(result.is_err());
  }

  // ========================================================================
  // Option Validation (Numeric) Tests
  // ========================================================================

  #[test]
  fn test_validate_option_numeric_none() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate_option(None, None).is_err());

    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    assert!(rule.validate_option(None, None).is_err());

    let rule = Rule::<f64>::Step(0.5);
    assert!(rule.validate_option(None, None).is_err());
  }

  #[test]
  fn test_validate_option_numeric_some_valid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_option(Some(50), None).is_ok());
  }

  #[test]
  fn test_validate_option_numeric_some_invalid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_option(Some(-5), None).is_err());
    assert!(rule.validate_option(Some(150), None).is_err());
  }

  #[test]
  fn test_validate_option_all_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100)).and(Rule::Step(10));

    assert!(rule.validate_option_all(None, None).is_err());
    assert!(rule.validate_option_all(Some(50), None).is_ok());

    let result = rule.validate_option_all(Some(55), None);
    assert!(result.is_err());
  }

  // ========================================================================
  // CompiledRule (Numeric) Tests
  // ========================================================================

  #[test]
  fn test_compiled_rule_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    let compiled = rule.compile();

    assert!(compiled.validate(50).is_ok());
    assert!(compiled.validate(0).is_ok());
    assert!(compiled.validate(100).is_ok());
    assert!(compiled.validate(-1).is_err());
    assert!(compiled.validate(101).is_err());
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
}

