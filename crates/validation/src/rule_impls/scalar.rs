use std::cmp::Ordering;

use crate::rule::{Rule, RuleResult};
use crate::traits::{IsEmpty, Validate, ValidateRef};
use crate::{ScalarValue, Violation, ViolationType, Violations};

impl<T: ScalarValue + IsEmpty> Rule<T> {
  /// Validates a scalar value against this rule.
  pub(crate) fn validate_scalar(&self, value: T) -> RuleResult {
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

      #[cfg(feature = "async")]
      Rule::CustomAsync(_) => Ok(()),

      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),

      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let eff = locale.as_deref().or(inherited_locale);
        match message {
          Some(msg) => msg.wrap_result(rule.validate_scalar_inner(value, eff), &value, eff),
          None => rule.validate_scalar_inner(value, eff),
        }
      }

      // Step and string-only rules are pass-through for scalar types.
      Rule::Step(_)
      | Rule::MinLength(_)
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

  /// Validates a scalar value and collects *all* violations (fail-slow).
  ///
  /// Returns `Ok(())` when every rule passes, or `Err(Violations)` containing
  /// every failure discovered during tree traversal.
  #[allow(dead_code)] // Reserved for a future `validate_all` public API
  pub(crate) fn validate_scalar_all(&self, value: T) -> Result<(), Violations> {
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
  #[allow(dead_code)] // Reserved for a future `validate_option` public API
  pub(crate) fn validate_scalar_option(&self, value: Option<T>) -> RuleResult {
    match value {
      Some(v) => self.validate_scalar(v),
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
    }
  }

  /// Validates an optional scalar value and collects all violations.
  #[allow(dead_code)] // Reserved for a future `validate_option_all` public API
  pub(crate) fn validate_scalar_option_all(&self, value: Option<T>) -> Result<(), Violations> {
    match value {
      Some(v) => self.validate_scalar_all(v),
      None if self.requires_value() => Err(Violations::from(Violation::value_missing())),
      None => Ok(()),
    }
  }

  /// Recursively collects all violations into `violations` (fail-slow traversal).
  #[allow(dead_code)] // Called transitively from validate_scalar_all
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
        let eff = locale.as_deref().or(inherited_locale);
        match message {
          Some(msg) => {
            let mut inner_violations = Violations::default();
            rule.collect_violations_scalar(value, eff, &mut inner_violations);
            msg.wrap_violations(inner_violations, &value, eff, violations);
          }
          None => rule.collect_violations_scalar(value, eff, violations),
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

impl ValidateRef<bool> for Rule<bool> {
  fn validate_ref(&self, value: &bool) -> crate::traits::ValidatorResult {
    self.validate_scalar(*value)
  }
}

impl Validate<char> for Rule<char> {
  fn validate(&self, value: char) -> crate::traits::ValidatorResult {
    self.validate_scalar(value)
  }
}

impl ValidateRef<char> for Rule<char> {
  fn validate_ref(&self, value: &char) -> crate::traits::ValidatorResult {
    self.validate_scalar(*value)
  }
}

impl Validate<Option<bool>> for Rule<bool> {
  fn validate(&self, value: Option<bool>) -> crate::traits::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate(v),
    }
  }
}

impl ValidateRef<Option<bool>> for Rule<bool> {
  fn validate_ref(&self, value: &Option<bool>) -> crate::traits::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate(*v),
    }
  }
}

impl Validate<Option<char>> for Rule<char> {
  fn validate(&self, value: Option<char>) -> crate::traits::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate(v),
    }
  }
}

impl ValidateRef<Option<char>> for Rule<char> {
  fn validate_ref(&self, value: &Option<char>) -> crate::traits::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate(*v),
    }
  }
}

// ============================================================================
// Async Scalar Validation (bool / char)
// ============================================================================

#[cfg(feature = "async")]
impl<T: ScalarValue + IsEmpty + Send + Sync> Rule<T> {
  /// Validates a scalar value asynchronously.
  ///
  /// Runs all rules: sync rules execute inline, `CustomAsync` rules are awaited.
  pub(crate) async fn validate_scalar_async(&self, value: T) -> RuleResult {
    self.validate_scalar_async_inner(value, None).await
  }

  /// Internal async validation with inherited locale.
  fn validate_scalar_async_inner<'a>(
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
              .validate_scalar_async_inner(value, inherited_locale)
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
              .validate_scalar_async_inner(value, inherited_locale)
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
            .validate_scalar_async_inner(value, inherited_locale)
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
              .validate_scalar_async_inner(value, inherited_locale)
              .await
          } else {
            match else_rule {
              Some(rule) => {
                rule
                  .validate_scalar_async_inner(value, inherited_locale)
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
          match message {
            Some(msg) => msg.wrap_result(
              rule.validate_scalar_async_inner(value, eff).await,
              &value,
              eff,
            ),
            None => rule.validate_scalar_async_inner(value, eff).await,
          }
        }

        // All sync rules — delegate to sync validation
        other => other.validate_scalar_inner(value, inherited_locale),
      }
    })
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<bool> for Rule<bool> {
  async fn validate_async(&self, value: bool) -> crate::ValidatorResult {
    self.validate_scalar_async(value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<bool> for Rule<bool> {
  async fn validate_ref_async(&self, value: &bool) -> crate::ValidatorResult {
    self.validate_scalar_async(*value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<Option<bool>> for Rule<bool> {
  async fn validate_async(&self, value: Option<bool>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_scalar_async(v).await,
    }
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<Option<bool>> for Rule<bool> {
  async fn validate_ref_async(&self, value: &Option<bool>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_scalar_async(*v).await,
    }
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<char> for Rule<char> {
  async fn validate_async(&self, value: char) -> crate::ValidatorResult {
    self.validate_scalar_async(value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<char> for Rule<char> {
  async fn validate_ref_async(&self, value: &char) -> crate::ValidatorResult {
    self.validate_scalar_async(*value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<Option<char>> for Rule<char> {
  async fn validate_async(&self, value: Option<char>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_scalar_async(v).await,
    }
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<Option<char>> for Rule<char> {
  async fn validate_ref_async(&self, value: &Option<char>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_scalar_async(*v).await,
    }
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
    assert!(rule.validate_scalar(0).is_err()); // passes Min(0) → NOT fails
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
    assert!(rule.validate_scalar(0).is_ok()); // condition false → skip
    assert!(rule.validate_scalar(5).is_ok()); // condition true, 5 <= 10
    assert!(rule.validate_scalar(11).is_err()); // condition true, 11 > 10
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
    assert!(rule.validate_scalar(0).is_ok()); // else branch passes
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
      message: Some(crate::Message::from("Out of range.")),
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
  // NaN Validation Tests (scalar path)
  // ==========================================================================

  #[test]
  fn test_validate_scalar_nan_f64_min() {
    let rule = Rule::<f64>::Min(0.0);
    assert!(rule.validate_scalar(f64::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f64_max() {
    let rule = Rule::<f64>::Max(100.0);
    assert!(rule.validate_scalar(f64::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f64_range() {
    let rule = Rule::<f64>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_scalar(f64::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f32_min() {
    let rule = Rule::<f32>::Min(0.0);
    assert!(rule.validate_scalar(f32::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f32_max() {
    let rule = Rule::<f32>::Max(100.0);
    assert!(rule.validate_scalar(f32::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f32_range() {
    let rule = Rule::<f32>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_scalar(f32::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_violation_type() {
    use crate::ViolationType;
    let rule = Rule::<f64>::Min(0.0);
    let err = rule.validate_scalar(f64::NAN).unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::TypeMismatch);
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

  // ==========================================================================
  // Option<bool> / Option<char> Validation (trait impls)
  // ==========================================================================

  #[test]
  fn test_option_bool_none_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<bool>::Required;
    assert!(Validate::<Option<bool>>::validate(&rule, None).is_err());
    assert!(ValidateRef::<Option<bool>>::validate_ref(&rule, &None).is_err());
  }

  #[test]
  fn test_option_bool_none_not_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<bool>::Equals(true);
    assert!(Validate::<Option<bool>>::validate(&rule, None).is_ok());
    assert!(ValidateRef::<Option<bool>>::validate_ref(&rule, &None).is_ok());
  }

  #[test]
  fn test_option_bool_some_valid() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<bool>::Equals(true);
    assert!(Validate::<Option<bool>>::validate(&rule, Some(true)).is_ok());
    assert!(ValidateRef::<Option<bool>>::validate_ref(&rule, &Some(true)).is_ok());
  }

  #[test]
  fn test_option_bool_some_invalid() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<bool>::Equals(true);
    assert!(Validate::<Option<bool>>::validate(&rule, Some(false)).is_err());
    assert!(ValidateRef::<Option<bool>>::validate_ref(&rule, &Some(false)).is_err());
  }

  #[test]
  fn test_option_char_none_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<char>::Required;
    assert!(Validate::<Option<char>>::validate(&rule, None).is_err());
    assert!(ValidateRef::<Option<char>>::validate_ref(&rule, &None).is_err());
  }

  #[test]
  fn test_option_char_none_not_required() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<char>::Equals('a');
    assert!(Validate::<Option<char>>::validate(&rule, None).is_ok());
    assert!(ValidateRef::<Option<char>>::validate_ref(&rule, &None).is_ok());
  }

  #[test]
  fn test_option_char_some_valid() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<char>::Equals('a');
    assert!(Validate::<Option<char>>::validate(&rule, Some('a')).is_ok());
    assert!(ValidateRef::<Option<char>>::validate_ref(&rule, &Some('a')).is_ok());
  }

  #[test]
  fn test_option_char_some_invalid() {
    use crate::{Validate, ValidateRef};
    let rule = Rule::<char>::Equals('a');
    assert!(Validate::<Option<char>>::validate(&rule, Some('b')).is_err());
    assert!(ValidateRef::<Option<char>>::validate_ref(&rule, &Some('b')).is_err());
  }

  // ==========================================================================
  // ValidateRef<T> (non-Option) for bool / char
  // ==========================================================================

  #[test]
  fn test_validate_ref_bool() {
    use crate::ValidateRef;
    let rule = Rule::<bool>::Equals(true);
    assert!(ValidateRef::<bool>::validate_ref(&rule, &true).is_ok());
    assert!(ValidateRef::<bool>::validate_ref(&rule, &false).is_err());
  }

  #[test]
  fn test_validate_ref_char() {
    use crate::ValidateRef;
    let rule = Rule::<char>::Range { min: 'a', max: 'z' };
    assert!(ValidateRef::<char>::validate_ref(&rule, &'m').is_ok());
    assert!(ValidateRef::<char>::validate_ref(&rule, &'A').is_err());
  }

  // ==========================================================================
  // NaN Validation Tests — Step, Equals, OneOf (scalar path)
  // ==========================================================================

  #[test]
  fn test_validate_scalar_nan_f64_step() {
    let rule = Rule::<f64>::Step(5.0);
    // Step is a pass-through in scalar validation
    assert!(rule.validate_scalar(f64::NAN).is_ok());
  }

  #[test]
  fn test_validate_scalar_nan_f32_step() {
    let rule = Rule::<f32>::Step(5.0);
    assert!(rule.validate_scalar(f32::NAN).is_ok());
  }

  #[test]
  fn test_validate_scalar_nan_f64_equals() {
    let rule = Rule::<f64>::Equals(42.0);
    assert!(rule.validate_scalar(f64::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f32_equals() {
    let rule = Rule::<f32>::Equals(42.0);
    assert!(rule.validate_scalar(f32::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f64_one_of() {
    let rule = Rule::<f64>::OneOf(vec![1.0, 2.0, 3.0]);
    assert!(rule.validate_scalar(f64::NAN).is_err());
  }

  #[test]
  fn test_validate_scalar_nan_f32_one_of() {
    let rule = Rule::<f32>::OneOf(vec![1.0, 2.0, 3.0]);
    assert!(rule.validate_scalar(f32::NAN).is_err());
  }

  // ==========================================================================
  // INFINITY Validation Tests (scalar path)
  // ==========================================================================

  #[test]
  fn test_validate_scalar_inf_f64_min() {
    let rule = Rule::<f64>::Min(0.0);
    assert!(rule.validate_scalar(f64::INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_inf_f64_max() {
    let rule = Rule::<f64>::Max(100.0);
    assert!(rule.validate_scalar(f64::INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_inf_f64_range() {
    let rule = Rule::<f64>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_scalar(f64::INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_inf_f64_step() {
    let rule = Rule::<f64>::Step(5.0);
    // Step is a pass-through in scalar validation
    assert!(rule.validate_scalar(f64::INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_inf_f64_equals() {
    let rule = Rule::<f64>::Equals(42.0);
    assert!(rule.validate_scalar(f64::INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_inf_f64_one_of() {
    let rule = Rule::<f64>::OneOf(vec![1.0, 2.0, 3.0]);
    assert!(rule.validate_scalar(f64::INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_inf_f32_min() {
    let rule = Rule::<f32>::Min(0.0);
    assert!(rule.validate_scalar(f32::INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_inf_f32_max() {
    let rule = Rule::<f32>::Max(100.0);
    assert!(rule.validate_scalar(f32::INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_inf_f32_range() {
    let rule = Rule::<f32>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_scalar(f32::INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_inf_f32_step() {
    let rule = Rule::<f32>::Step(5.0);
    assert!(rule.validate_scalar(f32::INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_inf_f32_equals() {
    let rule = Rule::<f32>::Equals(42.0);
    assert!(rule.validate_scalar(f32::INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_inf_f32_one_of() {
    let rule = Rule::<f32>::OneOf(vec![1.0, 2.0, 3.0]);
    assert!(rule.validate_scalar(f32::INFINITY).is_err());
  }

  // ==========================================================================
  // NEG_INFINITY Validation Tests (scalar path)
  // ==========================================================================

  #[test]
  fn test_validate_scalar_neg_inf_f64_min() {
    let rule = Rule::<f64>::Min(0.0);
    assert!(rule.validate_scalar(f64::NEG_INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f64_max() {
    let rule = Rule::<f64>::Max(100.0);
    assert!(rule.validate_scalar(f64::NEG_INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f64_range() {
    let rule = Rule::<f64>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_scalar(f64::NEG_INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f64_step() {
    let rule = Rule::<f64>::Step(5.0);
    assert!(rule.validate_scalar(f64::NEG_INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f64_equals() {
    let rule = Rule::<f64>::Equals(42.0);
    assert!(rule.validate_scalar(f64::NEG_INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f64_one_of() {
    let rule = Rule::<f64>::OneOf(vec![1.0, 2.0, 3.0]);
    assert!(rule.validate_scalar(f64::NEG_INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f32_min() {
    let rule = Rule::<f32>::Min(0.0);
    assert!(rule.validate_scalar(f32::NEG_INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f32_max() {
    let rule = Rule::<f32>::Max(100.0);
    assert!(rule.validate_scalar(f32::NEG_INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f32_range() {
    let rule = Rule::<f32>::Range {
      min: 0.0,
      max: 100.0,
    };
    assert!(rule.validate_scalar(f32::NEG_INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f32_step() {
    let rule = Rule::<f32>::Step(5.0);
    assert!(rule.validate_scalar(f32::NEG_INFINITY).is_ok());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f32_equals() {
    let rule = Rule::<f32>::Equals(42.0);
    assert!(rule.validate_scalar(f32::NEG_INFINITY).is_err());
  }

  #[test]
  fn test_validate_scalar_neg_inf_f32_one_of() {
    let rule = Rule::<f32>::OneOf(vec![1.0, 2.0, 3.0]);
    assert!(rule.validate_scalar(f32::NEG_INFINITY).is_err());
  }

  // ==========================================================================
  // Rule::Ref tests (#143)
  // ==========================================================================

  #[test]
  fn test_validate_scalar_ref_returns_err() {
    let rule = Rule::<i32>::Ref("my_ref".into());
    let result = rule.validate_scalar(42);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violation_type(), crate::ViolationType::CustomError);
    assert!(err.message().contains("my_ref"));
  }

  #[test]
  fn test_validate_scalar_ref_inside_all() {
    let rule = Rule::<i32>::All(vec![Rule::Min(0), Rule::Ref("my_ref".into())]);
    assert!(rule.validate_scalar(42).is_err());
  }

  #[test]
  fn test_validate_scalar_ref_inside_any() {
    // One branch passes → overall passes
    let rule = Rule::<i32>::Any(vec![Rule::Ref("my_ref".into()), Rule::Min(0)]);
    assert!(rule.validate_scalar(42).is_ok());

    // All branches fail → overall fails
    let rule_all_fail = Rule::<i32>::Any(vec![Rule::Ref("my_ref".into()), Rule::Min(100)]);
    assert!(rule_all_fail.validate_scalar(42).is_err());
  }

  #[test]
  fn test_validate_scalar_ref_inside_not() {
    // Not(Ref) → Ref fails → Not succeeds
    let rule = Rule::<i32>::Not(Box::new(Rule::Ref("my_ref".into())));
    assert!(rule.validate_scalar(42).is_ok());
  }

  // ==========================================================================
  // Deeply nested combinator tests (#145)
  // ==========================================================================

  #[test]
  fn test_nested_depth2_all_containing_all_and_any() {
    // All([All([Min(0), Max(100)]), Any([Equals(50), Equals(75)])])
    let rule = Rule::<i32>::All(vec![
      Rule::All(vec![Rule::Min(0), Rule::Max(100)]),
      Rule::Any(vec![Rule::Equals(50), Rule::Equals(75)]),
    ]);
    assert!(rule.validate_scalar(50).is_ok());
    assert!(rule.validate_scalar(75).is_ok());
    assert!(rule.validate_scalar(25).is_err()); // Any fails
    assert!(rule.validate_scalar(150).is_err()); // Max fails
  }

  #[test]
  fn test_nested_depth2_when_with_nested_all_then() {
    // When { condition: GreaterThan(0), then: All([Min(1), Max(10)]), else: None }
    let rule = Rule::<i32>::When {
      condition: Condition::GreaterThan(0),
      then_rule: Box::new(Rule::All(vec![Rule::Min(1), Rule::Max(10)])),
      else_rule: None,
    };
    assert!(rule.validate_scalar(5).is_ok());
    assert!(rule.validate_scalar(11).is_err());
    assert!(rule.validate_scalar(0).is_ok()); // condition false → pass
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
    assert!(rule.validate_scalar(1).is_err());
    assert!(rule.validate_scalar(2).is_err());
    assert!(rule.validate_scalar(5).is_ok());
  }

  #[test]
  fn test_nested_any_with_not_and_all() {
    // Any([Not(Min(0)), All([Min(0), Max(10)])])
    // -1: Not(Min(0)) → Min fails → Not passes → Any passes
    // 5: Not fails, All passes → Any passes
    // 15: Not fails, All fails → Any fails
    let rule = Rule::<i32>::Any(vec![
      Rule::Not(Box::new(Rule::Min(0))),
      Rule::All(vec![Rule::Min(0), Rule::Max(10)]),
    ]);
    assert!(rule.validate_scalar(-1).is_ok());
    assert!(rule.validate_scalar(5).is_ok());
    assert!(rule.validate_scalar(15).is_err());
  }

  #[test]
  fn test_nested_depth3_when_else_any_not() {
    // When { condition: GreaterThan(50), then: Max(100),
    //        else: Any([Not(Min(0)), Equals(25)]) }
    let rule = Rule::<i32>::When {
      condition: Condition::GreaterThan(50),
      then_rule: Box::new(Rule::Max(100)),
      else_rule: Some(Box::new(Rule::Any(vec![
        Rule::Not(Box::new(Rule::Min(0))),
        Rule::Equals(25),
      ]))),
    };
    // > 50 → then: Max(100)
    assert!(rule.validate_scalar(75).is_ok());
    assert!(rule.validate_scalar(101).is_err());
    // <= 50 → else: Any([Not(Min(0)), Equals(25)])
    assert!(rule.validate_scalar(25).is_ok()); // Equals(25) passes
    assert!(rule.validate_scalar(-1).is_ok()); // Not(Min(0)) passes
    assert!(rule.validate_scalar(10).is_err()); // Neither passes
  }

  #[test]
  fn test_empty_any_passes() {
    let rule = Rule::<i32>::Any(vec![]);
    assert!(rule.validate_scalar(42).is_ok());
  }

  // ==========================================================================
  // Async tests for bool / char
  // ==========================================================================

  #[cfg(feature = "async")]
  mod async_scalar_tests {
    use crate::rule::Rule;
    use crate::{ValidateAsync, ValidateRefAsync};

    // --- bool ---

    #[tokio::test]
    async fn test_async_validate_bool() {
      let rule = Rule::<bool>::Equals(true);
      assert!(rule.validate_async(true).await.is_ok());
      assert!(rule.validate_async(false).await.is_err());
    }

    #[tokio::test]
    async fn test_async_validate_ref_bool() {
      let rule = Rule::<bool>::Equals(true);
      assert!(rule.validate_ref_async(&true).await.is_ok());
      assert!(rule.validate_ref_async(&false).await.is_err());
    }

    #[tokio::test]
    async fn test_async_option_bool_none_required() {
      let rule = Rule::<bool>::Required;
      assert!(rule.validate_async(None::<bool>).await.is_err());
      assert!(rule.validate_ref_async(&None::<bool>).await.is_err());
    }

    #[tokio::test]
    async fn test_async_option_bool_none_not_required() {
      let rule = Rule::<bool>::Equals(true);
      assert!(rule.validate_async(None::<bool>).await.is_ok());
      assert!(rule.validate_ref_async(&None::<bool>).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_option_bool_some() {
      let rule = Rule::<bool>::Equals(true);
      assert!(rule.validate_async(Some(true)).await.is_ok());
      assert!(rule.validate_ref_async(&Some(true)).await.is_ok());
      assert!(rule.validate_async(Some(false)).await.is_err());
      assert!(rule.validate_ref_async(&Some(false)).await.is_err());
    }

    // --- char ---

    #[tokio::test]
    async fn test_async_validate_char() {
      let rule = Rule::<char>::Range { min: 'a', max: 'z' };
      assert!(rule.validate_async('m').await.is_ok());
      assert!(rule.validate_async('A').await.is_err());
    }

    #[tokio::test]
    async fn test_async_validate_ref_char() {
      let rule = Rule::<char>::Range { min: 'a', max: 'z' };
      assert!(rule.validate_ref_async(&'m').await.is_ok());
      assert!(rule.validate_ref_async(&'A').await.is_err());
    }

    #[tokio::test]
    async fn test_async_option_char_none_required() {
      let rule = Rule::<char>::Required;
      assert!(rule.validate_async(None::<char>).await.is_err());
      assert!(rule.validate_ref_async(&None::<char>).await.is_err());
    }

    #[tokio::test]
    async fn test_async_option_char_none_not_required() {
      let rule = Rule::<char>::Equals('a');
      assert!(rule.validate_async(None::<char>).await.is_ok());
      assert!(rule.validate_ref_async(&None::<char>).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_option_char_some() {
      let rule = Rule::<char>::Equals('a');
      assert!(rule.validate_async(Some('a')).await.is_ok());
      assert!(rule.validate_ref_async(&Some('a')).await.is_ok());
      assert!(rule.validate_async(Some('b')).await.is_err());
      assert!(rule.validate_ref_async(&Some('b')).await.is_err());
    }
  }
}
