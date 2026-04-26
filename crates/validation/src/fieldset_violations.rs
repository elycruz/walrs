use std::fmt;

use indexmap::IndexMap;

use crate::{Violation, Violations};

/// A key-value map of field names to their validation violations.
///
/// Used consistently across `walrs_validation` and `walrs_fieldfilter` to
/// represent multi-field validation errors.
///
/// - Keys are field names (e.g., `"email"`, `"address.street"`).
/// - Values are `Violations` (a vec of `Violation` instances).
/// - Cross-field / form-level violations use the key `""` (empty string).
///
/// # Example
///
/// ```rust
/// use walrs_validation::{FieldsetViolations, Violation, ViolationType};
///
/// let mut fv = FieldsetViolations::new();
/// fv.add("email", Violation::invalid_email());
/// fv.add("", Violation::new(ViolationType::NotEqual, "Passwords must match"));
///
/// assert!(!fv.is_empty());
/// assert!(fv.get("email").is_some());
/// assert!(fv.form_violations().is_some());
/// ```
#[derive(Clone, Debug, Default)]
pub struct FieldsetViolations(pub IndexMap<String, Violations>);

impl FieldsetViolations {
  /// Creates a new, empty `FieldsetViolations`.
  pub fn new() -> Self {
    Self::default()
  }

  /// Returns `true` if all fields have no violations.
  ///
  /// ```rust
  /// use walrs_validation::{FieldsetViolations, Violation};
  ///
  /// let fv = FieldsetViolations::new();
  /// assert!(fv.is_empty());
  ///
  /// let mut fv2 = FieldsetViolations::new();
  /// fv2.add("email", Violation::invalid_email());
  /// assert!(!fv2.is_empty());
  /// ```
  pub fn is_empty(&self) -> bool {
    self.0.values().all(|v| v.is_empty())
  }

  /// Returns the total number of violations across all fields.
  ///
  /// ```rust
  /// use walrs_validation::{FieldsetViolations, Violation};
  ///
  /// let mut fv = FieldsetViolations::new();
  /// fv.add("email", Violation::invalid_email());
  /// fv.add("email", Violation::value_missing());
  /// fv.add("name", Violation::value_missing());
  ///
  /// assert_eq!(fv.len(), 3);
  /// ```
  pub fn len(&self) -> usize {
    self.0.values().map(|v| v.len()).sum()
  }

  /// Returns a reference to the violations for the given field, if any.
  pub fn get(&self, field: &str) -> Option<&Violations> {
    self.0.get(field)
  }

  /// Returns a mutable reference to the violations for the given field, if any.
  pub fn get_mut(&mut self, field: &str) -> Option<&mut Violations> {
    self.0.get_mut(field)
  }

  /// Adds a single violation under the given field name.
  ///
  /// ```rust
  /// use walrs_validation::{FieldsetViolations, Violation};
  ///
  /// let mut fv = FieldsetViolations::new();
  /// fv.add("email", Violation::invalid_email());
  ///
  /// assert_eq!(fv.get("email").unwrap().len(), 1);
  /// ```
  pub fn add(&mut self, field: impl Into<String>, violation: Violation) -> &mut Self {
    self
      .0
      .entry(field.into())
      .or_insert_with(Violations::empty)
      .push(violation);
    self
  }

  /// Extends the violations under the given field name with multiple violations.
  pub fn add_many(&mut self, field: impl Into<String>, violations: Violations) -> &mut Self {
    self
      .0
      .entry(field.into())
      .or_insert_with(Violations::empty)
      .extend(violations);
    self
  }

  /// Returns form-level violations (those stored under the empty-string key).
  pub fn form_violations(&self) -> Option<&Violations> {
    self.get("")
  }

  /// Adds a form-level violation (stored under the empty-string key).
  pub fn add_form_violation(&mut self, violation: Violation) -> &mut Self {
    self.add("", violation)
  }

  /// Returns an iterator over the field names.
  pub fn field_names(&self) -> impl Iterator<Item = &String> {
    self.0.keys()
  }

  /// Returns an iterator over (field name, violations) pairs.
  pub fn iter(&self) -> impl Iterator<Item = (&String, &Violations)> {
    self.0.iter()
  }

  /// Merges all entries from `other` into `self`.
  ///
  /// For each key in `other`, violations are appended to the existing
  /// entry in `self` (or a new entry is created).
  pub fn merge(&mut self, other: FieldsetViolations) -> &mut Self {
    for (field, violations) in other.0 {
      self
        .0
        .entry(field)
        .or_insert_with(Violations::empty)
        .extend(violations);
    }
    self
  }

  /// Merges entries from `other` with dot-prefixed keys.
  ///
  /// Each key in `other` is prefixed with `prefix.` (e.g., prefix `"address"`
  /// and key `"street"` becomes `"address.street"`). An empty key in `other`
  /// maps to `prefix` itself.
  pub fn merge_prefixed(&mut self, prefix: &str, other: FieldsetViolations) -> &mut Self {
    for (field, violations) in other.0 {
      let prefixed_key = if field.is_empty() {
        prefix.to_string()
      } else {
        format!("{}.{}", prefix, field)
      };
      self
        .0
        .entry(prefixed_key)
        .or_insert_with(Violations::empty)
        .extend(violations);
    }
    self
  }

  /// Removes all entries from this container.
  pub fn clear(&mut self) -> &mut Self {
    self.0.clear();
    self
  }
}

impl fmt::Display for FieldsetViolations {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut first = true;
    for (field, violations) in &self.0 {
      if violations.is_empty() {
        continue;
      }
      if !first {
        write!(f, "; ")?;
      }
      first = false;
      let label = if field.is_empty() { "(form)" } else { field };
      write!(f, "{}: {}", label, violations)?;
    }
    Ok(())
  }
}

impl std::error::Error for FieldsetViolations {}

impl From<FieldsetViolations> for Result<(), FieldsetViolations> {
  fn from(fv: FieldsetViolations) -> Self {
    if fv.is_empty() { Ok(()) } else { Err(fv) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ViolationType;

  #[test]
  fn test_new_is_empty() {
    let fv = FieldsetViolations::new();
    assert!(fv.is_empty());
    assert_eq!(fv.len(), 0);
  }

  #[test]
  fn test_add_single_violation() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());

    assert!(!fv.is_empty());
    assert_eq!(fv.len(), 1);
    assert!(fv.get("email").is_some());
    assert_eq!(fv.get("email").unwrap().len(), 1);
  }

  #[test]
  fn test_add_many_violations() {
    let mut fv = FieldsetViolations::new();
    let violations = Violations::new(vec![Violation::invalid_email(), Violation::value_missing()]);
    fv.add_many("email", violations);

    assert_eq!(fv.len(), 2);
    assert_eq!(fv.get("email").unwrap().len(), 2);
  }

  #[test]
  fn test_form_violations() {
    let mut fv = FieldsetViolations::new();
    fv.add(
      "",
      Violation::new(ViolationType::NotEqual, "Passwords must match"),
    );

    assert!(fv.form_violations().is_some());
    assert_eq!(fv.form_violations().unwrap().len(), 1);
  }

  #[test]
  fn test_add_form_violation() {
    let mut fv = FieldsetViolations::new();
    fv.add_form_violation(Violation::new(
      ViolationType::CustomError,
      "Form-level error",
    ));

    assert!(fv.form_violations().is_some());
    assert_eq!(fv.form_violations().unwrap().len(), 1);
  }

  #[test]
  fn test_merge() {
    let mut fv1 = FieldsetViolations::new();
    fv1.add("email", Violation::invalid_email());

    let mut fv2 = FieldsetViolations::new();
    fv2.add("email", Violation::value_missing());
    fv2.add("name", Violation::value_missing());

    fv1.merge(fv2);

    assert_eq!(fv1.len(), 3);
    assert_eq!(fv1.get("email").unwrap().len(), 2);
    assert_eq!(fv1.get("name").unwrap().len(), 1);
  }

  #[test]
  fn test_merge_prefixed() {
    let mut fv1 = FieldsetViolations::new();

    let mut fv2 = FieldsetViolations::new();
    fv2.add("street", Violation::value_missing());
    fv2.add("city", Violation::value_missing());

    fv1.merge_prefixed("address", fv2);

    assert_eq!(fv1.len(), 2);
    assert!(fv1.get("address.street").is_some());
    assert!(fv1.get("address.city").is_some());
  }

  #[test]
  fn test_merge_prefixed_empty_key() {
    let mut fv1 = FieldsetViolations::new();

    let mut fv2 = FieldsetViolations::new();
    fv2.add(
      "",
      Violation::new(ViolationType::CustomError, "Sub-form error"),
    );

    fv1.merge_prefixed("address", fv2);

    assert_eq!(fv1.len(), 1);
    assert!(fv1.get("address").is_some());
  }

  #[test]
  fn test_clear() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());
    fv.add("name", Violation::value_missing());

    assert!(!fv.is_empty());
    fv.clear();
    assert!(fv.is_empty());
    assert_eq!(fv.len(), 0);
  }

  #[test]
  fn test_len() {
    let mut fv = FieldsetViolations::new();
    assert_eq!(fv.len(), 0);

    fv.add("email", Violation::invalid_email());
    assert_eq!(fv.len(), 1);

    fv.add("email", Violation::value_missing());
    assert_eq!(fv.len(), 2);

    fv.add("name", Violation::value_missing());
    assert_eq!(fv.len(), 3);
  }

  #[test]
  fn test_fields_iterator() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());
    fv.add("name", Violation::value_missing());

    let fields: Vec<&String> = fv.field_names().collect();
    assert_eq!(fields.len(), 2);
    assert!(fields.contains(&&"email".to_string()));
    assert!(fields.contains(&&"name".to_string()));
  }

  #[test]
  fn test_iter() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());
    fv.add("name", Violation::value_missing());

    let items: Vec<_> = fv.iter().collect();
    assert_eq!(items.len(), 2);

    let (field, violations) = items[0];
    assert_eq!(field, "email");
    assert_eq!(violations.len(), 1);
  }

  #[test]
  fn test_display() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());
    fv.add(
      "",
      Violation::new(ViolationType::NotEqual, "Passwords must match"),
    );

    let display = format!("{}", fv);
    assert!(display.contains("email: Invalid email address."));
    assert!(display.contains("(form): Passwords must match"));
  }

  #[test]
  fn test_error_impl() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());

    let err: &dyn std::error::Error = &fv;
    assert!(err.source().is_none());
    assert!(!err.to_string().is_empty());
  }

  #[test]
  fn test_into_result_ok() {
    let fv = FieldsetViolations::new();
    let result: Result<(), FieldsetViolations> = fv.into();
    assert!(result.is_ok());
  }

  #[test]
  fn test_into_result_err() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());
    let result: Result<(), FieldsetViolations> = fv.into();
    assert!(result.is_err());
  }

  #[test]
  fn test_get_mut() {
    let mut fv = FieldsetViolations::new();
    fv.add("email", Violation::invalid_email());

    let violations = fv.get_mut("email").unwrap();
    violations.push(Violation::value_missing());

    assert_eq!(fv.get("email").unwrap().len(), 2);
  }
}
