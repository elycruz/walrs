//! Example: Form validation
//!
//! This example demonstrates a practical use case for validators
//! in a form validation scenario.
//!
//! Run with: `cargo run --example form_validation`
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use walrs_validation::{
  LengthValidatorBuilder, PatternValidatorBuilder, RangeValidatorBuilder, Validate, ValidateRef,
  Violations,
};
/// Represents a user registration form
struct RegistrationForm {
  username: String,
  email: String,
  password: String,
  age: i32,
}
/// Validates the registration form and returns all errors
fn validate_registration(form: &RegistrationForm) -> Result<(), HashMap<String, Violations>> {
  let mut errors: HashMap<String, Violations> = HashMap::new();
  // Username validation: 3-20 characters, alphanumeric
  let username_length = LengthValidatorBuilder::<str>::default()
    .min_length(3)
    .max_length(20)
    .build()
    .unwrap();
  let username_pattern = PatternValidatorBuilder::default()
    .pattern(Cow::Owned(Regex::new(r"^[a-zA-Z0-9_]+$").unwrap()))
    .build()
    .unwrap();
  let mut username_violations = Violations::default();
  if let Err(v) = username_length.validate_ref(&form.username) {
    username_violations.push(v);
  }
  if let Err(v) = username_pattern.validate_ref(&form.username) {
    username_violations.push(v);
  }
  if !username_violations.is_empty() {
    errors.insert("username".to_string(), username_violations);
  }
  // Email validation
  let email_pattern = PatternValidatorBuilder::default()
    .pattern(Cow::Owned(
      Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
    ))
    .build()
    .unwrap();
  if let Err(v) = email_pattern.validate_ref(&form.email) {
    errors.insert("email".to_string(), Violations::new(vec![v]));
  }
  // Password validation: minimum 8 characters
  let password_length = LengthValidatorBuilder::<str>::default()
    .min_length(8)
    .build()
    .unwrap();
  if let Err(v) = password_length.validate_ref(&form.password) {
    errors.insert("password".to_string(), Violations::new(vec![v]));
  }
  // Age validation: 13-120
  let age_range = RangeValidatorBuilder::<i32>::default()
    .min(13)
    .max(120)
    .build()
    .unwrap();
  if let Err(v) = age_range.validate(form.age) {
    errors.insert("age".to_string(), Violations::new(vec![v]));
  }
  if errors.is_empty() {
    Ok(())
  } else {
    Err(errors)
  }
}
fn print_validation_result(name: &str, form: &RegistrationForm) {
  println!("--- {} ---", name);
  println!("  Username: \"{}\"", form.username);
  println!("  Email: \"{}\"", form.email);
  println!("  Password: \"{}\"", "*".repeat(form.password.len()));
  println!("  Age: {}", form.age);
  println!();
  match validate_registration(form) {
    Ok(()) => {
      println!("  Result: VALID\n");
    }
    Err(errors) => {
      println!("  Validation errors:");
      for (field, violations) in &errors {
        for violation in violations.iter() {
          println!("    - {}: {}", field, violation);
        }
      }
      println!();
    }
  }
}
fn main() {
  println!("=== Form Validation Example ===\n");
  // Valid form
  let valid_form = RegistrationForm {
    username: "john_doe".to_string(),
    email: "john@example.com".to_string(),
    password: "securepassword123".to_string(),
    age: 25,
  };
  print_validation_result("Valid Registration", &valid_form);
  // Invalid username (too short)
  let short_username = RegistrationForm {
    username: "ab".to_string(),
    email: "ab@example.com".to_string(),
    password: "password123".to_string(),
    age: 25,
  };
  print_validation_result("Short Username", &short_username);
  // Invalid email
  let invalid_email = RegistrationForm {
    username: "validuser".to_string(),
    email: "not-an-email".to_string(),
    password: "password123".to_string(),
    age: 25,
  };
  print_validation_result("Invalid Email", &invalid_email);
  // Multiple errors
  let multiple_errors = RegistrationForm {
    username: "a!".to_string(),
    email: "bad-email".to_string(),
    password: "short".to_string(),
    age: 5,
  };
  print_validation_result("Multiple Errors", &multiple_errors);
  println!("=== Example Complete ===");
}
