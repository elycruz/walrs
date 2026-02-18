//! Basic Field<T> usage example.
//!
//! This example demonstrates the core functionality of the `Field<T>` struct
//! for single-field validation and filtering.
//!
//! Run with: `cargo run --example field_basics`

use walrs_inputfilter::field::FieldBuilder;
use walrs_inputfilter::filter_enum::Filter;
use walrs_validator::Rule;

fn main() {
    println!("=== Field<T> Basic Examples ===\n");

    // Example 1: Simple required field
    println!("1. Simple required field:");
    let username_field = FieldBuilder::<String>::default()
        .name("username")
        .rule(Rule::Required)
        .build()
        .unwrap();

    let empty = "".to_string();
    let valid = "john_doe".to_string();

    println!("   Validating empty string: {:?}", username_field.validate(&empty));
    println!("   Validating 'john_doe': {:?}", username_field.validate(&valid));

    // Example 2: Field with multiple rules using .and() combinator
    println!("\n2. Field with multiple rules:");
    let password_field = FieldBuilder::<String>::default()
        .name("password")
        .rule(Rule::Required.and(Rule::MinLength(8)).and(Rule::MaxLength(128)))
        .build()
        .unwrap();

    let short_password = "abc".to_string();
    let valid_password = "securepassword123".to_string();

    println!("   Validating 'abc': {:?}", password_field.validate(&short_password));
    println!("   Validating 'securepassword123': {:?}", password_field.validate(&valid_password));

    // Example 3: Field with filters
    println!("\n3. Field with filters:");
    let email_field = FieldBuilder::<String>::default()
        .name("email")
        .rule(Rule::Required.and(Rule::Email))
        .filters(vec![Filter::Trim, Filter::Lowercase])
        .build()
        .unwrap();

    let messy_email = "  USER@EXAMPLE.COM  ".to_string();
    let filtered = email_field.filter(messy_email.clone());
    println!("   Original: '{}'", messy_email);
    println!("   Filtered: '{}'", filtered);
    println!("   Validation: {:?}", email_field.validate(&filtered));

    // Example 4: Process (filter + validate in one step)
    println!("\n4. Using process() for filter + validate:");
    let input = "  ADMIN@COMPANY.ORG  ".to_string();
    match email_field.process(input.clone()) {
        Ok(result) => println!("   Input '{}' -> Result: '{}'", input, result),
        Err(violations) => println!("   Input '{}' -> Errors: {:?}", input, violations),
    }

    // Example 5: Collecting all violations
    println!("\n5. Collecting all violations:");
    let strict_field = FieldBuilder::<String>::default()
        .name("code")
        .rule(Rule::Required.and(Rule::MinLength(5)).and(Rule::MaxLength(10)))
        .break_on_failure(false) // Collect all violations
        .build()
        .unwrap();

    let too_short = "ab".to_string();
    match strict_field.validate(&too_short) {
        Ok(()) => println!("   Valid!"),
        Err(violations) => {
            println!("   Found {} violation(s):", violations.len());
            for v in violations.iter() {
                println!("      - {:?}: {}", v.violation_type(), v.message());
            }
        }
    }

    // Example 6: Break on first failure
    println!("\n6. Break on first failure:");
    let fast_fail_field = FieldBuilder::<String>::default()
        .name("code")
        .rule(Rule::Required.and(Rule::MinLength(5)).and(Rule::MaxLength(10)))
        .break_on_failure(true) // Stop at first error
        .build()
        .unwrap();

    match fast_fail_field.validate(&too_short) {
        Ok(()) => println!("   Valid!"),
        Err(violations) => {
            println!("   Stopped after {} violation(s):", violations.len());
            for v in violations.iter() {
                println!("      - {:?}: {}", v.violation_type(), v.message());
            }
        }
    }

    println!("\n=== Examples Complete ===");
}

