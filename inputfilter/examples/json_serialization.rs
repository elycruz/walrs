//! JSON serialization example.
//!
//! This example demonstrates how to serialize and deserialize
//! Field configurations to/from JSON for config-driven validation.
//!
//! Run with: `cargo run --example json_serialization`

use walrs_form_core::Value;
use walrs_inputfilter::field::{Field, FieldBuilder};
use walrs_inputfilter::filter_enum::Filter;
use walrs_validator::Rule;

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
    .rule(Rule::Required.and(Rule::Email))
    .filters(vec![Filter::Trim, Filter::Lowercase])
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
    .and(Rule::Pattern(r"[A-Z]".to_string()))
    .and(Rule::Pattern(r"[0-9]".to_string()));

  let json = serde_json::to_string_pretty(&complex_rule).unwrap();
  println!("{}", json);

  // Example 5: Serialize Rule::Any
  println!("\n5. Serialize Rule::Any:");
  let any_rule = Rule::<String>::Any(vec![Rule::Email, Rule::Pattern(r"^\d{10}$".to_string())]);

  let json = serde_json::to_string_pretty(&any_rule).unwrap();
  println!("{}", json);

  // Example 6: Field with Value type for dynamic forms
  println!("\n6. Field<Value> for dynamic forms:");
  let dynamic_field = FieldBuilder::<Value>::default()
    .name("dynamic_input")
    .rule(Rule::Required)
    .filters(vec![Filter::Trim])
    .build()
    .unwrap();

  let json = serde_json::to_string_pretty(&dynamic_field).unwrap();
  println!("{}", json);

  // Example 7: Round-trip serialization
  println!("\n7. Round-trip serialization:");
  let original = FieldBuilder::<String>::default()
    .name("password")
    .rule(Rule::Required.and(Rule::MinLength(8)))
    .filters(vec![Filter::Trim])
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
    let result = product_code_field.validate(&value.to_string());
    println!("   '{}' -> {:?}", value, result.is_ok());
  }

  // Example 9: FieldFilter serialization
  println!("\n9. FieldFilter serialization:");
  use std::collections::HashMap;
  use walrs_inputfilter::field_filter::{CrossFieldRule, CrossFieldRuleType, FieldFilter};

  let mut field_filter = FieldFilter::new();

  // Fluent API - chain add_field and add_cross_field_rule
  field_filter
    .add_field(
      "email",
      FieldBuilder::<Value>::default()
        .rule(Rule::Required)
        .build()
        .unwrap(),
    )
    // Note: CrossFieldRules with Custom functions won't serialize
    .add_cross_field_rule(CrossFieldRule {
      name: Some("password_match".to_string()),
      fields: vec!["password".to_string(), "password_confirm".to_string()],
      rule: CrossFieldRuleType::FieldsEqual {
        field_a: "password".to_string(),
        field_b: "password_confirm".to_string(),
      },
    });

  let json = serde_json::to_string_pretty(&field_filter).unwrap();
  println!("{}", json);

  // Example 10: Deserialize and use FieldFilter
  println!("\n10. Deserialize and use FieldFilter:");
  let filter_config = r#"{
        "fields": {
            "username": {
                "name": "username",
                "rule": { "type": "required" }
            }
        },
        "cross_field_rules": []
    }"#;

  let loaded_filter: FieldFilter = serde_json::from_str(filter_config).unwrap();

  let valid_data: HashMap<String, Value> =
    [("username".to_string(), Value::String("john".to_string()))]
      .into_iter()
      .collect();

  let invalid_data: HashMap<String, Value> = HashMap::new();

  println!(
    "   Valid data: {:?}",
    loaded_filter.validate(&valid_data).is_ok()
  );
  println!(
    "   Invalid data (missing username): {:?}",
    loaded_filter.validate(&invalid_data).is_ok()
  );

  println!("\n=== Examples Complete ===");
}
