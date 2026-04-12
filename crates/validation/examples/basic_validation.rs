//! # Basic Validation Example
//!
//! Demonstrates core validation rules: string length, numeric range,
//! required fields, and regex pattern matching.

use walrs_validation::{Rule, Validate, ValidateRef};

fn main() {
  // -------------------------------------------------------------------------
  // String length validation
  // -------------------------------------------------------------------------
  let username_rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(20));

  assert!(username_rule.validate_ref("alice").is_ok());
  assert!(username_rule.validate_ref("ab").is_err());
  assert!(
    username_rule
      .validate_ref("this_username_is_way_too_long_for_our_system")
      .is_err()
  );

  println!("String length validation: OK");

  // -------------------------------------------------------------------------
  // Exact length
  // -------------------------------------------------------------------------
  let pin_rule = Rule::<String>::ExactLength(4);
  assert!(pin_rule.validate_ref("1234").is_ok());
  assert!(pin_rule.validate_ref("12345").is_err());

  println!("Exact length validation: OK");

  // -------------------------------------------------------------------------
  // Required (non-empty)
  // -------------------------------------------------------------------------
  let required_rule = Rule::<String>::Required;
  assert!(required_rule.validate_ref("hello").is_ok());
  assert!(required_rule.validate_ref("").is_err());
  assert!(required_rule.validate_ref("   ").is_err()); // whitespace-only is empty

  println!("Required validation: OK");

  // -------------------------------------------------------------------------
  // Regex pattern matching
  // -------------------------------------------------------------------------
  let slug_rule = Rule::<String>::pattern(r"(?i)^[\w\-]{1,100}$").unwrap();
  assert!(slug_rule.validate_ref("my-article-title").is_ok());
  assert!(slug_rule.validate_ref("invalid slug!").is_err());

  println!("Pattern validation: OK");

  // -------------------------------------------------------------------------
  // Numeric range
  // -------------------------------------------------------------------------
  let age_rule = Rule::<u32>::Min(0).and(Rule::Max(150));
  assert!(age_rule.validate(25).is_ok());
  assert!(age_rule.validate(200).is_err());

  let score_rule = Rule::<f64>::Range {
    min: 0.0,
    max: 100.0,
  };
  assert!(score_rule.validate(85.5).is_ok());
  assert!(score_rule.validate(-1.0).is_err());

  println!("Numeric range validation: OK");

  // -------------------------------------------------------------------------
  // Step validation
  // -------------------------------------------------------------------------
  let step_rule = Rule::<i32>::Step(5);
  assert!(step_rule.validate(15).is_ok());
  assert!(step_rule.validate(7).is_err());

  println!("Step validation: OK");

  // -------------------------------------------------------------------------
  // Equals / OneOf
  // -------------------------------------------------------------------------
  let status_rule = Rule::<i32>::OneOf(vec![0, 1, 2]);
  assert!(status_rule.validate(1).is_ok());
  assert!(status_rule.validate(99).is_err());

  let exact_rule = Rule::<i32>::Equals(42);
  assert!(exact_rule.validate(42).is_ok());
  assert!(exact_rule.validate(0).is_err());

  println!("Equals / OneOf validation: OK");

  println!("\nAll basic validation examples passed!");
}
