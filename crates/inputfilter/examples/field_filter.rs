//! FieldFilter example for multi-field validation.
//!
//! This example demonstrates how to use `FieldFilter` for validating
//! multiple fields with cross-field validation rules.
//!
//! Run with: `cargo run --example field_filter`

use std::collections::HashMap;
use walrs_validation::Value;
use walrs_inputfilter::field::FieldBuilder;
use walrs_inputfilter::field_filter::{CrossFieldRule, CrossFieldRuleType, FieldFilter};
use walrs_validation::{Rule, Rule::*};

fn main() {
  println!("=== FieldFilter Multi-Field Validation Examples ===\n");

  // Create a field filter for a user registration form using fluent API
  let mut filter = FieldFilter::new();
  let password_rule: Rule<Value> = Required.and(MinLength(8)).and(MaxLength(128));

  filter
    .add_field(
      "email",
      FieldBuilder::<Value>::default()
        .name("email")
          // Note: Rules that match html attributes will be exported when serializing to JSON;
          // E.g., json output will resemble:
          // { "type": "email", "name": "email", "required": true, "minlength": 5, "maxlength": 128 }
          // Allows rules to be sent to front-end clients so that they're shared
          // instead of isolated.
        .rule(All(vec![Required, MinLength(5), MaxLength(128), Email(Default::default())]))
        .build()
        .unwrap(),
    )
    .add_field(
      "password",
      FieldBuilder::<Value>::default()
        .name("password")
        .rule(password_rule.clone())
        .build()
        .unwrap(),
    )
    .add_field(
      "password_confirm",
      FieldBuilder::<Value>::default()
        .name("password_confirm")
        .rule(password_rule.clone())
        .build()
        .unwrap(),
    )
    // Add cross-field rule: passwords must match
    .add_cross_field_rule(CrossFieldRule {
      name: Some("password_match".into()),
      fields: vec!["password".to_string(), "password_confirm".to_string()],
      rule: CrossFieldRuleType::FieldsEqual {
        field_a: "password".to_string(),
        field_b: "password_confirm".to_string(),
      },
    });

  // Example 1: Valid data
  println!("1. Valid registration data:");
  let valid_data = make_data(&[
    ("email", "user@example.com"),
    ("password", "secretpassword123"),
    ("password_confirm", "secretpassword123"),
  ]);

  match filter.validate(&valid_data) {
    Ok(()) => println!("   ✓ All validations passed!\n"),
    Err(violations) => print_violations(&violations),
  }

  // Example 2: Missing required field
  println!("2. Missing email field:");
  let missing_email = make_data(&[
    ("password", "secretpassword123"),
    ("password_confirm", "secretpassword123"),
  ]);

  match filter.validate(&missing_email) {
    Ok(()) => println!("   ✓ All validations passed!\n"),
    Err(violations) => print_violations(&violations),
  }

  // Example 3: Passwords don't match
  println!("3. Passwords don't match:");
  let mismatched_passwords = make_data(&[
    ("email", "user@example.com"),
    ("password", "password123"),
    ("password_confirm", "different456"),
  ]);

  match filter.validate(&mismatched_passwords) {
    Ok(()) => println!("   ✓ All validations passed!\n"),
    Err(violations) => print_violations(&violations),
  }

  // Example 4: OneOfRequired - at least one contact method
  println!("4. OneOfRequired - at least one contact method:");
  let mut contact_filter = FieldFilter::new();
  contact_filter.add_cross_field_rule(CrossFieldRule {
    name: Some("contact_required".into()),
    fields: vec!["email".to_string(), "phone".to_string()],
    rule: CrossFieldRuleType::OneOfRequired(vec!["email".to_string(), "phone".to_string()]),
  });

  let no_contact = make_data(&[("name", "John")]);
  let with_email = make_data(&[("name", "John"), ("email", "john@example.com")]);
  let with_phone = make_data(&[("name", "John"), ("phone", "555-1234")]);

  println!(
    "   No contact info: {:?}",
    contact_filter.validate(&no_contact).is_ok()
  );
  println!(
    "   With email: {:?}",
    contact_filter.validate(&with_email).is_ok()
  );
  println!(
    "   With phone: {:?}",
    contact_filter.validate(&with_phone).is_ok()
  );

  // Example 5: MutuallyExclusive - only one payment method
  println!("\n5. MutuallyExclusive - only one payment method:");
  let mut payment_filter = FieldFilter::new();
  payment_filter.add_cross_field_rule(CrossFieldRule {
    name: Some("payment_exclusive".into()),
    fields: vec![
      "credit_card".to_string(),
      "paypal".to_string(),
      "bank_transfer".to_string(),
    ],
    rule: CrossFieldRuleType::MutuallyExclusive(vec![
      "credit_card".to_string(),
      "paypal".to_string(),
      "bank_transfer".to_string(),
    ]),
  });

  let one_payment = make_data(&[("credit_card", "4111111111111111")]);
  let multiple_payments = make_data(&[
    ("credit_card", "4111111111111111"),
    ("paypal", "user@paypal.com"),
  ]);

  println!(
    "   One payment method: {:?}",
    payment_filter.validate(&one_payment).is_ok()
  );
  println!(
    "   Multiple payment methods: {:?}",
    payment_filter.validate(&multiple_payments).is_ok()
  );

  // Example 6: DependentRequired - billing address required if shipping differs
  println!("\n6. DependentRequired - billing address when needed:");
  let mut address_filter = FieldFilter::new();
  address_filter.add_cross_field_rule(CrossFieldRule {
    name: Some("billing_required".into()),
    fields: vec![
      "different_billing".to_string(),
      "billing_address".to_string(),
    ],
    rule: CrossFieldRuleType::DependentRequired {
      field: "billing_address".to_string(),
      depends_on: "different_billing".to_string(),
    },
  });

  let same_address = make_data(&[("shipping_address", "123 Main St")]);
  let different_billing_no_address = make_data(&[
    ("shipping_address", "123 Main St"),
    ("different_billing", "true"),
  ]);
  let different_billing_with_address = make_data(&[
    ("shipping_address", "123 Main St"),
    ("different_billing", "true"),
    ("billing_address", "456 Other St"),
  ]);

  println!(
    "   Same address (no flag): {:?}",
    address_filter.validate(&same_address).is_ok()
  );
  println!(
    "   Different billing flag, no address: {:?}",
    address_filter
      .validate(&different_billing_no_address)
      .is_ok()
  );
  println!(
    "   Different billing with address: {:?}",
    address_filter
      .validate(&different_billing_with_address)
      .is_ok()
  );

  println!("\n=== Examples Complete ===");
}

fn make_data(pairs: &[(&str, &str)]) -> HashMap<String, Value> {
  pairs
    .iter()
    .map(|(k, v)| (k.to_string(), Value::Str(v.to_string())))
    .collect()
}

fn print_violations(violations: &walrs_inputfilter::FormViolations) {
  println!("   ✗ Validation failed:");
  for field_name in violations.field_names() {
    if let Some(field_violations) = violations.for_field(field_name) {
      for v in field_violations.iter() {
        println!("      - {}: {}", field_name, v.message());
      }
    }
  }
  for v in violations.form.iter() {
    println!("      - [form]: {}", v.message());
  }
  println!();
}
