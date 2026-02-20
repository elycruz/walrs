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
          Err(Violation::range_underflow(min).into())
        } else {
          Ok(())
        }
      }
      Rule::Max(max) => {
        if value > *max {
          Err(Violation::range_overflow(max).into())
        } else {
          Ok(())
        }
      }
      Rule::Range { min, max } => {
        if value < *min || value > *max {
          Err(
            Violation(
              ViolationType::ValueMissing,
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
          Err(Violation::value_missing().into())
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
