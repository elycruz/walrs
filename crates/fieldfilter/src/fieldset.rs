//! Typed struct validation and filtering.
//!
//! This module provides the [`Fieldset`] trait for compile-time-checked
//! validation and filtering of typed structs — the recommended replacement
//! for the dynamic `FieldFilter` path when your fields are known at
//! compile time.

use walrs_validation::FieldsetViolations;

#[cfg(feature = "async")]
use std::future::Future;

/// Trait for typed struct validation and filtering.
///
/// Implement this trait (or derive it with `#[derive(Fieldset)]`) on a struct
/// to get compile-time-checked validation and filtering.
///
/// # Example
///
/// ```rust
/// use walrs_fieldfilter::Fieldset;
/// use walrs_validation::FieldsetViolations;
///
/// struct LoginForm {
///     email: String,
///     password: String,
/// }
///
/// impl Fieldset for LoginForm {
///     fn validate(&self) -> Result<(), FieldsetViolations> {
///         let mut violations = FieldsetViolations::new();
///         if self.email.is_empty() {
///             violations.add("email", walrs_validation::Violation::value_missing());
///         }
///         if self.password.len() < 8 {
///             violations.add("password", walrs_validation::Violation::new(
///                 walrs_validation::ViolationType::TooShort, "Password must be at least 8 characters"
///             ));
///         }
///         violations.into()
///     }
///
///     fn filter(self) -> Result<Self, FieldsetViolations> {
///         Ok(Self {
///             email: self.email.trim().to_lowercase(),
///             password: self.password,
///         })
///     }
/// }
///
/// let form = LoginForm { email: " Test@Example.com ".to_string(), password: "secret123".to_string() };
/// let cleaned = form.clean().unwrap();
/// assert_eq!(cleaned.email, "test@example.com");
/// ```
pub trait Fieldset: Sized {
  /// If `true`, validation stops after the first field with violations.
  const BREAK_ON_FAILURE: bool = false;

  /// Validate all fields, returning any violations.
  fn validate(&self) -> Result<(), FieldsetViolations>;

  /// Apply filters to all fields, returning the filtered struct.
  fn filter(self) -> Result<Self, FieldsetViolations>;

  /// Filter and then validate (convenience method).
  fn clean(self) -> Result<Self, FieldsetViolations> {
    let filtered = self.filter()?;
    filtered.validate()?;
    Ok(filtered)
  }
}

/// Async version of [`Fieldset`].
///
/// Provides async validation and filtering for structs that need
/// async validators (e.g., database uniqueness checks).
#[cfg(feature = "async")]
pub trait FieldsetAsync: Fieldset + Send {
  /// Validate all fields asynchronously.
  fn validate_async(&self) -> impl Future<Output = Result<(), FieldsetViolations>> + Send;

  /// Apply filters to all fields asynchronously.
  fn filter_async(self) -> impl Future<Output = Result<Self, FieldsetViolations>> + Send;

  /// Filter and then validate asynchronously (convenience method).
  fn clean_async(self) -> impl Future<Output = Result<Self, FieldsetViolations>> + Send {
    async {
      let filtered = self.filter_async().await?;
      filtered.validate_async().await?;
      Ok(filtered)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use walrs_validation::{Violation, ViolationType};

  // --- Test struct ---

  #[derive(Debug, Clone, PartialEq)]
  struct ContactForm {
    name: String,
    email: String,
  }

  impl Fieldset for ContactForm {
    fn validate(&self) -> Result<(), FieldsetViolations> {
      let mut fv = FieldsetViolations::new();
      if self.name.is_empty() {
        fv.add("name", Violation::value_missing());
      }
      if self.email.is_empty() {
        fv.add("email", Violation::value_missing());
      } else if !self.email.contains('@') {
        fv.add("email", Violation::invalid_email());
      }
      fv.into()
    }

    fn filter(self) -> Result<Self, FieldsetViolations> {
      Ok(Self {
        name: self.name.trim().to_string(),
        email: self.email.trim().to_lowercase(),
      })
    }
  }

  // --- Struct with BREAK_ON_FAILURE ---

  #[derive(Debug, Clone, PartialEq)]
  struct StrictForm {
    a: String,
    b: String,
  }

  impl Fieldset for StrictForm {
    const BREAK_ON_FAILURE: bool = true;

    fn validate(&self) -> Result<(), FieldsetViolations> {
      let mut fv = FieldsetViolations::new();
      if self.a.is_empty() {
        fv.add("a", Violation::value_missing());
        if Self::BREAK_ON_FAILURE {
          return fv.into();
        }
      }
      if self.b.is_empty() {
        fv.add("b", Violation::value_missing());
      }
      fv.into()
    }

    fn filter(self) -> Result<Self, FieldsetViolations> {
      Ok(self)
    }
  }

  // --- Struct whose filter can fail ---

  #[derive(Debug, Clone, PartialEq)]
  struct ParsedForm {
    age: String,
  }

  impl Fieldset for ParsedForm {
    fn validate(&self) -> Result<(), FieldsetViolations> {
      FieldsetViolations::new().into()
    }

    fn filter(self) -> Result<Self, FieldsetViolations> {
      // Simulate a filter that rejects invalid data
      if self.age.parse::<u32>().is_err() {
        let mut fv = FieldsetViolations::new();
        fv.add(
          "age",
          Violation::new(ViolationType::TypeMismatch, "Must be a number"),
        );
        Err(fv)
      } else {
        Ok(self)
      }
    }
  }

  // 1. Test manual Fieldset impl — validate, filter, clean
  #[test]
  fn test_validate_pass() {
    let form = ContactForm {
      name: "Alice".into(),
      email: "alice@example.com".into(),
    };
    assert!(form.validate().is_ok());
  }

  #[test]
  fn test_validate_fail() {
    let form = ContactForm {
      name: "".into(),
      email: "bad".into(),
    };
    let err = form.validate().unwrap_err();
    assert!(err.get("name").is_some());
    assert!(err.get("email").is_some());
  }

  #[test]
  fn test_filter() {
    let form = ContactForm {
      name: "  Bob  ".into(),
      email: "  BOB@EXAMPLE.COM  ".into(),
    };
    let filtered = form.filter().unwrap();
    assert_eq!(filtered.name, "Bob");
    assert_eq!(filtered.email, "bob@example.com");
  }

  #[test]
  fn test_clean_success() {
    let form = ContactForm {
      name: "  Alice  ".into(),
      email: "  ALICE@EXAMPLE.COM  ".into(),
    };
    let cleaned = form.clean().unwrap();
    assert_eq!(cleaned.name, "Alice");
    assert_eq!(cleaned.email, "alice@example.com");
  }

  // 2. Test BREAK_ON_FAILURE const override
  #[test]
  fn test_break_on_failure_const() {
    assert!(StrictForm::BREAK_ON_FAILURE);
    assert!(!ContactForm::BREAK_ON_FAILURE);
  }

  #[test]
  fn test_break_on_failure_stops_early() {
    let form = StrictForm {
      a: "".into(),
      b: "".into(),
    };
    let err = form.validate().unwrap_err();
    // Only "a" should be present because BREAK_ON_FAILURE is true
    assert!(err.get("a").is_some());
    assert!(err.get("b").is_none());
    assert_eq!(err.len(), 1);
  }

  // 3. Test that clean = filter + validate
  #[test]
  fn test_clean_equals_filter_then_validate() {
    let form1 = ContactForm {
      name: "  Alice  ".into(),
      email: "  ALICE@EXAMPLE.COM  ".into(),
    };
    let form2 = form1.clone();

    let via_clean = form1.clean().unwrap();

    let filtered = form2.filter().unwrap();
    filtered.validate().unwrap();
    let via_manual = filtered;

    assert_eq!(via_clean, via_manual);
  }

  // 4. Test clean with filter error
  #[test]
  fn test_clean_filter_error() {
    let form = ParsedForm {
      age: "not-a-number".into(),
    };
    let err = form.clean().unwrap_err();
    assert!(err.get("age").is_some());
    assert_eq!(err.len(), 1);
  }

  // 5. Test clean with validation error
  #[test]
  fn test_clean_validation_error() {
    let form = ContactForm {
      name: "  ".into(),
      email: "  bad  ".into(),
    };
    // Filter trims, then validate sees empty name and bad email
    let err = form.clean().unwrap_err();
    assert!(err.get("name").is_some() || err.get("email").is_some());
  }

  // 8. Test async trait
  #[cfg(feature = "async")]
  mod async_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct AsyncForm {
      email: String,
    }

    impl Fieldset for AsyncForm {
      fn validate(&self) -> Result<(), FieldsetViolations> {
        let mut fv = FieldsetViolations::new();
        if self.email.is_empty() {
          fv.add("email", Violation::value_missing());
        }
        fv.into()
      }

      fn filter(self) -> Result<Self, FieldsetViolations> {
        Ok(Self {
          email: self.email.trim().to_lowercase(),
        })
      }
    }

    impl FieldsetAsync for AsyncForm {
      fn validate_async(&self) -> impl Future<Output = Result<(), FieldsetViolations>> + Send {
        async { self.validate() }
      }

      fn filter_async(self) -> impl Future<Output = Result<Self, FieldsetViolations>> + Send {
        async { self.filter() }
      }
    }

    #[tokio::test]
    async fn test_validate_async() {
      let form = AsyncForm {
        email: "test@example.com".into(),
      };
      assert!(form.validate_async().await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_async_fail() {
      let form = AsyncForm { email: "".into() };
      let err = form.validate_async().await.unwrap_err();
      assert!(err.get("email").is_some());
    }

    #[tokio::test]
    async fn test_filter_async() {
      let form = AsyncForm {
        email: "  TEST@EXAMPLE.COM  ".into(),
      };
      let filtered = form.filter_async().await.unwrap();
      assert_eq!(filtered.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_clean_async() {
      let form = AsyncForm {
        email: "  VALID@EXAMPLE.COM  ".into(),
      };
      let cleaned = form.clean_async().await.unwrap();
      assert_eq!(cleaned.email, "valid@example.com");
    }

    #[tokio::test]
    async fn test_clean_async_validation_error() {
      let form = AsyncForm {
        email: "   ".into(),
      };
      let err = form.clean_async().await.unwrap_err();
      assert!(err.get("email").is_some());
    }
  }
}
