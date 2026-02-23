use crate::rule::{Rule, RuleResult};
use crate::traits::{IsEmpty, Validate};
use crate::{SteppableValue, Violation};
use crate::CompiledRule;

impl<T: SteppableValue + IsEmpty> Rule<T> {
  /// Validates a numeric value against this rule.
  pub fn validate_step(&self, value: T) -> RuleResult {
    self.validate_step_inner(value, None)
  }

  /// Internal validation with inherited locale from an outer `WithMessage`.
  fn validate_step_inner(&self, value: T, inherited_locale: Option<&str>) -> RuleResult {
    match self {
      Rule::Required => {
        // Numeric values are always "present"
        Ok(())
      }
      Rule::Min(min) => {
        if value < *min {
          Err(Violation::range_underflow(min))
        } else {
          Ok(())
        }
      }
      Rule::Max(max) => {
        if value > *max {
          Err(Violation::range_overflow(max))
        } else {
          Ok(())
        }
      }
      Rule::Range { min, max } => {
        if value < *min {
          Err(Violation::range_underflow(min))
        } else if value > *max {
          Err(Violation::range_overflow(max))
        } else {
          Ok(())
        }
      }
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
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage { rule, message, locale } => {
        let effective_locale = locale.as_deref().or(inherited_locale);
        match rule.validate_step_inner(value, effective_locale) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg = message.resolve_or(&value, violation.message(), effective_locale);
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
        }
      },
      // String rules don't apply to numbers - pass through
      Rule::MinLength(_)
      | Rule::MaxLength(_)
      | Rule::ExactLength(_)
      | Rule::Pattern(_)
      | Rule::Email
      | Rule::Url
      | Rule::Uri(_)
      | Rule::Ip(_) => Ok(()),
    }
  }

  /// Validates a numeric value and collects all violations.
  pub fn validate_step_all(&self, value: T) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_violations(value, None, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional numeric value.
  pub fn validate_option_step(&self, value: Option<T>) -> RuleResult {
    match value {
      Some(v) => self.validate_step(v),
      None => Err(Violation::value_missing()),
    }
  }

  /// Validates an optional numeric value and collects all violations.
  pub fn validate_option_step_all(
    &self,
    value: Option<T>,
  ) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_step_all(v),
      None => Err(crate::Violations::from(Violation::value_missing())),
    }
  }

  /// Helper to collect all violations recursively.
  fn collect_violations(&self, value: T, inherited_locale: Option<&str>, violations: &mut crate::Violations) {
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
          any_violations.extend(rule_violations.into_iter());
        }
        if !any_passed && !rules.is_empty() {
          violations.extend(any_violations.into_iter());
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
      Rule::WithMessage { rule, message, locale } => {
        let effective_locale = locale.as_deref().or(inherited_locale);
        let mut inner_violations = crate::Violations::default();
        rule.collect_violations(value, effective_locale, &mut inner_violations);
        for violation in inner_violations {
          let custom_msg = message.resolve_or(&value, violation.message(), effective_locale);
          violations.push(Violation::new(violation.violation_type(), custom_msg));
        }
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

impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for CompiledRule<T> {
  fn validate(&self, value: T) -> crate::ValidatorResult {
    self.rule.validate_step(value)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> CompiledRule<T> {
  /// Validates a numeric value using the compiled rule.
  pub fn validate_step(&self, value: T) -> RuleResult {
    self.rule.validate_step(value)
  }

  /// Validates a numeric value and collects all violations.
  pub fn validate_step_all(&self, value: T) -> Result<(), crate::Violations> {
    self.rule.validate_step_all(value)
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
  // CompiledRule (Numeric) Tests
  // ========================================================================

  #[test]
  fn test_compiled_rule_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    let compiled = rule.compile();

    assert!(compiled.validate_step(50).is_ok());
    assert!(compiled.validate_step(0).is_ok());
    assert!(compiled.validate_step(100).is_ok());
    assert!(compiled.validate_step(-1).is_err());
    assert!(compiled.validate_step(101).is_err());
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

