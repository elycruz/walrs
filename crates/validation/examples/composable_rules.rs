//! # Composable Rules Example
//!
//! Demonstrates rule composition using `.and()`, `.or()`, `.not()`,
//! `.when()`, and `.when_else()` combinators.

use walrs_validation::{Rule, Validate, ValidateRef};
use walrs_validation::rule::Condition;

fn main() {
    // -------------------------------------------------------------------------
    // `.and()` — both rules must pass (fail-fast)
    // -------------------------------------------------------------------------
    let password_rule = Rule::<String>::MinLength(8)
        .and(Rule::MaxLength(64))
        .and(Rule::pattern(r"[A-Z]").unwrap())  // must contain uppercase
        .and(Rule::pattern(r"[0-9]").unwrap()); // must contain digit

    assert!(password_rule.validate_ref("SecurePass1").is_ok());
    assert!(password_rule.validate_ref("short1A").is_err());   // too short
    assert!(password_rule.validate_ref("alllowercase1").is_err()); // no uppercase
    assert!(password_rule.validate_ref("NoDigitsHere").is_err()); // no digit

    println!(".and() combinator: OK");

    // -------------------------------------------------------------------------
    // `.or()` — at least one rule must pass
    // -------------------------------------------------------------------------
    // Accept either a US phone (+1...) or an international phone (+44...)
    let phone_rule = Rule::<String>::pattern(r"^\+1\d{10}$").unwrap()
        .or(Rule::pattern(r"^\+44\d{10}$").unwrap());

    assert!(phone_rule.validate_ref("+12345678901").is_ok());
    assert!(phone_rule.validate_ref("+44123456789_").is_err()); // wrong length
    assert!(phone_rule.validate_ref("+49000000000").is_err());  // neither pattern

    println!(".or() combinator: OK");

    // -------------------------------------------------------------------------
    // `.not()` — negates the inner rule
    // -------------------------------------------------------------------------
    let no_reserved_rule = Rule::<String>::OneOf(vec![
        "admin".to_string(),
        "root".to_string(),
        "system".to_string(),
    ])
    .not();

    assert!(no_reserved_rule.validate_ref("alice").is_ok());
    assert!(no_reserved_rule.validate_ref("admin").is_err());

    println!(".not() combinator: OK");

    // -------------------------------------------------------------------------
    // `.when()` — apply a rule only when a condition holds
    // -------------------------------------------------------------------------
    // When the value is > 0, it must also be a multiple of 10.
    let conditional_rule = Rule::<i32>::Step(10).when(Condition::GreaterThan(0));

    assert!(conditional_rule.validate(0).is_ok());   // condition false → skip
    assert!(conditional_rule.validate(10).is_ok());  // condition true, 10 % 10 == 0
    assert!(conditional_rule.validate(7).is_err());  // condition true, 7 % 10 != 0
    assert!(conditional_rule.validate(-5).is_ok());  // condition false (-5 > 0 is false) → skip

    println!(".when() combinator: OK");

    // -------------------------------------------------------------------------
    // `.when_else()` — conditional with an else branch
    // -------------------------------------------------------------------------
    // If value >= 18: must be <= 65 (adult working age).
    // Otherwise (< 18): must equal 16 or 17 (older minors only allowed).
    let age_rule = Rule::<u32>::Range { min: 18, max: 65 }
        .when_else(
            Condition::Custom(std::sync::Arc::new(|v: &u32| *v >= 18)),
            Rule::OneOf(vec![16, 17]),
        );

    assert!(age_rule.validate(25).is_ok());  // adult: 18 ≤ 25 ≤ 65
    assert!(age_rule.validate(70).is_err()); // adult: 70 > 65
    assert!(age_rule.validate(17).is_ok());  // minor: allowed
    assert!(age_rule.validate(15).is_err()); // minor: not in [16, 17]

    println!(".when_else() combinator: OK");

    // -------------------------------------------------------------------------
    // Complex composition: username validation
    // -------------------------------------------------------------------------
    let username_rule = Rule::<String>::Required
        .and(Rule::MinLength(3))
        .and(Rule::MaxLength(30))
        .and(Rule::pattern(r"^[a-zA-Z][a-zA-Z0-9_\-]*$").unwrap())
        .and(
            Rule::OneOf(vec![
                "admin".to_string(),
                "root".to_string(),
            ])
            .not(),
        );

    assert!(username_rule.validate_ref("alice_dev").is_ok());
    assert!(username_rule.validate_ref("admin").is_err());   // reserved
    assert!(username_rule.validate_ref("12abc").is_err());   // must start with letter
    assert!(username_rule.validate_ref("").is_err());        // required

    println!("Complex composition: OK");

    println!("\nAll composable rules examples passed!");
}
