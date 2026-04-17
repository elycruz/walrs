//! Date validation helpers using the `chrono` crate.

use chrono::NaiveDate;
use chrono::NaiveDateTime;

use crate::options::{DateFormat, DateOptions, DateRangeOptions};
use crate::rule::{Rule, RuleResult};
use crate::traits::{IsEmpty, Validate, ValidateRef};
use crate::{Violation, ViolationType};

// ============================================================================
// IsEmpty Implementations
// ============================================================================

impl IsEmpty for NaiveDate {
  fn is_empty(&self) -> bool {
    false
  }
}

impl IsEmpty for NaiveDateTime {
  fn is_empty(&self) -> bool {
    false
  }
}

// ============================================================================
// String Parsing Helpers
// ============================================================================

/// Format string for US-style date: `MM/DD/YYYY`
const US_DATE_FMT: &str = "%m/%d/%Y";
/// Format string for EU-style date: `DD/MM/YYYY`
const EU_DATE_FMT: &str = "%d/%m/%Y";
/// Format string for US-style datetime: `MM/DD/YYYY HH:MM:SS`
const US_DATETIME_FMT: &str = "%m/%d/%Y %H:%M:%S";
/// Format string for EU-style datetime: `DD/MM/YYYY HH:MM:SS`
const EU_DATETIME_FMT: &str = "%d/%m/%Y %H:%M:%S";

/// Parses a date string using the given `DateFormat`, returning a `NaiveDate`.
pub(crate) fn parse_date_str(value: &str, format: &DateFormat) -> Result<NaiveDate, ()> {
  match format {
    DateFormat::Iso8601 => NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| ()),
    DateFormat::UsDate => NaiveDate::parse_from_str(value, US_DATE_FMT).map_err(|_| ()),
    DateFormat::EuDate => NaiveDate::parse_from_str(value, EU_DATE_FMT).map_err(|_| ()),
    DateFormat::Rfc2822 => {
      // RFC 2822 typically includes time; try parsing as datetime and extract date
      chrono::DateTime::parse_from_rfc2822(value)
        .map(|dt| dt.date_naive())
        .map_err(|_| ())
    }
    DateFormat::Custom(fmt) => NaiveDate::parse_from_str(value, fmt).map_err(|_| ()),
  }
}

/// Parses a datetime string using the given `DateFormat`, returning a `NaiveDateTime`.
pub(crate) fn parse_datetime_str(value: &str, format: &DateFormat) -> Result<NaiveDateTime, ()> {
  match format {
    DateFormat::Iso8601 => {
      // Try with 'T' separator first, then space
      NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S"))
        .map_err(|_| ())
    }
    DateFormat::UsDate => NaiveDateTime::parse_from_str(value, US_DATETIME_FMT).map_err(|_| ()),
    DateFormat::EuDate => NaiveDateTime::parse_from_str(value, EU_DATETIME_FMT).map_err(|_| ()),
    DateFormat::Rfc2822 => chrono::DateTime::parse_from_rfc2822(value)
      .map(|dt| dt.naive_local())
      .map_err(|_| ()),
    DateFormat::Custom(fmt) => NaiveDateTime::parse_from_str(value, fmt).map_err(|_| ()),
  }
}

/// Parses a bound string as a `NaiveDate` using the given `DateFormat`.
fn parse_bound_date(s: &str, format: &DateFormat) -> Result<NaiveDate, ()> {
  parse_date_str(s, format)
}

/// Parses a bound string as a `NaiveDateTime` using the given `DateFormat`.
fn parse_bound_datetime(s: &str, format: &DateFormat) -> Result<NaiveDateTime, ()> {
  parse_datetime_str(s, format)
}

// ============================================================================
// String Validation Functions
// ============================================================================

/// Validates a string as a date per `DateOptions`.
pub(crate) fn validate_date_str(value: &str, opts: &DateOptions) -> RuleResult {
  if opts.allow_time {
    // Try datetime first, fall back to date-only
    if parse_datetime_str(value, &opts.format).is_ok() {
      return Ok(());
    }
    if parse_date_str(value, &opts.format).is_ok() {
      return Ok(());
    }
  } else {
    if parse_date_str(value, &opts.format).is_ok() {
      return Ok(());
    }
  }
  Err(Violation::invalid_date())
}

/// Validates a string as a date within a range per `DateRangeOptions`.
pub(crate) fn validate_date_range_str(value: &str, opts: &DateRangeOptions) -> RuleResult {
  if opts.allow_time {
    // Try datetime first
    if let Ok(dt) = parse_datetime_str(value, &opts.format) {
      if let Some(min_str) = &opts.min {
        // Try datetime bound first; fall back to date-only bound (compare date component)
        if let Ok(min_dt) = parse_bound_datetime(min_str, &opts.format) {
          if dt < min_dt {
            return Err(Violation::date_range_underflow(min_str));
          }
        } else if let Ok(min_d) = parse_bound_date(min_str, &opts.format) {
          if dt.date() < min_d {
            return Err(Violation::date_range_underflow(min_str));
          }
        } else if !min_str.is_empty() {
          // Misconfigured min bound: non-empty but unparseable
          return Err(Violation::new(
            ViolationType::CustomError,
            format!(
              "Invalid min date bound: '{}' cannot be parsed in the configured format.",
              min_str
            ),
          ));
        }
      }
      if let Some(max_str) = &opts.max {
        // Try datetime bound first; fall back to date-only bound (compare date component)
        if let Ok(max_dt) = parse_bound_datetime(max_str, &opts.format) {
          if dt > max_dt {
            return Err(Violation::date_range_overflow(max_str));
          }
        } else if let Ok(max_d) = parse_bound_date(max_str, &opts.format) {
          if dt.date() > max_d {
            return Err(Violation::date_range_overflow(max_str));
          }
        } else if !max_str.is_empty() {
          // Misconfigured max bound: non-empty but unparseable
          return Err(Violation::new(
            ViolationType::CustomError,
            format!(
              "Invalid max date bound: '{}' cannot be parsed in the configured format.",
              max_str
            ),
          ));
        }
      }
      return Ok(());
    }
    // Fall back to date-only
    if let Ok(d) = parse_date_str(value, &opts.format) {
      return check_date_bounds(d, &opts.min, &opts.max, &opts.format);
    }
  } else {
    if let Ok(d) = parse_date_str(value, &opts.format) {
      return check_date_bounds(d, &opts.min, &opts.max, &opts.format);
    }
  }
  Err(Violation::invalid_date())
}

fn check_date_bounds(
  d: NaiveDate,
  min: &Option<String>,
  max: &Option<String>,
  format: &DateFormat,
) -> RuleResult {
  if let Some(min_str) = min {
    // Try date-only bound first; fall back to datetime bound (extract date component)
    let min_d = parse_bound_date(min_str, format)
      .or_else(|_| parse_bound_datetime(min_str, format).map(|dt| dt.date()));
    match min_d {
      Ok(min_d) => {
        if d < min_d {
          return Err(Violation::date_range_underflow(min_str));
        }
      }
      Err(_) if !min_str.is_empty() => {
        // Misconfigured min bound: non-empty but unparseable
        return Err(Violation::new(
          ViolationType::CustomError,
          format!(
            "Invalid min date bound: '{}' cannot be parsed in the configured format.",
            min_str
          ),
        ));
      }
      _ => {}
    }
  }
  if let Some(max_str) = max {
    // Try date-only bound first; fall back to datetime bound (extract date component)
    let max_d = parse_bound_date(max_str, format)
      .or_else(|_| parse_bound_datetime(max_str, format).map(|dt| dt.date()));
    match max_d {
      Ok(max_d) => {
        if d > max_d {
          return Err(Violation::date_range_overflow(max_str));
        }
      }
      Err(_) if !max_str.is_empty() => {
        // Misconfigured max bound: non-empty but unparseable
        return Err(Violation::new(
          ViolationType::CustomError,
          format!(
            "Invalid max date bound: '{}' cannot be parsed in the configured format.",
            max_str
          ),
        ));
      }
      _ => {}
    }
  }
  Ok(())
}

// ============================================================================
// Native Type Validation: Rule<NaiveDate>
// ============================================================================

impl Rule<NaiveDate> {
  /// Validates a `NaiveDate` value against this rule.
  pub fn validate_date(&self, value: &NaiveDate) -> RuleResult {
    self.validate_date_inner(value, None)
  }

  fn validate_date_inner(&self, value: &NaiveDate, inherited_locale: Option<&str>) -> RuleResult {
    match self {
      Rule::Required => Ok(()), // A present NaiveDate is never empty
      Rule::Min(min) => {
        if value < min {
          Err(Violation::range_underflow(min))
        } else {
          Ok(())
        }
      }
      Rule::Max(max) => {
        if value > max {
          Err(Violation::range_overflow(max))
        } else {
          Ok(())
        }
      }
      Rule::Range { min, max } => {
        if value < min {
          Err(Violation::range_underflow(min))
        } else if value > max {
          Err(Violation::range_overflow(max))
        } else {
          Ok(())
        }
      }
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(Violation::not_equal(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.contains(value) {
          Ok(())
        } else {
          Err(Violation::not_one_of())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_date_inner(value, inherited_locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_date_inner(value, inherited_locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_date_inner(value, inherited_locale) {
        Ok(()) => Err(Violation::negation_failed()),
        Err(_) => Ok(()),
      },
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
        match message {
          Some(msg) => msg.wrap_result(rule.validate_date_inner(value, eff), value, eff),
          None => rule.validate_date_inner(value, eff),
        }
      }
      // Inapplicable rules pass through
      _ => Ok(()),
    }
  }
}

impl Validate<NaiveDate> for Rule<NaiveDate> {
  fn validate(&self, value: NaiveDate) -> crate::ValidatorResult {
    self.validate_date(&value)
  }
}

impl ValidateRef<NaiveDate> for Rule<NaiveDate> {
  fn validate_ref(&self, value: &NaiveDate) -> crate::ValidatorResult {
    self.validate_date(value)
  }
}

impl Validate<Option<NaiveDate>> for Rule<NaiveDate> {
  fn validate(&self, value: Option<NaiveDate>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_date(&v),
    }
  }
}

impl ValidateRef<Option<NaiveDate>> for Rule<NaiveDate> {
  fn validate_ref(&self, value: &Option<NaiveDate>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_date(v),
    }
  }
}

// ============================================================================
// Async NaiveDate Validation
// ============================================================================

#[cfg(feature = "async")]
impl Rule<NaiveDate> {
  /// Validates a `chrono::NaiveDate` value asynchronously.
  ///
  /// Runs all rules: sync rules execute inline, `CustomAsync` rules are awaited.
  pub(crate) async fn validate_date_async(&self, value: &NaiveDate) -> RuleResult {
    self.validate_date_async_inner(value, None).await
  }

  fn validate_date_async_inner<'a>(
    &'a self,
    value: &'a NaiveDate,
    inherited_locale: Option<&'a str>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RuleResult> + Send + 'a>> {
    Box::pin(async move {
      match self {
        Rule::CustomAsync(f) => f(value).await,

        Rule::All(rules) => {
          for rule in rules {
            rule
              .validate_date_async_inner(value, inherited_locale)
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
              .validate_date_async_inner(value, inherited_locale)
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
            .validate_date_async_inner(value, inherited_locale)
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
          if condition.evaluate(value) {
            then_rule
              .validate_date_async_inner(value, inherited_locale)
              .await
          } else {
            match else_rule {
              Some(rule) => {
                rule
                  .validate_date_async_inner(value, inherited_locale)
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
            Some(msg) => msg.wrap_result(rule.validate_date_async_inner(value, eff).await, value, eff),
            None => rule.validate_date_async_inner(value, eff).await,
          }
        }

        // All sync rules — delegate to sync validation
        other => other.validate_date_inner(value, inherited_locale),
      }
    })
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<NaiveDate> for Rule<NaiveDate> {
  async fn validate_async(&self, value: NaiveDate) -> crate::ValidatorResult {
    self.validate_date_async(&value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<NaiveDate> for Rule<NaiveDate> {
  async fn validate_ref_async(&self, value: &NaiveDate) -> crate::ValidatorResult {
    self.validate_date_async(value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<Option<NaiveDate>> for Rule<NaiveDate> {
  async fn validate_async(&self, value: Option<NaiveDate>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(ref v) => self.validate_date_async(v).await,
    }
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<Option<NaiveDate>> for Rule<NaiveDate> {
  async fn validate_ref_async(&self, value: &Option<NaiveDate>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_date_async(v).await,
    }
  }
}

// ============================================================================
// Native Type Validation: Rule<NaiveDateTime>
// ============================================================================

impl Rule<NaiveDateTime> {
  /// Validates a `NaiveDateTime` value against this rule.
  pub fn validate_datetime(&self, value: &NaiveDateTime) -> RuleResult {
    self.validate_datetime_inner(value, None)
  }

  fn validate_datetime_inner(
    &self,
    value: &NaiveDateTime,
    inherited_locale: Option<&str>,
  ) -> RuleResult {
    match self {
      Rule::Required => Ok(()),
      Rule::Min(min) => {
        if value < min {
          Err(Violation::range_underflow(min))
        } else {
          Ok(())
        }
      }
      Rule::Max(max) => {
        if value > max {
          Err(Violation::range_overflow(max))
        } else {
          Ok(())
        }
      }
      Rule::Range { min, max } => {
        if value < min {
          Err(Violation::range_underflow(min))
        } else if value > max {
          Err(Violation::range_overflow(max))
        } else {
          Ok(())
        }
      }
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(Violation::not_equal(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.contains(value) {
          Ok(())
        } else {
          Err(Violation::not_one_of())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_datetime_inner(value, inherited_locale)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_datetime_inner(value, inherited_locale) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_datetime_inner(value, inherited_locale) {
        Ok(()) => Err(Violation::negation_failed()),
        Err(_) => Ok(()),
      },
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
        match message {
          Some(msg) => msg.wrap_result(rule.validate_datetime_inner(value, eff), value, eff),
          None => rule.validate_datetime_inner(value, eff),
        }
      }
      _ => Ok(()),
    }
  }
}

impl Validate<NaiveDateTime> for Rule<NaiveDateTime> {
  fn validate(&self, value: NaiveDateTime) -> crate::ValidatorResult {
    self.validate_datetime(&value)
  }
}

impl ValidateRef<NaiveDateTime> for Rule<NaiveDateTime> {
  fn validate_ref(&self, value: &NaiveDateTime) -> crate::ValidatorResult {
    self.validate_datetime(value)
  }
}

impl Validate<Option<NaiveDateTime>> for Rule<NaiveDateTime> {
  fn validate(&self, value: Option<NaiveDateTime>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_datetime(&v),
    }
  }
}

impl ValidateRef<Option<NaiveDateTime>> for Rule<NaiveDateTime> {
  fn validate_ref(&self, value: &Option<NaiveDateTime>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_datetime(v),
    }
  }
}

// ============================================================================
// Async NaiveDateTime Validation
// ============================================================================

#[cfg(feature = "async")]
impl Rule<NaiveDateTime> {
  /// Validates a `chrono::NaiveDateTime` value asynchronously.
  ///
  /// Runs all rules: sync rules execute inline, `CustomAsync` rules are awaited.
  pub(crate) async fn validate_datetime_async(&self, value: &NaiveDateTime) -> RuleResult {
    self.validate_datetime_async_inner(value, None).await
  }

  fn validate_datetime_async_inner<'a>(
    &'a self,
    value: &'a NaiveDateTime,
    inherited_locale: Option<&'a str>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RuleResult> + Send + 'a>> {
    Box::pin(async move {
      match self {
        Rule::CustomAsync(f) => f(value).await,

        Rule::All(rules) => {
          for rule in rules {
            rule
              .validate_datetime_async_inner(value, inherited_locale)
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
              .validate_datetime_async_inner(value, inherited_locale)
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
            .validate_datetime_async_inner(value, inherited_locale)
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
          if condition.evaluate(value) {
            then_rule
              .validate_datetime_async_inner(value, inherited_locale)
              .await
          } else {
            match else_rule {
              Some(rule) => {
                rule
                  .validate_datetime_async_inner(value, inherited_locale)
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
              rule.validate_datetime_async_inner(value, eff).await,
              value,
              eff,
            ),
            None => rule.validate_datetime_async_inner(value, eff).await,
          }
        }

        // All sync rules — delegate to sync validation
        other => other.validate_datetime_inner(value, inherited_locale),
      }
    })
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<NaiveDateTime> for Rule<NaiveDateTime> {
  async fn validate_async(&self, value: NaiveDateTime) -> crate::ValidatorResult {
    self.validate_datetime_async(&value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<NaiveDateTime> for Rule<NaiveDateTime> {
  async fn validate_ref_async(&self, value: &NaiveDateTime) -> crate::ValidatorResult {
    self.validate_datetime_async(value).await
  }
}

#[cfg(feature = "async")]
impl crate::ValidateAsync<Option<NaiveDateTime>> for Rule<NaiveDateTime> {
  async fn validate_async(&self, value: Option<NaiveDateTime>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(ref v) => self.validate_datetime_async(v).await,
    }
  }
}

#[cfg(feature = "async")]
impl crate::ValidateRefAsync<Option<NaiveDateTime>> for Rule<NaiveDateTime> {
  async fn validate_ref_async(&self, value: &Option<NaiveDateTime>) -> crate::ValidatorResult {
    match value {
      None if self.requires_value() => Err(Violation::value_missing()),
      None => Ok(()),
      Some(v) => self.validate_datetime_async(v).await,
    }
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ViolationType;

  // --- String parsing tests ---

  #[test]
  fn test_parse_iso_date() {
    assert!(parse_date_str("2026-02-23", &DateFormat::Iso8601).is_ok());
    assert!(parse_date_str("not-a-date", &DateFormat::Iso8601).is_err());
    assert!(parse_date_str("02/23/2026", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_parse_us_date() {
    assert!(parse_date_str("02/23/2026", &DateFormat::UsDate).is_ok());
    assert!(parse_date_str("2026-02-23", &DateFormat::UsDate).is_err());
  }

  #[test]
  fn test_parse_eu_date() {
    assert!(parse_date_str("23/02/2026", &DateFormat::EuDate).is_ok());
    assert!(parse_date_str("02/23/2026", &DateFormat::EuDate).is_err());
  }

  #[test]
  fn test_parse_custom_date() {
    let fmt = DateFormat::Custom("%d %b %Y".into());
    assert!(parse_date_str("23 Feb 2026", &fmt).is_ok());
    assert!(parse_date_str("2026-02-23", &fmt).is_err());
  }

  #[test]
  fn test_parse_iso_datetime() {
    assert!(parse_datetime_str("2026-02-23T18:00:00", &DateFormat::Iso8601).is_ok());
    assert!(parse_datetime_str("2026-02-23 18:00:00", &DateFormat::Iso8601).is_ok());
    assert!(parse_datetime_str("2026-02-23", &DateFormat::Iso8601).is_err());
  }

  // --- String validation tests ---

  #[test]
  fn test_validate_date_str_iso() {
    let opts = DateOptions::default();
    assert!(validate_date_str("2026-02-23", &opts).is_ok());
    assert!(validate_date_str("not-valid", &opts).is_err());
  }

  #[test]
  fn test_validate_date_str_with_time() {
    let opts = DateOptions {
      format: DateFormat::Iso8601,
      allow_time: true,
    };
    assert!(validate_date_str("2026-02-23T18:30:00", &opts).is_ok());
    assert!(validate_date_str("2026-02-23", &opts).is_ok());
  }

  #[test]
  fn test_validate_date_str_rejects_time_when_not_allowed() {
    let opts = DateOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
    };
    assert!(validate_date_str("2026-02-23T18:30:00", &opts).is_err());
  }

  #[test]
  fn test_validate_date_range_str() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2020-01-01".into()),
      max: Some("2030-12-31".into()),
    };
    assert!(validate_date_range_str("2025-06-15", &opts).is_ok());
    assert_eq!(
      validate_date_range_str("2019-12-31", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeUnderflow,
    );
    assert_eq!(
      validate_date_range_str("2031-01-01", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeOverflow,
    );
  }

  #[test]
  fn test_validate_date_range_str_no_bounds() {
    let opts = DateRangeOptions::default();
    assert!(validate_date_range_str("2099-12-31", &opts).is_ok());
  }

  #[test]
  fn test_validate_date_range_datetime_with_date_only_bounds() {
    // allow_time = true but bounds are date-only: should compare by date component
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: true,
      min: Some("2020-01-01".into()),
      max: Some("2030-12-31".into()),
    };
    // datetime within range
    assert!(validate_date_range_str("2025-06-15T12:00:00", &opts).is_ok());
    // datetime before min date
    assert_eq!(
      validate_date_range_str("2019-12-31T23:59:59", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeUnderflow,
    );
    // datetime after max date
    assert_eq!(
      validate_date_range_str("2031-01-01T00:00:00", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeOverflow,
    );
  }

  #[test]
  fn test_validate_date_range_date_only_with_datetime_bounds() {
    // allow_time = false but bounds include time: date component of bound is used
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2020-01-01T00:00:00".into()),
      max: Some("2030-12-31T23:59:59".into()),
    };
    assert!(validate_date_range_str("2025-06-15", &opts).is_ok());
    assert_eq!(
      validate_date_range_str("2019-12-31", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeUnderflow,
    );
    assert_eq!(
      validate_date_range_str("2031-01-01", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeOverflow,
    );
  }

  // --- Native NaiveDate tests ---

  #[test]
  fn test_rule_naive_date_min() {
    let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let rule = Rule::<NaiveDate>::Min(min);

    let ok_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    let bad_date = NaiveDate::from_ymd_opt(2019, 12, 31).unwrap();

    assert!(rule.validate_date(&ok_date).is_ok());
    assert!(rule.validate_date(&bad_date).is_err());
  }

  #[test]
  fn test_rule_naive_date_range() {
    let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let max = NaiveDate::from_ymd_opt(2030, 12, 31).unwrap();
    let rule = Rule::<NaiveDate>::Range { min, max };

    let in_range = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    let below = NaiveDate::from_ymd_opt(2019, 12, 31).unwrap();
    let above = NaiveDate::from_ymd_opt(2031, 1, 1).unwrap();

    assert!(rule.validate_date(&in_range).is_ok());
    assert!(rule.validate_date(&below).is_err());
    assert!(rule.validate_date(&above).is_err());
  }

  #[test]
  fn test_rule_naive_date_equals() {
    let target = NaiveDate::from_ymd_opt(2026, 2, 23).unwrap();
    let rule = Rule::<NaiveDate>::Equals(target);

    assert!(rule.validate_date(&target).is_ok());
    let other = NaiveDate::from_ymd_opt(2026, 2, 24).unwrap();
    assert!(rule.validate_date(&other).is_err());
  }

  #[test]
  fn test_rule_naive_date_one_of() {
    let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2026, 7, 4).unwrap();
    let rule = Rule::<NaiveDate>::OneOf(vec![d1, d2]);

    assert!(rule.validate_date(&d1).is_ok());
    let other = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
    assert!(rule.validate_date(&other).is_err());
  }

  #[test]
  fn test_rule_naive_date_composites() {
    let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let max = NaiveDate::from_ymd_opt(2030, 12, 31).unwrap();
    let rule = Rule::<NaiveDate>::Min(min).and(Rule::Max(max));

    let ok = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    assert!(rule.validate_date(&ok).is_ok());

    let bad = NaiveDate::from_ymd_opt(2031, 1, 1).unwrap();
    assert!(rule.validate_date(&bad).is_err());
  }

  // --- Native NaiveDateTime tests ---

  #[test]
  fn test_rule_naive_datetime_range() {
    let min = NaiveDate::from_ymd_opt(2020, 1, 1)
      .unwrap()
      .and_hms_opt(0, 0, 0)
      .unwrap();
    let max = NaiveDate::from_ymd_opt(2030, 12, 31)
      .unwrap()
      .and_hms_opt(23, 59, 59)
      .unwrap();
    let rule = Rule::<NaiveDateTime>::Range { min, max };

    let in_range = NaiveDate::from_ymd_opt(2025, 6, 15)
      .unwrap()
      .and_hms_opt(12, 0, 0)
      .unwrap();
    assert!(rule.validate_datetime(&in_range).is_ok());

    let below = NaiveDate::from_ymd_opt(2019, 12, 31)
      .unwrap()
      .and_hms_opt(23, 59, 59)
      .unwrap();
    assert!(rule.validate_datetime(&below).is_err());
  }

  // --- Validate/ValidateRef trait tests ---

  #[test]
  fn test_validate_trait_naive_date() {
    let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let rule = Rule::<NaiveDate>::Min(min);

    let ok_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    assert!(Validate::validate(&rule, ok_date).is_ok());
    assert!(ValidateRef::validate_ref(&rule, &ok_date).is_ok());
  }

  // --- Invalid bound error handling tests ---

  #[test]
  fn test_invalid_min_bound_returns_error() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("invalid-date".into()),
      max: None,
    };

    let result = validate_date_range_str("2025-06-15", &opts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::CustomError);
    assert!(err.message().contains("Invalid min date bound"));
  }

  #[test]
  fn test_invalid_max_bound_returns_error() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: None,
      max: Some("not-a-date".into()),
    };

    let result = validate_date_range_str("2025-06-15", &opts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::CustomError);
    assert!(err.message().contains("Invalid max date bound"));
  }

  #[test]
  fn test_empty_min_bound_is_ignored() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("".into()),
      max: Some("2030-12-31".into()),
    };

    // Empty min bound should be silently ignored
    let result = validate_date_range_str("2025-06-15", &opts);
    assert!(result.is_ok());
  }

  #[test]
  fn test_empty_max_bound_is_ignored() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2020-01-01".into()),
      max: Some("".into()),
    };

    // Empty max bound should be silently ignored
    let result = validate_date_range_str("2025-06-15", &opts);
    assert!(result.is_ok());
  }

  #[test]
  fn test_invalid_min_bound_with_allow_time() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: true,
      min: Some("bad-datetime".into()),
      max: None,
    };

    let result = validate_date_range_str("2025-06-15T12:00:00", &opts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violation_type(), ViolationType::CustomError);
    assert!(err.message().contains("Invalid min date bound"));
  }

  // ========================================================================
  // Edge-case tests: leap years, impossible dates, boundaries, formats
  // ========================================================================

  // --- Leap year tests ---

  #[test]
  fn test_leap_year_feb29_valid() {
    // 2024 is a leap year (divisible by 4)
    assert!(parse_date_str("2024-02-29", &DateFormat::Iso8601).is_ok());
    assert!(validate_date_str("2024-02-29", &DateOptions::default()).is_ok());
  }

  #[test]
  fn test_leap_year_divisible_by_400_valid() {
    // 2000 is a leap year (divisible by 400)
    assert!(parse_date_str("2000-02-29", &DateFormat::Iso8601).is_ok());
    assert!(validate_date_str("2000-02-29", &DateOptions::default()).is_ok());
  }

  #[test]
  fn test_non_leap_year_feb29_invalid() {
    // 2023 is not a leap year
    assert!(parse_date_str("2023-02-29", &DateFormat::Iso8601).is_err());
    assert!(validate_date_str("2023-02-29", &DateOptions::default()).is_err());
  }

  #[test]
  fn test_century_non_leap_year_feb29_invalid() {
    // 1900 is not a leap year (divisible by 100 but not 400)
    assert!(parse_date_str("1900-02-29", &DateFormat::Iso8601).is_err());
    assert!(validate_date_str("1900-02-29", &DateOptions::default()).is_err());
  }

  // --- Impossible date tests ---

  #[test]
  fn test_impossible_feb30() {
    assert!(parse_date_str("2024-02-30", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_apr31() {
    assert!(parse_date_str("2024-04-31", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_jun31() {
    assert!(parse_date_str("2024-06-31", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_sep31() {
    assert!(parse_date_str("2024-09-31", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_nov31() {
    assert!(parse_date_str("2024-11-31", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_month_13() {
    assert!(parse_date_str("2024-13-01", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_month_00() {
    assert!(parse_date_str("2024-00-01", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_day_32() {
    assert!(parse_date_str("2024-01-32", &DateFormat::Iso8601).is_err());
  }

  #[test]
  fn test_impossible_day_00() {
    assert!(parse_date_str("2024-01-00", &DateFormat::Iso8601).is_err());
  }

  // --- Year boundary / DateRange boundary tests ---

  #[test]
  fn test_date_range_at_exact_min_boundary() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2024-01-01".into()),
      max: Some("2024-12-31".into()),
    };
    assert!(validate_date_range_str("2024-01-01", &opts).is_ok());
  }

  #[test]
  fn test_date_range_at_exact_max_boundary() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2024-01-01".into()),
      max: Some("2024-12-31".into()),
    };
    assert!(validate_date_range_str("2024-12-31", &opts).is_ok());
  }

  #[test]
  fn test_date_range_day_before_min() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2024-01-01".into()),
      max: Some("2024-12-31".into()),
    };
    assert_eq!(
      validate_date_range_str("2023-12-31", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeUnderflow,
    );
  }

  #[test]
  fn test_date_range_day_after_max() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2024-01-01".into()),
      max: Some("2024-12-31".into()),
    };
    assert_eq!(
      validate_date_range_str("2025-01-01", &opts)
        .unwrap_err()
        .violation_type(),
      ViolationType::RangeOverflow,
    );
  }

  // --- Format variant tests ---

  #[test]
  fn test_format_iso8601_valid_and_invalid() {
    let opts = DateOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
    };
    assert!(validate_date_str("2024-06-15", &opts).is_ok());
    assert!(validate_date_str("15/06/2024", &opts).is_err());
  }

  #[test]
  fn test_format_us_date_valid_and_invalid() {
    let opts = DateOptions {
      format: DateFormat::UsDate,
      allow_time: false,
    };
    assert!(validate_date_str("06/15/2024", &opts).is_ok());
    assert!(validate_date_str("2024-06-15", &opts).is_err());
  }

  #[test]
  fn test_format_eu_date_valid_and_invalid() {
    let opts = DateOptions {
      format: DateFormat::EuDate,
      allow_time: false,
    };
    assert!(validate_date_str("15/06/2024", &opts).is_ok());
    // month 15 is invalid
    assert!(validate_date_str("06/15/2024", &opts).is_err());
  }

  #[test]
  fn test_format_rfc2822_valid_and_invalid() {
    let opts = DateOptions {
      format: DateFormat::Rfc2822,
      allow_time: true,
    };
    assert!(validate_date_str("Sat, 15 Jun 2024 12:00:00 +0000", &opts).is_ok());
    assert!(validate_date_str("not-a-date", &opts).is_err());
  }

  #[test]
  fn test_format_custom_valid_and_invalid() {
    let opts = DateOptions {
      format: DateFormat::Custom("%d %B %Y".into()),
      allow_time: false,
    };
    assert!(validate_date_str("15 June 2024", &opts).is_ok());
    assert!(validate_date_str("2024-06-15", &opts).is_err());
  }

  // --- Leap year tests with non-ISO formats ---

  #[test]
  fn test_leap_year_us_date_format() {
    let opts = DateOptions {
      format: DateFormat::UsDate,
      allow_time: false,
    };
    assert!(validate_date_str("02/29/2024", &opts).is_ok());
    assert!(validate_date_str("02/29/2023", &opts).is_err());
  }

  #[test]
  fn test_leap_year_eu_date_format() {
    let opts = DateOptions {
      format: DateFormat::EuDate,
      allow_time: false,
    };
    assert!(validate_date_str("29/02/2024", &opts).is_ok());
    assert!(validate_date_str("29/02/2023", &opts).is_err());
  }

  // --- Impossible dates with non-ISO formats ---

  #[test]
  fn test_impossible_dates_us_format() {
    let opts = DateOptions {
      format: DateFormat::UsDate,
      allow_time: false,
    };
    assert!(validate_date_str("02/30/2024", &opts).is_err());
    assert!(validate_date_str("04/31/2024", &opts).is_err());
    assert!(validate_date_str("06/31/2024", &opts).is_err());
  }

  #[test]
  fn test_impossible_dates_eu_format() {
    let opts = DateOptions {
      format: DateFormat::EuDate,
      allow_time: false,
    };
    assert!(validate_date_str("30/02/2024", &opts).is_err());
    assert!(validate_date_str("31/04/2024", &opts).is_err());
    assert!(validate_date_str("31/06/2024", &opts).is_err());
  }

  // ========================================================================
  // Option<NaiveDate> / Option<NaiveDateTime> Validation (trait impls)
  // ========================================================================

  #[test]
  fn test_option_date_none_required() {
    let rule = Rule::<NaiveDate>::Required;
    assert!(rule.validate(None::<NaiveDate>).is_err());
    assert!(rule.validate_ref(&None::<NaiveDate>).is_err());
  }

  #[test]
  fn test_option_date_none_not_required() {
    let d = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let rule = Rule::<NaiveDate>::Min(d);
    assert!(rule.validate(None::<NaiveDate>).is_ok());
    assert!(rule.validate_ref(&None::<NaiveDate>).is_ok());
  }

  #[test]
  fn test_option_date_some_valid() {
    let min = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let val = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let rule = Rule::<NaiveDate>::Min(min);
    assert!(rule.validate(Some(val)).is_ok());
    assert!(rule.validate_ref(&Some(val)).is_ok());
  }

  #[test]
  fn test_option_date_some_invalid() {
    let min = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let val = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let rule = Rule::<NaiveDate>::Min(min);
    assert!(rule.validate(Some(val)).is_err());
    assert!(rule.validate_ref(&Some(val)).is_err());
  }

  #[test]
  fn test_option_datetime_none_required() {
    let rule = Rule::<NaiveDateTime>::Required;
    assert!(rule.validate(None::<NaiveDateTime>).is_err());
    assert!(rule.validate_ref(&None::<NaiveDateTime>).is_err());
  }

  #[test]
  fn test_option_datetime_none_not_required() {
    let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
      .unwrap()
      .and_hms_opt(0, 0, 0)
      .unwrap();
    let rule = Rule::<NaiveDateTime>::Min(dt);
    assert!(rule.validate(None::<NaiveDateTime>).is_ok());
    assert!(rule.validate_ref(&None::<NaiveDateTime>).is_ok());
  }

  #[test]
  fn test_option_datetime_some_valid() {
    let min = NaiveDate::from_ymd_opt(2024, 1, 1)
      .unwrap()
      .and_hms_opt(0, 0, 0)
      .unwrap();
    let val = NaiveDate::from_ymd_opt(2024, 6, 15)
      .unwrap()
      .and_hms_opt(12, 0, 0)
      .unwrap();
    let rule = Rule::<NaiveDateTime>::Min(min);
    assert!(rule.validate(Some(val)).is_ok());
    assert!(rule.validate_ref(&Some(val)).is_ok());
  }

  #[test]
  fn test_option_datetime_some_invalid() {
    let min = NaiveDate::from_ymd_opt(2024, 6, 1)
      .unwrap()
      .and_hms_opt(0, 0, 0)
      .unwrap();
    let val = NaiveDate::from_ymd_opt(2024, 1, 15)
      .unwrap()
      .and_hms_opt(12, 0, 0)
      .unwrap();
    let rule = Rule::<NaiveDateTime>::Min(min);
    assert!(rule.validate(Some(val)).is_err());
    assert!(rule.validate_ref(&Some(val)).is_err());
  }

  // ========================================================================
  // IsEmpty Tests
  // ========================================================================

  #[test]
  fn test_is_empty_naive_date() {
    let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    assert!(!IsEmpty::is_empty(&d));
  }

  #[test]
  fn test_is_empty_naive_datetime() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 15)
      .unwrap()
      .and_hms_opt(12, 0, 0)
      .unwrap();
    assert!(!IsEmpty::is_empty(&dt));
  }

  // ========================================================================
  // Async NaiveDate/NaiveDateTime Validation
  // ========================================================================

  #[cfg(feature = "async")]
  mod async_date_tests {
    use crate::rule::Rule;
    use crate::{ValidateAsync, ValidateRefAsync};
    use chrono::{NaiveDate, NaiveDateTime};

    #[tokio::test]
    async fn test_async_naive_date_min() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
      let rule = Rule::<NaiveDate>::Min(min);
      assert!(
        rule
          .validate_async(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap())
          .await
          .is_ok()
      );
      assert!(
        rule
          .validate_async(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
          .await
          .is_err()
      );
    }

    #[tokio::test]
    async fn test_async_naive_date_ref() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
      let rule = Rule::<NaiveDate>::Min(min);
      let val = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
      assert!(rule.validate_ref_async(&val).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_naive_date_option_none_required() {
      let rule = Rule::<NaiveDate>::Required;
      assert!(rule.validate_async(None::<NaiveDate>).await.is_err());
      assert!(rule.validate_ref_async(&None::<NaiveDate>).await.is_err());
    }

    #[tokio::test]
    async fn test_async_naive_date_option_none_not_required() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
      let rule = Rule::<NaiveDate>::Min(min);
      assert!(rule.validate_async(None::<NaiveDate>).await.is_ok());
      assert!(rule.validate_ref_async(&None::<NaiveDate>).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_naive_date_option_some_valid() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
      let rule = Rule::<NaiveDate>::Min(min);
      let val = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
      assert!(rule.validate_async(Some(val)).await.is_ok());
      assert!(rule.validate_ref_async(&Some(val)).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_naive_datetime_min() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
      let rule = Rule::<NaiveDateTime>::Min(min);
      let ok_val = NaiveDate::from_ymd_opt(2024, 6, 15)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
      let err_val = NaiveDate::from_ymd_opt(2023, 6, 15)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
      assert!(rule.validate_async(ok_val).await.is_ok());
      assert!(rule.validate_async(err_val).await.is_err());
    }

    #[tokio::test]
    async fn test_async_naive_datetime_ref() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
      let rule = Rule::<NaiveDateTime>::Min(min);
      let val = NaiveDate::from_ymd_opt(2024, 6, 15)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
      assert!(rule.validate_ref_async(&val).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_naive_datetime_option_none_required() {
      let rule = Rule::<NaiveDateTime>::Required;
      assert!(rule.validate_async(None::<NaiveDateTime>).await.is_err());
      assert!(
        rule
          .validate_ref_async(&None::<NaiveDateTime>)
          .await
          .is_err()
      );
    }

    #[tokio::test]
    async fn test_async_naive_datetime_option_none_not_required() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
      let rule = Rule::<NaiveDateTime>::Min(min);
      assert!(rule.validate_async(None::<NaiveDateTime>).await.is_ok());
      assert!(
        rule
          .validate_ref_async(&None::<NaiveDateTime>)
          .await
          .is_ok()
      );
    }

    #[tokio::test]
    async fn test_async_naive_datetime_option_some_valid() {
      let min = NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
      let rule = Rule::<NaiveDateTime>::Min(min);
      let val = NaiveDate::from_ymd_opt(2024, 6, 15)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
      assert!(rule.validate_async(Some(val)).await.is_ok());
      assert!(rule.validate_ref_async(&Some(val)).await.is_ok());
    }
  }
}
