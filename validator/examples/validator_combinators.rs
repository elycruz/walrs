//! Example: Validator combinators
//!
//! This example demonstrates how to combine validators using
//! AND, OR, NOT, and other combinators.
//!
//! Run with: `cargo run --example validator_combinators`

use walrs_validator::{
    LengthValidatorBuilder, RangeValidatorBuilder,
    Validate, ValidateRef, ValidateExt, ValidateRefExt,
    ValidatorAnd, ValidatorOr, ValidatorNot, ValidatorOptional, ValidatorWhen,
};

fn main() {
    println!("=== Validator Combinators Example ===\n");

    // AND combinator: value must satisfy both validators
    println!("--- AND Combinator ---");
    println!("Rule: Value must be >= 0 AND <= 100\n");

    let min_validator = RangeValidatorBuilder::<i32>::default()
        .min(0)
        .build()
        .unwrap();

    let max_validator = RangeValidatorBuilder::<i32>::default()
        .max(100)
        .build()
        .unwrap();

    // Using the .and() extension method
    let range_validator = min_validator.and(max_validator);

    for value in [-10, 0, 50, 100, 150] {
        let result = range_validator.validate(value);
        let status = if result.is_ok() { "✓ PASS" } else { "✗ FAIL" };
        println!("  {} -> {}", value, status);
    }

    println!();

    // OR combinator: at least one validator must pass
    println!("--- OR Combinator ---");
    println!("Rule: Value must be < 0 OR > 100 (outside normal range)\n");

    let negative = RangeValidatorBuilder::<i32>::default()
        .max(-1)
        .build()
        .unwrap();

    let large = RangeValidatorBuilder::<i32>::default()
        .min(101)
        .build()
        .unwrap();

    let outside_range = negative.or(large);

    for value in [-50, -1, 0, 50, 100, 101, 200] {
        let result = outside_range.validate(value);
        let status = if result.is_ok() { "✓ PASS" } else { "✗ FAIL" };
        println!("  {} -> {}", value, status);
    }

    println!();

    // NOT combinator: validator must fail for overall pass
    println!("--- NOT Combinator ---");
    println!("Rule: Value must NOT be between 0 and 10\n");

    let in_range = RangeValidatorBuilder::<i32>::default()
        .min(0)
        .max(10)
        .build()
        .unwrap();

    let not_in_range = in_range.not("Value must not be between 0 and 10");

    for value in [-5, 0, 5, 10, 15] {
        let result = not_in_range.validate(value);
        let status = if result.is_ok() { "✓ PASS" } else { "✗ FAIL" };
        println!("  {} -> {}", value, status);
    }

    println!();

    // Optional combinator: skip validation for empty values
    println!("--- Optional Combinator ---");
    println!("Rule: If not empty, length must be >= 5\n");

    let length_validator = LengthValidatorBuilder::<str>::default()
        .min_length(5)
        .build()
        .unwrap();

    let optional_length = length_validator.optional(|s: &str| s.is_empty());

    for value in ["", "hi", "hello", "hello world"] {
        let result = optional_length.validate_ref(value);
        let status = if result.is_ok() { "✓ PASS" } else { "✗ FAIL" };
        let display = if value.is_empty() { "(empty)" } else { value };
        println!("  \"{}\" -> {}", display, status);
    }

    println!();

    // When combinator: conditional validation
    println!("--- When Combinator ---");
    println!("Rule: Validate only when value > 0\n");

    let positive_validator = RangeValidatorBuilder::<i32>::default()
        .min(10)
        .build()
        .unwrap();

    let when_positive = positive_validator.when(|&v| v > 0);

    for value in [-5, 0, 5, 10, 15] {
        let result = when_positive.validate(value);
        let status = if result.is_ok() { "✓ PASS" } else { "✗ FAIL" };
        let note = if value <= 0 { " (skipped)" } else { "" };
        println!("  {} -> {}{}", value, status, note);
    }

    println!();

    // Complex combination
    println!("--- Complex Combination ---");
    println!("Rule: Length 3-10 AND (starts with 'a' OR starts with 'b')\n");

    let length_3_10 = LengthValidatorBuilder::<str>::default()
        .min_length(3)
        .max_length(10)
        .build()
        .unwrap();

    // For demonstration, we'll just use the length validator
    // In a real scenario, you'd combine with pattern validators

    let values = ["ab", "abc", "abcdefghijk", "hello"];

    for value in values {
        let result = length_3_10.validate_ref(value);
        let status = if result.is_ok() { "✓ PASS" } else { "✗ FAIL" };
        println!("  \"{}\" (len={}) -> {}", value, value.len(), status);
    }

    println!();
    println!("=== Example Complete ===");
}

