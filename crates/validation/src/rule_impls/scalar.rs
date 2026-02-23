use crate::rule::{Rule, RuleResult};
use crate::traits::{IsEmpty, Validate};
use crate::{ScalarValue, Violation, Violations};

impl<T: ScalarValue + IsEmpty> Rule<T> {
  /// Validates a scalar value against this rule.
  pub fn validate_scalar(&self, value: T) -> RuleResult {
    self.validate_scalar_inner(value, None)
  }

  /// Internal validation with inherited locale from an outer `WithMessage`.
  ///
  /// The `inherited_locale` is passed down through the rule tree so that inner
  /// `WithMessage` nodes can use it when their own locale is `None`.
  fn validate_scalar_inner(&self, value: T, inherited_locale: Option<&str>) -> RuleResult {
    match self {
      // Scalar values are always present — Required is a no-op here.
      Rule::Required => Ok(()),

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
          rule.validate_scalar_inner(value, inherited_locale)?;
        }
        Ok(())
      }

      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_scalar_inner(value, inherited_locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }

      Rule::Not(inner) => match inner.validate_scalar_inner(value, inherited_locale) {
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
          then_rule.validate_scalar_inner(value, inherited_locale)
        } else {
          match else_rule {
            Some(rule) => rule.validate_scalar_inner(value, inherited_locale),
            None => Ok(()),
          }
        }
      }

      Rule::Custom(f) => f(&value),

      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),

      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let effective_locale = locale.as_deref().or(inherited_locale);
        match rule.validate_scalar_inner(value, effective_locale) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg =
              message.resolve_or(&value, violation.message(), effective_locale);
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
        }
      }

      // Step and string-only rules are pass-through for scalar types.
      Rule::Step(_)
      | Rule::MinLength(_)
      | Rule::MaxLength(_)
      | Rule::ExactLength(_)
      | Rule::Pattern(_)
      | Rule::Email
      | Rule::Url(_)
      | Rule::Uri(_)
      | Rule::Ip(_) => Ok(()),
    }
  }

  /// Validates a scalar value and collects *all* violations (fail-slow).
  ///
  /// Returns `Ok(())` when every rule passes, or `Err(Violations)` containing
  /// every failure discovered during tree traversal.
  pub fn validate_scalar_all(&self, value: T) -> Result<(), Violations> {
    let mut violations = Violations::default();
    self.collect_violations_scalar(value, None, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional scalar value.
  ///
  /// `None` is treated as a missing value and fails only when this rule
  /// (or any nested rule) contains `Required`.
  pub fn validate_scalar_option(&self, value: Option<T>) -> RuleResult {
    match value {
      Some(v) => self.validate_scalar(v),
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
    }
  }

  /// Validates an optional scalar value and collects all violations.
  pub fn validate_scalar_option_all(&self, value: Option<T>) -> Result<(), Violations> {
    match value {
      Some(v) => self.validate_scalar_all(v),
      None if self.requires_value() => Err(Violations::from(Violation::value_missing())),
      None => Ok(()),
    }
  }

  /// Recursively collects all violations into `violations` (fail-slow traversal).
  fn collect_violations_scalar(
    &self,
    value: T,
    inherited_locale: Option<&str>,
    violations: &mut Violations,
  ) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_violations_scalar(value, inherited_locale, violations);
        }
      }

      Rule::Any(rules) => {
        let mut any_violations = Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = Violations::default();
          rule.collect_violations_scalar(value, inherited_locale, &mut rule_violations);
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
          then_rule.collect_violations_scalar(value, inherited_locale, violations);
        } else if let Some(rule) = else_rule {
          rule.collect_violations_scalar(value, inherited_locale, violations);
        }
      }

      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let effective_locale = locale.as_deref().or(inherited_locale);
        let mut inner_violations = Violations::default();
        rule.collect_violations_scalar(value, effective_locale, &mut inner_violations);
        for violation in inner_violations {
          let custom_msg =
            message.resolve_or(&value, violation.message(), effective_locale);
          violations.push(Violation::new(violation.violation_type(), custom_msg));
        }
      }

      _ => {
        if let Err(v) = self.validate_scalar_inner(value, inherited_locale) {
          violations.push(v);
        }
      }
    }
  }
}

// `Validate<T>` for numeric ScalarValue types is covered by the SteppableValue
// impl in steppable.rs.  Here we only add the two ScalarValue types that are
// *not* SteppableValue so every ScalarValue type has a Validate impl.

impl Validate<bool> for Rule<bool> {
  fn validate(&self, value: bool) -> crate::traits::ValidatorResult {
    self.validate_scalar(value)
  }
}

impl Validate<char> for Rule<char> {
  fn validate(&self, value: char) -> crate::traits::ValidatorResult {
    self.validate_scalar(value)
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use crate::rule::{Condition, Rule};

  // ==========================================================================
  // Min / Max / Range
  // ==========================================================================

  #[test]
  fn test_validate_scalar_min() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate_scalar(0).is_ok());
    assert!(rule.validate_scalar(100).is_ok());
    assert!(rule.validate_scalar(-1).is_err());
  }

  #[test]
  fn test_validate_scalar_max() {
    let rule = Rule::<i32>::Max(100);
    assert!(rule.validate_scalar(100).is_ok());
    assert!(rule.validate_scalar(0).is_ok());
    assert!(rule.validate_scalar(101).is_err());
  }

  #[test]
  fn test_validate_scalar_range() {
    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    assert!(rule.validate_scalar(0).is_ok());
    assert!(rule.validate_scalar(50).is_ok());
    assert!(rule.validate_scalar(100).is_ok());
    assert!(rule.validate_scalar(-1).is_err());
    assert!(rule.validate_scalar(101).is_err());
  }

  // ==========================================================================
  // Equals / OneOf
  // ==========================================================================

  #[test]
  fn test_validate_scalar_equals() {
    let rule = Rule::<i32>::Equals(42);
    assert!(rule.validate_scalar(42).is_ok());
    assert!(rule.validate_scalar(0).is_err());
  }

  #[test]
  fn test_validate_scalar_one_of() {
    let rule = Rule::<i32>::OneOf(vec![1, 2, 3]);
    assert!(rule.validate_scalar(1).is_ok());
    assert!(rule.validate_scalar(3).is_ok());
    assert!(rule.validate_scalar(4).is_err());
  }

  // ==========================================================================
  // All / Any / Not
  // ==========================================================================

  #[test]
  fn test_validate_scalar_all_combinator() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_scalar(50).is_ok());
    assert!(rule.validate_scalar(-1).is_err());
    assert!(rule.validate_scalar(101).is_err());
  }

  #[test]
  fn test_validate_scalar_any_combinator() {
    let rule = Rule::<i32>::Equals(0).or(Rule::Equals(100));
    assert!(rule.validate_scalar(0).is_ok());
    assert!(rule.validate_scalar(100).is_ok());
    assert!(rule.validate_scalar(50).is_err());
  }

  #[test]
  fn test_validate_scalar_not() {
    let rule = Rule::<i32>::Min(0).not();
    assert!(rule.validate_scalar(-1).is_ok()); // fails Min(0) → NOT passes
    assert!(rule.validate_scalar(0).is_err());  // passes Min(0) → NOT fails
  }

  // ==========================================================================
  // When / Condition
  // ==========================================================================

  #[test]
  fn test_validate_scalar_when() {
    // When value > 0 it must be <= 10.
    let rule = Rule::<i32>::When {
      condition: Condition::GreaterThan(0),
      then_rule: Box::new(Rule::Max(10)),
      else_rule: None,
    };
    assert!(rule.validate_scalar(0).is_ok());    // condition false → skip
    assert!(rule.validate_scalar(5).is_ok());    // condition true, 5 <= 10
    assert!(rule.validate_scalar(11).is_err());  // condition true, 11 > 10
  }

  #[test]
  fn test_validate_scalar_when_else() {
    // When value > 50 → must equal 100, else → must equal 0.
    let rule = Rule::<i32>::When {
      condition: Condition::GreaterThan(50),
      then_rule: Box::new(Rule::Equals(100)),
      else_rule: Some(Box::new(Rule::Equals(0))),
    };
    assert!(rule.validate_scalar(100).is_ok()); // then branch passes
    assert!(rule.validate_scalar(0).is_ok());   // else branch passes
    assert!(rule.validate_scalar(50).is_err()); // else branch: 50 ≠ 0
    assert!(rule.validate_scalar(75).is_err()); // then branch: 75 ≠ 100
  }

  // ==========================================================================
  // WithMessage — custom message and locale propagation
  // ==========================================================================

  #[test]
  fn test_validate_scalar_with_message() {
    let rule = Rule::<i32>::Min(0).with_message("Must be non-negative.");
    let result = rule.validate_scalar(-5);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Must be non-negative.");
  }

  #[test]
  fn test_validate_scalar_with_message_passes_through_ok() {
    let rule = Rule::<i32>::Min(0).with_message("Must be non-negative.");
    assert!(rule.validate_scalar(5).is_ok());
  }

  #[test]
  fn test_validate_scalar_with_message_nested_all() {
    // WithMessage wrapping an All — all collected violations get the custom msg.
    let rule = Rule::<i32>::WithMessage {
      rule: Box::new(Rule::Min(0).and(Rule::Max(10))),
      message: crate::Message::from("Out of range."),
      locale: None,
    };
    assert!(rule.validate_scalar(5).is_ok());
    let err = rule.validate_scalar(-1).unwrap_err();
    assert_eq!(err.message(), "Out of range.");
  }

  // ==========================================================================
  // Step — pass-through for scalar impl
  // ==========================================================================

  #[test]
  fn test_validate_scalar_step_passthrough() {
    let rule = Rule::<i32>::Step(3);
    assert!(rule.validate_scalar(1).is_ok());
    assert!(rule.validate_scalar(7).is_ok());
  }

  // ==========================================================================
  // Required — no-op for non-Option scalars
  // ==========================================================================

  #[test]
  fn test_validate_scalar_required_noop() {
    let rule = Rule::<i32>::Required;
    assert!(rule.validate_scalar(0).is_ok());
    assert!(rule.validate_scalar(-1).is_ok());
  }

  // ==========================================================================
  // validate_scalar_all — fail-slow collection
  // ==========================================================================

  #[test]
  fn test_validate_scalar_all_collects_multiple() {
    // Min(0) AND Max(10) AND Equals(5): value -1 violates Min and Equals.
    let rule = Rule::<i32>::Min(0).and(Rule::Max(10)).and(Rule::Equals(5));
    let result = rule.validate_scalar_all(-1);
    assert!(result.is_err());
    // -1 < 0 (Min fails) AND -1 ≠ 5 (Equals fails) → 2 violations
    assert_eq!(result.unwrap_err().0.len(), 2);
  }

  // ==========================================================================
  // Option variants
  // ==========================================================================

  #[test]
  fn test_validate_scalar_option_some_valid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_scalar_option(Some(50)).is_ok());
  }

  #[test]
  fn test_validate_scalar_option_some_invalid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_scalar_option(Some(-1)).is_err());
  }

  #[test]
  fn test_validate_scalar_option_none_without_required() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate_scalar_option(None).is_ok());
  }

  #[test]
  fn test_validate_scalar_option_none_with_required() {
    let rule = Rule::<i32>::Required;
    assert!(rule.validate_scalar_option(None).is_err());
  }

  #[test]
  fn test_validate_scalar_option_all_none_required() {
    let rule = Rule::<i32>::Required;
    assert!(rule.validate_scalar_option_all(None).is_err());
  }

  #[test]
  fn test_validate_scalar_option_all_some_collects_violations() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(10)).and(Rule::Equals(5));
    let result = rule.validate_scalar_option_all(Some(-1));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0.len(), 2);
  }

  // ==========================================================================
  // bool and char — non-numeric ScalarValue types
  // ==========================================================================

  #[test]
  fn test_validate_scalar_bool_equals() {
    let rule = Rule::<bool>::Equals(true);
    assert!(rule.validate_scalar(true).is_ok());
    assert!(rule.validate_scalar(false).is_err());
  }

  #[test]
  fn test_validate_scalar_char_range() {
    let rule = Rule::<char>::Range { min: 'a', max: 'z' };
    assert!(rule.validate_scalar('m').is_ok());
    assert!(rule.validate_scalar('a').is_ok());
    assert!(rule.validate_scalar('z').is_ok());
    assert!(rule.validate_scalar('A').is_err());
  }
}
