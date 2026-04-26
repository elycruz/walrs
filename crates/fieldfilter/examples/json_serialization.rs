//! JSON serialization example.
//!
//! This example demonstrates how to serialize and deserialize
//! Field configurations to/from JSON for config-driven validation.
//!
//! Run with: `cargo run --example json_serialization`

#![allow(deprecated)]

use walrs_fieldfilter::field::{Field, FieldBuilder};
use walrs_filter::FilterOp;
use walrs_validation::Rule;
use walrs_validation::Value;

fn main() {
  println!("=== JSON Serialization Examples ===\n");

  // Example 1: Serialize a simple field
  println!("1. Serialize a simple field:");
  let username_field = FieldBuilder::<String>::default()
    .name("username")
    .rule(
      Rule::Required
        .and(Rule::MinLength(3))
        .and(Rule::MaxLength(20)),
    )
    .build()
    .unwrap();

  let json = serde_json::to_string_pretty(&username_field).unwrap();
  println!("{}", json);

  // Example 2: Serialize a field with filters
  println!("\n2. Serialize a field with filters:");
  let email_field = FieldBuilder::<String>::default()
    .name("email")
    .rule(Rule::Required.and(Rule::Email(Default::default())))
    .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
    .build()
    .unwrap();

  let json = serde_json::to_string_pretty(&email_field).unwrap();
  println!("{}", json);

  // Example 3: Deserialize a field from JSON
  println!("\n3. Deserialize a field from JSON:");
  let json_config = r#"{
        "name": "age",
        "rule": {
            "type": "all",
            "config": [
                { "type": "min", "config": 0 },
                { "type": "max", "config": 150 }
            ]
        }
    }"#;

  let age_field: Field<i32> = serde_json::from_str(json_config).unwrap();
  println!("   Deserialized field name: {:?}", age_field.name);
  println!("   Has rule: {}", age_field.rule.is_some());

  // Example 4: Serialize complex rule compositions
  println!("\n4. Serialize complex rule compositions:");
  let complex_rule = Rule::<String>::Required
    .and(Rule::MinLength(8))
    .and(Rule::pattern(r"[A-Z]").unwrap())
    .and(Rule::pattern(r"[0-9]").unwrap());

  let json = serde_json::to_string_pretty(&complex_rule).unwrap();
  println!("{}", json);

  // Example 5: Serialize Rule::Any
  println!("\n5. Serialize Rule::Any:");
  let any_rule = Rule::<String>::Any(vec![
    Rule::Email(Default::default()),
    Rule::pattern(r"^\d{10}$").unwrap(),
  ]);

  let json = serde_json::to_string_pretty(&any_rule).unwrap();
  println!("{}", json);

  // Example 6: Field with Value type for dynamic forms
  println!("\n6. Field<Value> for dynamic forms:");
  let dynamic_field = FieldBuilder::<Value>::default()
    .name("dynamic_input")
    .rule(Rule::Required)
    .filters(vec![FilterOp::Trim])
    .build()
    .unwrap();

  let json = serde_json::to_string_pretty(&dynamic_field).unwrap();
  println!("{}", json);

  // Example 7: Round-trip serialization
  println!("\n7. Round-trip serialization:");
  let original = FieldBuilder::<String>::default()
    .name("password")
    .rule(Rule::Required.and(Rule::MinLength(8)))
    .filters(vec![FilterOp::Trim])
    .break_on_failure(true)
    .build()
    .unwrap();

  let serialized = serde_json::to_string(&original).unwrap();
  let deserialized: Field<String> = serde_json::from_str(&serialized).unwrap();

  println!("   Original name: {:?}", original.name);
  println!("   Deserialized name: {:?}", deserialized.name);
  println!(
    "   break_on_failure matches: {}",
    original.break_on_failure == deserialized.break_on_failure
  );

  // Example 8: Config-driven validation
  println!("\n8. Config-driven validation:");
  let config_json = r#"{
        "name": "product_code",
        "rule": {
            "type": "all",
            "config": [
                { "type": "required" },
                { "type": "exactlength", "config": 8 },
                { "type": "pattern", "config": "^[A-Z0-9]+$" }
            ]
        }
    }"#;

  let product_code_field: Field<String> = serde_json::from_str(config_json).unwrap();

  let test_values = vec!["", "ABC", "ABCD1234", "abcd1234", "ABCD12345"];
  for value in test_values {
    let result = product_code_field.validate_ref(value);
    println!("   '{}' -> {:?}", value, result.is_ok());
  }

  println!("\n=== Examples Complete ===");
}
