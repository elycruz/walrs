//! Form-level violations for multi-field validation.
//!
//! This module provides the `FormViolations` struct for collecting validation
//! errors from multiple fields and cross-field validation rules.

use std::collections::HashMap;
use walrs_validator::Violations;

/// Collection of validation violations for a form.
///
/// `FormViolations` separates per-field violations from cross-field (form-level)
/// violations, making it easy to display errors next to their respective fields
/// while also showing form-wide validation failures.
///
/// # Example
///
/// ```rust
/// use walrs_inputfilter::form_violations::FormViolations;
/// use walrs_validator::{Violation, ViolationType, Violations};
///
/// let mut form_violations = FormViolations::new();
///
/// // Add a field-level violation
/// let mut email_violations = Violations::empty();
/// email_violations.push(Violation::new(ViolationType::TypeMismatch, "Invalid email"));
/// form_violations.add_field_violations("email", email_violations);
///
/// // Add a form-level violation
/// form_violations.add_form_violation(Violation::new(
///     ViolationType::CustomError,
///     "Passwords do not match"
/// ));
///
/// assert!(!form_violations.is_empty());
/// assert!(form_violations.for_field("email").is_some());
/// ```
#[derive(Clone, Debug, Default)]
pub struct FormViolations {
    /// Per-field violations, keyed by field name.
    pub fields: HashMap<String, Violations>,

    /// Cross-field (form-level) violations.
    pub form: Violations,
}

impl FormViolations {
    /// Creates a new empty `FormViolations`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if there are no violations.
    pub fn is_empty(&self) -> bool {
        self.fields.values().all(|v| v.is_empty()) && self.form.is_empty()
    }

    /// Returns the total number of violations across all fields and form-level.
    pub fn len(&self) -> usize {
        self.fields.values().map(|v| v.len()).sum::<usize>() + self.form.len()
    }

    /// Gets violations for a specific field.
    pub fn for_field(&self, name: &str) -> Option<&Violations> {
        self.fields.get(name)
    }

    /// Gets mutable violations for a specific field.
    pub fn for_field_mut(&mut self, name: &str) -> Option<&mut Violations> {
        self.fields.get_mut(name)
    }

    /// Adds violations for a field.
    pub fn add_field_violations<S: Into<String>>(&mut self, field_name: S, violations: Violations) {
        let name = field_name.into();
        self.fields
            .entry(name)
            .or_insert_with(Violations::empty)
            .extend(violations);
    }

    /// Adds a single violation for a field.
    pub fn add_field_violation<S: Into<String>>(
        &mut self,
        field_name: S,
        violation: walrs_validator::Violation,
    ) {
        let name = field_name.into();
        self.fields
            .entry(name)
            .or_insert_with(Violations::empty)
            .push(violation);
    }

    /// Adds a form-level violation.
    pub fn add_form_violation(&mut self, violation: walrs_validator::Violation) {
        self.form.push(violation);
    }

    /// Adds multiple form-level violations.
    pub fn add_form_violations(&mut self, violations: Violations) {
        self.form.extend(violations);
    }

    /// Returns an iterator over all field names with violations.
    pub fn field_names(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }

    /// Merges another `FormViolations` into this one.
    pub fn merge(&mut self, other: FormViolations) {
        for (field, violations) in other.fields {
            self.add_field_violations(field, violations);
        }
        self.form.extend(other.form);
    }

    /// Clears all violations.
    pub fn clear(&mut self) {
        self.fields.clear();
        self.form.clear();
    }
}

impl From<FormViolations> for Result<(), FormViolations> {
    fn from(violations: FormViolations) -> Self {
        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use walrs_validator::{Violation, ViolationType};

    #[test]
    fn test_new_is_empty() {
        let violations = FormViolations::new();
        assert!(violations.is_empty());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_add_field_violations() {
        let mut form_violations = FormViolations::new();
        let mut field_violations = Violations::empty();
        field_violations.push(Violation::new(ViolationType::ValueMissing, "Required"));

        form_violations.add_field_violations("email", field_violations);

        assert!(!form_violations.is_empty());
        assert_eq!(form_violations.len(), 1);
        assert!(form_violations.for_field("email").is_some());
    }

    #[test]
    fn test_add_field_violation() {
        let mut form_violations = FormViolations::new();
        form_violations.add_field_violation(
            "username",
            Violation::new(ViolationType::TooShort, "Too short"),
        );

        assert_eq!(form_violations.len(), 1);
        assert!(form_violations.for_field("username").is_some());
    }

    #[test]
    fn test_add_form_violation() {
        let mut form_violations = FormViolations::new();
        form_violations.add_form_violation(Violation::new(
            ViolationType::CustomError,
            "Passwords must match",
        ));

        assert!(!form_violations.is_empty());
        assert_eq!(form_violations.len(), 1);
        assert_eq!(form_violations.form.len(), 1);
    }

    #[test]
    fn test_merge() {
        let mut violations1 = FormViolations::new();
        violations1.add_field_violation("email", Violation::new(ViolationType::TypeMismatch, "Invalid"));

        let mut violations2 = FormViolations::new();
        violations2.add_field_violation("username", Violation::new(ViolationType::TooShort, "Too short"));
        violations2.add_form_violation(Violation::new(ViolationType::CustomError, "Form error"));

        violations1.merge(violations2);

        assert_eq!(violations1.len(), 3);
        assert!(violations1.for_field("email").is_some());
        assert!(violations1.for_field("username").is_some());
        assert_eq!(violations1.form.len(), 1);
    }

    #[test]
    fn test_field_names() {
        let mut violations = FormViolations::new();
        violations.add_field_violation("email", Violation::new(ViolationType::TypeMismatch, "Invalid"));
        violations.add_field_violation("username", Violation::new(ViolationType::TooShort, "Too short"));

        let names: Vec<_> = violations.field_names().collect();
        assert_eq!(names.len(), 2);
    }
}

