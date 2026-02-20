use crate::rule::{
    Rule, RuleResult,
    value_missing_violation, too_short_violation, too_long_violation, exact_length_violation,
    pattern_mismatch_violation, invalid_email_violation, invalid_url_violation,
    not_equal_violation, not_one_of_violation, negation_failed_violation, unresolved_ref_violation
};
use crate::Violation;
use crate::traits::ValidateRef;
use crate::CompiledRule;

/// Cached validators for a compiled rule.
///
/// This struct holds compiled regex patterns for string validation rules.
/// Included in `CompiledRule` for all types, but only populated for String rules.
#[derive(Debug, Default)]
pub struct CachedStringValidators {
  /// Cached regex for Pattern rules
  pub(crate) pattern_regex: Option<regex::Regex>,
  /// Cached email regex
  pub(crate) email_regex: Option<regex::Regex>,
  /// Cached URL regex
  pub(crate) url_regex: Option<regex::Regex>,
}

impl CachedStringValidators {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Rule<String> {
  /// Validates a string value against this rule.
  pub fn validate_ref(&self, value: &str, locale: Option<&str>) -> RuleResult {
    match self {
      Rule::Required => {
        if value.trim().is_empty() {
          Err(value_missing_violation())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.chars().count();
        if len < *min {
          Err(too_short_violation(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.chars().count();
        if len > *max {
          Err(too_long_violation(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.chars().count();
        if len != *expected {
          Err(exact_length_violation(*expected, len))
        } else {
          Ok(())
        }
      }
      Rule::Pattern(pattern) => match regex::Regex::new(pattern) {
        Ok(re) => {
          if re.is_match(value) {
            Ok(())
          } else {
            Err(pattern_mismatch_violation(pattern))
          }
        }
        Err(_) => Err(pattern_mismatch_violation(pattern)),
      },
      Rule::Email => {
        // Simple email validation using regex
        let email_re =
          regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if email_re.is_match(value) {
          Ok(())
        } else {
          Err(invalid_email_violation())
        }
      }
      Rule::Url => {
        // Simple URL validation using regex
        let url_re = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        if url_re.is_match(value) {
          Ok(())
        } else {
          Err(invalid_url_violation())
        }
      }
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(not_equal_violation(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.iter().any(|v| v == value) {
          Ok(())
        } else {
          Err(not_one_of_violation())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_ref(value, locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_ref(value, locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_ref(value, locale) {
        Ok(()) => Err(negation_failed_violation()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          then_rule.validate_ref(value, locale)
        } else {
          match else_rule {
            Some(rule) => rule.validate_ref(value, locale),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value.to_string()),
      Rule::Ref(name) => Err(unresolved_ref_violation(name)),
      Rule::WithMessage { rule, message } => match rule.validate_ref(value, locale) {
        Ok(()) => Ok(()),
        Err(violation) => {
          let custom_msg = message.resolve(&value.to_string(), locale);
          Err(Violation::new(violation.violation_type(), custom_msg))
        }
      },
      // Numeric rules don't apply to strings - pass through
      Rule::Min(_) | Rule::Max(_) | Rule::Range { .. } | Rule::Step(_) => Ok(()),
    }
  }

  /// Validates a string value and collects all violations.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violations)` with all failures.
  pub fn validate_ref_all(
    &self,
    value: &str,
    locale: Option<&str>,
  ) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_violations_ref(value, locale, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional string value.
  pub fn validate_ref_option(&self, value: Option<&str>, locale: Option<&str>) -> RuleResult {
    match value {
      Some(v) => self.validate_ref(v, locale),
      None if self.requires_value() => Err(value_missing_violation()),
      None => Ok(()),
    }
  }

  /// Validates an optional string value and collects all violations.
  pub fn validate_ref_option_all(
    &self,
    value: Option<&str>,
    locale: Option<&str>,
  ) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_ref_all(v, locale),
      None if self.requires_value() => Err(crate::Violations::from(value_missing_violation())),
      None => Ok(()),
    }
  }

  /// Helper to collect all violations recursively.
  fn collect_violations_ref(
    &self,
    value: &str,
    locale: Option<&str>,
    violations: &mut crate::Violations,
  ) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_violations_ref(value, locale, violations);
        }
      }
      Rule::Any(rules) => {
        // For Any, we only add violations if ALL rules fail
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_violations_ref(value, locale, &mut rule_violations);
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
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          then_rule.collect_violations_ref(value, locale, violations);
        } else if let Some(rule) = else_rule {
          rule.collect_violations_ref(value, locale, violations);
        }
      }
      Rule::WithMessage { rule, message } => {
        let mut inner_violations = crate::Violations::default();
        rule.collect_violations_ref(value, locale, &mut inner_violations);
        for violation in inner_violations {
          let custom_msg = message.resolve(&value.to_string(), locale);
          violations.push(Violation::new(violation.violation_type(), custom_msg));
        }
      }
      _ => {
        if let Err(v) = self.validate_ref(value, locale) {
          violations.push(v);
        }
      }
    }
  }
}

impl ValidateRef<str> for Rule<String> {
  fn validate_ref(&self, value: &str) -> crate::ValidatorResult {
    Rule::validate_ref(self, value, None)
  }
}

impl ValidateRef<str> for CompiledRule<String> {
  fn validate_ref(&self, value: &str) -> crate::ValidatorResult {
    CompiledRule::validate_ref(self, value)
  }
}

impl CompiledRule<String> {
  /// Gets or initializes the cached string validators.
  fn get_or_init_cache(&self) -> &CachedStringValidators {
    self.string_cache.get_or_init(|| {
      let mut cache = CachedStringValidators::new();

      // Pre-compile pattern regex if applicable
      if let Rule::Pattern(pattern) = &self.rule {
        cache.pattern_regex = regex::Regex::new(pattern).ok();
      }

      // Pre-compile email regex
      cache.email_regex =
        regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").ok();

      // Pre-compile URL regex
      cache.url_regex = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").ok();

      cache
    })
  }

  /// Validates a string value using cached validators.
  pub fn validate_ref(&self, value: &str) -> RuleResult {
    self.validate_ref_with_cache(value, self.get_or_init_cache())
  }

  fn validate_ref_with_cache(&self, value: &str, cache: &CachedStringValidators) -> RuleResult {
    match &self.rule {
      Rule::Required => {
        if value.trim().is_empty() {
          Err(value_missing_violation())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.chars().count();
        if len < *min {
          Err(too_short_violation(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.chars().count();
        if len > *max {
          Err(too_long_violation(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.chars().count();
        if len != *expected {
          Err(exact_length_violation(*expected, len))
        } else {
          Ok(())
        }
      }
      Rule::Pattern(pattern) => {
        // Use cached regex if available
        let matches = cache
          .pattern_regex
          .as_ref()
          .map(|re| re.is_match(value))
          .unwrap_or_else(|| {
            regex::Regex::new(pattern)
              .map(|re| re.is_match(value))
              .unwrap_or(false)
          });
        if matches {
          Ok(())
        } else {
          Err(pattern_mismatch_violation(pattern))
        }
      }
      Rule::Email => {
        let matches = cache
          .email_regex
          .as_ref()
          .map(|re| re.is_match(value))
          .unwrap_or(false);
        if matches {
          Ok(())
        } else {
          Err(invalid_email_violation())
        }
      }
      Rule::Url => {
        let matches = cache
          .url_regex
          .as_ref()
          .map(|re| re.is_match(value))
          .unwrap_or(false);
        if matches {
          Ok(())
        } else {
          Err(invalid_url_violation())
        }
      }
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(not_equal_violation(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.iter().any(|v| v == value) {
          Ok(())
        } else {
          Err(not_one_of_violation())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          CompiledRule::new(rule.clone()).validate_ref(value)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match CompiledRule::new(rule.clone()).validate_ref(value) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match CompiledRule::new((**inner).clone()).validate_ref(value) {
        Ok(()) => Err(negation_failed_violation()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          CompiledRule::new((**then_rule).clone()).validate_ref(value)
        } else {
          match else_rule {
            Some(rule) => CompiledRule::new((**rule).clone()).validate_ref(value),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value.to_string()),
      Rule::Ref(name) => Err(unresolved_ref_violation(name)),
      Rule::WithMessage { rule, message } => {
        match CompiledRule::new((**rule).clone()).validate_ref(value) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg = message.resolve(&value.to_string(), None);
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
        }
      }
      Rule::Min(_) | Rule::Max(_) | Rule::Range { .. } | Rule::Step(_) => Ok(()),
    }
  }

  /// Validates a string value and collects all violations.
  pub fn validate_ref_all(&self, value: &str) -> Result<(), crate::Violations> {
    self.rule.validate_ref_all(value, None)
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use crate::rule::{Condition, Rule};

  // ========================================================================
  // String Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_ref_required() {
    let rule = Rule::<String>::Required;
    assert!(rule.validate_ref("hello", None).is_ok());
    assert!(rule.validate_ref("", None).is_err());
    assert!(rule.validate_ref("   ", None).is_err());
  }

  #[test]
  fn test_validate_ref_min_length() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_ref("hello", None).is_ok());
    assert!(rule.validate_ref("abc", None).is_ok());
    assert!(rule.validate_ref("ab", None).is_err());
    assert!(rule.validate_ref("", None).is_err());
  }

  #[test]
  fn test_validate_ref_max_length() {
    let rule = Rule::<String>::MaxLength(5);
    assert!(rule.validate_ref("hello", None).is_ok());
    assert!(rule.validate_ref("hi", None).is_ok());
    assert!(rule.validate_ref("", None).is_ok());
    assert!(rule.validate_ref("hello!", None).is_err());
  }

  #[test]
  fn test_validate_ref_exact_length() {
    let rule = Rule::<String>::ExactLength(5);
    assert!(rule.validate_ref("hello", None).is_ok());
    assert!(rule.validate_ref("hi", None).is_err());
    assert!(rule.validate_ref("hello!", None).is_err());
  }

  #[test]
  fn test_validate_ref_pattern() {
    let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
    assert!(rule.validate_ref("123", None).is_ok());
    assert!(rule.validate_ref("abc", None).is_err());
    assert!(rule.validate_ref("12a", None).is_err());
  }

  #[test]
  fn test_validate_ref_email() {
    let rule = Rule::<String>::Email;
    assert!(rule.validate_ref("user@example.com", None).is_ok());
    assert!(rule.validate_ref("user@sub.example.com", None).is_ok());
    assert!(rule.validate_ref("invalid", None).is_err());
    assert!(rule.validate_ref("@example.com", None).is_err());
  }

  #[test]
  fn test_validate_ref_url() {
    let rule = Rule::<String>::Url;
    assert!(rule.validate_ref("http://example.com", None).is_ok());
    assert!(rule.validate_ref("https://example.com/path", None).is_ok());
    assert!(rule.validate_ref("not-a-url", None).is_err());
    assert!(rule.validate_ref("ftp://example.com", None).is_err()); // Only http/https
  }

  #[test]
  fn test_validate_ref_equals() {
    let rule = Rule::<String>::Equals("secret".to_string());
    assert!(rule.validate_ref("secret", None).is_ok());
    assert!(rule.validate_ref("wrong", None).is_err());
  }

  #[test]
  fn test_validate_ref_one_of() {
    let rule = Rule::<String>::OneOf(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert!(rule.validate_ref("a", None).is_ok());
    assert!(rule.validate_ref("b", None).is_ok());
    assert!(rule.validate_ref("d", None).is_err());
  }

  #[test]
  fn test_validate_ref_all() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    assert!(rule.validate_ref("hello", None).is_ok());
    assert!(rule.validate_ref("hi", None).is_err());
    assert!(rule.validate_ref("hello world!", None).is_err());
  }

  #[test]
  fn test_validate_ref_any() {
    let rule = Rule::<String>::Email.or(Rule::Url);
    assert!(rule.validate_ref("user@example.com", None).is_ok());
    assert!(rule.validate_ref("http://example.com", None).is_ok());
    assert!(rule.validate_ref("neither", None).is_err());
  }

  #[test]
  fn test_validate_ref_not() {
    let rule = Rule::<String>::MinLength(5).not();
    assert!(rule.validate_ref("hi", None).is_ok()); // Less than 5 chars, so NOT passes
    assert!(rule.validate_ref("hello", None).is_err()); // 5 chars, so NOT fails
  }

  #[test]
  fn test_validate_ref_when() {
    let rule = Rule::<String>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::MinLength(5)),
      else_rule: None,
    };
    assert!(rule.validate_ref("", None).is_ok()); // Empty, condition false, no rule applied
    assert!(rule.validate_ref("hello", None).is_ok()); // Not empty, MinLength(5) passes
    assert!(rule.validate_ref("hi", None).is_err()); // Not empty, MinLength(5) fails
  }

  #[test]
  fn test_validate_ref_with_message() {
    let rule = Rule::<String>::MinLength(8).with_message("Password too short.");
    let result = rule.validate_ref("hi", None);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(violation.message(), "Password too short.");
  }

  #[test]
  fn test_validate_ref_all_violations() {
    let rule = Rule::<String>::MinLength(3)
      .and(Rule::MaxLength(5))
      .and(Rule::Pattern(r"^\d+$".to_string()));

    assert!(rule.validate_ref_all("123", None).is_ok());

    let result = rule.validate_ref_all("ab", None);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.len() >= 1); // At least TooShort
  }

  // ========================================================================
  // Option Validation (String) Tests
  // ========================================================================

  #[test]
  fn test_validate_ref_option_none_non_required() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_ref_option(None, None).is_ok());

    let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
    assert!(rule.validate_ref_option(None, None).is_ok());

    let rule = Rule::<String>::Email;
    assert!(rule.validate_ref_option(None, None).is_ok());
  }

  #[test]
  fn test_validate_ref_option_none_required() {
    let rule = Rule::<String>::Required;
    assert!(rule.validate_ref_option(None, None).is_err());

    let violation = rule.validate_ref_option(None, None).unwrap_err();
    assert_eq!(
      violation.violation_type(),
      crate::ViolationType::ValueMissing
    );
  }

  #[test]
  fn test_validate_ref_option_none_all_with_required() {
    let rule = Rule::<String>::Required.and(Rule::MinLength(3));
    assert!(rule.validate_ref_option(None, None).is_err());
  }

  #[test]
  fn test_validate_ref_option_none_all_without_required() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    assert!(rule.validate_ref_option(None, None).is_ok());
  }

  #[test]
  fn test_validate_ref_option_some_valid() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_ref_option(Some("hello"), None).is_ok());
  }

  #[test]
  fn test_validate_ref_option_some_invalid() {
    let rule = Rule::<String>::MinLength(5);
    assert!(rule.validate_ref_option(Some("hi"), None).is_err());
  }

  #[test]
  fn test_validate_ref_option_all() {
    let rule = Rule::<String>::Required.and(Rule::MinLength(3));

    let result = rule.validate_ref_option_all(None, None);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1);

    assert!(rule.validate_ref_option_all(Some("hello"), None).is_ok());
    assert!(rule.validate_ref_option_all(Some("hi"), None).is_err());
  }

  // ========================================================================
  // CompiledRule (String) Tests
  // ========================================================================

  #[test]
  fn test_compiled_rule_string_basic() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    let compiled = rule.compile();

    assert!(compiled.validate_ref("hello").is_ok());
    assert!(compiled.validate_ref("hi").is_err());
    assert!(compiled.validate_ref("hello world!").is_err());
  }

  #[test]
  fn test_compiled_rule_pattern_cached() {
    let rule = Rule::<String>::Pattern(r"^\d{3}-\d{4}$".to_string());
    let compiled = rule.compile();

    assert!(compiled.validate_ref("123-4567").is_ok());
    assert!(compiled.validate_ref("999-0000").is_ok());
    assert!(compiled.validate_ref("abc-defg").is_err());
    assert!(compiled.validate_ref("12-345").is_err());
  }

  #[test]
  fn test_compiled_rule_email() {
    let rule = Rule::<String>::Email;
    let compiled = rule.compile();

    assert!(compiled.validate_ref("user@example.com").is_ok());
    assert!(compiled.validate_ref("test@sub.domain.org").is_ok());
    assert!(compiled.validate_ref("invalid").is_err());
  }

  #[test]
  fn test_compiled_rule_url() {
    let rule = Rule::<String>::Url;
    let compiled = rule.compile();

    assert!(compiled.validate_ref("http://example.com").is_ok());
    assert!(compiled.validate_ref("https://example.com/path?query=1").is_ok());
    assert!(compiled.validate_ref("not-a-url").is_err());
  }

  #[test]
  fn test_compiled_rule_clone() {
    let rule = Rule::<String>::Pattern(r"^\w+$".to_string());
    let compiled = rule.compile();

    assert!(compiled.validate_ref("hello").is_ok());

    let cloned = compiled.clone();
    assert!(cloned.validate_ref("world").is_ok());
  }

  #[test]
  fn test_compiled_rule_debug() {
    let rule = Rule::<String>::MinLength(5);
    let compiled = rule.compile();
    let debug_str = format!("{:?}", compiled);
    assert!(debug_str.contains("CompiledRule"));
    assert!(debug_str.contains("MinLength"));
  }

  #[test]
  fn test_compiled_rule_into_rule() {
    let rule = Rule::<String>::Required;
    let compiled = rule.clone().compile();
    let recovered = compiled.into_rule();
    assert_eq!(recovered, rule);
  }

  #[test]
  fn test_compiled_rule_with_trait() {
    use crate::ValidateRef;

    let rule = Rule::<String>::MinLength(3);
    let compiled = rule.compile();

    let validator: &dyn ValidateRef<str> = &compiled;
    assert!(validator.validate_ref("hello").is_ok());
    assert!(validator.validate_ref("hi").is_err());
  }

  #[test]
  fn test_compiled_rule_validate_all() {
    let rule = Rule::<String>::MinLength(3)
      .and(Rule::MaxLength(5))
      .and(Rule::Pattern(r"^[a-z]+$".to_string()));
    let compiled = rule.compile();

    assert!(compiled.validate_ref_all("abc").is_ok());

    let result = compiled.validate_ref_all("AB");
    assert!(result.is_err());
  }
}

