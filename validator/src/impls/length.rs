use crate::rule::{Rule, RuleResult};
use crate::traits::WithLength;
use crate::Violation;

impl<T: WithLength> Rule<T> {
  /// Validates a collection's length against this rule.
  pub fn validate_len(&self, value: &T) -> RuleResult {
    match self {
      Rule::Required => {
        if value.length() == 0 {
          Err(Violation::value_missing())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.length();
        if len < *min {
          Err(Violation::too_short(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.length();
        if len > *max {
          Err(Violation::too_long(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.length();
        if len != *expected {
          Err(Violation::exact_length(*expected, len))
        } else {
          Ok(())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_len(value)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_len(value) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_len(value) {
        Ok(()) => Err(Violation::negation_failed()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition: _,
        then_rule,
        else_rule: _,
      } => {
        // For collections, we only support simple condition evaluation based on emptiness
        // Full condition evaluation would require additional trait bounds
        // For now, always apply then_rule if value is not empty
        if value.length() > 0 {
          then_rule.validate_len(value)?;
        }
        Ok(())
      }
      Rule::Custom(_) => {
        // Custom rules are not supported for generic WithLength validation
        // as they require the specific type T
        Ok(())
      }
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage { rule, .. } => {
        // For WithLength types, we can't easily resolve messages without more bounds
        // Just delegate to inner rule
        rule.validate_len(value)
      }
      // Non-length rules don't apply to collections - pass through
      Rule::Pattern(_)
      | Rule::Email
      | Rule::Url
      | Rule::Min(_)
      | Rule::Max(_)
      | Rule::Range { .. }
      | Rule::Step(_)
      | Rule::Equals(_)
      | Rule::OneOf(_) => Ok(()),
    }
  }

  /// Validates a collection's length and collects all violations.
  pub fn validate_len_all(&self, value: &T) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_len_violations(value, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional collection's length.
  pub fn validate_option_len(&self, value: Option<&T>) -> RuleResult {
    match value {
      Some(v) => self.validate_len(v),
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
    }
  }

  /// Validates an optional collection's length and collects all violations.
  pub fn validate_option_len_all(&self, value: Option<&T>) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_len_all(v),
      None if self.requires_value() => Err(crate::Violations::from(Violation::value_missing())),
      None => Ok(()),
    }
  }

  /// Helper to collect all length violations recursively.
  fn collect_len_violations(&self, value: &T, violations: &mut crate::Violations) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_len_violations(value, violations);
        }
      }
      Rule::Any(rules) => {
        // For Any, we only add violations if ALL rules fail
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_len_violations(value, &mut rule_violations);
          if rule_violations.is_empty() {
            any_passed = true;
            break;
          }
          any_violations.extend(rule_violations.into_iter());
        }
        if !any_passed && !rules.is_empty() {
          // Just add the last violation for Any
          if let Some(v) = any_violations.0.pop() {
            violations.push(v);
          }
        }
      }
      Rule::When {
        condition: _,
        then_rule,
        else_rule: _,
      } => {
        // For collections, apply then_rule if not empty
        if value.length() > 0 {
          then_rule.collect_len_violations(value, violations);
        }
      }
      Rule::WithMessage { rule, message: _, locale: _ } => {
        // Delegate to inner rule
        rule.collect_len_violations(value, violations);
      }
      _ => {
        if let Err(v) = self.validate_len(value) {
          violations.push(v);
        }
      }
    }
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use crate::rule::Rule;

  // ========================================================================
  // Collection Length Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_len_min_length() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_len(&vec![1, 2]).is_ok());
    assert!(rule.validate_len(&vec![1, 2, 3]).is_ok());
    assert!(rule.validate_len(&vec![1]).is_err());
    assert!(rule.validate_len(&vec![]).is_err());
  }

  #[test]
  fn test_validate_len_max_length() {
    let rule = Rule::<Vec<i32>>::MaxLength(3);
    assert!(rule.validate_len(&vec![1]).is_ok());
    assert!(rule.validate_len(&vec![1, 2, 3]).is_ok());
    assert!(rule.validate_len(&vec![1, 2, 3, 4]).is_err());
  }

  #[test]
  fn test_validate_len_exact_length() {
    let rule = Rule::<Vec<i32>>::ExactLength(3);
    assert!(rule.validate_len(&vec![1, 2, 3]).is_ok());
    assert!(rule.validate_len(&vec![1, 2]).is_err());
    assert!(rule.validate_len(&vec![1, 2, 3, 4]).is_err());
  }

  #[test]
  fn test_validate_len_required() {
    let rule = Rule::<Vec<i32>>::Required;
    assert!(rule.validate_len(&vec![1]).is_ok());
    assert!(rule.validate_len(&vec![]).is_err());
  }

  #[test]
  fn test_validate_len_all_combinator() {
    let rule = Rule::<Vec<i32>>::MinLength(2).and(Rule::MaxLength(5));
    assert!(rule.validate_len(&vec![1, 2]).is_ok());
    assert!(rule.validate_len(&vec![1, 2, 3, 4, 5]).is_ok());
    assert!(rule.validate_len(&vec![1]).is_err());
    assert!(rule.validate_len(&vec![1, 2, 3, 4, 5, 6]).is_err());
  }

  #[test]
  fn test_validate_len_any_combinator() {
    // Either exactly 2 items OR exactly 5 items
    let rule = Rule::<Vec<i32>>::ExactLength(2).or(Rule::ExactLength(5));
    assert!(rule.validate_len(&vec![1, 2]).is_ok());
    assert!(rule.validate_len(&vec![1, 2, 3, 4, 5]).is_ok());
    assert!(rule.validate_len(&vec![1, 2, 3]).is_err());
  }

  #[test]
  fn test_validate_len_not_combinator() {
    // NOT empty (must have at least 1 item)
    let rule = Rule::<Vec<i32>>::MaxLength(0).not();
    assert!(rule.validate_len(&vec![1]).is_ok());
    assert!(rule.validate_len(&vec![]).is_err());
  }

  // Note: Slice validation ([T]) is not supported because Rule<T> requires T: Sized.
  // Use Vec<T> or other sized collection types instead.
  // For slice validation, use LengthValidator<[T]> directly.

  #[test]
  fn test_validate_len_hashmap() {
    use std::collections::HashMap;

    let rule = Rule::<HashMap<String, i32>>::MinLength(1).and(Rule::MaxLength(3));

    let mut map = HashMap::new();
    map.insert("a".to_string(), 1);
    assert!(rule.validate_len(&map).is_ok());

    map.insert("b".to_string(), 2);
    map.insert("c".to_string(), 3);
    assert!(rule.validate_len(&map).is_ok());

    map.insert("d".to_string(), 4);
    assert!(rule.validate_len(&map).is_err());

    let empty_map: HashMap<String, i32> = HashMap::new();
    assert!(rule.validate_len(&empty_map).is_err());
  }

  #[test]
  fn test_validate_len_all_violations() {
    // Contradictory rule - will always fail
    let rule = Rule::<Vec<i32>>::MinLength(3).and(Rule::MaxLength(2));

    let result = rule.validate_len_all(&vec![1, 2]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1); // MinLength fails
  }

  #[test]
  fn test_validate_option_len_none_not_required() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_option_len(None).is_ok());
  }

  #[test]
  fn test_validate_option_len_none_required() {
    let rule = Rule::<Vec<i32>>::Required;
    assert!(rule.validate_option_len(None).is_err());
  }

  #[test]
  fn test_validate_option_len_some_valid() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_option_len(Some(&vec![1, 2, 3])).is_ok());
  }

  #[test]
  fn test_validate_option_len_some_invalid() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_option_len(Some(&vec![1])).is_err());
  }

  #[test]
  fn test_validate_option_len_all_with_required() {
    let rule = Rule::<Vec<i32>>::Required.and(Rule::MinLength(2));

    assert!(rule.validate_option_len(None).is_err());
    assert!(rule.validate_option_len(Some(&vec![1, 2])).is_ok());
    assert!(rule.validate_option_len(Some(&vec![1])).is_err());
  }

  #[test]
  fn test_validate_len_violation_messages() {
    let rule = Rule::<Vec<i32>>::MinLength(3);
    let result = rule.validate_len(&vec![1]);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(
      violation.message(),
      "Value length must be at least 3;  Received 1."
    );

    let rule = Rule::<Vec<i32>>::MaxLength(2);
    let result = rule.validate_len(&vec![1, 2, 3, 4]);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(
      violation.message(),
      "Value length must at most 2;  Received 4."
    );

    let rule = Rule::<Vec<i32>>::ExactLength(3);
    let result = rule.validate_len(&vec![1, 2]);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(
      violation.message(),
      "Value length must be exactly 3 (got 2)."
    );
  }
}
