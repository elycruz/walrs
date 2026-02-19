//! Rule composition examples.
//!
//! This example demonstrates how to compose validation rules using
//! the `Rule<T>` enum combinators.
//!
//! Run with: `cargo run --example rule_composition`

use walrs_validator::{Condition, Rule};

fn main() {
  println!("=== Rule<T> Composition Examples ===\n");

  // Example 1: Simple rules
  println!("1. Simple rules:");
  let required = Rule::<String>::Required;
  let min_length = Rule::<String>::MinLength(3);
  let _max_length = Rule::<String>::MaxLength(50);

  println!(
    "   Required rule on 'hello': {:?}",
    required.validate_ref("hello", None)
  );
  println!(
    "   Required rule on '': {:?}",
    required.validate_ref("", None)
  );
  println!(
    "   MinLength(3) on 'hi': {:?}",
    min_length.validate_ref("hi", None)
  );
  println!(
    "   MinLength(3) on 'hello': {:?}",
    min_length.validate_ref("hello", None)
  );

  // Example 2: Combining rules with .and()
  println!("\n2. Combining rules with .and() (All must pass):");
  let username_rule = Rule::<String>::Required
    .and(Rule::MinLength(3))
    .and(Rule::MaxLength(20));

  println!(
    "   Combined rule on 'ab': {:?}",
    username_rule.validate_ref("ab", None)
  );
  println!(
    "   Combined rule on 'john': {:?}",
    username_rule.validate_ref("john", None)
  );
  println!(
    "   Combined rule on 'this_username_is_way_too_long': {:?}",
    username_rule.validate_ref("this_username_is_way_too_long", None)
  );

  // Example 3: Using Rule::All directly
  println!("\n3. Using Rule::All directly:");
  let password_rule = Rule::<String>::All(vec![
    Rule::Required,
    Rule::MinLength(8),
    Rule::MaxLength(128),
    Rule::Pattern(r"[A-Z]".to_string()), // Must contain uppercase
  ]);

  println!(
    "   'password': {:?}",
    password_rule.validate_ref("password", None)
  );
  println!(
    "   'Password123': {:?}",
    password_rule.validate_ref("Password123", None)
  );

  // Example 4: Using .or() combinator (Any must pass)
  println!("\n4. Using .or() combinator (Any must pass):");
  let contact_rule = Rule::<String>::Email.or(Rule::Pattern(r"^\d{3}-\d{4}$".to_string()));

  println!(
    "   'user@example.com': {:?}",
    contact_rule.validate_ref("user@example.com", None)
  );
  println!(
    "   '555-1234': {:?}",
    contact_rule.validate_ref("555-1234", None)
  );
  println!(
    "   'invalid': {:?}",
    contact_rule.validate_ref("invalid", None)
  );

  // Example 5: Using Rule::Any directly
  println!("\n5. Using Rule::Any directly:");
  let flexible_id = Rule::<String>::Any(vec![
    Rule::Email,
    Rule::Pattern(r"^\d{5,10}$".to_string()), // Numeric ID
    Rule::Pattern(r"^[A-Z]{2}\d{6}$".to_string()), // Code format
  ]);

  println!(
    "   'user@example.com': {:?}",
    flexible_id.validate_ref("user@example.com", None)
  );
  println!(
    "   '123456': {:?}",
    flexible_id.validate_ref("123456", None)
  );
  println!(
    "   'AB123456': {:?}",
    flexible_id.validate_ref("AB123456", None)
  );
  println!(
    "   'invalid': {:?}",
    flexible_id.validate_ref("invalid", None)
  );

  // Example 6: Negation with .not()
  println!("\n6. Negation with .not():");
  let not_empty = Rule::<String>::MinLength(1);
  let is_empty = not_empty.clone().not();

  println!(
    "   not(MinLength(1)) on '': {:?}",
    is_empty.validate_ref("", None)
  );
  println!(
    "   not(MinLength(1)) on 'hello': {:?}",
    is_empty.validate_ref("hello", None)
  );

  // Example 7: Conditional rules with .when()
  println!("\n7. Conditional rules with .when():");
  let conditional_rule = Rule::<String>::MinLength(8).when(Condition::IsNotEmpty);

  println!(
    "   MinLength(8).when(IsNotEmpty) on '': {:?}",
    conditional_rule.validate_ref("", None)
  );
  println!(
    "   MinLength(8).when(IsNotEmpty) on 'short': {:?}",
    conditional_rule.validate_ref("short", None)
  );
  println!(
    "   MinLength(8).when(IsNotEmpty) on 'longenough': {:?}",
    conditional_rule.validate_ref("longenough", None)
  );

  // Example 8: Conditional with else using .when_else()
  println!("\n8. Conditional with else using .when_else():");
  let with_else = Rule::<String>::MinLength(8).when_else(
    Condition::IsNotEmpty,
    Rule::Required, // Else rule: require value if empty check fails
  );

  // Note: This creates a When variant with both then_rule and else_rule
  println!("   Rule structure: {:?}", with_else);

  // Example 9: Pattern matching
  println!("\n9. Pattern matching:");
  let email_pattern = Rule::<String>::Email;
  let url_pattern = Rule::<String>::Url;
  let custom_pattern = Rule::<String>::Pattern(r"^[a-z]+$".to_string());

  println!(
    "   Email on 'test@example.com': {:?}",
    email_pattern.validate_ref("test@example.com", None)
  );
  println!(
    "   Email on 'invalid': {:?}",
    email_pattern.validate_ref("invalid", None)
  );
  println!(
    "   Url on 'https://example.com': {:?}",
    url_pattern.validate_ref("https://example.com", None)
  );
  println!(
    "   Pattern([a-z]+) on 'hello': {:?}",
    custom_pattern.validate_ref("hello", None)
  );
  println!(
    "   Pattern([a-z]+) on 'Hello': {:?}",
    custom_pattern.validate_ref("Hello", None)
  );

  // Example 10: Numeric rules
  println!("\n10. Numeric rules:");
  let age_rule = Rule::<i32>::Min(0).and(Rule::Max(150));
  let percentage_rule = Rule::<f64>::Range {
    min: 0.0,
    max: 100.0,
  };

  println!("   Age rule on -5: {:?}", age_rule.validate(-5, None));
  println!("   Age rule on 25: {:?}", age_rule.validate(25, None));
  println!("   Age rule on 200: {:?}", age_rule.validate(200, None));
  println!(
    "   Percentage on 50.5: {:?}",
    percentage_rule.validate(50.5, None)
  );
  println!(
    "   Percentage on 150.0: {:?}",
    percentage_rule.validate(150.0, None)
  );

  // Example 11: Exact length
  println!("\n11. Exact length:");
  let pin_code = Rule::<String>::ExactLength(4);
  let zip_code = Rule::<String>::ExactLength(5);

  println!(
    "   PIN(4) on '123': {:?}",
    pin_code.validate_ref("123", None)
  );
  println!(
    "   PIN(4) on '1234': {:?}",
    pin_code.validate_ref("1234", None)
  );
  println!(
    "   ZIP(5) on '12345': {:?}",
    zip_code.validate_ref("12345", None)
  );

  // Example 12: OneOf for enum-like values
  println!("\n12. OneOf for enum-like values:");
  let status_rule = Rule::<String>::OneOf(vec![
    "pending".to_string(),
    "active".to_string(),
    "completed".to_string(),
    "cancelled".to_string(),
  ]);

  println!(
    "   Status 'active': {:?}",
    status_rule.validate_ref("active", None)
  );
  println!(
    "   Status 'invalid': {:?}",
    status_rule.validate_ref("invalid", None)
  );

  // Example 13: Custom messages
  println!("\n13. Custom error messages:");
  let with_message =
    Rule::<String>::MinLength(8).with_message("Password must be at least 8 characters");

  match with_message.validate_ref("short", None) {
    Ok(()) => println!("   Valid"),
    Err(violation) => println!("   Error: {}", violation.message()),
  }

  // Example 14: Collecting all violations
  println!("\n14. Collecting all violations:");
  let strict_rule = Rule::<String>::Required
    .and(Rule::MinLength(8))
    .and(Rule::Pattern(r"[0-9]".to_string()).with_message("Must contain a number"))
    .and(Rule::Pattern(r"[A-Z]".to_string()).with_message("Must contain uppercase"));

  match strict_rule.validate_ref_all("abc", None) {
    Ok(()) => println!("   All rules passed!"),
    Err(violations) => {
      println!("   Found {} violations:", violations.len());
      for v in violations.iter() {
        println!("      - {}", v.message());
      }
    }
  }

  println!("\n=== Examples Complete ===");
}
