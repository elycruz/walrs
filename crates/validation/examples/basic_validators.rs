//! Example: Basic validator usage
//!
//! Run with: `cargo run --example basic_validators`

use regex::Regex;
use std::borrow::Cow;
use walrs_validation::{
  EqualityValidatorBuilder, LengthValidatorBuilder, PatternValidatorBuilder, RangeValidatorBuilder,
  StepValidatorBuilder, Validate, ValidateExt, ValidateRef,
};

fn main() {
  println!("=== walrs_validation Examples ===\n");

  // LengthValidator example
  println!("--- LengthValidator ---");
  let length_validator = LengthValidatorBuilder::<str>::default()
    .min_length(3)
    .max_length(20)
    .build()
    .unwrap();

  let strings = [
    "hi",
    "hello",
    "hello world",
    "this is way too long for our validator",
  ];

  for s in strings {
    let result = length_validator.validate_ref(s);
    let status = if result.is_ok() {
      "✓ PASS"
    } else {
      "✗ FAIL"
    };
    println!("  \"{}\" (len={}) -> {}", s, s.len(), status);
    if let Err(violation) = result {
      println!("    Error: {}", violation);
    }
  }

  println!();

  // RangeValidator example
  println!("--- RangeValidator ---");
  let range_validator = RangeValidatorBuilder::<i32>::default()
    .min(1)
    .max(100)
    .build()
    .unwrap();

  let numbers = [0, 1, 50, 100, 101];

  for n in numbers {
    let result = range_validator.validate(n);
    let status = if result.is_ok() {
      "✓ PASS"
    } else {
      "✗ FAIL"
    };
    println!("  {} -> {}", n, status);
    if let Err(violation) = result {
      println!("    Error: {}", violation);
    }
  }

  println!();

  // RangeValidator + StepValidator combined example
  println!("--- RangeValidator + StepValidator (combined) ---");
  let range_with_step = RangeValidatorBuilder::<i32>::default()
    .min(0)
    .max(100)
    .build()
    .unwrap();
  let step_check = StepValidatorBuilder::<i32>::default()
    .step(5)
    .build()
    .unwrap();
  let combined_validator = range_with_step.and(step_check);

  let numbers = [0, 5, 7, 25, 100, 103];

  for n in numbers {
    let result = combined_validator.validate(n);
    let status = if result.is_ok() {
      "✓ PASS"
    } else {
      "✗ FAIL"
    };
    println!("  {} -> {}", n, status);
    if let Err(violation) = result {
      println!("    Error: {}", violation);
    }
  }

  println!();

  // StepValidator example
  println!("--- StepValidator ---");
  let step_validator = StepValidatorBuilder::<i32>::default()
    .step(5)
    .build()
    .unwrap();

  let numbers = [0, 5, 7, 10, 25, 26, -10, -7];

  for n in numbers {
    let result = step_validator.validate(n);
    let status = if result.is_ok() {
      "✓ PASS"
    } else {
      "✗ FAIL"
    };
    println!("  {} -> {}", n, status);
    if let Err(violation) = result {
      println!("    Error: {}", violation);
    }
  }

  println!();

  // PatternValidator example
  println!("--- PatternValidator (email-like pattern) ---");
  let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
  let pattern_validator = PatternValidatorBuilder::default()
    .pattern(Cow::Owned(email_regex))
    .build()
    .unwrap();

  let emails = [
    "valid@example.com",
    "user.name+tag@domain.org",
    "invalid-email",
    "@missing-local.com",
    "missing-domain@",
  ];

  for email in emails {
    let result = pattern_validator.validate_ref(email);
    let status = if result.is_ok() {
      "✓ PASS"
    } else {
      "✗ FAIL"
    };
    println!("  \"{}\" -> {}", email, status);
  }

  println!();

  // EqualityValidator example
  println!("--- EqualityValidator ---");
  let equality_validator = EqualityValidatorBuilder::<&str>::default()
    .rhs_value("secret")
    .build()
    .unwrap();

  let passwords = ["secret", "wrong", "SECRET", "secret123"];

  for pwd in passwords {
    let result = equality_validator.validate(pwd);
    let status = if result.is_ok() {
      "✓ PASS"
    } else {
      "✗ FAIL"
    };
    println!("  \"{}\" -> {}", pwd, status);
  }

  println!();
  println!("=== Examples Complete ===");
}
