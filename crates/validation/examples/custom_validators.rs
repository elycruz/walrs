//! # Custom Validators Example
//!
//! Demonstrates `Rule::Custom` with closures, `WithMessage` for custom error
//! messages, and locale-aware message providers.

use std::sync::Arc;
use walrs_validation::{Rule, ValidateRef, Validate, Violation, ViolationType};

fn main() {
    // -------------------------------------------------------------------------
    // Rule::Custom — closure-based validation
    // -------------------------------------------------------------------------
    let no_spaces_rule = Rule::<String>::custom(Arc::new(|value: &String| {
        if value.contains(' ') {
            Err(Violation::new(
                ViolationType::PatternMismatch,
                "Value must not contain spaces.",
            ))
        } else {
            Ok(())
        }
    }));

    assert!(no_spaces_rule.validate_ref("hello").is_ok());
    assert!(no_spaces_rule.validate_ref("hello world").is_err());

    println!("Rule::Custom (no spaces): OK");

    // -------------------------------------------------------------------------
    // Custom numeric validator
    // -------------------------------------------------------------------------
    let even_rule = Rule::<i32>::custom(Arc::new(|&value: &i32| {
        if value % 2 == 0 {
            Ok(())
        } else {
            Err(Violation::new(
                ViolationType::StepMismatch,
                "Value must be even.",
            ))
        }
    }));

    assert!(even_rule.validate(4).is_ok());
    assert!(even_rule.validate(3).is_err());

    println!("Rule::Custom (even numbers): OK");

    // -------------------------------------------------------------------------
    // `.with_message()` — override violation message
    // -------------------------------------------------------------------------
    let age_rule = Rule::<u32>::Min(18).with_message("You must be at least 18 years old.");
    let result = age_rule.validate(16);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().message(),
        "You must be at least 18 years old."
    );

    println!(".with_message() override: OK");

    // -------------------------------------------------------------------------
    // `.with_message()` on composed rules
    // -------------------------------------------------------------------------
    let password_rule = Rule::<String>::MinLength(8)
        .and(Rule::MaxLength(64))
        .with_message("Password must be between 8 and 64 characters.");

    let result = password_rule.validate_ref("short");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().message(),
        "Password must be between 8 and 64 characters."
    );

    println!(".with_message() on composed rules: OK");

    // -------------------------------------------------------------------------
    // `.with_message_provider()` — dynamic message based on value
    // -------------------------------------------------------------------------
    let len_rule = Rule::<String>::MinLength(5).with_message_provider(
        |ctx: &walrs_validation::MessageContext<String>| {
            format!(
                "\"{}\" is too short — needs at least 5 characters, got {}.",
                ctx.value,
                ctx.value.chars().count()
            )
        },
        None,
    );

    let result = len_rule.validate_ref("hi");
    assert!(result.is_err());
    let msg = result.unwrap_err().message().to_string();
    assert!(msg.contains("hi"), "message should contain the value");
    assert!(msg.contains("2"), "message should contain actual length");

    println!(".with_message_provider() dynamic messages: OK");

    // -------------------------------------------------------------------------
    // `.with_locale()` — associate a locale with a rule
    // -------------------------------------------------------------------------
    let localized_rule = Rule::<String>::Required
        .with_message("Este campo es obligatorio.")
        .with_locale("es");

    assert!(localized_rule.validate_ref("").is_err());

    println!(".with_locale() locale tagging: OK");

    println!("\nAll custom validator examples passed!");
}
