//! Common types and utilities shared between `Input` and `RefInput` modules.

use crate::{Violation, ViolationType::ValueMissing, Violations};

/// Runs validation logic against a custom validator and a list of validators.
///
/// Returns `Ok(())` if all validations pass, or `Err(Violations)` with all collected violations.
///
/// # Arguments
/// * `custom_result` - Result from the custom validator (if any)
/// * `validators_results` - Iterator of results from the validators list
/// * `break_on_failure` - Whether to stop at the first failure
#[inline]
pub fn collect_violations<I>(
  custom_result: Option<Result<(), Violation>>,
  validators_results: I,
  break_on_failure: bool,
) -> Result<(), Violations>
where
  I: Iterator<Item = Result<(), Violation>>,
{
  let mut violations = Vec::new();

  // Handle custom validator result
  if let Some(Err(violation)) = custom_result {
    violations.push(violation);
    if break_on_failure {
      return Err(Violations(violations));
    }
  }

  // Handle validators list results
  for result in validators_results {
    if let Err(violation) = result {
      violations.push(violation);
      if break_on_failure {
        break;
      }
    }
  }

  if violations.is_empty() {
    Ok(())
  } else {
    Err(Violations(violations))
  }
}

/// Handles the "value is missing" case for optional validation methods.
///
/// Returns `Ok(())` if not required, or `Err(Violations)` with ValueMissing violation if required.
#[inline]
pub fn handle_missing_value<F>(required: bool, value_missing_msg: F) -> Result<(), Violations>
where
  F: FnOnce() -> String,
{
  if required {
    Err(Violations(vec![Violation(
      ValueMissing,
      value_missing_msg(),
    )]))
  } else {
    Ok(())
  }
}

/// Handles the "value is missing" case for optional filter methods.
///
/// Returns `Ok(default_value)` if not required, or `Err(Violations)` with ValueMissing violation if required.
#[inline]
pub fn handle_missing_value_for_filter<FT, F, D>(
  required: bool,
  value_missing_msg: F,
  get_default: Option<D>,
) -> Result<Option<FT>, Violations>
where
  F: FnOnce() -> String,
  D: FnOnce() -> Option<FT>,
{
  if required {
    Err(Violations(vec![Violation(
      ValueMissing,
      value_missing_msg(),
    )]))
  } else {
    Ok(get_default.and_then(|f| f()))
  }
}

/// Helper macro for generating Debug format strings for closure fields.
#[macro_export]
macro_rules! debug_closure_field {
  ($value:expr, $some_str:literal) => {
    if $value.is_some() { $some_str } else { "None" }
  };
}

/// Helper macro for generating Debug format strings for Vec fields containing closures.
#[macro_export]
macro_rules! debug_vec_closure_field {
  ($value:expr, $type_name:literal) => {
    if let Some(vs) = $value.as_deref() {
      format!("Some(Vec<{}>{{ len: {} }})", $type_name, vs.len())
    } else {
      "None".to_string()
    }
  };
}
