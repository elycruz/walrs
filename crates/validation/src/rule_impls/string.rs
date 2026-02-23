use crate::rule::{Rule, RuleResult};
use crate::Violation;
use crate::traits::ValidateRef;
use crate::CompiledRule;
use crate::options::{UriOptions, UrlOptions, IpOptions};

// ============================================================================
// URI / IP Validation Helpers
// ============================================================================

/// Validates a string as a URI according to the given options.
fn validate_uri(value: &str, opts: &UriOptions) -> RuleResult {
  if !opts.allow_absolute && !opts.allow_relative {
    return Err(Violation::invalid_uri());
  }

  // Try parsing as an absolute URI
  match url::Url::parse(value) {
    Ok(parsed) => {
      if !opts.allow_absolute {
        return Err(Violation::invalid_uri());
      }
      // Check allowed schemes
      if let Some(schemes) = &opts.allowed_schemes {
        let scheme = parsed.scheme();
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case(scheme)) {
          return Err(Violation::invalid_uri());
        }
      }
      Ok(())
    }
    Err(_) => {
      // Not an absolute URI — check if relative is allowed
      if opts.allow_relative && !value.is_empty() {
        Ok(())
      } else {
        Err(Violation::invalid_uri())
      }
    }
  }
}

/// Validates a string as a URL using the `url` crate's WHATWG parser.
fn validate_url(value: &str, opts: &UrlOptions) -> RuleResult {
  match url::Url::parse(value) {
    Ok(parsed) => {
      if let Some(schemes) = &opts.allowed_schemes {
        let scheme = parsed.scheme();
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case(scheme)) {
          return Err(Violation::invalid_url());
        }
      }
      Ok(())
    }
    Err(_) => Err(Violation::invalid_url()),
  }
}

/// Validates a string as an IP address according to the given options.
fn validate_ip(value: &str, opts: &IpOptions) -> RuleResult {
  // Handle bracket-literal notation
  let inner = if value.starts_with('[') && value.ends_with(']') {
    if !opts.allow_literal {
      return Err(Violation::invalid_ip());
    }
    &value[1..value.len() - 1]
  } else {
    value
  };

  // Try IPvFuture: v<hex>.<unreserved/sub-delims/:>
  if opts.allow_ipvfuture && is_ipvfuture(inner) {
    return Ok(());
  }

  // Try IPv4
  if opts.allow_ipv4 {
    if let Ok(_) = inner.parse::<std::net::Ipv4Addr>() {
      return Ok(());
    }
  }

  // Try IPv6
  if opts.allow_ipv6 {
    if let Ok(_) = inner.parse::<std::net::Ipv6Addr>() {
      return Ok(());
    }
  }

  Err(Violation::invalid_ip())
}

/// Checks if a string matches IPvFuture syntax per RFC 3986 §3.2.2:
/// `v<hex-digit>+.<unreserved / sub-delims / ":">`
fn is_ipvfuture(s: &str) -> bool {
  let bytes = s.as_bytes();
  if bytes.len() < 4 {
    return false;
  }
  if bytes[0] != b'v' && bytes[0] != b'V' {
    return false;
  }

  // Find the dot separator
  let dot_pos = match bytes.iter().position(|&b| b == b'.') {
    Some(p) if p > 1 => p,
    _ => return false,
  };

  // Hex digits between 'v' and '.'
  if !bytes[1..dot_pos].iter().all(|b| b.is_ascii_hexdigit()) {
    return false;
  }

  // After the dot: unreserved / sub-delims / ":"
  if dot_pos + 1 >= bytes.len() {
    return false;
  }
  bytes[dot_pos + 1..].iter().all(|&b| {
    b.is_ascii_alphanumeric()
      || b"-._~!$&'()*+,;=:".contains(&b)
  })
}

/// Cached validators for a compiled rule.
///
/// This struct holds compiled regex patterns for string validation rules.
/// Included in `CompiledRule` for all types, but only populated for String rules.
#[derive(Debug, Default)]
pub(crate) struct CachedStringValidators {
  /// Cached regex for Pattern rules
  pub(crate) pattern_regex: Option<regex::Regex>,
  /// Cached email regex
  pub(crate) email_regex: Option<regex::Regex>,
}

impl CachedStringValidators {
  pub(crate) fn new() -> Self {
    Self::default()
  }
}

impl Rule<String> {
  /// Validates a string value against this rule.
  pub fn validate_str(&self, value: &str) -> RuleResult {
    self.validate_str_inner(value, None)
  }

  /// Internal validation with inherited locale from an outer `WithMessage`.
  ///
  /// The `inherited_locale` is passed down through the rule tree so that
  /// inner `WithMessage` nodes can use it when their own locale is `None`.
  fn validate_str_inner(&self, value: &str, inherited_locale: Option<&str>) -> RuleResult {
    match self {
      Rule::Required => {
        if value.trim().is_empty() {
          Err(Violation::value_missing())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.chars().count();
        if len < *min {
          Err(Violation::too_short(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.chars().count();
        if len > *max {
          Err(Violation::too_long(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.chars().count();
        if len != *expected {
          Err(Violation::exact_length(*expected, len))
        } else {
          Ok(())
        }
      }
      Rule::Pattern(pattern) => match regex::Regex::new(pattern) {
        Ok(re) => {
          if re.is_match(value) {
            Ok(())
          } else {
            Err(Violation::pattern_mismatch(pattern))
          }
        }
        Err(_) => Err(Violation::pattern_mismatch(pattern)),
      },
      Rule::Email => {
        // Simple email validation using regex
        let email_re =
          regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if email_re.is_match(value) {
          Ok(())
        } else {
          Err(Violation::invalid_email())
        }
      }
      Rule::Url(opts) => validate_url(value, opts),
      Rule::Uri(opts) => validate_uri(value, opts),
      Rule::Ip(opts) => validate_ip(value, opts),
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(Violation::not_equal(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.iter().any(|v| v.as_str() == value) {
          Ok(())
        } else {
          Err(Violation::not_one_of())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_str_inner(value, inherited_locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_str_inner(value, inherited_locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_str_inner(value, inherited_locale) {
        Ok(()) => Err(Violation::negation_failed()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          then_rule.validate_str_inner(value, inherited_locale)
        } else {
          match else_rule {
            Some(rule) => rule.validate_str_inner(value, inherited_locale),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value.to_string()),
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage { rule, message, locale } => {
        // Use this variant's locale if set, otherwise inherit from parent
        let effective_locale = locale.as_deref().or(inherited_locale);
        match rule.validate_str_inner(value, effective_locale) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg = message.resolve_or(&value.to_string(), violation.message(), effective_locale);
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
        }
      },
      // Numeric rules don't apply to strings - pass through
      Rule::Min(_) | Rule::Max(_) | Rule::Range { .. } | Rule::Step(_) => Ok(()),
    }
  }

  /// Validates a string value and collects all violations.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violations)` with all failures.
  pub fn validate_str_all(
    &self,
    value: &str,
  ) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_violations_str(value, None, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional string value.
  pub fn validate_str_option(&self, value: Option<&str>) -> RuleResult {
    match value {
      Some(v) => self.validate_str(v),
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
    }
  }

  /// Validates an optional string value and collects all violations.
  pub fn validate_str_option_all(
    &self,
    value: Option<&str>,
  ) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_str_all(v),
      None if self.requires_value() => Err(crate::Violations::from(Violation::value_missing())),
      None => Ok(()),
    }
  }

  /// Helper to collect all violations recursively.
  fn collect_violations_str(
    &self,
    value: &str,
    inherited_locale: Option<&str>,
    violations: &mut crate::Violations,
  ) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_violations_str(value, inherited_locale, violations);
        }
      }
      Rule::Any(rules) => {
        // For Any, we only add violations if ALL rules fail
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_violations_str(value, inherited_locale, &mut rule_violations);
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
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          then_rule.collect_violations_str(value, inherited_locale, violations);
        } else if let Some(rule) = else_rule {
          rule.collect_violations_str(value, inherited_locale, violations);
        }
      }
      Rule::WithMessage { rule, message, locale } => {
        let effective_locale = locale.as_deref().or(inherited_locale);
        let mut inner_violations = crate::Violations::default();
        rule.collect_violations_str(value, effective_locale, &mut inner_violations);
        for violation in inner_violations {
          let custom_msg = message.resolve_or(&value.to_string(), violation.message(), effective_locale);
          violations.push(Violation::new(violation.violation_type(), custom_msg));
        }
      }
      _ => {
        if let Err(v) = self.validate_str_inner(value, inherited_locale) {
          violations.push(v);
        }
      }
    }
  }
}

impl ValidateRef<str> for Rule<String> {
  fn validate_ref(&self, value: &str) -> crate::ValidatorResult {
    Rule::validate_str(self, value)
  }
}

impl ValidateRef<str> for CompiledRule<String> {
  fn validate_ref(&self, value: &str) -> crate::ValidatorResult {
    CompiledRule::validate_str(self, value)
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

      cache
    })
  }

  /// Validates a string value using cached validators.
  pub fn validate_str(&self, value: &str) -> RuleResult {
    self.validate_str_with_cache(value, self.get_or_init_cache())
  }

  fn validate_str_with_cache(&self, value: &str, cache: &CachedStringValidators) -> RuleResult {
    match &self.rule {
      Rule::Required => {
        if value.trim().is_empty() {
          Err(Violation::value_missing())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.chars().count();
        if len < *min {
          Err(Violation::too_short(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.chars().count();
        if len > *max {
          Err(Violation::too_long(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.chars().count();
        if len != *expected {
          Err(Violation::exact_length(*expected, len))
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
          Err(Violation::pattern_mismatch(pattern))
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
          Err(Violation::invalid_email())
        }
      }
      Rule::Url(opts) => validate_url(value, opts),
      Rule::Uri(opts) => validate_uri(value, opts),
      Rule::Ip(opts) => validate_ip(value, opts),
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
      Rule::All(rules) => {
        for rule in rules {
          CompiledRule::new(rule.clone()).validate_str(value)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match CompiledRule::new(rule.clone()).validate_str(value) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match CompiledRule::new((**inner).clone()).validate_str(value) {
        Ok(()) => Err(Violation::negation_failed()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          CompiledRule::new((**then_rule).clone()).validate_str(value)
        } else {
          match else_rule {
            Some(rule) => CompiledRule::new((**rule).clone()).validate_str(value),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value.to_string()),
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage { rule, message, locale } => {
        let effective_locale = locale.as_deref();
        match CompiledRule::new((**rule).clone()).validate_str(value) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg = message.resolve_or(&value.to_string(), violation.message(), effective_locale);
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
        }
      }
      Rule::Min(_) | Rule::Max(_) | Rule::Range { .. } | Rule::Step(_) => Ok(()),
    }
  }

  /// Validates a string value and collects all violations.
  pub fn validate_str_all(&self, value: &str) -> Result<(), crate::Violations> {
    self.rule.validate_str_all(value)
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
  fn test_validate_str_required() {
    let rule = Rule::<String>::Required;
    assert!(rule.validate_str("hello").is_ok());
    assert!(rule.validate_str("").is_err());
    assert!(rule.validate_str("   ").is_err());
  }

  #[test]
  fn test_validate_str_min_length() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_str("hello").is_ok());
    assert!(rule.validate_str("abc").is_ok());
    assert!(rule.validate_str("ab").is_err());
    assert!(rule.validate_str("").is_err());
  }

  #[test]
  fn test_validate_str_max_length() {
    let rule = Rule::<String>::MaxLength(5);
    assert!(rule.validate_str("hello").is_ok());
    assert!(rule.validate_str("hi").is_ok());
    assert!(rule.validate_str("").is_ok());
    assert!(rule.validate_str("hello!").is_err());
  }

  #[test]
  fn test_validate_str_exact_length() {
    let rule = Rule::<String>::ExactLength(5);
    assert!(rule.validate_str("hello").is_ok());
    assert!(rule.validate_str("hi").is_err());
    assert!(rule.validate_str("hello!").is_err());
  }

  #[test]
  fn test_validate_str_pattern() {
    let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
    assert!(rule.validate_str("123").is_ok());
    assert!(rule.validate_str("abc").is_err());
    assert!(rule.validate_str("12a").is_err());
  }

  #[test]
  fn test_validate_str_email() {
    let rule = Rule::<String>::Email;
    assert!(rule.validate_str("user@example.com").is_ok());
    assert!(rule.validate_str("user@sub.example.com").is_ok());
    assert!(rule.validate_str("invalid").is_err());
    assert!(rule.validate_str("@example.com").is_err());
  }

  #[test]
  fn test_validate_str_url() {
    let rule = Rule::<String>::Url(crate::UrlOptions::default());
    assert!(rule.validate_str("http://example.com").is_ok());
    assert!(rule.validate_str("https://example.com/path").is_ok());
    assert!(rule.validate_str("not-a-url").is_err());
    assert!(rule.validate_str("ftp://example.com").is_err()); // Only http/https by default
  }

  #[test]
  fn test_validate_str_url_any_scheme() {
    let rule = Rule::<String>::Url(crate::UrlOptions {
      allowed_schemes: None,
    });
    assert!(rule.validate_str("http://example.com").is_ok());
    assert!(rule.validate_str("https://example.com").is_ok());
    assert!(rule.validate_str("ftp://example.com").is_ok());
    assert!(rule.validate_str("custom://example.com").is_ok());
    assert!(rule.validate_str("not-a-url").is_err());
  }

  #[test]
  fn test_validate_str_url_custom_schemes() {
    let rule = Rule::<String>::Url(crate::UrlOptions {
      allowed_schemes: Some(vec!["ftp".into(), "ftps".into()]),
    });
    assert!(rule.validate_str("ftp://example.com").is_ok());
    assert!(rule.validate_str("ftps://example.com").is_ok());
    assert!(rule.validate_str("http://example.com").is_err());
    assert!(rule.validate_str("https://example.com").is_err());
  }

  #[test]
  fn test_validate_str_url_scheme_case_insensitive() {
    let rule = Rule::<String>::Url(crate::UrlOptions {
      allowed_schemes: Some(vec!["https".into()]),
    });
    assert!(rule.validate_str("https://example.com").is_ok());
    assert!(rule.validate_str("HTTPS://example.com").is_ok());
    assert!(rule.validate_str("http://example.com").is_err());
  }

  #[test]
  fn test_validate_str_url_composed_with_required() {
    let rule = Rule::<String>::Required.and(Rule::Url(crate::UrlOptions::default()));
    assert!(rule.validate_str("https://example.com").is_ok());
    assert!(rule.validate_str("").is_err());
    assert!(rule.validate_str("not-a-url").is_err());
  }

  #[test]
  fn test_validate_str_url_compiled() {
    let compiled = Rule::<String>::Url(crate::UrlOptions::default()).compile();
    assert!(compiled.validate_str("http://example.com").is_ok());
    assert!(compiled.validate_str("https://example.com/path?q=1#frag").is_ok());
    assert!(compiled.validate_str("not-a-url").is_err());
    assert!(compiled.validate_str("ftp://example.com").is_err());
  }

  #[test]
  fn test_validate_str_url_with_message() {
    let rule = Rule::<String>::Url(crate::UrlOptions::default())
      .with_message("Please enter a valid URL");
    let err = rule.validate_str("bad").unwrap_err();
    assert_eq!(err.message(), "Please enter a valid URL");
  }

  #[test]
  fn test_validate_str_equals() {
    let rule = Rule::<String>::Equals("secret".to_string());
    assert!(rule.validate_str("secret").is_ok());
    assert!(rule.validate_str("wrong").is_err());
  }

  #[test]
  fn test_validate_str_one_of() {
    let rule = Rule::<String>::OneOf(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert!(rule.validate_str("a").is_ok());
    assert!(rule.validate_str("b").is_ok());
    assert!(rule.validate_str("d").is_err());
  }

  #[test]
  fn test_validate_str_all() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    assert!(rule.validate_str("hello").is_ok());
    assert!(rule.validate_str("hi").is_err());
    assert!(rule.validate_str("hello world!").is_err());
  }

  #[test]
  fn test_validate_str_any() {
    let rule = Rule::<String>::Email.or(Rule::Url(Default::default()));
    assert!(rule.validate_str("user@example.com").is_ok());
    assert!(rule.validate_str("http://example.com").is_ok());
    assert!(rule.validate_str("neither").is_err());
  }

  #[test]
  fn test_validate_str_not() {
    let rule = Rule::<String>::MinLength(5).not();
    assert!(rule.validate_str("hi").is_ok()); // Less than 5 chars, so NOT passes
    assert!(rule.validate_str("hello").is_err()); // 5 chars, so NOT fails
  }

  #[test]
  fn test_validate_str_when() {
    let rule = Rule::<String>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::MinLength(5)),
      else_rule: None,
    };
    assert!(rule.validate_str("").is_ok()); // Empty, condition false, no rule applied
    assert!(rule.validate_str("hello").is_ok()); // Not empty, MinLength(5) passes
    assert!(rule.validate_str("hi").is_err()); // Not empty, MinLength(5) fails
  }

  #[test]
  fn test_validate_str_with_message() {
    let rule = Rule::<String>::MinLength(8).with_message("Password too short.");
    let result = rule.validate_str("hi");
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(violation.message(), "Password too short.");
  }

  #[test]
  fn test_validate_str_all_violations() {
    let rule = Rule::<String>::MinLength(3)
      .and(Rule::MaxLength(5))
      .and(Rule::Pattern(r"^\d+$".to_string()));

    assert!(rule.validate_str_all("123").is_ok());

    let result = rule.validate_str_all("ab");
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.len() >= 1); // At least TooShort
  }

  // ========================================================================
  // Option Validation (String) Tests
  // ========================================================================

  #[test]
  fn test_validate_str_option_none_non_required() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_str_option(None).is_ok());

    let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
    assert!(rule.validate_str_option(None).is_ok());

    let rule = Rule::<String>::Email;
    assert!(rule.validate_str_option(None).is_ok());
  }

  #[test]
  fn test_validate_str_option_none_required() {
    let rule = Rule::<String>::Required;
    assert!(rule.validate_str_option(None).is_err());

    let violation = rule.validate_str_option(None).unwrap_err();
    assert_eq!(
      violation.violation_type(),
      crate::ViolationType::ValueMissing
    );
  }

  #[test]
  fn test_validate_str_option_none_all_with_required() {
    let rule = Rule::<String>::Required.and(Rule::MinLength(3));
    assert!(rule.validate_str_option(None).is_err());
  }

  #[test]
  fn test_validate_str_option_none_all_without_required() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    assert!(rule.validate_str_option(None).is_ok());
  }

  #[test]
  fn test_validate_str_option_some_valid() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_str_option(Some("hello")).is_ok());
  }

  #[test]
  fn test_validate_str_option_some_invalid() {
    let rule = Rule::<String>::MinLength(5);
    assert!(rule.validate_str_option(Some("hi")).is_err());
  }

  #[test]
  fn test_validate_str_option_all() {
    let rule = Rule::<String>::Required.and(Rule::MinLength(3));

    let result = rule.validate_str_option_all(None);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1);

    assert!(rule.validate_str_option_all(Some("hello")).is_ok());
    assert!(rule.validate_str_option_all(Some("hi")).is_err());
  }

  // ========================================================================
  // CompiledRule (String) Tests
  // ========================================================================

  #[test]
  fn test_compiled_rule_string_basic() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    let compiled = rule.compile();

    assert!(compiled.validate_str("hello").is_ok());
    assert!(compiled.validate_str("hi").is_err());
    assert!(compiled.validate_str("hello world!").is_err());
  }

  #[test]
  fn test_compiled_rule_pattern_cached() {
    let rule = Rule::<String>::Pattern(r"^\d{3}-\d{4}$".to_string());
    let compiled = rule.compile();

    assert!(compiled.validate_str("123-4567").is_ok());
    assert!(compiled.validate_str("999-0000").is_ok());
    assert!(compiled.validate_str("abc-defg").is_err());
    assert!(compiled.validate_str("12-345").is_err());
  }

  #[test]
  fn test_compiled_rule_email() {
    let rule = Rule::<String>::Email;
    let compiled = rule.compile();

    assert!(compiled.validate_str("user@example.com").is_ok());
    assert!(compiled.validate_str("test@sub.domain.org").is_ok());
    assert!(compiled.validate_str("invalid").is_err());
  }

  #[test]
  fn test_compiled_rule_url() {
    let rule = Rule::<String>::Url(crate::UrlOptions::default());
    let compiled = rule.compile();

    assert!(compiled.validate_str("http://example.com").is_ok());
    assert!(compiled.validate_str("https://example.com/path?query=1").is_ok());
    assert!(compiled.validate_str("not-a-url").is_err());
  }

  #[test]
  fn test_compiled_rule_clone() {
    let rule = Rule::<String>::Pattern(r"^\w+$".to_string());
    let compiled = rule.compile();

    assert!(compiled.validate_str("hello").is_ok());

    let cloned = compiled.clone();
    assert!(cloned.validate_str("world").is_ok());
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

    assert!(compiled.validate_str_all("abc").is_ok());

    let result = compiled.validate_str_all("AB");
    assert!(result.is_err());
  }

  // ========================================================================
  // URI Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_uri_absolute_default() {
    let rule = Rule::<String>::Uri(crate::UriOptions::default());
    assert!(rule.validate_str("http://example.com").is_ok());
    assert!(rule.validate_str("https://example.com/path?q=1").is_ok());
    assert!(rule.validate_str("ftp://files.example.com").is_ok());
  }

  #[test]
  fn test_validate_uri_relative_default() {
    let rule = Rule::<String>::Uri(crate::UriOptions::default());
    assert!(rule.validate_str("/path/to/resource").is_ok());
    assert!(rule.validate_str("relative/path").is_ok());
    assert!(rule.validate_str("../parent").is_ok());
  }

  #[test]
  fn test_validate_uri_empty_string() {
    let rule = Rule::<String>::Uri(crate::UriOptions::default());
    // Empty string is not a valid relative URI
    assert!(rule.validate_str("").is_err());
  }

  #[test]
  fn test_validate_uri_absolute_only() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: true,
      allow_relative: false,
      allowed_schemes: None,
    });
    assert!(rule.validate_str("http://example.com").is_ok());
    assert!(rule.validate_str("relative/path").is_err());
  }

  #[test]
  fn test_validate_uri_relative_only() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: false,
      allow_relative: true,
      allowed_schemes: None,
    });
    assert!(rule.validate_str("http://example.com").is_err());
    assert!(rule.validate_str("/path/to/resource").is_ok());
  }

  #[test]
  fn test_validate_uri_both_disabled() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: false,
      allow_relative: false,
      allowed_schemes: None,
    });
    assert!(rule.validate_str("http://example.com").is_err());
    assert!(rule.validate_str("relative/path").is_err());
  }

  #[test]
  fn test_validate_uri_allowed_schemes() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: true,
      allow_relative: false,
      allowed_schemes: Some(vec!["https".into()]),
    });
    assert!(rule.validate_str("https://example.com").is_ok());
    assert!(rule.validate_str("http://example.com").is_err());
    assert!(rule.validate_str("ftp://example.com").is_err());
  }

  #[test]
  fn test_validate_uri_scheme_case_insensitive() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: true,
      allow_relative: false,
      allowed_schemes: Some(vec!["HTTPS".into()]),
    });
    assert!(rule.validate_str("https://example.com").is_ok());
  }

  #[test]
  fn test_validate_uri_with_message() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: true,
      allow_relative: false,
      allowed_schemes: Some(vec!["https".into()]),
    })
    .with_message("Must be a secure URL.");

    let result = rule.validate_str("http://example.com");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Must be a secure URL.");
  }

  #[test]
  fn test_validate_uri_composed() {
    let rule = Rule::<String>::Required.and(Rule::Uri(crate::UriOptions::default()));
    assert!(rule.validate_str("http://example.com").is_ok());
    assert!(rule.validate_str("").is_err());
  }

  #[test]
  fn test_compiled_rule_uri() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: true,
      allow_relative: false,
      allowed_schemes: Some(vec!["https".into()]),
    });
    let compiled = rule.compile();
    assert!(compiled.validate_str("https://example.com").is_ok());
    assert!(compiled.validate_str("http://example.com").is_err());
  }

  // ========================================================================
  // IP Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_ip_v4_default() {
    let rule = Rule::<String>::Ip(crate::IpOptions::default());
    assert!(rule.validate_str("192.168.1.1").is_ok());
    assert!(rule.validate_str("0.0.0.0").is_ok());
    assert!(rule.validate_str("255.255.255.255").is_ok());
  }

  #[test]
  fn test_validate_ip_v4_invalid() {
    let rule = Rule::<String>::Ip(crate::IpOptions::default());
    assert!(rule.validate_str("256.1.1.1").is_err());
    assert!(rule.validate_str("not-an-ip").is_err());
    assert!(rule.validate_str("").is_err());
  }

  #[test]
  fn test_validate_ip_v6_default() {
    let rule = Rule::<String>::Ip(crate::IpOptions::default());
    assert!(rule.validate_str("::1").is_ok());
    assert!(rule.validate_str("2001:db8::1").is_ok());
    assert!(rule.validate_str("fe80::1%25eth0").is_err()); // zone id not valid for std parser
  }

  #[test]
  fn test_validate_ip_v4_only() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipv4: true,
      allow_ipv6: false,
      ..Default::default()
    });
    assert!(rule.validate_str("192.168.1.1").is_ok());
    assert!(rule.validate_str("::1").is_err());
  }

  #[test]
  fn test_validate_ip_v6_only() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipv4: false,
      allow_ipv6: true,
      ..Default::default()
    });
    assert!(rule.validate_str("192.168.1.1").is_err());
    assert!(rule.validate_str("::1").is_ok());
  }

  #[test]
  fn test_validate_ip_literal_brackets() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_literal: true,
      ..Default::default()
    });
    assert!(rule.validate_str("[::1]").is_ok());
    assert!(rule.validate_str("[192.168.1.1]").is_ok());
  }

  #[test]
  fn test_validate_ip_literal_disabled() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_literal: false,
      ..Default::default()
    });
    assert!(rule.validate_str("[::1]").is_err());
    assert!(rule.validate_str("::1").is_ok());
  }

  #[test]
  fn test_validate_ip_ipvfuture() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipvfuture: true,
      ..Default::default()
    });
    assert!(rule.validate_str("v1.test").is_ok());
    assert!(rule.validate_str("vFF.hello:world").is_ok());
  }

  #[test]
  fn test_validate_ip_ipvfuture_disabled() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipvfuture: false,
      allow_ipv4: false,
      allow_ipv6: false,
      allow_literal: false,
    });
    assert!(rule.validate_str("v1.test").is_err());
  }

  #[test]
  fn test_validate_ip_ipvfuture_in_brackets() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipvfuture: true,
      allow_literal: true,
      ..Default::default()
    });
    assert!(rule.validate_str("[v1.test]").is_ok());
  }

  #[test]
  fn test_validate_ip_with_message() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipv4: true,
      allow_ipv6: false,
      ..Default::default()
    })
    .with_message("Must be an IPv4 address.");

    let result = rule.validate_str("::1");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Must be an IPv4 address.");
  }

  #[test]
  fn test_validate_ip_composed() {
    let rule = Rule::<String>::Required.and(Rule::Ip(crate::IpOptions::default()));
    assert!(rule.validate_str("192.168.1.1").is_ok());
    assert!(rule.validate_str("").is_err());
  }

  #[test]
  fn test_compiled_rule_ip() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipv4: true,
      allow_ipv6: false,
      ..Default::default()
    });
    let compiled = rule.compile();
    assert!(compiled.validate_str("192.168.1.1").is_ok());
    assert!(compiled.validate_str("::1").is_err());
  }

  #[test]
  fn test_validate_ip_all_violations() {
    let rule = Rule::<String>::Required
      .and(Rule::Ip(crate::IpOptions { allow_ipv4: true, allow_ipv6: false, ..Default::default() }));

    assert!(rule.validate_str_all("192.168.1.1").is_ok());

    let result = rule.validate_str_all("::1");
    assert!(result.is_err());
  }

  // ========================================================================
  // Serialization Tests for Uri / Ip
  // ========================================================================

  #[test]
  fn test_uri_rule_serialization() {
    let rule = Rule::<String>::Uri(crate::UriOptions {
      allow_absolute: true,
      allow_relative: false,
      allowed_schemes: Some(vec!["https".into()]),
    });
    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(rule, deserialized);
  }

  #[test]
  fn test_ip_rule_serialization() {
    let rule = Rule::<String>::Ip(crate::IpOptions {
      allow_ipv4: true,
      allow_ipv6: false,
      allow_ipvfuture: true,
      allow_literal: false,
    });
    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(rule, deserialized);
  }

  // ========================================================================
  // Convenience Constructor Tests
  // ========================================================================

  #[test]
  fn test_uri_constructor() {
    let opts = crate::UriOptions::default();
    let rule = Rule::<String>::uri(opts.clone());
    assert_eq!(rule, Rule::Uri(opts));
  }

  #[test]
  fn test_ip_constructor() {
    let opts = crate::IpOptions::default();
    let rule = Rule::<String>::ip(opts.clone());
    assert_eq!(rule, Rule::Ip(opts));
  }
}

