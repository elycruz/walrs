//! FormViolations example.
//!
//! This example demonstrates how to work with `FormViolations` for
//! collecting and organizing validation errors from multiple fields.
//!
//! Run with: `cargo run --example form_violations`

use walrs_inputfilter::form_violations::FormViolations;
use walrs_validator::{Violation, ViolationType, Violations};

fn main() {
  println!("=== FormViolations Examples ===\n");

  // Example 1: Creating and populating FormViolations
  println!("1. Creating and adding violations:");
  let mut form_violations = FormViolations::new();

  // Add field-level violations
  form_violations.add_field_violation(
    "email",
    Violation::new(ViolationType::TypeMismatch, "Invalid email format"),
  );

  form_violations.add_field_violation(
    "password",
    Violation::new(
      ViolationType::TooShort,
      "Password must be at least 8 characters",
    ),
  );

  form_violations.add_field_violation(
    "password",
    Violation::new(
      ViolationType::PatternMismatch,
      "Password must contain a number",
    ),
  );

  // Add form-level violation
  form_violations.add_form_violation(Violation::new(
    ViolationType::CustomError,
    "Passwords do not match",
  ));

  println!("   Total violations: {}", form_violations.len());
  println!("   Is empty: {}", form_violations.is_empty());

  // Example 2: Querying violations
  println!("\n2. Querying violations by field:");
  for field_name in form_violations.field_names() {
    if let Some(violations) = form_violations.for_field(field_name) {
      println!(
        "   Field '{}' has {} violation(s):",
        field_name,
        violations.len()
      );
      for v in violations.iter() {
        println!("      - [{:?}] {}", v.violation_type(), v.message());
      }
    }
  }

  println!("\n   Form-level violations:");
  for v in form_violations.form.iter() {
    println!("      - [{:?}] {}", v.violation_type(), v.message());
  }

  // Example 3: Adding multiple violations at once
  println!("\n3. Adding multiple violations at once:");
  let mut bulk_violations = FormViolations::new();

  let mut username_violations = Violations::empty();
  username_violations.push(Violation::new(ViolationType::TooShort, "Too short"));
  username_violations.push(Violation::new(
    ViolationType::PatternMismatch,
    "Invalid characters",
  ));

  bulk_violations.add_field_violations("username", username_violations);

  println!(
    "   Username violations: {}",
    bulk_violations
      .for_field("username")
      .map(|v| v.len())
      .unwrap_or(0)
  );

  // Example 4: Merging FormViolations
  println!("\n4. Merging FormViolations:");
  let mut violations_a = FormViolations::new();
  violations_a.add_field_violation(
    "email",
    Violation::new(ViolationType::ValueMissing, "Required"),
  );

  let mut violations_b = FormViolations::new();
  violations_b.add_field_violation(
    "phone",
    Violation::new(ViolationType::ValueMissing, "Required"),
  );
  violations_b.add_form_violation(Violation::new(ViolationType::CustomError, "Form error"));

  println!("   Before merge:");
  println!("      Violations A: {} total", violations_a.len());
  println!("      Violations B: {} total", violations_b.len());

  violations_a.merge(violations_b);

  println!("   After merge:");
  println!("      Violations A: {} total", violations_a.len());
  println!(
    "      Fields with violations: {:?}",
    violations_a.field_names().collect::<Vec<_>>()
  );

  // Example 5: Converting to Result
  println!("\n5. Converting to Result:");
  let empty_violations = FormViolations::new();
  let result_ok: Result<(), FormViolations> = empty_violations.into();
  println!("   Empty violations -> Result: {:?}", result_ok.is_ok());

  let mut non_empty = FormViolations::new();
  non_empty.add_field_violation("test", Violation::new(ViolationType::ValueMissing, "Error"));
  let result_err: Result<(), FormViolations> = non_empty.into();
  println!(
    "   Non-empty violations -> Result: {:?}",
    result_err.is_ok()
  );

  // Example 6: Practical use case - Form validation summary
  println!("\n6. Practical example - Form validation summary:");
  let violations = simulate_form_validation();
  print_validation_summary(&violations);

  // Example 7: Clearing violations
  println!("\n7. Clearing violations:");
  let mut clearable = FormViolations::new();
  clearable.add_field_violation("test", Violation::new(ViolationType::ValueMissing, "Error"));
  println!("   Before clear: {} violations", clearable.len());
  clearable.clear();
  println!("   After clear: {} violations", clearable.len());

  println!("\n=== Examples Complete ===");
}

fn simulate_form_validation() -> FormViolations {
  let mut violations = FormViolations::new();

  // Simulate validation of a registration form using fluent interface
  violations
    .add_field_violation(
      "username",
      Violation::new(
        ViolationType::TooShort,
        "Username must be at least 3 characters",
      ),
    )
    .add_field_violation(
      "email",
      Violation::new(
        ViolationType::TypeMismatch,
        "Please enter a valid email address",
      ),
    )
    .add_field_violation(
      "password",
      Violation::new(
        ViolationType::TooShort,
        "Password must be at least 8 characters",
      ),
    )
    .add_field_violation(
      "password",
      Violation::new(
        ViolationType::PatternMismatch,
        "Password must contain at least one uppercase letter",
      ),
    )
    .add_form_violation(Violation::new(
      ViolationType::CustomError,
      "Password confirmation does not match",
    ));

  violations
}

fn print_validation_summary(violations: &FormViolations) {
  if violations.is_empty() {
    println!("   ✓ Form is valid!");
    return;
  }

  println!("   ✗ Form has {} error(s):\n", violations.len());

  // Print field errors grouped by field
  let mut field_names: Vec<_> = violations.field_names().collect();
  field_names.sort();

  for field_name in field_names {
    if let Some(field_violations) = violations.for_field(field_name) {
      println!("   {}:", field_name);
      for v in field_violations.iter() {
        println!("      • {}", v.message());
      }
    }
  }

  // Print form-level errors
  if !violations.form.is_empty() {
    println!("\n   Form errors:");
    for v in violations.form.iter() {
      println!("      • {}", v.message());
    }
  }
}
