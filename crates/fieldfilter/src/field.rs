//! Field validation configuration.
//!
//! This module provides the `Field<T>` struct for defining validation and filtering
//! rules for a single form field. It replaces the old `Input`/`RefInput` API with
//! a unified, serializable design.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use walrs_filter::{FilterOp, TryFilterOp};
#[allow(deprecated)]
use walrs_validation::{Rule, ValidateRef, Violation, Violations};

#[cfg(feature = "async")]
use walrs_validation::ValidateRefAsync;

/// Validation configuration for a single field.
///
/// `Field<T>` provides a unified API for field validation and filtering,
/// replacing the old `Input`/`RefInput` split. It supports:
///
/// - Rule-based validation using the `Rule<T>` enum
/// - Filter-based transformation using the `FilterOp<T>` enum
/// - Fallible filter-based transformation using the `TryFilterOp<T>` enum
/// - Builder pattern via `FieldBuilder`
/// - JSON/YAML serialization for config-driven forms
///
/// # Example
///
/// ```rust
/// use walrs_fieldfilter::field::{Field, FieldBuilder};
/// use walrs_filter::{FilterOp, TryFilterOp, FilterError};
/// use walrs_validation::Rule;
/// use std::sync::Arc;
///
/// // Simple field with just a rule (no filters)
/// let field = FieldBuilder::<String>::default()
///     .name("username")
///     .rule(Rule::Required)
///     .build()
///     .unwrap();
///
/// // Field with infallible and fallible filters
/// let field = FieldBuilder::<String>::default()
///     .name("encoded_data")
///     .filters(vec![FilterOp::Trim])
///     .try_filters(vec![
///         TryFilterOp::TryCustom(Arc::new(|s: String| {
///             if s.contains('\0') {
///                 Err(FilterError::new("input contains null bytes"))
///             } else {
///                 Ok(s)
///             }
///         })),
///     ])
///     .build()
///     .unwrap();
///
/// // sanitize() applies infallible filters, then fallible filters, then validates
/// assert!(field.sanitize("  hello  ".to_string()).is_ok());
/// assert!(field.sanitize("  \0bad  ".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option), default)]
pub struct Field<T>
where
  T: Clone,
{
  /// Optional field name for error reporting.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub name: Option<Cow<'static, str>>,

  /// Optional locale for localized error messages.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub locale: Option<Cow<'static, str>>,

  /// Validation rule to apply. Use `Rule::All` for multiple rules.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub rule: Option<Rule<T>>,

  /// Filters to apply before validation. Use `FilterOp::Chain` for multiple filters.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub filters: Option<Vec<FilterOp<T>>>,

  /// Fallible filters to apply after infallible filters, before validation.
  ///
  /// If any fallible filter fails, the error is converted to a `Violation`
  /// and returned as part of the validation error pipeline.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default = "None")]
  pub try_filters: Option<Vec<TryFilterOp<T>>>,

  /// When true, stops validation at the first error.
  #[builder(default = "false")]
  pub break_on_failure: bool,
}

impl<T: Clone> Default for Field<T> {
  fn default() -> Self {
    Self {
      name: None,
      locale: None,
      rule: None,
      filters: None,
      try_filters: None,
      break_on_failure: false,
    }
  }
}

impl<T: Clone + PartialEq> PartialEq for Field<T>
where
  Rule<T>: PartialEq,
  FilterOp<T>: PartialEq,
  TryFilterOp<T>: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
      && self.locale == other.locale
      && self.rule == other.rule
      && self.filters == other.filters
      && self.try_filters == other.try_filters
      && self.break_on_failure == other.break_on_failure
  }
}

impl<T: Clone> Field<T> {
  /// Bakes the stored locale into the rule so that subsequent
  /// `validate_ref` calls avoid cloning the rule.
  ///
  /// After this call `locale` is `None` and the rule carries the locale
  /// internally via [`Rule::WithMessage`].
  ///
  /// This is an opt-in performance optimisation — calling code that never
  /// sets a locale is unaffected.
  pub fn apply_locale(&mut self) {
    if let Some(locale) = &self.locale
      && let Some(rule) = self.rule.take()
    {
      self.rule = Some(rule.with_locale(locale.as_ref()));
      self.locale = None;
    }
  }
}

// ============================================================================
// FieldOps trait — private abstraction for Field<String>
// ============================================================================

/// Private trait for `Field<String>` method bodies.
trait FieldOps: Clone + Sized {
  /// The reference type for validation (`String` → `str`).
  type ValueRef: ?Sized;

  /// Apply infallible filters starting from a reference, returning owned.
  fn apply_filters_from_ref(filters: &[FilterOp<Self>], value: &Self::ValueRef) -> Self;

  /// Apply infallible filters to an owned value.
  fn apply_filters(filters: &[FilterOp<Self>], value: Self) -> Self;

  /// Clone/convert from reference to owned.
  fn ref_to_owned(value: &Self::ValueRef) -> Self;

  /// Borrow self as the validation reference type.
  fn as_value_ref(&self) -> &Self::ValueRef;

  /// Apply a single fallible filter to an owned value.
  fn try_apply_filter(
    filter: &TryFilterOp<Self>,
    value: Self,
  ) -> Result<Self, walrs_filter::FilterError>;
}

impl FieldOps for String {
  type ValueRef = str;

  fn apply_filters_from_ref(filters: &[FilterOp<Self>], value: &str) -> String {
    let mut result = value.to_string();
    for f in filters {
      match f.apply_ref(&result) {
        Cow::Borrowed(_) => {} // No change, keep result as-is
        Cow::Owned(s) => result = s,
      }
    }
    result
  }

  fn apply_filters(filters: &[FilterOp<Self>], value: String) -> String {
    let mut result = value;
    for f in filters {
      match f.apply_ref(&result) {
        Cow::Borrowed(_) => {} // No change, keep result as-is
        Cow::Owned(s) => result = s,
      }
    }
    result
  }

  fn ref_to_owned(value: &str) -> String {
    value.to_string()
  }

  fn as_value_ref(&self) -> &str {
    self.as_str()
  }

  fn try_apply_filter(
    filter: &TryFilterOp<Self>,
    value: Self,
  ) -> Result<Self, walrs_filter::FilterError> {
    filter.try_apply(value)
  }
}

// ============================================================================
// Private generic helper functions
// ============================================================================

fn filter_ref_impl<T: FieldOps>(field: &Field<T>, value: &T::ValueRef) -> T {
  match &field.filters {
    Some(filters) => T::apply_filters_from_ref(filters, value),
    None => T::ref_to_owned(value),
  }
}

fn filter_impl<T: FieldOps>(field: &Field<T>, value: T) -> T {
  match &field.filters {
    Some(filters) => T::apply_filters(filters, value),
    None => value,
  }
}

fn try_filter_ref_impl<T: FieldOps>(
  field: &Field<T>,
  value: &T::ValueRef,
) -> Result<T, Violations> {
  match &field.try_filters {
    Some(try_filters) => {
      let mut result = T::ref_to_owned(value);
      for f in try_filters {
        result = T::try_apply_filter(f, result).map_err(|e| -> Violations {
          let violation: Violation = e.into();
          Violations::new(vec![violation])
        })?;
      }
      Ok(result)
    }
    None => Ok(T::ref_to_owned(value)),
  }
}

fn try_filter_impl<T: FieldOps>(field: &Field<T>, value: T) -> Result<T, Violations> {
  match &field.try_filters {
    Some(try_filters) => {
      let mut result = value;
      for f in try_filters {
        result = T::try_apply_filter(f, result).map_err(|e| -> Violations {
          let violation: Violation = e.into();
          Violations::new(vec![violation])
        })?;
      }
      Ok(result)
    }
    None => Ok(value),
  }
}

fn validate_ref_impl<T: FieldOps>(field: &Field<T>, value: &T::ValueRef) -> Result<(), Violations>
where
  Rule<T>: ValidateRef<T::ValueRef>,
{
  match &field.rule {
    Some(rule) => {
      // Apply locale to rule if set, then validate via trait method
      // @todo `locale` should be set directly on `rule`.
      let result = if let Some(locale) = &field.locale {
        rule
          .clone()
          .with_locale(locale.as_ref())
          .validate_ref(value)
      } else {
        rule.validate_ref(value)
      };
      result.map_err(|v| {
        let mut violations = Violations::empty();
        violations.push(v);
        violations
      })
    }
    None => Ok(()),
  }
}

fn sanitize_impl<T: FieldOps>(field: &Field<T>, value: T) -> Result<T, Violations>
where
  Rule<T>: ValidateRef<T::ValueRef>,
{
  let filtered = filter_impl(field, value);
  let filtered = try_filter_impl(field, filtered)?;
  validate_ref_impl(field, filtered.as_value_ref())?;
  Ok(filtered)
}

#[cfg(feature = "async")]
async fn validate_ref_async_impl<T: FieldOps>(
  field: &Field<T>,
  value: &T::ValueRef,
) -> Result<(), Violations>
where
  T::ValueRef: Sync,
  Rule<T>: ValidateRefAsync<T::ValueRef>,
{
  match &field.rule {
    Some(rule) => {
      let result = if let Some(locale) = &field.locale {
        rule
          .clone()
          .with_locale(locale.as_ref())
          .validate_ref_async(value)
          .await
      } else {
        rule.validate_ref_async(value).await
      };
      result.map_err(|v| {
        let mut violations = Violations::empty();
        violations.push(v);
        violations
      })
    }
    None => Ok(()),
  }
}

#[cfg(feature = "async")]
async fn sanitize_async_impl<T: FieldOps>(field: &Field<T>, value: T) -> Result<T, Violations>
where
  T::ValueRef: Sync,
  Rule<T>: ValidateRefAsync<T::ValueRef>,
{
  let filtered = filter_impl(field, value);
  let filtered = try_filter_impl(field, filtered)?;
  validate_ref_async_impl(field, filtered.as_value_ref()).await?;
  Ok(filtered)
}

// ============================================================================
// String Field Implementation
// ============================================================================

impl Field<String> {
  /// Apply all filters to a `&str` reference, returning an owned `String`.
  ///
  /// Prefer this method when you already have a `&str`, avoiding an
  /// allocation at the call site.
  pub fn filter_ref(&self, value: &str) -> String {
    filter_ref_impl(self, value)
  }

  /// Apply all filters to the value sequentially.
  pub fn filter(&self, value: String) -> String {
    filter_impl(self, value)
  }

  /// Apply all fallible filters to a `&str` reference.
  ///
  /// Returns `Ok(filtered_value)` if all filters succeed, or `Err(Violations)` with
  /// the filter error converted to a `Violation`.
  pub fn try_filter_ref(&self, value: &str) -> Result<String, Violations> {
    try_filter_ref_impl(self, value)
  }

  /// Apply all fallible filters to an owned `String`.
  ///
  /// Returns `Ok(filtered_value)` if all filters succeed, or `Err(Violations)` with
  /// the filter error converted to a `Violation`.
  pub fn try_filter(&self, value: String) -> Result<String, Violations> {
    try_filter_impl(self, value)
  }

  /// Validate the value against the rule, short-circuiting on the first violation.
  ///
  /// Accepts a `&str` directly, avoiding any allocation at the call site.
  ///
  /// Returns `Ok(())` if the rule passes, or `Err(Violations)` with the first failure.
  /// If the field has a locale set, it is applied to the rule for internationalized
  /// error messages.
  /// Whether the calling context stops processing further fields on failure is
  /// controlled by the `break_on_failure` flag (used by `Fieldset`).
  pub fn validate_ref(&self, value: &str) -> Result<(), Violations> {
    validate_ref_impl(self, value)
  }

  /// Validate a `&String` value. Delegates to [`validate_ref`](Self::validate_ref).
  pub fn validate(&self, value: String) -> Result<(), Violations> {
    self.validate_ref(value.as_str())
  }

  /// Filter the value and then validate it.
  ///
  /// Applies infallible filters first, then fallible filters, then validates.
  /// Returns `Ok(filtered_value)` if all steps pass, or `Err(Violations)`.
  pub fn sanitize(&self, value: String) -> Result<String, Violations> {
    sanitize_impl(self, value)
  }

  /// Filter a `&str` and then validate the result.
  ///
  /// Like [`sanitize`](Self::sanitize) but starts from a `&str` reference,
  /// avoiding the need for the caller to allocate a `String` up-front.
  pub fn sanitize_ref(&self, value: &str) -> Result<String, Violations> {
    let filtered = self.filter_ref(value);
    let filtered = self.try_filter(filtered)?;
    self.validate_ref(&filtered)?;
    Ok(filtered)
  }
}

// ============================================================================
// Async Field Implementations
// ============================================================================

#[cfg(feature = "async")]
impl Field<String> {
  /// Validate the value asynchronously against the rule.
  ///
  /// Works like [`validate_ref`](Self::validate_ref) but supports
  /// `Rule::CustomAsync` validators.
  pub async fn validate_ref_async(&self, value: &str) -> Result<(), Violations> {
    validate_ref_async_impl(self, value).await
  }

  /// Validate a `String` value asynchronously.
  pub async fn validate_async(&self, value: String) -> Result<(), Violations> {
    self.validate_ref_async(value.as_str()).await
  }

  /// Filter the value synchronously (infallible + fallible), then validate asynchronously.
  ///
  /// Returns `Ok(filtered_value)` if all steps pass, or `Err(Violations)`.
  pub async fn sanitize_async(&self, value: String) -> Result<String, Violations> {
    sanitize_async_impl(self, value).await
  }

  /// Like [`sanitize_async`](Self::sanitize_async) but starts from a `&str` reference.
  pub async fn sanitize_ref_async(&self, value: &str) -> Result<String, Violations> {
    let filtered = self.filter_ref(value);
    let filtered = self.try_filter(filtered)?;
    self.validate_ref_async(&filtered).await?;
    Ok(filtered)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::Arc;
  use walrs_filter::{FilterError, TryFilterOp};
  use walrs_validation::Rule;

  #[test]
  fn test_field_builder_defaults() {
    let field = FieldBuilder::<String>::default().build().unwrap();
    assert_eq!(field.name, None);
    assert!(field.rule.is_none());
    assert!(field.filters.is_none());
  }

  #[test]
  fn test_field_builder_with_values() {
    let field = FieldBuilder::<String>::default()
      .name("email")
      .rule(Rule::Required.and(Rule::MinLength(5)))
      .filters(vec![FilterOp::Trim])
      .build()
      .unwrap();

    assert_eq!(field.name.as_deref(), Some("email"));
    assert!(field.rule.is_some());
    assert_eq!(field.filters.as_ref().map(|f| f.len()), Some(1));
  }

  #[test]
  fn test_string_field_filter() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
      .build()
      .unwrap();

    let result = field.filter("  HELLO  ".to_string());
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_string_field_validate_ref() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::Required.and(Rule::MinLength(3)))
      .build()
      .unwrap();

    assert!(field.validate_ref("hello").is_ok());
    assert!(field.validate_ref("hi").is_err());
    assert!(field.validate_ref("").is_err());
  }

  #[test]
  fn test_string_field_validate_passes() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::MinLength(3))
      .build()
      .unwrap();

    assert!(field.validate_ref("hello").is_ok());
  }

  #[test]
  fn test_string_field_validate_fails() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::MinLength(10))
      .build()
      .unwrap();

    assert!(field.validate_ref("hello").is_err());
  }

  #[test]
  fn test_string_field_required() {
    let field = FieldBuilder::<String>::default()
      .rule(Rule::Required)
      .build()
      .unwrap();

    assert!(field.validate_ref("").is_err());
    assert!(field.validate_ref("   ").is_err());
    assert!(field.validate_ref("hello").is_ok());
  }

  #[test]
  fn test_string_field_sanitize() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![FilterOp::Trim])
      .rule(Rule::MinLength(3))
      .build()
      .unwrap();

    let result = field.sanitize("  hello  ".to_string());
    assert_eq!(result.unwrap(), "hello");
  }

  #[test]
  fn test_break_on_failure() {
    // `break_on_failure` signals the Fieldset to stop processing further
    // fields when this field fails; the `validate` method itself always
    // short-circuits on the first encountered violation.
    let field = FieldBuilder::<String>::default()
      .rule(
        Rule::Required
          .and(Rule::MinLength(5))
          .and(Rule::MaxLength(10)),
      )
      .break_on_failure(true)
      .build()
      .unwrap();

    // Empty string fails on the first encountered violation
    let result = field.validate_ref("");
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1); // Always returns the first violation only
  }

  #[test]
  fn test_field_serialization() {
    let field = FieldBuilder::<String>::default()
      .name("username")
      .rule(Rule::Required)
      .build()
      .unwrap();

    let json = serde_json::to_string(&field).unwrap();
    assert!(json.contains("username"));
    assert!(json.contains("required")); // lowercase due to serde rename_all
  }

  // ====================================================================
  // filter_ref tests
  // ====================================================================

  #[test]
  fn test_string_field_filter_ref() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
      .build()
      .unwrap();

    // filter_ref accepts &str directly — no .to_string() needed
    let result = field.filter_ref("  HELLO  ");
    assert_eq!(result, "hello");
  }

  #[test]
  fn test_string_field_filter_ref_no_filters() {
    let field = FieldBuilder::<String>::default().build().unwrap();
    let result = field.filter_ref("unchanged");
    assert_eq!(result, "unchanged");
  }

  // ====================================================================
  // try_filter / try_filter_ref tests — Field<String>
  // ====================================================================

  #[test]
  fn test_string_field_try_filter_success() {
    let field = FieldBuilder::<String>::default()
      .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|s: String| {
        Ok(s.to_uppercase())
      }))])
      .build()
      .unwrap();

    let result = field.try_filter("hello".to_string());
    assert_eq!(result.unwrap(), "HELLO");
  }

  #[test]
  fn test_string_field_try_filter_failure() {
    let field = FieldBuilder::<String>::default()
      .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|s: String| {
        if s.is_empty() {
          Err(FilterError::new("empty after trim"))
        } else {
          Ok(s)
        }
      }))])
      .build()
      .unwrap();

    assert!(field.try_filter("hello".to_string()).is_ok());
    assert!(field.try_filter("".to_string()).is_err());
  }

  #[test]
  fn test_string_field_try_filter_ref() {
    let field = FieldBuilder::<String>::default()
      .try_filters(vec![TryFilterOp::Infallible(FilterOp::Lowercase)])
      .build()
      .unwrap();

    let result = field.try_filter_ref("HELLO");
    assert_eq!(result.unwrap(), "hello");
  }

  #[test]
  fn test_string_field_try_filter_none() {
    let field = FieldBuilder::<String>::default().build().unwrap();
    let result = field.try_filter("hello".to_string());
    assert_eq!(result.unwrap(), "hello");
  }

  #[test]
  fn test_string_field_sanitize_with_try_filters() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![FilterOp::Trim])
      .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|s: String| {
        if s.is_empty() {
          Err(FilterError::new("value must not be empty after trimming"))
        } else {
          Ok(s)
        }
      }))])
      .rule(Rule::MinLength(3))
      .build()
      .unwrap();

    // Happy path: trim -> try_filter passes -> validation passes
    assert_eq!(field.sanitize("  hello  ".to_string()).unwrap(), "hello");

    // Try filter fails (empty after trim)
    let err = field.sanitize("     ".to_string()).unwrap_err();
    assert_eq!(err.len(), 1);
    assert!(err[0].message().contains("empty after trimming"));

    // Try filter passes but validation fails (too short)
    let err = field.sanitize("  hi  ".to_string()).unwrap_err();
    assert_eq!(err.len(), 1);
  }

  #[test]
  fn test_string_field_sanitize_try_filter_short_circuits() {
    let field = FieldBuilder::<String>::default()
      .try_filters(vec![
        TryFilterOp::TryCustom(Arc::new(|_| Err(FilterError::new("first fails")))),
        TryFilterOp::TryCustom(Arc::new(|_| {
          panic!("should not reach second filter");
        })),
      ])
      .build()
      .unwrap();

    let err = field.sanitize("hello".to_string()).unwrap_err();
    assert!(err[0].message().contains("first fails"));
  }

  // ====================================================================
  // Builder with try_filters
  // ====================================================================

  #[test]
  fn test_field_builder_defaults_include_try_filters() {
    let field = FieldBuilder::<String>::default().build().unwrap();
    assert!(field.try_filters.is_none());
  }

  #[test]
  fn test_field_builder_with_try_filters() {
    let field = FieldBuilder::<String>::default()
      .try_filters(vec![TryFilterOp::Infallible(FilterOp::Trim)])
      .build()
      .unwrap();

    assert!(field.try_filters.is_some());
    assert_eq!(field.try_filters.as_ref().map(|f| f.len()), Some(1));
  }

  // ====================================================================
  // sanitize_ref tests — Field<String>
  // ====================================================================

  #[test]
  fn test_string_field_sanitize_ref() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![FilterOp::Trim])
      .rule(Rule::MinLength(3))
      .build()
      .unwrap();

    // Happy path: starts from &str
    assert_eq!(field.sanitize_ref("  hello  ").unwrap(), "hello");

    // Validation fails
    assert!(field.sanitize_ref("  hi  ").is_err());
  }

  #[test]
  fn test_string_field_sanitize_ref_with_try_filters() {
    let field = FieldBuilder::<String>::default()
      .filters(vec![FilterOp::Trim])
      .try_filters(vec![TryFilterOp::TryCustom(Arc::new(|s: String| {
        if s.is_empty() {
          Err(FilterError::new("empty after trim"))
        } else {
          Ok(s)
        }
      }))])
      .rule(Rule::MinLength(3))
      .build()
      .unwrap();

    assert_eq!(field.sanitize_ref("  hello  ").unwrap(), "hello");
    assert!(field.sanitize_ref("     ").is_err()); // empty after trim
    assert!(field.sanitize_ref("  hi  ").is_err()); // too short
  }

  #[test]
  fn test_string_field_sanitize_ref_no_filters_no_rule() {
    let field = FieldBuilder::<String>::default().build().unwrap();
    assert_eq!(field.sanitize_ref("hello").unwrap(), "hello");
  }

  // ====================================================================
  // apply_locale tests
  // ====================================================================

  #[test]
  fn test_apply_locale_bakes_into_rule() {
    let mut field = FieldBuilder::<String>::default()
      .locale("es")
      .rule(Rule::Required)
      .build()
      .unwrap();

    assert!(field.locale.is_some());
    field.apply_locale();
    assert!(field.locale.is_none());
    // Rule still validates correctly
    assert!(field.validate_ref("").is_err());
    assert!(field.validate_ref("hello").is_ok());
  }

  #[test]
  fn test_apply_locale_no_op_without_locale() {
    let mut field = FieldBuilder::<String>::default()
      .rule(Rule::Required)
      .build()
      .unwrap();

    let rule_before = field.rule.clone();
    field.apply_locale();
    assert_eq!(field.rule, rule_before);
  }

  #[test]
  fn test_apply_locale_no_op_without_rule() {
    let mut field = FieldBuilder::<String>::default()
      .locale("es")
      .build()
      .unwrap();

    field.apply_locale();
    // locale preserved when rule is None
    assert!(field.locale.is_some());
    assert!(field.rule.is_none());
  }
}
