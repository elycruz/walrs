use crate::Violation;
use crate::rule::{Rule, RuleResult};
use crate::traits::WithLength;

impl<T: WithLength> Rule<T> {
  /// Validates a collection's length against this rule.
  #[allow(dead_code)] // Reserved for a future public API
  pub(crate) fn validate_len(&self, value: &T) -> RuleResult {
    self.validate_len_inner(value, None)
  }

  /// Internal validation with inherited locale from an outer `WithMessage`.
  fn validate_len_inner(&self, value: &T, inherited_locale: Option<&str>) -> RuleResult {
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
          rule.validate_len_inner(value, inherited_locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_len_inner(value, inherited_locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_len_inner(value, inherited_locale) {
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
          then_rule.validate_len_inner(value, inherited_locale)?;
        }
        Ok(())
      }
      Rule::Custom(_) => {
        // Custom rules are not supported for generic WithLength validation
        // as they require the specific type T
        Ok(())
      }
      #[cfg(feature = "async")]
      Rule::CustomAsync(_) => Ok(()),
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let eff = locale.as_deref().or(inherited_locale);
        message.wrap_result(rule.validate_len_inner(value, eff), value, eff)
      }
      // Non-length rules don't apply to collections - pass through
      Rule::Pattern(_)
      | Rule::Email(_)
      | Rule::Url(_)
      | Rule::Uri(_)
      | Rule::Ip(_)
      | Rule::Hostname(_)
      | Rule::Date(_)
      | Rule::DateRange(_)
      | Rule::Min(_)
      | Rule::Max(_)
      | Rule::Range { .. }
      | Rule::Step(_)
      | Rule::Equals(_)
      | Rule::OneOf(_) => Ok(()),
    }
  }

  /// Validates a collection's length and collects all violations.
  #[allow(dead_code)] // Reserved for a future `validate_all` public API
  pub(crate) fn validate_len_all(&self, value: &T) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_len_violations(value, None, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional collection's length.
  #[allow(dead_code)] // Reserved for a future `validate_option` public API
  pub(crate) fn validate_option_len(&self, value: Option<&T>) -> RuleResult {
    match value {
      Some(v) => self.validate_len(v),
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
    }
  }

  /// Validates an optional collection's length and collects all violations.
  #[allow(dead_code)] // Reserved for a future `validate_option_all` public API
  pub(crate) fn validate_option_len_all(&self, value: Option<&T>) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_len_all(v),
      None if self.requires_value() => Err(crate::Violations::from(Violation::value_missing())),
      None => Ok(()),
    }
  }

  /// Helper to collect all length violations recursively.
  #[allow(dead_code)] // Called transitively from validate_len_all
  fn collect_len_violations(
    &self,
    value: &T,
    inherited_locale: Option<&str>,
    violations: &mut crate::Violations,
  ) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_len_violations(value, inherited_locale, violations);
        }
      }
      Rule::Any(rules) => {
        // For Any, we only add violations if ALL rules fail
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_len_violations(value, inherited_locale, &mut rule_violations);
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
        condition: _,
        then_rule,
        else_rule: _,
      } => {
        // For collections, apply then_rule if not empty
        if value.length() > 0 {
          then_rule.collect_len_violations(value, inherited_locale, violations);
        }
      }
      Rule::WithMessage {
        rule,
        message,
        locale,
      } => {
        let eff = locale.as_deref().or(inherited_locale);
        let mut inner_violations = crate::Violations::default();
        rule.collect_len_violations(value, eff, &mut inner_violations);
        message.wrap_violations(inner_violations, value, eff, violations);
      }
      _ => {
        if let Err(v) = self.validate_len_inner(value, inherited_locale) {
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
  fn test_validate_len_indexmap() {
    use indexmap::IndexMap;

    let rule = Rule::<IndexMap<String, i32>>::MinLength(1).and(Rule::MaxLength(3));

    let mut map = IndexMap::new();
    map.insert("a".to_string(), 1);
    assert!(rule.validate_len(&map).is_ok());

    map.insert("b".to_string(), 2);
    map.insert("c".to_string(), 3);
    assert!(rule.validate_len(&map).is_ok());

    map.insert("d".to_string(), 4);
    assert!(rule.validate_len(&map).is_err());

    let empty_map: IndexMap<String, i32> = IndexMap::new();
    assert!(rule.validate_len(&empty_map).is_err());
  }

  #[test]
  fn test_validate_len_indexset() {
    use indexmap::IndexSet;

    let rule = Rule::<IndexSet<String>>::MinLength(2).and(Rule::MaxLength(4));

    let mut set = IndexSet::new();
    set.insert("a".to_string());
    assert!(rule.validate_len(&set).is_err()); // too short

    set.insert("b".to_string());
    assert!(rule.validate_len(&set).is_ok());

    set.insert("c".to_string());
    set.insert("d".to_string());
    assert!(rule.validate_len(&set).is_ok());

    set.insert("e".to_string());
    assert!(rule.validate_len(&set).is_err()); // too long

    let empty_set: IndexSet<String> = IndexSet::new();
    assert!(rule.validate_len(&empty_set).is_err());
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

  // ========================================================================
  // WithMessage + Locale Tests
  // ========================================================================

  #[test]
  fn test_validate_len_with_message_static() {
    let rule = Rule::<Vec<i32>>::MinLength(3).with_message("Too few items");

    let result = rule.validate_len(&vec![1]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Too few items");

    // Passing case should still succeed
    assert!(rule.validate_len(&vec![1, 2, 3]).is_ok());
  }

  #[test]
  fn test_validate_len_with_message_provider_and_locale() {
    let rule = Rule::<Vec<i32>>::MinLength(2).with_message_provider(
      |ctx: &crate::MessageContext<Vec<i32>>| match ctx.locale {
        Some("es") => format!(
          "Se requieren al menos 2 elementos, recibidos: {}",
          ctx.value.len()
        ),
        Some("fr") => format!("Au moins 2 éléments requis, reçu : {}", ctx.value.len()),
        _ => format!("At least 2 items required, got: {}", ctx.value.len()),
      },
      Some("es"),
    );

    let result = rule.validate_len(&vec![1]);
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().message(),
      "Se requieren al menos 2 elementos, recibidos: 1"
    );
  }

  #[test]
  fn test_validate_len_with_message_provider_default_locale() {
    let rule = Rule::<Vec<i32>>::MinLength(2).with_message_provider(
      |ctx: &crate::MessageContext<Vec<i32>>| match ctx.locale {
        Some("es") => "Muy pocos".to_string(),
        _ => format!("Need at least 2, got {}", ctx.value.len()),
      },
      None,
    );

    // No locale set → default arm
    let result = rule.validate_len(&vec![1]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Need at least 2, got 1");
  }

  #[test]
  fn test_validate_len_all_with_message_and_locale() {
    let rule = Rule::<Vec<i32>>::MinLength(3)
      .and(Rule::MaxLength(5))
      .with_message("Longitud inválida")
      .with_locale("es".to_string());

    // Too short — static message overrides all inner violations
    let result = rule.validate_len_all(&vec![1]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].message(), "Longitud inválida");

    // Too long
    let result = rule.validate_len_all(&vec![1, 2, 3, 4, 5, 6]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].message(), "Longitud inválida");

    // Valid
    assert!(rule.validate_len_all(&vec![1, 2, 3, 4]).is_ok());
  }

  #[test]
  fn test_validate_len_nested_with_message_locale_inheritance() {
    // Inner WithMessage has a locale-aware provider, outer sets the locale
    let inner = Rule::<Vec<i32>>::MinLength(2).with_message_provider(
      |ctx: &crate::MessageContext<Vec<i32>>| match ctx.locale {
        Some("fr") => format!("Besoin d'au moins 2, reçu {}", ctx.value.len()),
        _ => format!("Need at least 2, got {}", ctx.value.len()),
      },
      None,
    );

    let outer = inner.with_locale("fr".to_string());

    let result = outer.validate_len(&vec![1]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Besoin d'au moins 2, reçu 1");
  }

  #[test]
  fn test_validate_len_with_message_empty_static_uses_default() {
    // Empty static message should fall back to the default violation message
    let rule = Rule::<Vec<i32>>::MinLength(3).with_locale("en".to_string());

    let result = rule.validate_len(&vec![1]);
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().message(),
      "Value length must be at least 3;  Received 1."
    );
  }

  #[test]
  fn test_collect_violations_with_message_locale() {
    // Contradictory rules wrapped with locale-aware message
    let rule = Rule::<Vec<i32>>::MinLength(5)
      .and(Rule::MaxLength(2))
      .with_message_provider(
        |ctx: &crate::MessageContext<Vec<i32>>| match ctx.locale {
          Some("es") => format!("Error de longitud ({})", ctx.value.len()),
          _ => format!("Length error ({})", ctx.value.len()),
        },
        Some("es"),
      );

    let result = rule.validate_len_all(&vec![1, 2, 3]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    // Both MinLength and MaxLength fail
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].message(), "Error de longitud (3)");
    assert_eq!(violations[1].message(), "Error de longitud (3)");
  }
}
