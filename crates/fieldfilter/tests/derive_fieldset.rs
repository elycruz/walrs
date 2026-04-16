//! Integration tests for `#[derive(Fieldset)]`.

#[cfg(feature = "derive")]
mod derive_tests {
  use walrs_fieldfilter::{DeriveFieldset, Fieldset};
  use walrs_validation::{Violation, ViolationType};

  // =========================================================================
  // Test 1: Simple struct with string validation + filters
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct UserAddress {
    #[validate(required, min_length = 3)]
    #[filter(trim)]
    street: String,

    #[validate(required, pattern = r"^\d{5}$")]
    #[filter(trim)]
    zip: String,
  }

  #[test]
  fn test_simple_validate_pass() {
    let addr = UserAddress {
      street: "123 Main St".into(),
      zip: "90210".into(),
    };
    assert!(addr.validate().is_ok());
  }

  #[test]
  fn test_simple_validate_fail() {
    let addr = UserAddress {
      street: "ab".into(), // too short
      zip: "bad".into(),   // doesn't match pattern
    };
    let err = addr.validate().unwrap_err();
    assert!(err.get("street").is_some());
    assert!(err.get("zip").is_some());
  }

  #[test]
  fn test_simple_filter() {
    let addr = UserAddress {
      street: "  123 Main St  ".into(),
      zip: "  90210  ".into(),
    };
    let filtered = addr.filter().unwrap();
    assert_eq!(filtered.street, "123 Main St");
    assert_eq!(filtered.zip, "90210");
  }

  #[test]
  fn test_simple_clean() {
    let addr = UserAddress {
      street: "  123 Main St  ".into(),
      zip: "  90210  ".into(),
    };
    let cleaned = addr.clean().unwrap();
    assert_eq!(cleaned.street, "123 Main St");
    assert_eq!(cleaned.zip, "90210");
  }

  #[test]
  fn test_clean_fails_validation_after_filter() {
    let addr = UserAddress {
      street: "  ab  ".into(), // trims to "ab", too short
      zip: "  bad  ".into(),   // trims to "bad", no match
    };
    let err = addr.clean().unwrap_err();
    assert!(err.get("street").is_some());
    assert!(err.get("zip").is_some());
  }

  // =========================================================================
  // Test 2: Nested struct
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct Registration {
    #[validate(required)]
    #[filter(trim)]
    name: String,

    #[validate(nested)]
    #[filter(nested)]
    address: UserAddress,
  }

  #[test]
  fn test_nested_validate_pass() {
    let reg = Registration {
      name: "Alice".into(),
      address: UserAddress {
        street: "123 Main St".into(),
        zip: "90210".into(),
      },
    };
    assert!(reg.validate().is_ok());
  }

  #[test]
  fn test_nested_validate_fail() {
    let reg = Registration {
      name: "".into(),
      address: UserAddress {
        street: "ab".into(),
        zip: "bad".into(),
      },
    };
    let err = reg.validate().unwrap_err();
    assert!(err.get("name").is_some());
    assert!(err.get("address.street").is_some());
    assert!(err.get("address.zip").is_some());
  }

  #[test]
  fn test_nested_filter() {
    let reg = Registration {
      name: "  Alice  ".into(),
      address: UserAddress {
        street: "  123 Main  ".into(),
        zip: "  90210  ".into(),
      },
    };
    let filtered = reg.filter().unwrap();
    assert_eq!(filtered.name, "Alice");
    assert_eq!(filtered.address.street, "123 Main");
    assert_eq!(filtered.address.zip, "90210");
  }

  // =========================================================================
  // Test 3: Cross-field validation
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  #[cross_validate(passwords_match)]
  struct PasswordForm {
    #[validate(required, min_length = 8)]
    password: String,

    #[validate(required)]
    confirm: String,
  }

  fn passwords_match(form: &PasswordForm) -> walrs_validation::RuleResult {
    if form.password == form.confirm {
      Ok(())
    } else {
      Err(Violation::new(
        ViolationType::NotEqual,
        "Passwords must match",
      ))
    }
  }

  #[test]
  fn test_cross_validate_pass() {
    let form = PasswordForm {
      password: "secretpass".into(),
      confirm: "secretpass".into(),
    };
    assert!(form.validate().is_ok());
  }

  #[test]
  fn test_cross_validate_fail() {
    let form = PasswordForm {
      password: "secretpass".into(),
      confirm: "different".into(),
    };
    let err = form.validate().unwrap_err();
    // Cross-field violations are under "" key (form-level)
    assert!(err.form_violations().is_some());
  }

  #[test]
  fn test_cross_validate_with_field_errors() {
    let form = PasswordForm {
      password: "short".into(), // min_length = 8 fails
      confirm: "different".into(),
    };
    let err = form.validate().unwrap_err();
    // Both field and form-level errors
    assert!(err.get("password").is_some());
    assert!(err.form_violations().is_some());
  }

  // =========================================================================
  // Test 4: Option<T> fields
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct OptionalForm {
    #[validate(required)]
    name: String,

    #[validate(email)]
    #[filter(trim, lowercase)]
    email: Option<String>,
  }

  #[test]
  fn test_option_none_skips() {
    let form = OptionalForm {
      name: "Alice".into(),
      email: None,
    };
    // email is None and not required — should pass
    assert!(form.validate().is_ok());
  }

  #[test]
  fn test_option_some_validates() {
    let form = OptionalForm {
      name: "Alice".into(),
      email: Some("not-an-email".into()),
    };
    let err = form.validate().unwrap_err();
    assert!(err.get("email").is_some());
  }

  #[test]
  fn test_option_some_passes() {
    let form = OptionalForm {
      name: "Alice".into(),
      email: Some("alice@example.com".into()),
    };
    assert!(form.validate().is_ok());
  }

  #[test]
  fn test_option_filter() {
    let form = OptionalForm {
      name: "Alice".into(),
      email: Some("  ALICE@EXAMPLE.COM  ".into()),
    };
    let filtered = form.filter().unwrap();
    assert_eq!(filtered.email, Some("alice@example.com".into()));
  }

  #[test]
  fn test_option_filter_none() {
    let form = OptionalForm {
      name: "Alice".into(),
      email: None,
    };
    let filtered = form.filter().unwrap();
    assert_eq!(filtered.email, None);
  }

  // =========================================================================
  // Test 5: break_on_failure
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  #[fieldset(break_on_failure)]
  struct StrictForm {
    #[validate(required)]
    first: String,

    #[validate(required)]
    second: String,
  }

  #[test]
  fn test_break_on_failure_const() {
    assert!(StrictForm::BREAK_ON_FAILURE);
  }

  #[test]
  fn test_break_on_failure_stops_early() {
    let form = StrictForm {
      first: "".into(),
      second: "".into(),
    };
    let err = form.validate().unwrap_err();
    // Only first field should be reported because break_on_failure stops early
    assert!(err.get("first").is_some());
    assert!(err.get("second").is_none());
    assert_eq!(err.len(), 1);
  }

  #[test]
  fn test_break_on_failure_passes() {
    let form = StrictForm {
      first: "hello".into(),
      second: "world".into(),
    };
    assert!(form.validate().is_ok());
  }

  // =========================================================================
  // Test 6: Numeric validation
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct NumericForm {
    #[validate(min = 0, max = 150)]
    age: i64,

    #[validate(range(min = 0.0, max = 100.0))]
    score: f64,
  }

  #[test]
  fn test_numeric_validate_pass() {
    let form = NumericForm {
      age: 25,
      score: 85.5,
    };
    assert!(form.validate().is_ok());
  }

  #[test]
  fn test_numeric_validate_fail_min() {
    let form = NumericForm {
      age: -1,
      score: 50.0,
    };
    let err = form.validate().unwrap_err();
    assert!(err.get("age").is_some());
  }

  #[test]
  fn test_numeric_validate_fail_max() {
    let form = NumericForm {
      age: 200,
      score: 50.0,
    };
    let err = form.validate().unwrap_err();
    assert!(err.get("age").is_some());
  }

  #[test]
  fn test_numeric_validate_fail_range() {
    let form = NumericForm {
      age: 25,
      score: 150.0,
    };
    let err = form.validate().unwrap_err();
    assert!(err.get("score").is_some());
  }

  // =========================================================================
  // Test 7: Message customization
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct CustomMessageForm {
    #[validate(required, message = "Name is required!")]
    name: String,
  }

  #[test]
  fn test_custom_message() {
    let form = CustomMessageForm { name: "".into() };
    let err = form.validate().unwrap_err();
    let violations = err.get("name").unwrap();
    assert_eq!(violations.0.len(), 1);
    assert_eq!(violations.0[0].message(), "Name is required!");
  }

  // =========================================================================
  // Test 8: Multiple filters
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct MultiFilterForm {
    #[filter(trim, lowercase)]
    email: String,

    #[filter(trim, uppercase)]
    code: String,
  }

  #[test]
  fn test_multiple_filters() {
    let form = MultiFilterForm {
      email: "  HELLO@WORLD.COM  ".into(),
      code: "  abc123  ".into(),
    };
    let filtered = form.filter().unwrap();
    assert_eq!(filtered.email, "hello@world.com");
    assert_eq!(filtered.code, "ABC123");
  }

  // =========================================================================
  // Test 9: Field with no attributes (passthrough)
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct MixedForm {
    #[validate(required)]
    name: String,

    // No annotations — just passes through
    extra: String,
  }

  #[test]
  fn test_passthrough_field() {
    let form = MixedForm {
      name: "Alice".into(),
      extra: "  some data  ".into(),
    };
    let filtered = form.filter().unwrap();
    assert_eq!(filtered.extra, "  some data  "); // not modified
  }

  // =========================================================================
  // Test 10: Numeric filter (clamp)
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct ClampForm {
    #[validate(min = 0, max = 100)]
    #[filter(clamp(min = 0, max = 100))]
    percentage: i32,
  }

  #[test]
  fn test_numeric_clamp_filter() {
    let form = ClampForm { percentage: 150 };
    let filtered = form.filter().unwrap();
    assert_eq!(filtered.percentage, 100);
  }

  #[test]
  fn test_numeric_clamp_clean() {
    let form = ClampForm { percentage: 150 };
    // After filter (clamp to 100), validation passes
    let cleaned = form.clean().unwrap();
    assert_eq!(cleaned.percentage, 100);
  }

  // =========================================================================
  // Test 11: Per-field break_on_failure override
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  #[fieldset(break_on_failure)]
  struct OverrideForm {
    #[validate(required)]
    #[fieldset(break_on_failure = false)]
    first: String,

    #[validate(required)]
    second: String,
  }

  #[test]
  fn test_per_field_break_override() {
    let form = OverrideForm {
      first: "".into(),
      second: "".into(),
    };
    let err = form.validate().unwrap_err();
    // first has break_on_failure=false override, so validation continues to second
    // but second has struct-level break_on_failure=true
    assert!(err.get("first").is_some());
    assert!(err.get("second").is_some());
  }

  // =========================================================================
  // Test 12: Option<T> with required
  // =========================================================================

  #[derive(Debug, DeriveFieldset)]
  struct RequiredOptionForm {
    #[validate(required, min_length = 3)]
    nickname: Option<String>,
  }

  #[test]
  fn test_required_option_none_fails() {
    let form = RequiredOptionForm { nickname: None };
    let err = form.validate().unwrap_err();
    assert!(err.get("nickname").is_some());
  }

  #[test]
  fn test_required_option_some_validates() {
    let form = RequiredOptionForm {
      nickname: Some("ab".into()), // too short
    };
    let err = form.validate().unwrap_err();
    assert!(err.get("nickname").is_some());
  }

  #[test]
  fn test_required_option_some_passes() {
    let form = RequiredOptionForm {
      nickname: Some("alice".into()),
    };
    assert!(form.validate().is_ok());
  }
}
