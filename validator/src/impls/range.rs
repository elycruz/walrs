use crate::ViolationType::ValueMissing;
use crate::rule::{range_overflow_violation, range_underflow_violation, value_missing_violation};
use crate::{
  Rule, ScalarValue, ValidationResult, Violation,
  ViolationType,
};

/// Validation rules for primitive scalar types.
///
/// Note: This implementation uses the upcoming validation result scheme which
/// returns `ValidationResult` for all validation methods (methods that return
/// singular `Err(Violation)` will be changed to always return `Err(Violations)`
/// soon).
///
impl<'a,  T: ScalarValue + Sized> Rule<T> {
  pub fn validate_copy(&self, value: T) -> ValidationResult {
    match self {
      Rule::Min(min) => {
        if value < *min {
          Err(range_underflow_violation(min).into())
        } else {
          Ok(())
        }
      }
      Rule::Max(max) => {
        if value > *max {
          Err(range_overflow_violation(max).into())
        } else {
          Ok(())
        }
      }
      Rule::Range { min, max } => {
        if value < *min || value > *max {
          Err(
            Violation(
              ValueMissing,
              format!("Value {} is not in range {}..={}", value, min, max),
            )
            .into(),
          )
        } else {
          Ok(())
        }
      }
      Rule::Equals( expected ) => {
        if value != *expected {
          Err(
            Violation(
              ViolationType::NotEqual,
              format!("Value {} does not equal expected {}", value, expected),
            )
            .into(),
          )
        } else {
          Ok(())
        }
      }
      // @todo Support `WithMessage` for copy types as well -
      //    The `locale` field on `WithMessage` is already available for use here.
      _ => Ok(()),
    }
  }

  pub fn validate_copy_option(&self, value: Option<T>) -> ValidationResult {
    match value {
      Some(v) => self.validate_copy(v),
      None => {
        if self.requires_value() {
          Err(value_missing_violation().into())
        } else {
          Ok(())
        }
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::Rule;

  #[test]
  fn test_validate_copy() {
    let rule = Rule::<usize>::Min(1);
    assert!(
      rule.validate_copy(0).is_err(),
      "Should fail validation for value 0"
    );

    let result = rule.validate_copy(1usize);
    if let Err(e) = &result {
      println!("Validation error: {}", e);
    }
    assert!(result.is_ok(), "Should validate successfully for value 1");
  }
}
