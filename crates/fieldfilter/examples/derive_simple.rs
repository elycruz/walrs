//! Simple derive(Fieldset) example with validation and filtering.

use walrs_fieldfilter::{DeriveFieldset, Fieldset};

#[derive(Debug, DeriveFieldset)]
struct ContactForm {
  #[validate(required, email)]
  #[filter(trim, lowercase)]
  email: String,

  #[validate(required, min_length = 2)]
  #[filter(trim)]
  name: String,
}

fn main() {
  let form = ContactForm {
    email: "  USER@EXAMPLE.COM  ".into(),
    name: "  Alice  ".into(),
  };

  match form.sanitize() {
    Ok(sanitized) => {
      println!("✓ Validation passed!");
      println!("  Email: {} (was:   USER@EXAMPLE.COM  )", sanitized.email);
      println!("  Name: {} (was:   Alice  )", sanitized.name);
    }
    Err(violations) => {
      eprintln!("✗ Validation failed:");
      for (field, field_violations) in violations.iter() {
        for v in field_violations.0.iter() {
          eprintln!("  {}: {}", field, v.message());
        }
      }
    }
  }

  // Example with validation errors
  println!("\n--- Testing with invalid data ---");
  let invalid_form = ContactForm {
    email: "".into(),
    name: "A".into(),
  };

  match invalid_form.sanitize() {
    Ok(_) => println!("✓ Unexpected success"),
    Err(violations) => {
      eprintln!("✗ Validation failed (expected):");
      for (field, field_violations) in violations.iter() {
        for v in field_violations.0.iter() {
          eprintln!("  {}: {}", field, v.message());
        }
      }
    }
  }
}
