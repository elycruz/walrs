//! Date validation helpers using the `chrono` crate.

use chrono::NaiveDate;
use chrono::NaiveDateTime;

use crate::options::{DateFormat, DateOptions, DateRangeOptions};
use crate::rule::{Rule, RuleResult};
use crate::traits::{Validate, ValidateRef};
use crate::Violation;

// ============================================================================
// String Parsing Helpers
// ============================================================================

/// Format string for US-style date: `MM/DD/YYYY`
const US_DATE_FMT: &str = "%m/%d/%Y";
/// Format string for EU-style date: `DD/MM/YYYY`
const EU_DATE_FMT: &str = "%d/%m/%Y";

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
pub(crate) fn parse_datetime_str(
  value: &str,
  format: &DateFormat,
) -> Result<NaiveDateTime, ()> {
  match format {
    DateFormat::Iso8601 => {
      // Try with 'T' separator first, then space
      NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S"))
        .map_err(|_| ())
    }
    DateFormat::UsDate => {
      NaiveDateTime::parse_from_str(value, &format!("{} %H:%M:%S", US_DATE_FMT))
        .map_err(|_| ())
    }
    DateFormat::EuDate => {
      NaiveDateTime::parse_from_str(value, &format!("{} %H:%M:%S", EU_DATE_FMT))
        .map_err(|_| ())
    }
    DateFormat::Rfc2822 => chrono::DateTime::parse_from_rfc2822(value)
      .map(|dt| dt.naive_local())
      .map_err(|_| ()),
    DateFormat::Custom(fmt) => {
      NaiveDateTime::parse_from_str(value, fmt).map_err(|_| ())
    }
  }
}

/// Parses a bound string as a `NaiveDate` (always ISO 8601).
fn parse_bound_date(s: &str) -> Result<NaiveDate, ()> {
  NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| ())
}

/// Parses a bound string as a `NaiveDateTime` (always ISO 8601).
fn parse_bound_datetime(s: &str) -> Result<NaiveDateTime, ()> {
  NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
    .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
    .map_err(|_| ())
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
        if let Ok(min_dt) = parse_bound_datetime(min_str) {
          if dt < min_dt {
            return Err(Violation::date_range_underflow(min_str));
          }
        } else if let Ok(min_d) = parse_bound_date(min_str) {
          if dt.date() < min_d {
            return Err(Violation::date_range_underflow(min_str));
          }
        }
      }
      if let Some(max_str) = &opts.max {
        // Try datetime bound first; fall back to date-only bound (compare date component)
        if let Ok(max_dt) = parse_bound_datetime(max_str) {
          if dt > max_dt {
            return Err(Violation::date_range_overflow(max_str));
          }
        } else if let Ok(max_d) = parse_bound_date(max_str) {
          if dt.date() > max_d {
            return Err(Violation::date_range_overflow(max_str));
          }
        }
      }
      return Ok(());
    }
    // Fall back to date-only
    if let Ok(d) = parse_date_str(value, &opts.format) {
      return check_date_bounds(d, &opts.min, &opts.max);
    }
  } else {
    if let Ok(d) = parse_date_str(value, &opts.format) {
      return check_date_bounds(d, &opts.min, &opts.max);
    }
  }
  Err(Violation::invalid_date())
}

fn check_date_bounds(
  d: NaiveDate,
  min: &Option<String>,
  max: &Option<String>,
) -> RuleResult {
  if let Some(min_str) = min {
    // Try date-only bound first; fall back to datetime bound (extract date component)
    let min_d = parse_bound_date(min_str)
      .or_else(|_| parse_bound_datetime(min_str).map(|dt| dt.date()));
    if let Ok(min_d) = min_d {
      if d < min_d {
        return Err(Violation::date_range_underflow(min_str));
      }
    }
  }
  if let Some(max_str) = max {
    // Try date-only bound first; fall back to datetime bound (extract date component)
    let max_d = parse_bound_date(max_str)
      .or_else(|_| parse_bound_datetime(max_str).map(|dt| dt.date()));
    if let Ok(max_d) = max_d {
      if d > max_d {
        return Err(Violation::date_range_overflow(max_str));
      }
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
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage { rule, message, locale } => {
        let effective_locale = locale.as_deref().or(inherited_locale);
        match rule.validate_date_inner(value, effective_locale) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg = message.resolve_or(value, violation.message(), effective_locale);
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
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
      Rule::Ref(name) => Err(Violation::unresolved_ref(name)),
      Rule::WithMessage { rule, message, locale } => {
        let effective_locale = locale.as_deref().or(inherited_locale);
        match rule.validate_datetime_inner(value, effective_locale) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg = message.resolve_or(value, violation.message(), effective_locale);
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
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
      validate_date_range_str("2019-12-31", &opts).unwrap_err().violation_type(),
      ViolationType::RangeUnderflow,
    );
    assert_eq!(
      validate_date_range_str("2031-01-01", &opts).unwrap_err().violation_type(),
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
      validate_date_range_str("2019-12-31T23:59:59", &opts).unwrap_err().violation_type(),
      ViolationType::RangeUnderflow,
    );
    // datetime after max date
    assert_eq!(
      validate_date_range_str("2031-01-01T00:00:00", &opts).unwrap_err().violation_type(),
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
      validate_date_range_str("2019-12-31", &opts).unwrap_err().violation_type(),
      ViolationType::RangeUnderflow,
    );
    assert_eq!(
      validate_date_range_str("2031-01-01", &opts).unwrap_err().violation_type(),
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
}
